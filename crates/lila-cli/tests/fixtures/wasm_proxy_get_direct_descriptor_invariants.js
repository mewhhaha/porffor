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

function returningGetProxy(target, result) {
  return new Proxy(target, {
    get: function () {
      return result;
    },
  });
}

var lockedLength = [1, 2];
Object.defineProperty(lockedLength, "length", { writable: false });
var lockedLengthProxy = returningGetProxy(lockedLength, 2);
assert(lockedLengthProxy.length === 2, "array length SameValue");
assert(Reflect.get(lockedLengthProxy, "length") === 2, "Reflect array length SameValue");
assert(throwsTypeError(function () {
  return returningGetProxy(lockedLength, 3).length;
}), "array length mismatch");
assert(throwsTypeError(function () {
  return Reflect.get(returningGetProxy(lockedLength, 3), "length");
}), "Reflect array length mismatch");

var dense = [NaN];
Object.defineProperty(dense, "0", {
  writable: false,
  configurable: false,
});
var denseValue = Reflect.get(returningGetProxy(dense, NaN), "0");
assert(denseValue !== denseValue, "dense Array NaN SameValue");
assert(throwsTypeError(function () {
  return returningGetProxy(dense, 1)[0];
}), "dense Array mismatch");

var sparse = [];
Object.defineProperty(sparse, "100", {
  value: "sparse",
  writable: false,
  configurable: false,
});
assert(returningGetProxy(sparse, "sparse")[100] === "sparse", "sparse Array SameValue");
assert(throwsTypeError(function () {
  return Reflect.get(returningGetProxy(sparse, "different"), "100");
}), "sparse Array mismatch");

var namedAndSymbol = [];
var symbolKey = Symbol("Proxy get descriptor key");
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
assert(
  1 / returningGetProxy(namedAndSymbol, -0).named === -Infinity,
  "named signed-zero SameValue"
);
assert(throwsTypeError(function () {
  return returningGetProxy(namedAndSymbol, +0).named;
}), "named signed-zero mismatch");
assert(
  returningGetProxy(namedAndSymbol, namedAndSymbol)[symbolKey] === namedAndSymbol,
  "symbol object-identity SameValue"
);
assert(throwsTypeError(function () {
  return returningGetProxy(namedAndSymbol, [])[symbolKey];
}), "symbol object-identity mismatch");

var missingGetter = {
  set value(next) {},
};
Object.freeze(missingGetter);
assert(
  returningGetProxy(missingGetter, undefined).value === undefined,
  "raw missing getter accepts undefined"
);
assert(throwsTypeError(function () {
  return returningGetProxy(missingGetter, 1).value;
}), "raw missing getter rejects a value");

var taggedMissingGetter = [];
Object.defineProperty(taggedMissingGetter, "0", {
  configurable: false,
  get: undefined,
  set: function () {},
});
assert(
  Reflect.get(returningGetProxy(taggedMissingGetter, undefined), "0") === undefined,
  "tagged missing getter accepts undefined"
);
assert(throwsTypeError(function () {
  return returningGetProxy(taggedMissingGetter, 1)[0];
}), "tagged missing getter rejects a value");

var getterCalls = 0;
var callableGetter = new Proxy(function () {
  getterCalls++;
  return "target getter";
}, {});
var accessorArray = [];
Object.defineProperty(accessorArray, "0", {
  configurable: false,
  get: callableGetter,
});
assert(
  returningGetProxy(accessorArray, "trap result")[0] === "trap result",
  "callable Proxy getter is not undefined"
);
assert(getterCalls === 0, "descriptor observation invoked Array getter");

var boxed = new String("xy");
assert(returningGetProxy(boxed, "x")[0] === "x", "boxed String index SameValue");
assert(throwsTypeError(function () {
  return returningGetProxy(boxed, "z")[0];
}), "boxed String index mismatch");
assert(returningGetProxy(boxed, 2).length === 2, "boxed String length SameValue");
assert(throwsTypeError(function () {
  return Reflect.get(returningGetProxy(boxed, 3), "length");
}), "boxed String length mismatch");

