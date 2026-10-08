function expectTypeError(action) {
  try { action(); } catch (error) {
    if (Object.getPrototypeOf(error) !== TypeError.prototype) throw 'native TypeError';
    return;
  }
  throw 'missing TypeError';
}
function ownKeys(result, target) {
  return new Proxy(target, {ownKeys() { return result; }});
}
let primitiveLengthReads = 0;
Object.defineProperty(Number.prototype, 'length', {configurable: true, get() {
  primitiveLengthReads++;
  throw 'nonobject must not Get length';
}});
expectTypeError(() => Reflect.ownKeys(ownKeys(7, {})));
expectTypeError(() => Reflect.ownKeys(ownKeys(null, {})));
expectTypeError(() => Reflect.ownKeys(ownKeys(undefined, {})));
if (primitiveLengthReads !== 0) throw 'primitive boxing';
delete Number.prototype.length;

const invalidTrace = [];
let conversions = 0;
const invalid = {length: 2, get 0() {
  invalidTrace.push('0');
  return {[Symbol.toPrimitive]() { conversions++; throw 'key coercion'; }};
}, get 1() {
  invalidTrace.push('1');
  throw 'invalid key must stop later Get';
}};
expectTypeError(() => Reflect.ownKeys(ownKeys(invalid, {})));
if (conversions !== 0 || invalidTrace.join(',') !== '0') throw 'immediate key type validation';

const duplicateTrace = [];
const duplicate = {get length() { duplicateTrace.push('length'); return 3; },
  get 0() { duplicateTrace.push('0'); return 'duplicate'; },
  get 1() { duplicateTrace.push('1'); return ['du', 'plicate'].join(''); },
  get 2() { duplicateTrace.push('2'); return 'later'; }};
expectTypeError(() => Reflect.ownKeys(ownKeys(duplicate, {})));
if (duplicateTrace.join(',') !== 'length,0,1,2') throw 'complete acquisition before String duplicate error';
const firstSymbol = Symbol('same-description');
const secondSymbol = Symbol('same-description');
const distinct = Reflect.ownKeys(ownKeys([firstSymbol, secondSymbol], {}));
if (distinct.length !== 2 || distinct[0] !== firstSymbol || distinct[1] !== secondSymbol) throw 'Symbol identity distinction';
expectTypeError(() => Reflect.ownKeys(ownKeys([firstSymbol, firstSymbol], {})));

const fixedSymbol = Symbol('fixed');
const target = {loose: 1};
Object.defineProperty(target, 'fixed', {value: 2, configurable: false});
Object.defineProperty(target, fixedSymbol, {value: 3, configurable: false});
const valid = Reflect.ownKeys(ownKeys([fixedSymbol, 'extra', 'fixed'], target));
if (valid.length !== 3 || valid[0] !== fixedSymbol || valid[1] !== 'extra' || valid[2] !== 'fixed') throw 'extensible target invariants';
expectTypeError(() => Reflect.ownKeys(ownKeys([fixedSymbol], target)));
expectTypeError(() => Object.getOwnPropertyNames(ownKeys(['fixed'], target)));
const omitted = Reflect.ownKeys(ownKeys([], {configurable: 1}));
if (omitted.length !== 0) throw 'configurable omission on extensible target';
const sealed = {name: 1};
sealed[firstSymbol] = 2;
Object.preventExtensions(sealed);
const exact = Reflect.ownKeys(ownKeys([firstSymbol, 'name'], sealed));
if (exact.length !== 2 || exact[0] !== firstSymbol || exact[1] !== 'name') throw 'nonextensible exact set';
expectTypeError(() => Reflect.ownKeys(ownKeys(['name'], sealed)));
expectTypeError(() => Reflect.ownKeys(ownKeys([firstSymbol, 'name', 'extra'], sealed)));
print('proxy-own-keys-validation:ok');
262;
