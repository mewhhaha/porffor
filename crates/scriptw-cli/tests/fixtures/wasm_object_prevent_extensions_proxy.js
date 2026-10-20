var failures = 0;

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

var callSentinel = new TypeError("call");
var observedCallError;
try {
  Reflect.preventExtensions(new Proxy({}, {
    preventExtensions: function() {
      throw callSentinel;
    }
  }));
} catch (error) {
  observedCallError = error;
}
if (observedCallError !== callSentinel) failures |= 1;

var lookupSentinel = new TypeError("lookup");
var lookupCalls = 0;
var abruptLookupHandler = {};
Object.defineProperty(abruptLookupHandler, "preventExtensions", {
  get: function() {
    lookupCalls = lookupCalls + 1;
    throw lookupSentinel;
  }
});
var observedLookupError;
try {
  Reflect.preventExtensions(new Proxy({}, abruptLookupHandler));
} catch (error) {
  observedLookupError = error;
}
if (observedLookupError !== lookupSentinel || lookupCalls !== 1) failures |= 4096;

var falseProxy = new Proxy({}, {
  preventExtensions: function() {
    return 0;
  }
});
if (Reflect.preventExtensions(falseProxy) !== false) failures |= 2;
if (!throwsTypeError(function() { Object.preventExtensions(falseProxy); })) failures |= 4;

var extensibleTarget = {};
var invariantProxy = new Proxy(extensibleTarget, {
  preventExtensions: function() {
    return true;
  }
});
if (!throwsTypeError(function() { Reflect.preventExtensions(invariantProxy); })) failures |= 8;

var fixedTarget = {};
Object.preventExtensions(fixedTarget);
var trueCalls = 0;
var trueProxy = new Proxy(fixedTarget, {
  preventExtensions: function(target) {
    trueCalls = trueCalls + 1;
    return !Object.isExtensible(target);
  }
});
if (Reflect.preventExtensions(trueProxy) !== true) failures |= 16;
if (trueCalls !== 1) failures |= 32;

var fallbackTarget = {};
var fallbackProxy = new Proxy(fallbackTarget, {});
if (Reflect.preventExtensions(fallbackProxy) !== true) failures |= 64;
fallbackTarget.x = 1;
if (fallbackTarget.x !== undefined) failures |= 128;

var nestedTarget = {};
var nestedInner = new Proxy(nestedTarget, {});
var nestedOuterNull = new Proxy(nestedInner, {
  preventExtensions: null,
});
Object.preventExtensions(nestedOuterNull);
nestedTarget.y = 1;
if (nestedTarget.y !== undefined) failures |= 256;

var nestedFalseTarget = new Proxy({}, {
  preventExtensions: function() {
    return false;
  },
});
var nestedOuterMissing = new Proxy(nestedFalseTarget, {});
if (!throwsTypeError(function() { Object.preventExtensions(nestedOuterMissing); })) failures |= 512;

var deepTarget = {};
Object.preventExtensions(deepTarget);
var deepProxy = deepTarget;
deepProxy = new Proxy(deepProxy, {});
deepProxy = new Proxy(deepProxy, { preventExtensions: null });
deepProxy = new Proxy(deepProxy, { preventExtensions: undefined });
deepProxy = new Proxy(deepProxy, {});
deepProxy = new Proxy(deepProxy, { preventExtensions: null });
deepProxy = new Proxy(deepProxy, { preventExtensions: undefined });
if (Reflect.preventExtensions(deepProxy) !== true) failures |= 16384;

function verifyHandler(handler, target, bit) {
  var getterThis;
  var trapThis;
  var trapTarget;
  Object.defineProperty(handler, "preventExtensions", {
    configurable: true,
    get: function() {
      getterThis = this;
      return function(actualTarget) {
        trapThis = this;
        trapTarget = actualTarget;
        Object.preventExtensions(actualTarget);
        return true;
      };
    }
  });
  var proxy = new Proxy(target, handler);
  if (Reflect.preventExtensions(proxy) !== true) failures |= bit;
  if (getterThis !== handler || trapThis !== handler || trapTarget !== target) failures |= bit;
}

function functionHandler() {}
verifyHandler(functionHandler, {}, 32768);

var arrayHandler = [];
verifyHandler(arrayHandler, [], 65536);

var argumentsHandler = (function() { return arguments; })(1, 2);
verifyHandler(argumentsHandler, (function() { return arguments; })(3), 131072);

var proxyHandlerTarget = {};
var proxyHandlerGets = 0;
var proxyHandler = new Proxy(proxyHandlerTarget, {
  get: function(target, key, receiver) {
    proxyHandlerGets = proxyHandlerGets + 1;
    return Reflect.get(target, key, receiver);
  }
});
verifyHandler(proxyHandler, {}, 262144);
if (proxyHandlerGets !== 1) failures |= 262144;

var callableProxyTrapThis;
var callableProxyTrapTarget;
var callableProxyApplyCalls = 0;
var callableProxyTrap = new Proxy(function(target) {
  callableProxyTrapThis = this;
  callableProxyTrapTarget = target;
  Object.preventExtensions(target);
  return true;
}, {
  apply: function(target, thisArg, args) {
    callableProxyApplyCalls = callableProxyApplyCalls + 1;
    return Reflect.apply(target, thisArg, args);
  }
});
var callableProxyHandler = { preventExtensions: callableProxyTrap };
var callableProxyTarget = {};
if (Reflect.preventExtensions(new Proxy(callableProxyTarget, callableProxyHandler)) !== true) failures |= 524288;
if (callableProxyApplyCalls !== 1 || callableProxyTrapThis !== callableProxyHandler || callableProxyTrapTarget !== callableProxyTarget) failures |= 524288;

var nonCallableProxy = new Proxy({}, {
  preventExtensions: 1,
});
if (!throwsTypeError(function() { Reflect.preventExtensions(nonCallableProxy); })) failures |= 1024;

var revoked = Proxy.revocable({}, {});
revoked.revoke();
if (!throwsTypeError(function() { Object.preventExtensions(revoked.proxy); })) failures |= 2048;

failures === 0;
