const realm = __lilaCreateRealm();
const other = realm.global;
const ForeignProxy = other.Proxy;
const ForeignError = other.Error;
const foreignTypeErrorPrototype = other.TypeError.prototype;
const localMethods = [Reflect.preventExtensions, Object.preventExtensions];
const foreignMethods = [other.Reflect.preventExtensions, other.Object.preventExtensions];
const marker = new ForeignError('prevent-extensions-marker');
other.TypeError = function wrongTypeError() {throw 'mutable TypeError constructor';};
other.Object = {};
other.Reflect = {};
function expectTypeError(action, prototype) {
  try {action();} catch (error) {
    if (Object.getPrototypeOf(error) !== prototype) throw 'preventExtensions native error Realm';
    return;
  }
  throw 'missing preventExtensions TypeError';
}
function expectMarker(action) {
  let result = 'before';
  let finallyRuns = 0;
  try {
    try {result = action();} finally {finallyRuns++;}
    throw 'missing preventExtensions marker';
  } catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== ForeignError.prototype) throw 'preventExtensions original thrown identity';
  }
  if (result !== 'before' || finallyRuns !== 1) throw 'preventExtensions abrupt prior effects';
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
    expectTypeError(() => method(new ProxyConstructor({}, {preventExtensions: 0})), errorPrototype);
    expectTypeError(() => method(new ProxyConstructor({}, {preventExtensions() {return true;}})), errorPrototype);
    const invalidTarget = new ProxyConstructor({}, {isExtensible: 0});
    expectTypeError(() => method(new ProxyConstructor(invalidTarget, {preventExtensions() {return true;}})), errorPrototype);
    let extensibilityReads = 0;
    let falseTrapCalls = 0;
    const falseTarget = new ProxyConstructor({}, {get isExtensible() {
      extensibilityReads++;
      throw 'false preventExtensions must not observe target extensibility';
    }});
    const falseResult = new ProxyConstructor(falseTarget, {preventExtensions() {falseTrapCalls++; return false;}});
    if (index === 0) {
      if (method(falseResult) !== false) throw 'Reflect preventExtensions false result';
    } else {
      expectTypeError(() => method(falseResult), errorPrototype);
    }
    if (falseTrapCalls !== 1 || extensibilityReads !== 0) throw 'false-result Boolean or throw policy';
    expectMarker(() => method(new ProxyConstructor({}, {get preventExtensions() {throw marker;}})));
    expectMarker(() => method(new ProxyConstructor({}, {preventExtensions() {throw marker;}})));
    const base = {};
    const delegated = new ProxyConstructor(new ProxyConstructor(base, {}), {});
    const delegatedResult = method(delegated);
    if (delegatedResult !== (index === 0 ? true : delegated) || Object.isExtensible(base)) throw 'recursive preventExtensions success policy';
    let conversions = 0;
    const trapBase = {};
    const successful = new ProxyConstructor(trapBase, {preventExtensions(actual) {
      Reflect.preventExtensions(actual);
      return {valueOf() {conversions++; throw 'ToBoolean must not coerce';}};
    }});
    const result = method(successful);
    if (result !== (index === 0 ? true : successful) || Object.isExtensible(trapBase) || conversions !== 0) throw 'truthy trap success policy';
    cases++;
  }
}
if (cases !== 4) throw 'paired preventExtensions Realm census';
print('proxy-prevent-extensions-realm:ok');
262;
