function expectTypeError(action, prototype) {
  try {action();} catch (error) {
    if (Object.getPrototypeOf(error) !== prototype) throw 'outer builtin TypeError Realm';
    return;
  }
  throw 'missing invariant TypeError';
}
function withKeys(target, keys) {
  return new Proxy(target, {ownKeys() {return keys;}});
}
const boxed = new String('ab');
expectTypeError(() => Reflect.ownKeys(withKeys(boxed, ['length', '1'])), TypeError.prototype);
expectTypeError(() => Object.getOwnPropertyNames(withKeys(boxed, ['0', '1'])), TypeError.prototype);
const boxedKeys = Reflect.ownKeys(withKeys(boxed, ['1', 'length', '0', 'extra']));
if (boxedKeys.join(',') !== '1,length,0,extra') throw 'boxed String virtual own descriptors';
function ordinary() {'use strict';}
expectTypeError(() => Reflect.ownKeys(withKeys(ordinary, ['extra'])), TypeError.prototype);
const functionKeys = Reflect.ownKeys(withKeys(ordinary, ['prototype']));
if (functionKeys.join(',') !== 'prototype') throw 'function prototype own descriptor';
const array = [1];
expectTypeError(() => Reflect.ownKeys(withKeys(array, ['0'])), TypeError.prototype);
const arrayKeys = Reflect.ownKeys(withKeys(array, ['length']));
if (arrayKeys.join(',') !== 'length') throw 'Array nonconfigurable length';
const symbol = Symbol('typed-fixed');
const typed = new Uint8Array([4, 5]);
Object.defineProperty(typed, symbol, {value: 6, configurable: false});
Object.preventExtensions(typed);
const typedKeys = Reflect.ownKeys(withKeys(typed, [symbol, '1', '0']));
if (typedKeys.length !== 3 || typedKeys[0] !== symbol || typedKeys[1] !== '1' || typedKeys[2] !== '0') throw 'typed target complete own keys';
expectTypeError(() => Reflect.ownKeys(withKeys(typed, [symbol, '0'])), TypeError.prototype);
expectTypeError(() => Object.getOwnPropertySymbols(withKeys(typed, [symbol, '0', '1', 'extra'])), TypeError.prototype);

const other = __lilaCreateRealm().global;
const foreignTypeErrorPrototype = other.TypeError.prototype;
const foreignArrayPrototype = other.Array.prototype;
const ForeignError = other.Error;
const ForeignProxy = other.Proxy;
const localMethods = [Reflect.ownKeys, Object.getOwnPropertyNames, Object.getOwnPropertySymbols];
const foreignMethods = [other.Reflect.ownKeys, other.Object.getOwnPropertyNames, other.Object.getOwnPropertySymbols];
const marker = new ForeignError('recursive-target-marker');
other.TypeError = function wrongTypeError() {throw 'mutable TypeError constructor';};
other.Array = function wrongArray() {throw 'mutable Array constructor';};
other.Reflect = {};
other.Object = {};
let cases = 0;
for (let direction = 0; direction < 2; direction++) {
  const methods = direction === 0 ? foreignMethods : localMethods;
  const ProxyConstructor = direction === 0 ? Proxy : ForeignProxy;
  const errorPrototype = direction === 0 ? foreignTypeErrorPrototype : TypeError.prototype;
  const arrayPrototype = direction === 0 ? foreignArrayPrototype : Array.prototype;
  for (let index = 0; index < methods.length; index++) {
    const method = methods[index];
    const revoked = ProxyConstructor.revocable({}, {});
    revoked.revoke();
    let resultGets = 0;
    const list = {length: 1, get 0() {resultGets++; return 'key';}};
    expectTypeError(() => method(new ProxyConstructor(revoked.proxy, {ownKeys() {return list;}})), errorPrototype);
    if (resultGets !== 1) throw 'target revocation follows completed trap list';
    expectTypeError(() => method(new ProxyConstructor(new ProxyConstructor({}, {isExtensible: 0}), {ownKeys() {return [];}})), errorPrototype);
    expectTypeError(() => method(new ProxyConstructor(new ProxyConstructor({}, {isExtensible() {return false;}}), {ownKeys() {return [];}})), errorPrototype);
    expectTypeError(() => method(new ProxyConstructor(new ProxyConstructor({}, {ownKeys: 0}), {ownKeys() {return [];}})), errorPrototype);
    const descriptorTarget = new ProxyConstructor({key: 1}, {getOwnPropertyDescriptor: 0});
    expectTypeError(() => method(new ProxyConstructor(descriptorTarget, {ownKeys() {return ['key'];}})), errorPrototype);
    const throwingTarget = new ProxyConstructor({}, {get isExtensible() {throw marker;}});
    let finallyRuns = 0;
    let assignment = 'before';
    try {
      try {assignment = method(new ProxyConstructor(throwingTarget, {ownKeys() {return [];}}));}
      finally {finallyRuns++;}
      throw 'missing recursive marker';
    } catch (error) {
      if (error !== marker || Object.getPrototypeOf(error) !== ForeignError.prototype) throw 'recursive thrown identity';
    }
    if (assignment !== 'before' || finallyRuns !== 1) throw 'recursive abrupt prior effects';
    const successful = method(new ProxyConstructor(new ProxyConstructor({}, {}), {ownKeys() {return ['extra'];}}));
    if (Object.getPrototypeOf(successful) !== arrayPrototype || successful.length !== (index === 2 ? 0 : 1)) throw 'recursive target success Realm';
    cases++;
  }
}
if (cases !== 6) throw 'paired builtin Realm census';
print('proxy-own-keys-target-realms:ok');
262;
