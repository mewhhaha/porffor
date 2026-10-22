function check(value, message) { if (!value) throw new Error(message); }
const map = new Map();
const set = new Set();
const keys = [];
for (let i = 0; i < 1100; ++i) {
  const key = 'prefix\ud800\udc00\udfff:' + 'x'.repeat(257) + ':' + i;
  keys.push(key);
  map.set(key, i);
  set.add(key);
}
check(map.size === 1100 && set.size === 1100, 'growth sizes');
for (let i = 0; i < keys.length; ++i) {
  const sameContent = ('!' + keys[i]).slice(1);
  check(map.get(sameContent) === i && set.has(sameContent), 'content lookup after growth');
  map.set(sameContent, i + 2000);
  set.add(sameContent);
}
check(map.size === 1100 && set.size === 1100, 'duplicate inserts');
for (let i = 0; i < keys.length; i += 3) {
  check(map.delete(keys[i]) && set.delete(keys[i]), 'delete');
  map.set(keys[i], -i);
  set.add(keys[i]);
}
const expected = keys.filter((_, i) => i % 3 !== 0).concat(keys.filter((_, i) => i % 3 === 0));
check([...map.keys()].every((key, i) => key === expected[i]), 'map reinsertion order');
check([...set].every((key, i) => key === expected[i]), 'set reinsertion order');
const object1 = { toString() { throw new Error('key coercion'); }, valueOf() { throw new Error('key coercion'); } };
const object2 = {};
const symbol1 = Symbol('same');
const symbol2 = Symbol('same');
const mixed = [NaN, -0, 1, '1', 1n, undefined, null, false, object1, object2, symbol1, symbol2, 123456789012345678901234567890n];
const mixedMap = new Map();
const mixedSet = new Set();
for (let i = 0; i < mixed.length; ++i) { mixedMap.set(mixed[i], i); mixedSet.add(mixed[i]); }
if (typeof gc === 'function') gc();
mixedMap.set(0, 99); mixedMap.set(0 / 0, 100);
mixedSet.add(0); mixedSet.add(0 / 0);
check(mixedMap.size === mixed.length && mixedSet.size === mixed.length, 'SameValueZero size');
check(mixedMap.get(NaN) === 100 && mixedMap.get(-0) === 99 && mixedSet.has(-0), 'SameValueZero lookup');
check([...mixedMap.keys()].some(key => key === 0 && 1 / key === Infinity), 'stored map zero is positive');
check([...mixedSet].some(key => key === 0 && 1 / key === Infinity), 'stored set zero is positive');
for (let i = 2; i < mixed.length; ++i) check(mixedMap.get(mixed[i]) === i && mixedSet.has(mixed[i]), 'mixed keys');
const mapIterator = mixedMap.keys();
const setIterator = mixedSet.values();
check(Number.isNaN(mapIterator.next().value) && Number.isNaN(setIterator.next().value), 'live iterator first');
const retained = [];
for (let i = 0; i < 4500; ++i) {
  const object = { index: i, text: 'retained-' + i };
  retained.push(object);
  mixedMap.set(object, i + 200);
  mixedSet.add(object);
  if ((i === 31 || i === 513 || i === 4095) && typeof gc === 'function') gc();
}
for (let i = 0; i < retained.length; i += 31) check(mixedMap.get(retained[i]) === i + 200 && mixedSet.has(retained[i]), 'object roots through growth');
let mapCount = 1;
for (let item = mapIterator.next(); !item.done; item = mapIterator.next()) ++mapCount;
let setCount = 1;
for (let item = setIterator.next(); !item.done; item = setIterator.next()) ++setCount;
check(mapCount === mixedMap.size && setCount === mixedSet.size, 'live iterators through growth');
mixedMap.clear(); mixedSet.clear();
mixedMap.set(object1, 7); mixedSet.add(object1);
check(mixedMap.size === 1 && mixedMap.get(object1) === 7 && mixedSet.size === 1 && mixedSet.has(object1), 'clear and reuse');
print('prepared-collection-hash-ok');
