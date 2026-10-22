'use strict';
function check(value, message) { if (!value) throw new Error(message); }
const object = Object.create(null);
const keys = [];
for (let i = 0; i < 517; ++i) {
  const key = 'property\ud800\udc00\udfff:' + 'x'.repeat(257) + ':' + i;
  keys.push(key);
  object[key] = i;
}
if (typeof gc === 'function') gc();
check(Object.keys(object).length === keys.length, 'initial property growth');
for (let i = 0; i < keys.length; ++i) {
  const sameContent = ('!' + keys[i]).slice(1);
  check(object[sameContent] === i, 'content lookup');
  Object.defineProperty(object, sameContent, { value: i + 1000 });
}
check(Object.keys(object).every((key, i) => key === keys[i]), 'replacement preserves order');
for (let i = 0; i < keys.length; i += 3) {
  check(delete object[keys[i]], 'delete property');
  Object.defineProperty(object, keys[i], { value: -i, configurable: true, enumerable: true, writable: true });
}
const expected = keys.filter((_, i) => i % 3 !== 0).concat(keys.filter((_, i) => i % 3 === 0));
check(Object.keys(object).every((key, i) => key === expected[i]), 'holes and reinsertion order');
const symbol1 = Symbol('same');
const symbol2 = Symbol('same');
object[symbol1] = 17; object[symbol2] = 29;
check(object[symbol1] === 17 && object[symbol2] === 29, 'symbol identity');
check(Reflect.ownKeys(object).slice(-2).every((key, i) => key === [symbol1, symbol2][i]), 'symbol order');
const getter = function () { return 42; };
let setValue;
const setter = function (value) { setValue = value; };
Object.defineProperty(object, keys[0], { get: getter });
Object.defineProperty(object, keys[0], { set: setter });
if (typeof gc === 'function') gc();
const descriptor = Object.getOwnPropertyDescriptor(object, keys[0]);
check(descriptor.get === getter && descriptor.set === setter, 'independent accessor fields');
object[keys[0]] = 99;
check(setValue === 99 && object[keys[0]] === 42, 'accessor replacement');
const events = [];
const coercingKey = { [Symbol.toPrimitive](hint) { events.push(hint); return keys[1]; } };
object[coercingKey] = (events.push('rhs'), 123);
check(events.join(',') === 'rhs,string' && object[keys[1]] === 123, 'canonicalization ordering');
const marker = {};
try { object[{ toString() { throw marker; } }] = 1; throw new Error('missed key error'); }
catch (e) { check(e === marker, 'abrupt key conversion'); }
const backing = Object.create(null);
const traps = [];
const proxy = new Proxy(backing, { defineProperty(target, key, desc) {
  traps.push(key);
  return Reflect.defineProperty(target, key, desc);
} });
Object.defineProperty(proxy, coercingKey, { value: 11, writable: true, configurable: true });
check(traps.length === 1 && traps[0] === keys[1] && backing[keys[1]] === 11, 'proxy receives canonical key');
Object.seal(object);
check(Object.isSealed(object) && Reflect.ownKeys(object).length === keys.length + 2, 'seal after indexed growth');
Object.freeze(object);
check(Object.isFrozen(object), 'freeze descriptors');
try { object[keys[1]] = 5; throw new Error('missed frozen write'); }
catch (e) { check(e instanceof TypeError, 'frozen write rejects'); }
print('prepared-property-hash-ok');
