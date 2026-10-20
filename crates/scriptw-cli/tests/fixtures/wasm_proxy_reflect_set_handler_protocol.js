function assert(condition, message) {
  if (!condition) throw message;
}

function capture(operation) {
  try {
    operation();
  } catch (error) {
    return error;
  }
  return undefined;
}

var expectedHandler;
var expectedTarget;
var expectedKey;
var expectedValue;
var expectedReceiver;
var currentScenario;
var handlerLookups = 0;
var handlerCalls = 0;
var trapCallSentinel = {};

function ordinarySetTrap(target, key, value, receiver) {
  handlerCalls += 1;
  assert(arguments.length === 4, currentScenario + " trap arity");
  assert(this === expectedHandler, currentScenario + " trap this");
  assert(target === expectedTarget, currentScenario + " target");
  assert(key === expectedKey, currentScenario + " key");
  assert(value === expectedValue, currentScenario + " value");
  assert(receiver === expectedReceiver, currentScenario + " receiver");
  if (currentScenario === "Function handler") {
    assert(typeof target === "function", currentScenario + " target tag");
    assert(Array.isArray(receiver), currentScenario + " receiver tag");
  }
  return true;
}

function ordinarySetTrapGetter() {
  handlerLookups += 1;
  assert(this === expectedHandler, currentScenario + " getter this");
  return ordinarySetTrap;
}

function exerciseHandlerBrand(handler, target, key, value, receiver, label) {
  var lookupCountBefore = handlerLookups;
  var callCountBefore = handlerCalls;

  currentScenario = label;
  expectedHandler = handler;
  expectedTarget = target;
  expectedKey = key;
  expectedValue = value;
  expectedReceiver = receiver;

  assert(Reflect.set(new Proxy(target, handler), key, value, receiver), label + " result");
  assert(handlerLookups === lookupCountBefore + 1, label + " lookup count");
  assert(handlerCalls === callCountBefore + 1, label + " call count");
  assert(target[key] === undefined, label + " target mutation");
  assert(receiver[key] === undefined, label + " receiver mutation");
}

function functionHandler() {}
Object.defineProperty(functionHandler, "set", {
  configurable: true,
  get: ordinarySetTrapGetter,
});
function tagSensitiveTarget() {}
var tagSensitiveReceiver = [];
exerciseHandlerBrand(
  functionHandler,
  tagSensitiveTarget,
  "function-key",
  1,
  tagSensitiveReceiver,
  "Function handler"
);

var arrayHandler = [];
Object.defineProperty(arrayHandler, "set", {
  configurable: true,
  get: ordinarySetTrapGetter,
});
exerciseHandlerBrand(arrayHandler, {}, "array-key", 2, {}, "Array handler");

var argumentsHandler = (function () { return arguments; })(1, 2, 3);
Object.defineProperty(argumentsHandler, "set", {
  configurable: true,
  get: ordinarySetTrapGetter,
});
exerciseHandlerBrand(
  argumentsHandler,
  {},
  "arguments-key",
  3,
  {},
  "arguments handler"
);

var callableTrapTarget = function () {
  throw "callable Proxy target invoked directly";
};
var callableTrapHandler = {
  apply: function (target, thisArgument, argumentsList) {
    handlerCalls += 1;
    assert(this === callableTrapHandler, currentScenario + " apply this");
    assert(target === callableTrapTarget, currentScenario + " callable target");
    assert(thisArgument === expectedHandler, currentScenario + " trap this");
    assert(argumentsList.length === 4, currentScenario + " arguments length");
    assert(argumentsList[0] === expectedTarget, currentScenario + " target");
    assert(argumentsList[1] === expectedKey, currentScenario + " key");
    assert(argumentsList[2] === expectedValue, currentScenario + " value");
    assert(argumentsList[3] === expectedReceiver, currentScenario + " receiver");
    if (argumentsList[2] === trapCallSentinel) throw trapCallSentinel;
    return true;
  },
};
var callableProxyTrap = new Proxy(callableTrapTarget, callableTrapHandler);
var proxyHandlerTarget = {};
var proxyHandler;
var proxyLookupHandler = {
  get: function (target, key, receiver) {
    handlerLookups += 1;
    assert(this === proxyLookupHandler, currentScenario + " lookup this");
    assert(target === proxyHandlerTarget, currentScenario + " lookup target");
    assert(key === "set", currentScenario + " lookup key");
    assert(receiver === proxyHandler, currentScenario + " lookup receiver");
    return callableProxyTrap;
  },
};
proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);
var symbolKey = Symbol("proxy-reflect-set-key");
exerciseHandlerBrand(proxyHandler, {}, symbolKey, 4, {}, "Proxy handler");

var trapCallTarget = {};
Object.defineProperty(trapCallTarget, "fixed", {
  value: 1,
  writable: false,
  configurable: false,
});
currentScenario = "callable Proxy trap abrupt call";
expectedHandler = proxyHandler;
expectedTarget = trapCallTarget;
expectedKey = "fixed";
expectedValue = trapCallSentinel;
expectedReceiver = {};
var trapCallError = capture(function () {
  Reflect.set(
    new Proxy(trapCallTarget, proxyHandler),
    expectedKey,
    expectedValue,
    expectedReceiver
  );
});
assert(trapCallError === trapCallSentinel, "abrupt trap call sentinel");
assert(trapCallTarget.fixed === 1, "abrupt trap call mutated target");

var lookupSentinel = {};
var abruptHandler = new Proxy({}, {
  get: function (target, key) {
    assert(key === "set", "abrupt lookup key");
    throw lookupSentinel;
  },
});
var abruptTarget = {};
var lookupError = capture(function () {
  Reflect.set(new Proxy(abruptTarget, abruptHandler), "abrupt", 5, {});
});
assert(lookupError === lookupSentinel, "abrupt lookup sentinel");
assert(abruptTarget.abrupt === undefined, "abrupt lookup mutated target");

var nestedCalls = 0;
var nestedRawTarget = {};
var nestedHandler = {
  set: function (target, key, value, receiver) {
    nestedCalls += 1;
    assert(this === nestedHandler, "nested trap this");
    assert(target === nestedRawTarget, "nested trap target");
    assert(key === "null-fallback" || key === symbolKey, "nested trap key");
    assert(value === 6 || value === 7, "nested trap value");
    assert(receiver === expectedReceiver, "nested trap receiver");
    return true;
  },
};
var nestedTarget = new Proxy(nestedRawTarget, nestedHandler);
expectedReceiver = {};
assert(
  Reflect.set(
    new Proxy(nestedTarget, { set: null }),
    "null-fallback",
    6,
    expectedReceiver
  ),
  "null fallback result"
);
expectedReceiver = {};
assert(
  Reflect.set(
    new Proxy(nestedTarget, { set: undefined }),
    symbolKey,
    7,
    expectedReceiver
  ),
  "undefined fallback result"
);
assert(nestedCalls === 2, "nested fallback call count");

true;
