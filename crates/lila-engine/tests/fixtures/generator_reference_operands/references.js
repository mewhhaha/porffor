function check(condition, message) { if (!condition) throw new Error(message); }
function step(iterator, input, expected, done) {
  var result = iterator.next(input);
  check(result.value === expected && result.done === done, 'step:' + expected);
}
function throws(action, type) {
  var caught = false;
  try { action(); } catch (error) { caught = error instanceof type; }
  check(caught, 'expected:' + type.name);
}
var events = [], writes = 0, written;
var old = { [Symbol.toPrimitive]: function () { events.push('old-number'); return 7; } };
var right = { [Symbol.toPrimitive]: function () { events.push('right-number'); return 5; } };
var target = {
  get selected() { events.push('get'); return old; },
  set selected(value) { check(this === target, 'original-receiver'); events.push('set'); writes++; written = value; }
};
var key = { [Symbol.toPrimitive]: function () { events.push('key'); return 'selected'; } };
function* compound() { return (yield 'base')[yield 'key'] += (yield 'rhs'); }
var iterator = compound();
step(iterator, undefined, 'base', false);
step(iterator, target, 'key', false);
step(iterator, key, 'rhs', false);
check(events.join(',') === 'key,get', 'GetValue-before-RHS-without-numeric-coercion');
gc();
step(iterator, right, 12, true);
check(events.join(',') === 'key,get,old-number,right-number,set' && writes === 1 && written === 12, 'one-retained-reference-and-late-numeric-coercion');

function* andValue() { return (yield 'base')[yield 'key'] &&= (yield 'rhs'); }
function* orValue() { return (yield 'base')[yield 'key'] ||= (yield 'rhs'); }
function* nullishValue() { return (yield 'base')[yield 'key'] ??= (yield 'rhs'); }
for (var row of [[andValue,0,false],[andValue,2,true],[orValue,3,false],[orValue,0,true],[nullishValue,4,false],[nullishValue,null,true]]) {
  var reads = 0, sets = 0, conversions = 0, initial = row[1];
  var selected = { get value() { reads++; return initial; }, set value(value) { check(this === selected, 'logical-receiver'); sets++; written = value; } };
  var selectedKey = { [Symbol.toPrimitive]: function () { conversions++; return 'value'; } };
  iterator = row[0]();
  step(iterator, undefined, 'base', false);
  step(iterator, selected, 'key', false);
  if (row[2]) {
    step(iterator, selectedKey, 'rhs', false);
    gc();
    step(iterator, right, right, true);
    check(sets === 1 && written === right, 'selected-logical-whole-value');
  } else {
    step(iterator, selectedKey, initial, true);
    check(sets === 0, 'skipped-logical-does-not-set');
  }
  check(reads === 1 && conversions === 1, 'logical-LHS-once');
}

var whole = { reason: 29 }, finalizers = 0;
function* interrupted() { try { return (yield 'base')[yield 'key'] += (yield 'rhs'); } finally { finalizers++; } }
iterator = interrupted(); iterator.next(); iterator.next(target); iterator.next(key);
try { iterator.throw(whole); throw 'missing-throw'; } catch (error) { check(error === whole, 'whole-injected-throw'); }
check(writes === 1 && finalizers === 1, 'injection-before-PutValue');
iterator = interrupted(); iterator.next(); iterator.next(target); iterator.next(key);
var returned = iterator.return(whole);
check(returned.done && returned.value === whole && writes === 1 && finalizers === 2, 'return-before-PutValue');

class PrivateOwner {
  #value = 13n;
  *plain() { return (yield 'target').#value = (yield 'rhs'); }
  *compound() { return (yield 'target').#value += (yield 'rhs'); }
  *logical() { return (yield 'target').#value ||= (yield 'unreached'); }
  *read() { return (yield 'target').#value; }
  *has() { return #value in (yield 'target'); }
  *update() { return (yield 'target').#value++; }
}
var owner = new PrivateOwner();
iterator = owner.plain(); step(iterator, undefined, 'target', false); step(iterator, {}, 'rhs', false);
throws(function () { iterator.next(17n); }, TypeError);
iterator = owner.compound(); step(iterator, undefined, 'target', false);
throws(function () { iterator.next({}); }, TypeError);
iterator = owner.update(); step(iterator, undefined, 'target', false); gc(); step(iterator, owner, 13n, true);
iterator = owner.read(); iterator.next(); step(iterator, owner, 14n, true);
iterator = owner.logical(); iterator.next(); step(iterator, owner, 14n, true);
iterator = owner.has(); iterator.next(); step(iterator, owner, true, true);
iterator = owner.has(); iterator.next(); throws(function () { iterator.next(0); }, TypeError);

events = [];
var first = {
  get selected() { events.push('super-get'); return this.input; },
  set selected(value) { events.push('first-set'); this.output = value; }
};
var second = { set selected(value) { throw 'wrong-super-base'; } };
var home = {
  *compound() { return super[yield 'key'] += (yield 'rhs'); },
  *plain() { return super[yield 'key'] = (yield 'rhs'); },
  *read() { return super[yield 'key']; },
  *update() { return super[yield 'key']++; },
  *deleted() { return delete super[yield 'key']; }
};
Object.setPrototypeOf(home, first);
var receiver = { input: 19 };
iterator = home.compound.call(receiver);
step(iterator, undefined, 'key', false); step(iterator, key, 'rhs', false);
Object.setPrototypeOf(home, second); gc(); step(iterator, 4, 23, true);
check(receiver.output === 23 && events.join(',') === 'key,super-get,first-set', 'saved-Super-base-and-original-this');
Object.setPrototypeOf(home, first); events = [];
iterator = home.plain.call(receiver);
step(iterator, undefined, 'key', false); step(iterator, key, 'rhs', false);
check(events.length === 0, 'plain-Super-defers-key-conversion-and-Get');
Object.setPrototypeOf(home, second); step(iterator, whole, whole, true);
check(events.join(',') === 'key,first-set' && receiver.output === whole, 'plain-Super-saves-base-before-RHS');
Object.setPrototypeOf(home, first); events = [];
iterator = home.read.call(receiver); iterator.next(); step(iterator, 'selected', 19, true);
iterator = home.update.call(receiver); iterator.next(); step(iterator, 'selected', 19, true);
check(receiver.output === 20 && events.join(',') === 'super-get,super-get,first-set', 'shared-Super-read-and-update');
iterator = home.deleted.call(receiver); iterator.next();
throws(function () { iterator.next({ [Symbol.toPrimitive]: function () { throw 'delete-must-not-coerce'; } }); }, ReferenceError);

var numeric = { value: 31n };
function* propertyUpdate() { return ++(yield 'base')[yield 'key']; }
iterator = propertyUpdate(); iterator.next(); iterator.next(numeric); step(iterator, 'value', 32n, true);
check(numeric.value === 32n, 'property-update-dynamic-BigInt');
print('generator-reference-operands:ok');
