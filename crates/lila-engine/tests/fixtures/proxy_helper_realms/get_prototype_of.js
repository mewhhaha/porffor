const realm = __lilaCreateRealm();
const other = realm.global;
const ForeignProxy = other.Proxy;
const ForeignError = other.Error;
const foreignTypeErrorPrototype = other.TypeError.prototype;
const localMethods = [Reflect.getPrototypeOf, Object.getPrototypeOf];
const foreignMethods = [other.Reflect.getPrototypeOf, other.Object.getPrototypeOf];
const marker = new ForeignError('get-prototype-marker');
other.TypeError = function wrongTypeError() {throw 'mutable TypeError constructor';};
other.Object = {};
other.Reflect = {};
function expectTypeError(action, prototype) {
  try {action();} catch (error) {
    if (Object.getPrototypeOf(error) !== prototype) throw 'getPrototypeOf native error Realm';
    return;
  }
  throw 'missing getPrototypeOf TypeError';
}
function expectMarker(action) {
  let result = 'before';
  let finallyRuns = 0;
  try {
    try {result = action();} finally {finallyRuns++;}
    throw 'missing getPrototypeOf marker';
  } catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== ForeignError.prototype) throw 'getPrototypeOf original thrown identity';
  }
  if (result !== 'before' || finallyRuns !== 1) throw 'getPrototypeOf abrupt prior effects';
}
let cases = 0;
for (let direction = 0; direction < 2; direction++) {
  const methods = direction === 0 ? foreignMethods : localMethods;
  const ProxyConstructor = direction === 0 ? Proxy : ForeignProxy;
  const errorPrototype = direction === 0 ? foreignTypeErrorPrototype : TypeError.prototype;
  for (let index = 0; index < methods.length; index++) {
    const method = methods[index];
    const revoked = ProxyConstructor.revocable({}, {});
    revoked.revoke();
    expectTypeError(() => method(revoked.proxy), errorPrototype);
    expectTypeError(() => method(new ProxyConstructor(revoked.proxy, {})), errorPrototype);
    expectTypeError(() => method(new ProxyConstructor({}, {getPrototypeOf: 0})), errorPrototype);
    const trace = [];
    const invalidTarget = new ProxyConstructor({}, {isExtensible() {
      trace.push('extensible');
      throw 'invalid result must precede target extensibility';
    }});
    const invalid = new ProxyConstructor(invalidTarget, {get getPrototypeOf() {
      trace.push('get');
      return function() {trace.push('call'); return 7;};
    }});
    expectTypeError(() => method(invalid), errorPrototype);
    if (trace.join(',') !== 'get,call') throw 'getPrototypeOf result type precedes invariant operations';
    const prototype = {};
    const wrongPrototype = {};
    const fixed = Object.create(prototype);
    Object.preventExtensions(fixed);
    const wrappedFixed = new ProxyConstructor(fixed, {});
    expectTypeError(() => method(new ProxyConstructor(wrappedFixed, {getPrototypeOf() {return wrongPrototype;}})), errorPrototype);
    const inconsistentTarget = new ProxyConstructor({}, {isExtensible() {return false;}});
    expectTypeError(() => method(new ProxyConstructor(inconsistentTarget, {getPrototypeOf() {return prototype;}})), errorPrototype);
    expectMarker(() => method(new ProxyConstructor({}, {get getPrototypeOf() {throw marker;}})));
    expectMarker(() => method(new ProxyConstructor({}, {getPrototypeOf() {throw marker;}})));
    const delegated = new ProxyConstructor(new ProxyConstructor(Object.create(prototype), {}), {});
    if (method(delegated) !== prototype) throw 'recursive getPrototypeOf result identity';
    const nullPrototype = new ProxyConstructor({}, {getPrototypeOf() {return null;}});
    if (method(nullPrototype) !== null) throw 'normal null getPrototypeOf result';
    cases++;
  }
}
if (cases !== 4) throw 'paired getPrototypeOf Realm census';
print('proxy-get-prototype-realm:ok');
262;