function mappedArgument(value) {
  value = 9;
  Object.defineProperty(arguments, "0", {
    writable: false,
    configurable: false,
  });
  var proxy = returningGetProxy(arguments, 9);
  return proxy[0] === 9 && throwsTypeError(function () {
    return Reflect.get(returningGetProxy(arguments, 10), "0");
  });
}
assert(mappedArgument(1), "mapped arguments current value");

var argumentsLength = (function () { return arguments; })(1, 2);
Object.defineProperty(argumentsLength, "length", {
  writable: false,
  configurable: false,
});
assert(returningGetProxy(argumentsLength, 2).length === 2, "arguments length SameValue");
assert(throwsTypeError(function () {
  return returningGetProxy(argumentsLength, 3).length;
}), "arguments length mismatch");

var accessorArguments = (function () { return arguments; })(1);
Object.defineProperty(accessorArguments, "0", {
  configurable: false,
  get: undefined,
  set: function () {},
});
assert(
  returningGetProxy(accessorArguments, undefined)[0] === undefined,
  "arguments missing getter accepts undefined"
);
assert(throwsTypeError(function () {
  return returningGetProxy(accessorArguments, 1)[0];
}), "arguments missing getter rejects a value");

var accessorArgumentsLength = (function () { return arguments; })(1);
Object.defineProperty(accessorArgumentsLength, "length", {
  configurable: false,
  get: undefined,
  set: function () {},
});
assert(throwsTypeError(function () {
  return returningGetProxy(accessorArgumentsLength, 1).length;
}), "arguments special length getter");

var strictArguments = (function () {
  "use strict";
  return arguments;
})();
assert(
  returningGetProxy(strictArguments, "trap callee").callee === "trap callee",
  "arguments special callable callee getter"
);

function FrozenPrototypeTarget() {}
var frozenPrototype = { actualEntry: true };
Object.defineProperty(FrozenPrototypeTarget, "prototype", {
  value: frozenPrototype,
  writable: false,
});
assert(
  returningGetProxy(FrozenPrototypeTarget, frozenPrototype).prototype === frozenPrototype,
  "frozen Function prototype actual-entry SameValue"
);
assert(throwsTypeError(function () {
  return returningGetProxy(FrozenPrototypeTarget, {}).prototype;
}), "frozen Function prototype mismatch");

var dataViewPrototype = DataView.prototype;
assert(
  returningGetProxy(DataView, dataViewPrototype).prototype === dataViewPrototype,
  "DataView constructor prototype SameValue"
);
assert(throwsTypeError(function () {
  return returningGetProxy(DataView, {}).prototype;
}), "DataView constructor prototype mismatch");

var typed = new Uint8Array(1);
assert(
  returningGetProxy(typed, 255)[0] === 255,
  "configurable integer-indexed descriptor"
);

var configurable = {};
Object.defineProperty(configurable, "value", {
  value: 1,
  writable: false,
  configurable: true,
});
assert(
  returningGetProxy(configurable, 2).value === 2,
  "configurable non-writable target"
);

var absent = [];
Object.preventExtensions(absent);
assert(returningGetProxy(absent, 1).missing === 1, "absent non-extensible target");

var abruptMarker = {};
var abruptTarget = [];
Object.defineProperty(abruptTarget, "length", { writable: false });
var abruptProxy = new Proxy(abruptTarget, {
  get: function () {
    throw abruptMarker;
  },
});
var directAbruptPreserved = false;
try {
  abruptProxy.length;
} catch (error) {
  directAbruptPreserved = error === abruptMarker;
}
assert(directAbruptPreserved, "direct thrown trap was replaced by invariant error");

var reflectAbruptPreserved = false;
try {
  Reflect.get(abruptProxy, "length");
} catch (error) {
  reflectAbruptPreserved = error === abruptMarker;
}
assert(reflectAbruptPreserved, "Reflect thrown trap was replaced by invariant error");

true;
