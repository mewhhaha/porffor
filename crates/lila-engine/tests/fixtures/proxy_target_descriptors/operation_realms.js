const realm = __lilaCreateRealm();
const other = realm.global;
const ForeignProxy = other.Proxy;
const ForeignError = other.Error;
const foreignTypeErrorPrototype = other.TypeError.prototype;
const foreignErrorPrototype = ForeignError.prototype;
const localTypeErrorPrototype = TypeError.prototype;
const localMethods = [Reflect.get, Reflect.set, Reflect.has, Reflect.deleteProperty];
const foreignMethods = [other.Reflect.get, other.Reflect.set, other.Reflect.has, other.Reflect.deleteProperty];
const receiver = {};
const marker = new ForeignError('target-descriptor-foreign-marker');
other.TypeError = function wrongTypeError() {throw 'mutable TypeError constructor';};
other.Object = {};
other.Reflect = {};
other.Proxy = {};
function invoke(method, kind, proxy) {
  if (kind === 0) return method(proxy, 'x', receiver);
  if (kind === 1) return method(proxy, 'x', 9, receiver);
  return method(proxy, 'x');
}
function handler(kind) {
  if (kind === 0) return {get() {return 9;}};
  if (kind === 1) return {set() {return true;}};
  if (kind === 2) return {has() {return false;}};
  return {deleteProperty() {return true;}};
}
function expectTypeError(action, prototype) {
  try {action();} catch (error) {
    if (Object.getPrototypeOf(error) !== prototype) throw 'target descriptor error operation Realm';
    return;
  }
  throw 'missing target descriptor operation TypeError';
}
function expectMarker(action) {
  let result = 'before';
  let finallyRuns = 0;
  try {
    try {result = action();} finally {finallyRuns++;}
    throw 'missing original descriptor marker';
  } catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignErrorPrototype) throw 'original descriptor marker identity';
  }
  if (result !== 'before' || finallyRuns !== 1) throw 'original descriptor marker abrupt effects';
}
let cases = 0;
for (let direction = 0; direction < 2; direction++) {
  const methods = direction === 0 ? foreignMethods : localMethods;
  const ProxyConstructor = direction === 0 ? Proxy : ForeignProxy;
  const expectedPrototype = direction === 0 ? foreignTypeErrorPrototype : localTypeErrorPrototype;
  for (let kind = 0; kind < 4; kind++) {
    const method = methods[kind];
    const operationHandler = handler(kind);
    const revoked = ProxyConstructor.revocable({}, {});
    revoked.revoke();
    expectTypeError(() => invoke(method, kind, revoked.proxy), expectedPrototype);
    expectTypeError(() => invoke(method, kind, new ProxyConstructor(revoked.proxy, operationHandler)), expectedPrototype);
    const invalid = new ProxyConstructor({}, {getOwnPropertyDescriptor() {return 7;}});
    expectTypeError(() => invoke(method, kind, new ProxyConstructor(invalid, operationHandler)), expectedPrototype);
    const nonCallable = new ProxyConstructor({}, {getOwnPropertyDescriptor: 0});
    expectTypeError(() => invoke(method, kind, new ProxyConstructor(nonCallable, operationHandler)), expectedPrototype);
    const invalidGetter = new ProxyConstructor({}, {getOwnPropertyDescriptor() {
      return {get: 0, configurable: true};
    }});
    expectTypeError(() => invoke(method, kind, new ProxyConstructor(invalidGetter, operationHandler)), expectedPrototype);
    const fixed = {};
    Object.defineProperty(fixed, 'x', {value: 1, writable: false, configurable: false});
    const honestFixed = new ProxyConstructor(fixed, {});
    expectTypeError(() => invoke(method, kind, new ProxyConstructor(honestFixed, operationHandler)), expectedPrototype);
    const hiddenFixed = new ProxyConstructor(fixed, {getOwnPropertyDescriptor() {return undefined;}});
    expectTypeError(() => invoke(method, kind, new ProxyConstructor(hiddenFixed, operationHandler)), expectedPrototype);
    const throwingLookup = new ProxyConstructor(fixed, {get getOwnPropertyDescriptor() {throw marker;}});
    expectMarker(() => invoke(method, kind, new ProxyConstructor(throwingLookup, operationHandler)));
    const throwingCall = new ProxyConstructor(fixed, {getOwnPropertyDescriptor() {throw marker;}});
    expectMarker(() => invoke(method, kind, new ProxyConstructor(throwingCall, operationHandler)));
    if (kind >= 2) {
      const inconsistentExtensibility = new ProxyConstructor({x: 1}, {
        isExtensible() {return false;}
      });
      expectTypeError(() => invoke(method, kind, new ProxyConstructor(inconsistentExtensibility, operationHandler)), expectedPrototype);
      const nonExtensible = {x: 1};
      Object.preventExtensions(nonExtensible);
      expectTypeError(() => invoke(method, kind, new ProxyConstructor(new ProxyConstructor(nonExtensible, {}), operationHandler)), expectedPrototype);
    }
    const base = {x: 1};
    const normalTarget = new ProxyConstructor(base, {getOwnPropertyDescriptor(target, key) {
      return Reflect.getOwnPropertyDescriptor(target, key);
    }});
    const expectedResult = kind === 0 ? 9 : kind === 2 ? false : true;
    if (invoke(method, kind, new ProxyConstructor(normalTarget, operationHandler)) !== expectedResult || base.x !== 1) throw 'paired Realm normal Boolean or value';
    cases++;
  }
}
// Source operators select their own current function Realm even for foreign targets.
function sourceGet(proxy) {return proxy.x;}
function sourceSet(proxy) {return proxy.x = 9;}
function sourceHas(proxy) {return 'x' in proxy;}
function sourceDelete(proxy) {return delete proxy.x;}
const sourceOperations = [sourceGet, sourceSet, sourceHas, sourceDelete];
for (let kind = 0; kind < sourceOperations.length; kind++) {
  const invalid = new ForeignProxy({}, {getOwnPropertyDescriptor() {return 7;}});
  const outer = new ForeignProxy(invalid, handler(kind));
  expectTypeError(() => sourceOperations[kind](outer), localTypeErrorPrototype);
}
if (cases !== 8) throw 'paired four-operation Realm census';
print('proxy-target-descriptor-realms:ok');
262;
