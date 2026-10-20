function check(value, message) { if (!value) throw new Error(message); }
var trace = [], masks = { value: false }, whole = { marker: Symbol('with') };
whole.self = whole;
var target = { marker: 262 }, stored, originalReceiver = false;
Object.defineProperty(target, 'value', {
  configurable: true,
  get() { trace.push('get'); throw whole; },
  set(value) { originalReceiver = this.marker === 262; value.result = 9; stored = value; }
});
target[Symbol.unscopables] = masks;
var selected = new Proxy(target, {
  has(object, key) { if (key === 'value') trace.push('has'); return key in object; },
  get(object, key) {
    if (key === Symbol.unscopables) trace.push('unscopables');
    if (key === 'value') trace.push('proxy-get');
    return object[key];
  },
  set(object, key, value) { if (key === 'value') trace.push('set'); object[key] = value; return true; }
});
function* write() { with (selected) { return value = yield 'with'; } }
var iterator = write();
check(iterator.next().value === 'with' && trace.join(',') === 'has,unscopables',
      'with capture performs HasBinding and unscopables but no GetValue');
masks.value = true; selected = null; target = null; gc();
var result = iterator.next(whole);
check(result.done && result.value === whole && stored === whole && whole.result === 9 && originalReceiver,
      'selected original receiver remains rooted and setter can mutate the returned RHS');
check(trace.join(',') === 'has,unscopables,has,set', 'Put uses the saved object without new unscopables selection');

var mutating = { set value(value) { value.result = 9; } };
function* returnedObject() {
  with (mutating) { return (value = { result: 1, payload: yield 'rhs-object' }).result; }
}
iterator = returnedObject();
check(iterator.next().value === 'rhs-object' && iterator.next(2).value === 9,
      'setter mutation invalidates heap facts for the actual returned RHS object');

trace = []; masks.value = false; var originalWrite = false;
target = { value: 0, marker: 263 }; target[Symbol.unscopables] = masks;
selected = new Proxy(target, {
  has(object, key) { if (key === 'value') trace.push('has'); return key in object; },
  get(object, key) { if (key === Symbol.unscopables) trace.push('unscopables'); return object[key]; },
  set(object, key, value) { if (key === 'value') { trace.push('set'); originalWrite = object.marker === 263; } object[key] = value; return true; }
});
iterator = write(); check(iterator.next().value === 'with', 'second selected write suspends');
delete target.value; masks.value = true; target = null; selected = null; gc();
check(iterator.next(7).value === 7 && originalWrite && trace.join(',') === 'has,unscopables,has,set',
      'deleted property is recreated on the original sloppy object environment');

var added = {}; delete globalThis.plainWithMissing;
function* missing() { with (added) { return plainWithMissing = yield 'missing'; } }
iterator = missing(); check(iterator.next().value === 'missing', 'with miss still reaches RHS');
added.plainWithMissing = 100; gc();
check(iterator.next(8).value === 8 && added.plainWithMissing === 100 && globalThis.plainWithMissing === 8,
      'original unresolvable Reference creates a global instead of reselecting the changed with object');
delete globalThis.plainWithMissing;
function* missedTdz() { with ({}) { future = yield 'tdz'; } let future; }
iterator = missedTdz(); check(iterator.next().value === 'tdz', 'missing with binding defers fallback TDZ');
var caught = null;
try { iterator.next(1); } catch (error) { caught = error; }
check(caught instanceof ReferenceError, 'captured fallback cell checks TDZ only at Put');
function* hiddenTdz() { with ({ future: 1 }) { return future = yield 'hidden'; } let future; }
iterator = hiddenTdz();
check(iterator.next().value === 'hidden' && iterator.next(2).value === 2,
      'successful object selection does not write its uninitialized declarative fallback');

selected = new Proxy({ value: 1 }, { has(object, key) { if (key === 'value') throw whole; return key in object; } });
iterator = write(); caught = null;
try { iterator.next(); } catch (error) { caught = error; }
check(caught === whole, 'HasBinding abrupt still precedes the RHS in write-only mode');
selected = new Proxy({ value: 1 }, {
  has(object, key) { return key in object; },
  get(object, key) { return object[key]; },
  set(object, key, value) { if (key === 'value') throw whole; object[key] = value; return true; }
});
iterator = write(); check(iterator.next().value === 'with', 'throwing setter is not called during capture');
caught = null;
try { iterator.next(3); } catch (error) { caught = error; }
check(caught === whole && caught.self === caught, 'whole setter abrupt propagates from Put after the RHS');
print('generator-plain-with:ok');
