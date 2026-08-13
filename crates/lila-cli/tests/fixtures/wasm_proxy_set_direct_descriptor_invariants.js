function assert(condition, message) {
  if (!condition) {
    throw message;
  }
}

function throwsTypeError(operation) {
  try {
    operation();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

function acceptingSetProxy(target) {
  return new Proxy(target, {
    set: function () {
      return true;
    },
  });
}

var lockedLength = [1, 2];
Object.defineProperty(lockedLength, "length", { writable: false });
var lockedLengthProxy = acceptingSetProxy(lockedLength);
assert(Reflect.set(lockedLengthProxy, "length", 2), "array length SameValue");
assert(throwsTypeError(function () {
  Reflect.set(lockedLengthProxy, "length", 3);
}), "array length mismatch");
assert(throwsTypeError(function () {
  "use strict";
  lockedLengthProxy.length = 3;
}), "array length assignment mismatch");

var dense = [NaN];
Object.defineProperty(dense, "0", {
  writable: false,
  configurable: false,
});
var denseProxy = acceptingSetProxy(dense);
assert(Reflect.set(denseProxy, "0", NaN), "dense Array NaN SameValue");
assert(throwsTypeError(function () {
  Reflect.set(denseProxy, "0", 1);
}), "dense Array mismatch");

var sparse = [];
Object.defineProperty(sparse, "100", {
  value: "sparse",
  writable: false,
  configurable: false,
});
var sparseProxy = acceptingSetProxy(sparse);
assert(Reflect.set(sparseProxy, "100", "sparse"), "sparse Array SameValue");
assert(throwsTypeError(function () {
  Reflect.set(sparseProxy, "100", "different");
}), "sparse Array mismatch");

var namedAndSymbol = [];
var symbolKey = Symbol("Proxy set descriptor key");
Object.defineProperty(namedAndSymbol, "named", {
  value: -0,
  writable: false,
  configurable: false,
});
Object.defineProperty(namedAndSymbol, symbolKey, {
  value: namedAndSymbol,
  writable: false,
  configurable: false,
});
var namedAndSymbolProxy = acceptingSetProxy(namedAndSymbol);
assert(Reflect.set(namedAndSymbolProxy, "named", -0), "named signed-zero SameValue");
assert(throwsTypeError(function () {
  Reflect.set(namedAndSymbolProxy, "named", +0);
}), "named signed-zero mismatch");
assert(
  Reflect.set(namedAndSymbolProxy, symbolKey, namedAndSymbol),
  "symbol object-identity SameValue"
);
assert(throwsTypeError(function () {
  Reflect.set(namedAndSymbolProxy, symbolKey, []);
}), "symbol object-identity mismatch");

var boxed = new String("xy");
var boxedProxy = acceptingSetProxy(boxed);
assert(Reflect.set(boxedProxy, "0", "x"), "boxed String index SameValue");
assert(throwsTypeError(function () {
  Reflect.set(boxedProxy, "0", "z");
}), "boxed String index mismatch");
assert(Reflect.set(boxedProxy, "length", 2), "boxed String length SameValue");
assert(throwsTypeError(function () {
  Reflect.set(boxedProxy, "length", 3);
}), "boxed String length mismatch");

function mappedArgument(value) {
  value = 9;
  Object.defineProperty(arguments, "0", {
    writable: false,
    configurable: false,
  });
  var proxy = acceptingSetProxy(arguments);
  return Reflect.set(proxy, "0", 9) && throwsTypeError(function () {
    Reflect.set(proxy, "0", 10);
  });
}
assert(mappedArgument(1), "mapped arguments current value");

var argumentsLength = (function () { return arguments; })(1, 2);
Object.defineProperty(argumentsLength, "length", {
  writable: false,
  configurable: false,
});
var argumentsLengthProxy = acceptingSetProxy(argumentsLength);
assert(Reflect.set(argumentsLengthProxy, "length", 2), "arguments length SameValue");
assert(throwsTypeError(function () {
  Reflect.set(argumentsLengthProxy, "length", 3);
}), "arguments length mismatch");

var getterCalls = 0;
var accessorArguments = (function () {
  "use strict";
  return arguments;
})(1);
Object.defineProperty(accessorArguments, "0", {
  configurable: false,
  get: function () {
    getterCalls++;
    return 1;
  },
  set: undefined,
});
var accessorArgumentsProxy = acceptingSetProxy(accessorArguments);
assert(throwsTypeError(function () {
  Reflect.set(accessorArgumentsProxy, "0", 1);
}), "arguments undefined setter");
assert(getterCalls === 0, "descriptor observation invoked getter");

var callableSetter = new Proxy(function () {}, {});
var accessorTarget = {};
Object.defineProperty(accessorTarget, "value", {
  configurable: false,
  set: callableSetter,
});
assert(
  Reflect.set(acceptingSetProxy(accessorTarget), "value", 1),
  "callable Proxy setter is not undefined"
);

var configurable = {};
Object.defineProperty(configurable, "value", {
  value: 1,
  writable: false,
  configurable: true,
});
assert(
  Reflect.set(acceptingSetProxy(configurable), "value", 2),
  "configurable non-writable target"
);

function PrototypeTarget() {}
var originalPrototype = PrototypeTarget.prototype;
assert(
  Reflect.set(acceptingSetProxy(PrototypeTarget), "prototype", {}),
  "writable Function prototype"
);
assert(PrototypeTarget.prototype === originalPrototype, "truthy trap mutated target");

function FrozenPrototypeTarget() {}
var frozenPrototype = { actualEntry: true };
Object.defineProperty(FrozenPrototypeTarget, "prototype", {
  value: frozenPrototype,
  writable: false,
});
assert(
  FrozenPrototypeTarget.prototype === frozenPrototype,
  "frozen Function prototype actual entry"
);
var frozenPrototypeProxy = acceptingSetProxy(FrozenPrototypeTarget);
assert(
  Reflect.set(frozenPrototypeProxy, "prototype", frozenPrototype),
  "frozen Function prototype actual-entry SameValue"
);
assert(throwsTypeError(function () {
  Reflect.set(frozenPrototypeProxy, "prototype", {});
}), "frozen Function prototype mismatch");
assert(
  FrozenPrototypeTarget.prototype === frozenPrototype,
  "frozen Function truthy trap mutated target"
);

var typed = new Uint8Array(1);
assert(Reflect.set(acceptingSetProxy(typed), "0", 255), "integer-indexed descriptor");
assert(typed[0] === 0, "truthy typed-array trap mutated target");

var absent = [];
Object.preventExtensions(absent);
assert(
  Reflect.set(acceptingSetProxy(absent), "missing", 1),
  "absent property on non-extensible target"
);

true;
