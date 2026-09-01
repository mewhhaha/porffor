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
var expectedPrototype;
var currentScenario;
var handlerLookups = 0;
var handlerCalls = 0;

function ordinarySetPrototypeOfTrap(target, prototype) {
  handlerCalls += 1;
  assert(arguments.length === 2, currentScenario + " trap arity");
  assert(this === expectedHandler, currentScenario + " trap this");
  assert(target === expectedTarget, currentScenario + " target");
  assert(prototype === expectedPrototype, currentScenario + " prototype");
  return true;
}

function ordinarySetPrototypeOfTrapGetter() {
  handlerLookups += 1;
  assert(this === expectedHandler, currentScenario + " getter this");
  return ordinarySetPrototypeOfTrap;
}

function exerciseHandlerBrand(handler, target, label) {
  var proxy = new Proxy(target, handler);
  var lookupCountBefore = handlerLookups;
  var callCountBefore = handlerCalls;

  currentScenario = label + " Object.setPrototypeOf";
  expectedHandler = handler;
  expectedTarget = target;
  expectedPrototype = { scenario: currentScenario };
  var objectResult = Object.setPrototypeOf(proxy, expectedPrototype);
  assert(objectResult === proxy, currentScenario + " result");

  currentScenario = label + " Reflect.setPrototypeOf";
  expectedHandler = handler;
  expectedTarget = target;
  expectedPrototype = { scenario: currentScenario };
  var reflectResult = Reflect.setPrototypeOf(proxy, expectedPrototype);
  assert(reflectResult === true, currentScenario + " result");

  assert(handlerLookups === lookupCountBefore + 2, label + " lookup count");
  assert(handlerCalls === callCountBefore + 2, label + " call count");
}

function functionHandler() {}
Object.defineProperty(functionHandler, "setPrototypeOf", {
  configurable: true,
  get: ordinarySetPrototypeOfTrapGetter,
});
exerciseHandlerBrand(functionHandler, {}, "Function handler");

var arrayHandler = [];
Object.defineProperty(arrayHandler, "setPrototypeOf", {
  configurable: true,
  get: ordinarySetPrototypeOfTrapGetter,
});
exerciseHandlerBrand(arrayHandler, {}, "Array handler");

var argumentsHandler = (function () { return arguments; })(1, 2, 3);
Object.defineProperty(argumentsHandler, "setPrototypeOf", {
  configurable: true,
  get: ordinarySetPrototypeOfTrapGetter,
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
    assert(argumentsList.length === 2, currentScenario + " arguments length");
    assert(argumentsList[0] === expectedTarget, currentScenario + " target");
    assert(
      argumentsList[1] === expectedPrototype,
      currentScenario + " prototype"
    );
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
    assert(key === "setPrototypeOf", currentScenario + " lookup key");
    assert(receiver === proxyHandler, currentScenario + " lookup receiver");
    return callableProxyTrap;
  },
};
proxyHandler = new Proxy(proxyHandlerTarget, proxyLookupHandler);
exerciseHandlerBrand(proxyHandler, {}, "Proxy handler");

var lookupSentinel = {};
var abruptHandler = new Proxy({}, {
  get: function (target, key) {
    assert(key === "setPrototypeOf", "abrupt lookup key");
    throw lookupSentinel;
  },
});
var abruptTarget = {};
var originalPrototype = Object.getPrototypeOf(abruptTarget);
var lookupError = capture(function () {
  Reflect.setPrototypeOf(new Proxy(abruptTarget, abruptHandler), {});
});
assert(lookupError === lookupSentinel, "abrupt lookup sentinel");
assert(
  Object.getPrototypeOf(abruptTarget) === originalPrototype,
  "abrupt lookup mutated target"
);

var nestedCalls = 0;
var nestedRawTarget = {};
var nullPrototype = {};
var undefinedPrototype = {};
var nestedHandler = {
  setPrototypeOf: function (target, prototype) {
    nestedCalls += 1;
    assert(this === nestedHandler, "nested trap this");
    assert(target === nestedRawTarget, "nested trap target");
    assert(
      prototype === nullPrototype || prototype === undefinedPrototype,
      "nested trap prototype"
    );
    return true;
  },
};
var nestedTarget = new Proxy(nestedRawTarget, nestedHandler);
var nullFallback = new Proxy(nestedTarget, { setPrototypeOf: null });
assert(
  Object.setPrototypeOf(nullFallback, nullPrototype) === nullFallback,
  "null fallback result"
);
var undefinedFallback = new Proxy(nestedTarget, { setPrototypeOf: undefined });
assert(
  Reflect.setPrototypeOf(undefinedFallback, undefinedPrototype) === true,
  "undefined fallback result"
);
assert(nestedCalls === 2, "nested fallback call count");

var other = __lilaCreateRealm().global;
var nonCallableError = capture(function () {
  other.Reflect.setPrototypeOf(
    new Proxy({}, { setPrototypeOf: {} }),
    {}
  );
});
assert(nonCallableError !== undefined, "created Realm noncallable did not throw");
assert(
  Object.getPrototypeOf(nonCallableError) === other.TypeError.prototype,
  "created Realm noncallable prototype"
);
assert(nonCallableError instanceof other.TypeError, "created Realm noncallable instanceof");
assert(!(nonCallableError instanceof TypeError), "created Realm noncallable entry TypeError");

var invariantPrototype = {};
var invariantTarget = Object.create(invariantPrototype);
Object.preventExtensions(invariantTarget);
var invariantError = capture(function () {
  other.Reflect.setPrototypeOf(
    new Proxy(invariantTarget, {
      setPrototypeOf: function () {
        return true;
      },
    }),
    {}
  );
});
assert(invariantError !== undefined, "created Realm invariant did not throw");
assert(
  Object.getPrototypeOf(invariantError) === other.TypeError.prototype,
  "created Realm invariant prototype"
);
assert(invariantError instanceof other.TypeError, "created Realm invariant instanceof");
assert(!(invariantError instanceof TypeError), "created Realm invariant entry TypeError");

var revocable = Proxy.revocable({}, {});
revocable.revoke();
var revokedError = capture(function () {
  other.Object.setPrototypeOf(revocable.proxy, {});
});
assert(revokedError !== undefined, "created Realm revoked did not throw");
assert(
  Object.getPrototypeOf(revokedError) === other.TypeError.prototype,
  "created Realm revoked prototype"
);
assert(revokedError instanceof other.TypeError, "created Realm revoked instanceof");
assert(!(revokedError instanceof TypeError), "created Realm revoked entry TypeError");

true;
