const realm = __lilaCreateRealm();
const other = realm.global;
const getPrototypeOf = Object.getPrototypeOf;
const getOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;
const ownKeys = Reflect.ownKeys;
const localMethods = [Object.defineProperty, Reflect.defineProperty];
const foreignMethods = [other.Object.defineProperty, other.Reflect.defineProperty];
const localTypeErrorPrototype = TypeError.prototype;
const foreignTypeErrorPrototype = other.TypeError.prototype;
const localObjectPrototype = Object.prototype;
const foreignObjectPrototype = other.Object.prototype;
const LocalProxy = Proxy;
const ForeignProxy = other.Proxy;
const ForeignArray = other.Array;
const ForeignUint8Array = other.Uint8Array;
const ForeignString = other.String;
const marker = new other.Error('definition entry marker');
const markerPrototype = other.Error.prototype;
const localTargets = [[], function target() {}, (function (a) {return arguments;})(1), new Uint8Array(2), new String('ab')];
const foreignFunction = realm.evalScript('(function foreignTarget(a) {return a;})');
const foreignArguments = realm.evalScript('(function (a) {return arguments;})(1)');
const foreignTargets = [new ForeignArray(0), foreignFunction, foreignArguments, new ForeignUint8Array(2), new ForeignString('ab')];

// Public bindings do not own native errors or the descriptor's Object prototype.
TypeError = function wrongTypeError() {throw 'mutable TypeError';};
Object = {};
Reflect = {};
other.TypeError = function wrongForeignTypeError() {throw 'mutable foreign TypeError';};
other.Object = {};
other.Reflect = {};
other.Proxy = {};

function expectTypeError(action, prototype) {
  try {action();} catch (error) {
    if (getPrototypeOf(error) !== prototype) throw 'definition error Realm';
    return;
  }
  throw 'missing definition TypeError';
}
function expectMarker(action) {
  let returned = 'before';
  let finalized = 0;
  try {
    try {returned = action();} finally {finalized++;}
    throw 'missing definition marker';
  } catch (error) {
    if (error !== marker || getPrototypeOf(error) !== markerPrototype) throw 'definition original marker';
  }
  if (returned !== 'before' || finalized !== 1) throw 'definition abrupt assignment or finally';
}

let positiveCases = 0;
let trapCases = 0;
for (let direction = 0; direction < 2; direction++) {
  const methods = direction === 0 ? foreignMethods : localMethods;
  const targets = direction === 0 ? localTargets : foreignTargets;
  const ProxyConstructor = direction === 0 ? LocalProxy : ForeignProxy;
  const expectedTypeError = direction === 0 ? foreignTypeErrorPrototype : localTypeErrorPrototype;
  const expectedObjectPrototype = direction === 0 ? foreignObjectPrototype : localObjectPrototype;
  for (let kind = 0; kind < methods.length; kind++) {
    const method = methods[kind];
    let hooks = 0;
    const keyPoison = {[Symbol.toPrimitive]() {hooks++; throw marker;}};
    const attributesPoison = new ProxyConstructor({}, {has() {hooks++; throw marker;}});
    expectTypeError(() => method(1, keyPoison, attributesPoison), expectedTypeError);
    if (hooks !== 0) throw 'borrowed primitive target hook';

    for (const target of targets) {
      const result = method(target, 'added', {value: 9, writable: true, configurable: true});
      const descriptor = getOwnPropertyDescriptor(target, 'added');
      if (result !== (kind === 0 ? target : true) || descriptor.value !== 9 || !descriptor.writable || !descriptor.configurable) throw 'valid exotic target';
      positiveCases++;
    }

    const key = Symbol('definition key');
    const falseTrace = [];
    const falseTarget = new ProxyConstructor({}, {
      get getOwnPropertyDescriptor() {throw 'false trap descriptor';},
      get isExtensible() {throw 'false trap extensibility';}
    });
    let falseHandler;
    falseHandler = {defineProperty(target, property, descriptor) {
      if (this !== falseHandler || target !== falseTarget || property !== key) throw 'false trap roles';
      if (getPrototypeOf(descriptor) !== expectedObjectPrototype || ownKeys(descriptor).join(',') !== 'value' || descriptor.value !== 9) throw 'false partial trap descriptor';
      falseTrace.push('define');
      return false;
    }};
    const falseProxy = new ProxyConstructor(falseTarget, falseHandler);
    if (kind === 0) expectTypeError(() => method(falseProxy, key, {value: 9}), expectedTypeError);
    else if (method(falseProxy, key, {value: 9}) !== false) throw 'Reflect false definition';
    if (falseTrace.join(',') !== 'define') throw 'false trap target operation';
    trapCases++;

    const trueTrace = [];
    const base = {};
    const trueTarget = new ProxyConstructor(base, {
      getOwnPropertyDescriptor(target, property) {
        if (target !== base || property !== key) throw 'true target descriptor roles';
        trueTrace.push('descriptor');
        return undefined;
      },
      isExtensible(target) {
        if (target !== base) throw 'true extensibility target';
        trueTrace.push('extensible');
        return true;
      }
    });
    let trueHandler;
    trueHandler = {defineProperty(target, property, descriptor) {
      if (this !== trueHandler || target !== trueTarget || property !== key) throw 'true trap roles';
      if (getPrototypeOf(descriptor) !== expectedObjectPrototype || ownKeys(descriptor).join(',') !== 'value' || descriptor.value !== 9) throw 'true partial trap descriptor';
      trueTrace.push('define');
      descriptor.value = 99;
      return {valueOf() {throw 'ToBoolean must not coerce';}};
    }};
    const trueProxy = new ProxyConstructor(trueTarget, trueHandler);
    const trueResult = method(trueProxy, key, {value: 9});
    if (trueResult !== (kind === 0 ? trueProxy : true) || trueTrace.join(',') !== 'define,descriptor,extensible' || getOwnPropertyDescriptor(base, key) !== undefined) throw 'true trap result or snapshot order';
    trapCases++;

    expectMarker(() => method({}, 'x', new ProxyConstructor({}, {has() {throw marker;}})));
    expectMarker(() => method(new ProxyConstructor({}, {defineProperty() {throw marker;}}), 'x', {value: 9}));
  }
}
if (positiveCases !== 20 || trapCases !== 8) throw 'paired definition target cohort';
print('define-property-entry-realms:ok');
262;
