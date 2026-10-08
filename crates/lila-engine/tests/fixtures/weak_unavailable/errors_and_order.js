function assert(value, message) { if (!value) throw message; }
const foreign = __lilaCreateRealm().global;
const realms = [globalThis, foreign];
const marker = new foreign.TypeError('prototype marker');
const markerPrototype = foreign.TypeError.prototype;
let prototypeReads = 0;
let receiverReads = 0;
let closes = 0;
function expectTypeError(action, prototype) {
  let caught;
  try { action(); } catch (error) { caught = error; }
  assert(Object.getPrototypeOf(caught) === prototype, 'early error Realm');
}
function expectMarker(action) {
  let caught;
  try { action(); } catch (error) { caught = error; }
  assert(caught === marker && Object.getPrototypeOf(caught) === markerPrototype, 'prototype abrupt identity');
}
const NewTarget = function() {}.bind(null);
Object.defineProperty(NewTarget, 'prototype', {get() { prototypeReads++; throw marker; }});
for (const realm of realms) {
  const intrinsicPrototype = realm.TypeError.prototype;
  const constructors = [realm.WeakMap, realm.WeakSet, realm.WeakRef, realm.FinalizationRegistry];
  realm.TypeError = undefined;
  for (const C of constructors) {
    expectTypeError(function() { C({}); }, intrinsicPrototype);
  }
  for (const value of [undefined, null, true, 1, 'target', Symbol.for('registered')]) {
    expectTypeError(function() { Reflect.construct(realm.WeakRef, [value], NewTarget); }, intrinsicPrototype);
  }
  for (const value of [undefined, null, 1, {}, Symbol('callback')]) {
    expectTypeError(function() { Reflect.construct(realm.FinalizationRegistry, [value], NewTarget); }, intrinsicPrototype);
  }
  const receiver = new Proxy({}, {get() { receiverReads++; throw 'brand checks must not Get'; }});
  const methods = [
    realm.WeakMap.prototype.delete, realm.WeakMap.prototype.get,
    realm.WeakMap.prototype.getOrInsert, realm.WeakMap.prototype.getOrInsertComputed,
    realm.WeakMap.prototype.has, realm.WeakMap.prototype.set,
    realm.WeakSet.prototype.add, realm.WeakSet.prototype.delete, realm.WeakSet.prototype.has,
    realm.WeakRef.prototype.deref,
    realm.FinalizationRegistry.prototype.register, realm.FinalizationRegistry.prototype.unregister,
  ];
  for (const method of methods) {
    for (const value of [undefined, null, receiver, Object.create(realm.WeakMap.prototype)]) {
      expectTypeError(function() { method.call(value, {}, 'held', {}); }, intrinsicPrototype);
    }
  }
  expectMarker(function() { Reflect.construct(realm.WeakMap, [], NewTarget); });
  expectMarker(function() { Reflect.construct(realm.WeakSet, [], NewTarget); });
  expectMarker(function() { Reflect.construct(realm.WeakRef, [{}], NewTarget); });
  expectMarker(function() { Reflect.construct(realm.FinalizationRegistry, [function() {}], NewTarget); });
}
assert(prototypeReads === 8 && receiverReads === 0, 'validation before prototype and brand order');

const spreadTrace = [];
const iterable = {[Symbol.iterator]() {
  spreadTrace.push('iterator');
  return {next() {
    spreadTrace.push('next');
    return {get done() { spreadTrace.push('done'); return false; }, get value() { spreadTrace.push('value'); throw marker; }};
  }, return() { closes++; throw 'argument spread must not close'; }};
}};
expectMarker(function() { new WeakRef(...iterable); });
assert(spreadTrace.join(',') === 'iterator,next,done,value' && closes === 0, 'argument abrupt completion precedes construction');
const map = new Map([[1, 2]]);
const set = new Set([3]);
assert(map.getOrInsert(1, 8) === 2 && map.getOrInsert(4, 5) === 5 && set.has(3), 'strong collections remain ordinary');
print('weak-early-errors:ok');
262;
