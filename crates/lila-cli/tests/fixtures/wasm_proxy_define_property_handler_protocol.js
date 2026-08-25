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
var expectedInputDescriptor;
var currentScenario;
var handlerLookups = 0;
var handlerCalls = 0;

function assertTrapArguments(target, key, descriptor) {
  assert(target === expectedTarget, currentScenario + " target");
  assert(key === expectedKey, currentScenario + " key");
  assert(
    descriptor !== expectedInputDescriptor,
    currentScenario + " descriptor identity"
  );
  assert(
    Object.getPrototypeOf(descriptor) === Object.prototype,
    currentScenario + " descriptor prototype"
  );
  assert(descriptor.value === expectedValue, currentScenario + " descriptor value");
  assert(descriptor.writable === true, currentScenario + " descriptor writable");
  assert(descriptor.enumerable === false, currentScenario + " descriptor enumerable");
  assert(descriptor.configurable === true, currentScenario + " descriptor configurable");
  assert(
    Object.prototype.hasOwnProperty.call(descriptor, "value"),
    currentScenario + " value presence"
  );
  assert(
    Object.prototype.hasOwnProperty.call(descriptor, "writable"),
    currentScenario + " writable presence"
  );
  assert(
    Object.prototype.hasOwnProperty.call(descriptor, "enumerable"),
    currentScenario + " enumerable presence"
  );
  assert(
    Object.prototype.hasOwnProperty.call(descriptor, "configurable"),
    currentScenario + " configurable presence"
  );
  assert(
    !Object.prototype.hasOwnProperty.call(descriptor, "get"),
    currentScenario + " unexpected getter"
  );
  assert(
    !Object.prototype.hasOwnProperty.call(descriptor, "set"),
    currentScenario + " unexpected setter"
  );
}

function ordinaryDefineTrap(target, key, descriptor) {
  handlerCalls += 1;
  assert(arguments.length === 3, currentScenario + " trap arity");
  assert(this === expectedHandler, currentScenario + " trap this");
  assertTrapArguments(target, key, descriptor);
  return Reflect.defineProperty(target, key, descriptor);
}

function ordinaryDefineTrapGetter() {
  handlerLookups += 1;
  assert(this === expectedHandler, currentScenario + " getter this");
  return ordinaryDefineTrap;
}

function exerciseHandlerBrand(handler, target, label) {
  var proxy = new Proxy(target, handler);
  var lookupCountBefore = handlerLookups;
  var callCountBefore = handlerCalls;

  currentScenario = label + " Object.defineProperty";
  expectedHandler = handler;
  expectedTarget = target;
  expectedKey = label + "Object";
  expectedValue = { scenario: currentScenario };
  expectedInputDescriptor = {
    configurable: true,
    enumerable: false,
    value: expectedValue,
    writable: true,
  };
  var objectResult = Object.defineProperty(proxy, expectedKey, expectedInputDescriptor);
  assert(objectResult === proxy, currentScenario + " result");
  var objectTargetDescriptor = Object.getOwnPropertyDescriptor(target, expectedKey);
  assert(objectTargetDescriptor !== undefined, currentScenario + " target descriptor");
  assert(objectTargetDescriptor.value === expectedValue, currentScenario + " target value");
  assert(objectTargetDescriptor.writable === true, currentScenario + " target writable");
  assert(objectTargetDescriptor.enumerable === false, currentScenario + " target enumerable");
  assert(objectTargetDescriptor.configurable === true, currentScenario + " target configurable");

  currentScenario = label + " Reflect.defineProperty";
  expectedHandler = handler;
  expectedTarget = target;
  expectedKey = label + "Reflect";
  expectedValue = { scenario: currentScenario };
  expectedInputDescriptor = {
    configurable: true,
    enumerable: false,
    value: expectedValue,
    writable: true,
  };
  var reflectResult = Reflect.defineProperty(proxy, expectedKey, expectedInputDescriptor);
  assert(reflectResult === true, currentScenario + " result");
  var reflectTargetDescriptor = Object.getOwnPropertyDescriptor(target, expectedKey);
  assert(reflectTargetDescriptor !== undefined, currentScenario + " target descriptor");
  assert(reflectTargetDescriptor.value === expectedValue, currentScenario + " target value");
  assert(reflectTargetDescriptor.writable === true, currentScenario + " target writable");
  assert(reflectTargetDescriptor.enumerable === false, currentScenario + " target enumerable");
  assert(reflectTargetDescriptor.configurable === true, currentScenario + " target configurable");

  assert(handlerLookups === lookupCountBefore + 2, label + " lookup count");
  assert(handlerCalls === callCountBefore + 2, label + " call count");
}

