const other = __lilaCreateRealm().global;
const ForeignError = other.Error;
const ForeignTypeError = other.TypeError;
const foreignTypeErrorPrototype = ForeignTypeError.prototype;
const foreignArrayPrototype = other.Array.prototype;
const foreignOwnKeys = other.Reflect.ownKeys;
const foreignNames = other.Object.getOwnPropertyNames;
const foreignSymbols = other.Object.getOwnPropertySymbols;
const marker = new ForeignError('foreign-ownKeys-marker');
const symbol = Symbol('foreign-result');
const order = [];
const duplicateThenThrow = {get length() { order.push('length'); return 3; },
  get 0() { order.push('0'); return 'duplicate'; },
  get 1() { order.push('1'); return 'duplicate'; },
  get 2() { order.push('2'); throw marker; }};
function expectMarker(action) {
  try { action(); } catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== ForeignError.prototype) throw 'arbitrary foreign abrupt identity';
    return;
  }
  throw 'missing marker';
}
expectMarker(() => Reflect.ownKeys(new Proxy({}, {ownKeys() { return duplicateThenThrow; }})));
if (order.join(',') !== 'length,0,1,2') throw 'later abrupt beats earlier duplicate';
let indexes = 0;
const throwingLength = {get length() { throw marker; }, get 0() { indexes++; return 'key'; }};
expectMarker(() => Object.getOwnPropertyNames(new Proxy({}, {ownKeys() { return throwingLength; }})));
if (indexes !== 0) throw 'length abrupt precedes indices';
const conversionLength = {length: {[Symbol.toPrimitive](hint) {
  if (hint !== 'number') throw 'length hint';
  throw marker;
}}, get 0() { indexes++; return 'key'; }};
expectMarker(() => Object.getOwnPropertySymbols(new Proxy({}, {ownKeys() { return conversionLength; }})));
if (indexes !== 0) throw 'ToLength abrupt precedes indices';

other.Array = function wrongArray() { throw 'mutable Array constructor'; };
other.TypeError = function wrongTypeError() { throw 'mutable TypeError constructor'; };
other.Object = {};
other.Reflect = {};
const result = {length: 2, 0: 'name', 1: symbol};
let traps = 0;
const proxy = new Proxy({}, {ownKeys() { traps++; return result; }});
const foreignKeys = foreignOwnKeys(proxy);
const names = foreignNames(proxy);
const symbols = foreignSymbols(proxy);
for (const array of [foreignKeys, names, symbols]) {
  if (Object.getPrototypeOf(array) !== foreignArrayPrototype || Object.getPrototypeOf(array) === Array.prototype) throw 'defining-Realm array prototype';
  if (array === result) throw 'private result snapshot';
}
if (foreignKeys.length !== 2 || foreignKeys[0] !== 'name' || foreignKeys[1] !== symbol) throw 'foreign full keys';
if (names.length !== 1 || names[0] !== 'name' || symbols.length !== 1 || symbols[0] !== symbol || traps !== 3) throw 'foreign filtered publications';
function expectForeignTypeError(action) {
  try { action(); } catch (error) {
    if (Object.getPrototypeOf(error) !== foreignTypeErrorPrototype || error instanceof TypeError) throw 'defining-Realm TypeError';
    return;
  }
  throw 'missing foreign TypeError';
}
expectForeignTypeError(() => foreignOwnKeys(new Proxy({}, {ownKeys() { return ['same', 'same']; }})));
expectForeignTypeError(() => foreignNames(new Proxy({}, {ownKeys() { return {length: 1, 0: 7}; }})));
expectForeignTypeError(() => foreignSymbols(new Proxy({}, {ownKeys() { return null; }})));
const fixed = {};
Object.defineProperty(fixed, 'fixed', {value: 1, configurable: false});
expectForeignTypeError(() => foreignOwnKeys(new Proxy(fixed, {ownKeys() { return []; }})));
expectMarker(() => foreignOwnKeys(new Proxy({}, {ownKeys() { return duplicateThenThrow; }})));
if (order.join(',') !== 'length,0,1,2,length,0,1,2') throw 'borrowed abrupt acquisition';
print('proxy-own-keys-abrupt:ok');
262;
