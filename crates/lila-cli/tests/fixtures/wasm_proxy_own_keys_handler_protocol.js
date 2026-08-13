var failures = 0;

function check(condition) {
  if (!condition) failures += 1;
}

function capture(fn) {
  try {
    fn();
  } catch (error) {
    return error;
  }
  return undefined;
}

var functionTarget = {};
Object.defineProperty(functionTarget, "functionKey", {
  value: 1,
  enumerable: true,
  configurable: true
});
function functionHandler() {}
var functionGetterThis;
var functionTrapThis;
var functionTrapTarget;
var functionTrapTargetFunction = function(target) {
  functionTrapThis = this;
  functionTrapTarget = target;
  return ["functionKey"];
};
var callableProxyTrap = new Proxy(functionTrapTargetFunction, {});
Object.defineProperty(functionHandler, "ownKeys", {
  configurable: true,
  get: function() {
    functionGetterThis = this;
    return callableProxyTrap;
  }
});
var functionKeys = Reflect.ownKeys(new Proxy(functionTarget, functionHandler));
check(functionGetterThis === functionHandler);
check(functionTrapThis === functionHandler);
check(functionTrapTarget === functionTarget);
check(functionKeys.length === 1 && functionKeys[0] === "functionKey");

var arrayTarget = { arrayKey: 2 };
var arrayHandler = [];
var arrayGetterThis;
var arrayTrapThis;
var arrayTrapTarget;
Object.defineProperty(arrayHandler, "ownKeys", {
  configurable: true,
  get: function() {
    arrayGetterThis = this;
    return function(target) {
      arrayTrapThis = this;
      arrayTrapTarget = target;
      return ["arrayKey"];
    };
  }
});
var arrayKeys = Object.getOwnPropertyNames(new Proxy(arrayTarget, arrayHandler));
check(arrayGetterThis === arrayHandler);
check(arrayTrapThis === arrayHandler);
check(arrayTrapTarget === arrayTarget);
check(arrayKeys.length === 1 && arrayKeys[0] === "arrayKey");

function makeArgumentsHandler() {
  return arguments;
}
var argumentsTarget = { argumentsKey: 3 };
var argumentsHandler = makeArgumentsHandler(1);
var argumentsGetterThis;
var argumentsTrapThis;
var argumentsTrapTarget;
Object.defineProperty(argumentsHandler, "ownKeys", {
  configurable: true,
  get: function() {
    argumentsGetterThis = this;
    return function(target) {
      argumentsTrapThis = this;
      argumentsTrapTarget = target;
      return ["argumentsKey"];
    };
  }
});
var argumentsKeys = Object.keys(new Proxy(argumentsTarget, argumentsHandler));
check(argumentsGetterThis === argumentsHandler);
check(argumentsTrapThis === argumentsHandler);
check(argumentsTrapTarget === argumentsTarget);
check(argumentsKeys.length === 1 && argumentsKeys[0] === "argumentsKey");

var symbolKey = Symbol("proxy-handler-key");
var proxyTarget = {};
var proxyHandlerTarget = {};
var proxyLookupHandler = {};
var proxyLookupThis;
var proxyLookupTarget;
var proxyLookupKey;
var proxyLookupReceiver;
var proxyTrapThis;
var proxyTrapTarget;
var proxyHandler;
proxyLookupHandler.get = function(target, key, receiver) {
  proxyLookupThis = this;
  proxyLookupTarget = target;
  proxyLookupKey = key;
  proxyLookupReceiver = receiver;
  return function(outerTarget) {
    proxyTrapThis = this;
    proxyTrapTarget = outerTarget;
    return [symbolKey];
  };
};
proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);
var symbolKeys = Object.getOwnPropertySymbols(new Proxy(proxyTarget, proxyHandler));
check(proxyLookupThis === proxyLookupHandler);
check(proxyLookupTarget === proxyHandlerTarget);
check(proxyLookupKey === "ownKeys");
check(proxyLookupReceiver === proxyHandler);
check(proxyTrapThis === proxyHandler);
check(proxyTrapTarget === proxyTarget);
check(symbolKeys.length === 1 && symbolKeys[0] === symbolKey);

var lookupSentinel = {};
var abruptHandler = {};
Object.defineProperty(abruptHandler, "ownKeys", {
  configurable: true,
  get: function() {
    throw lookupSentinel;
  }
});
var lookupError = capture(function() {
  Reflect.ownKeys(new Proxy({}, abruptHandler));
});
check(lookupError === lookupSentinel);

var nestedCalls = 0;
var nestedTarget = new Proxy({ nestedKey: 4 }, {
  ownKeys: function() {
    nestedCalls += 1;
    return ["nestedKey"];
  }
});
var nestedKeys = Object.keys(new Proxy(nestedTarget, { ownKeys: null }));
check(nestedCalls === 1);
check(nestedKeys.length === 1 && nestedKeys[0] === "nestedKey");

var other = __lilaCreateRealm().global;
var nonCallableError = capture(function() {
  other.Reflect.ownKeys(new Proxy({}, { ownKeys: {} }));
});
check(nonCallableError !== undefined);
check(Object.getPrototypeOf(nonCallableError) === other.TypeError.prototype);
check(nonCallableError instanceof other.TypeError);
check(!(nonCallableError instanceof TypeError));

var revocable = Proxy.revocable({}, {});
revocable.revoke();
var revokedError = capture(function() {
  other.Object.keys(revocable.proxy);
});
check(revokedError !== undefined);
check(Object.getPrototypeOf(revokedError) === other.TypeError.prototype);
check(revokedError instanceof other.TypeError);
check(!(revokedError instanceof TypeError));

failures === 0;