function functionHandler() {}
Object.defineProperty(functionHandler, "defineProperty", {
  configurable: true,
  get: ordinaryDefineTrapGetter,
});
exerciseHandlerBrand(functionHandler, {}, "Function handler");

var arrayHandler = [];
Object.defineProperty(arrayHandler, "defineProperty", {
  configurable: true,
  get: ordinaryDefineTrapGetter,
});
exerciseHandlerBrand(arrayHandler, {}, "Array handler");

var argumentsHandler = (function () { return arguments; })(1, 2, 3);
Object.defineProperty(argumentsHandler, "defineProperty", {
  configurable: true,
  get: ordinaryDefineTrapGetter,
});
exerciseHandlerBrand(argumentsHandler, {}, "arguments handler");

var callableTrapTarget = function () {
  throw "callable Proxy target invoked directly";
};
var callableTrapHandler = {
  apply: function (target, thisArgument, argumentsList) {
    handlerCalls += 1;
    assert(this === callableTrapHandler, currentScenario + " apply this");
    assert(target === callableTrapTarget, currentScenario + " callable target");
    assert(thisArgument === expectedHandler, currentScenario + " trap this");
    assert(argumentsList.length === 3, currentScenario + " arguments length");
    assertTrapArguments(argumentsList[0], argumentsList[1], argumentsList[2]);
    return Reflect.defineProperty(argumentsList[0], argumentsList[1], argumentsList[2]);
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
    assert(key === "defineProperty", currentScenario + " lookup key");
    assert(receiver === proxyHandler, currentScenario + " lookup receiver");
    return callableProxyTrap;
  },
};
proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);
exerciseHandlerBrand(proxyHandler, {}, "Proxy handler");

var lookupSentinel = {};
var abruptHandler = {};
Object.defineProperty(abruptHandler, "defineProperty", {
  configurable: true,
  get: function () {
    assert(this === abruptHandler, "abrupt lookup getter this");
    throw lookupSentinel;
  },
});
var abruptTarget = {};
var lookupError = capture(function () {
  Reflect.defineProperty(
    new Proxy(abruptTarget, abruptHandler),
    "untouched",
    { value: 1 }
  );
});
assert(lookupError === lookupSentinel, "abrupt lookup sentinel");
assert(
  !Object.prototype.hasOwnProperty.call(abruptTarget, "untouched"),
  "abrupt lookup mutated target"
);

var nestedCalls = 0;
var nestedRawTarget = {};
var nestedHandler = {
  defineProperty: function (target, key, descriptor) {
    nestedCalls += 1;
    assert(this === nestedHandler, "nested trap this");
    assert(target === nestedRawTarget, "nested trap target");
    assert(key === "nullFallback" || key === "undefinedFallback", "nested trap key");
    return Reflect.defineProperty(target, key, descriptor);
  },
};
var nestedTarget = new Proxy(nestedRawTarget, nestedHandler);
var nullFallback = new Proxy(nestedTarget, { defineProperty: null });
assert(
  Object.defineProperty(nullFallback, "nullFallback", { value: 1 }) === nullFallback,
  "null fallback result"
);
var undefinedFallback = new Proxy(nestedTarget, { defineProperty: undefined });
assert(
  Reflect.defineProperty(undefinedFallback, "undefinedFallback", { value: 2 }) === true,
  "undefined fallback result"
);
assert(nestedCalls === 2, "nested fallback call count");
assert(nestedRawTarget.nullFallback === 1, "null fallback value");
assert(nestedRawTarget.undefinedFallback === 2, "undefined fallback value");

var other = __lilaCreateRealm().global;
var nonCallableError = capture(function () {
  other.Object.defineProperty(
    new Proxy({}, { defineProperty: {} }),
    "value",
    { value: 1 }
  );
});
assert(nonCallableError !== undefined, "created Realm noncallable did not throw");
assert(
  Object.getPrototypeOf(nonCallableError) === other.TypeError.prototype,
  "created Realm noncallable prototype"
);
assert(nonCallableError instanceof other.TypeError, "created Realm noncallable instanceof");
assert(!(nonCallableError instanceof TypeError), "created Realm noncallable entry TypeError");

var revocable = Proxy.revocable({}, {});
revocable.revoke();
var revokedError = capture(function () {
  other.Reflect.defineProperty(revocable.proxy, "value", { value: 1 });
});
assert(revokedError !== undefined, "created Realm revoked did not throw");
assert(Object.getPrototypeOf(revokedError) === other.TypeError.prototype, "created Realm revoked prototype");
assert(revokedError instanceof other.TypeError, "created Realm revoked instanceof");
assert(!(revokedError instanceof TypeError), "created Realm revoked entry TypeError");

true;
