function check(value, message) { if (!value) throw new Error(message); }
var trace = '';
var raw = { identity: 1 };
var symbol = Symbol('named');
var proto = { get home() { return this.tag; } };
var short = 17;
var key = { [Symbol.toPrimitive](hint) {
  check(hint === 'string', 'computed key hint'); trace += 'key;'; return 'data';
} };
var spread = { get copied() { trace += 'spread;'; return raw; } };
function* build() {
  return { __proto__: proto, short, [yield 'key']: yield 'value',
    [symbol]: function () { return 11; },
    get [yield 'getter']() { return this.slot; },
    set [yield 'setter'](value) { this.slot = value; },
    [yield 'method']() { return super.home; },
    ...yield 'spread', final: yield 'last' };
}
var iterator = build();
check(iterator.next().value === 'key', 'first property key'); gc();
check(iterator.next(key).value === 'value' && trace === 'key;', 'key conversion before value');
key[Symbol.toPrimitive] = function () { throw new Error('key converted twice'); }; gc();
check(iterator.next(raw).value === 'getter', 'value identity retained');
check(iterator.next('pair').value === 'setter', 'getter installed first'); gc();
check(iterator.next('pair').value === 'method', 'setter merged with getter');
check(iterator.next('method').value === 'spread', 'method installed first'); gc();
check(iterator.next(spread).value === 'last' && trace === 'key;spread;', 'spread copied before later yield');
Object.defineProperty(spread, 'copied', { value: 99 });
var result = iterator.next(raw);
var object = result.value;
check(result.done && object.data === raw && object.final === raw && object.copied === raw, 'raw values and copied values');
check(object.short === 17 && Object.getPrototypeOf(object) === proto, 'shorthand and prototype');
check(object.method.call({ tag: 73 }) === 73, 'method HomeObject and super receiver');
check(object[symbol].name === '[named]' && object[symbol]() === 11, 'computed function name');
var name = Object.getOwnPropertyDescriptor(object[symbol], 'name');
check(name.writable === false && name.enumerable === false && name.configurable === true, 'function name attributes');
var pair = Object.getOwnPropertyDescriptor(object, 'pair');
check(pair.get.name === 'get pair' && pair.set.name === 'set pair' && pair.enumerable && pair.configurable, 'accessor identity and descriptors');
var keys = Reflect.ownKeys(object);
check(keys.length === 7 && keys.slice(0, 6).join(',') === 'short,data,pair,method,copied,final' && keys[6] === symbol, 'property order');
object.pair = 28;
check(object.pair === 28 && object.slot === 28, 'merged accessor receiver');
keys = Reflect.ownKeys(object);
check(keys.length === 8 && keys[6] === 'slot' && keys[7] === symbol, 'setter appends its own data property');

function* prototypeCases() {
  return { __proto__: yield 'proto', ['__proto__']: yield 'own' };
}
iterator = prototypeCases();
check(iterator.next().value === 'proto', 'prototype yield');
check(iterator.next(null).value === 'own', 'null prototype before own data');
object = iterator.next(raw).value;
check(Object.getPrototypeOf(object) === null && object.__proto__ === raw, 'computed proto is ordinary data');
check(Object.getOwnPropertyDescriptor(object, '__proto__').writable === true, 'own proto descriptor');
iterator = prototypeCases(); iterator.next(); iterator.next(3); object = iterator.next(raw).value;
check(Object.getPrototypeOf(object) === Object.prototype, 'primitive proto ignored');
function* shorthandProto() { let __proto__ = 4; return { __proto__, final: yield 'last' }; }
iterator = shorthandProto(); iterator.next(); object = iterator.next(5).value;
check(object.__proto__ === 4 && Object.getPrototypeOf(object) === Object.prototype, 'shorthand proto is ordinary data');

var classTrace = '';
var classKey = { [Symbol.toPrimitive](hint) {
  check(hint === 'string', 'class key hint'); classTrace += 'key;'; return 'C';
} };
function* classBuild() {
  return { [yield 'class-key']: class {
    [yield 'class-field']() { return 8; }
    static label = (classTrace += 'static;', this.name);
  }, later: yield 'class-last' };
}
iterator = classBuild(); check(iterator.next().value === 'class-key', 'class key phase');
check(iterator.next(classKey).value === 'class-field' && classTrace === 'key;', 'class NamedEvaluation follows key conversion'); gc();
classKey[Symbol.toPrimitive] = function () { throw new Error('class key replay'); };
check(iterator.next('member').value === 'class-last' && classTrace === 'key;static;', 'class prepared once before later property');
object = iterator.next(raw).value;
check(object.C.name === 'C' && object.C.label === 'C' && new object.C().member() === 8, 'captured class name precedes static initialization');

var marker = { whole: true };
var badKey = { [Symbol.toPrimitive]() { trace += 'throw;'; throw marker; } };
function* abrupt() {
  try { return { [yield 'abrupt-key']: yield 'forbidden', later: trace += 'later;' }; }
  finally { yield 'finally'; }
}
iterator = abrupt(); check(iterator.next().value === 'abrupt-key', 'abrupt start');
check(iterator.next(badKey).value === 'finally', 'key throw reaches yielding finally before value'); gc();
try { iterator.next(); throw new Error('missing key throw'); } catch (error) { check(error === marker, 'whole key throw'); }
check(trace === 'key;spread;throw;', 'later property effects suppressed');
iterator = abrupt(); iterator.next();
check(iterator.return(marker).value === 'finally', 'injected return reaches finally'); gc();
result = iterator.next(); check(result.done && result.value === marker, 'whole injected return');
iterator = abrupt(); iterator.next(); iterator.next('safe');
check(iterator.throw(marker).value === 'finally', 'injected throw during value'); gc();
try { iterator.next(); throw new Error('missing injected throw'); } catch (error) { check(error === marker, 'whole injected throw'); }
check(trace === 'key;spread;throw;', 'abandoned construction does not define later properties');

var discardedTrace = '';
var discardedKey = { [Symbol.toPrimitive](hint) {
  check(hint === 'string', 'discarded key hint'); discardedTrace += 'key;'; return 'data';
} };
var discardedSpread = { get copied() { discardedTrace += 'spread;'; return raw; } };
var discardedLateKey = { [Symbol.toPrimitive](hint) {
  check(hint === 'string', 'later discarded key hint'); discardedTrace += 'late-key;'; return 'last';
} };
function* discarded() {
  ({ [yield 'discard-key']: (discardedTrace += 'value;', yield 'discard-value'),
    ...discardedSpread, [discardedLateKey]: yield 'discard-last' });
  return marker;
}
iterator = discarded();
check(iterator.next().value === 'discard-key' && discardedTrace === '', 'discarded literal begins at its first key');
check(iterator.next(discardedKey).value === 'discard-value' && discardedTrace === 'key;value;', 'discarded key conversion precedes value effects'); gc();
discardedKey[Symbol.toPrimitive] = function () { throw new Error('discarded key replay'); };
check(iterator.next(raw).value === 'discard-last' && discardedTrace === 'key;value;spread;late-key;', 'discarded property and spread precede the next key and value'); gc();
result = iterator.next(raw);
check(result.done && result.value === marker && discardedTrace === 'key;value;spread;late-key;', 'discarded construction resumes once and preserves later completion');
print('generator-object-literal:ok');
