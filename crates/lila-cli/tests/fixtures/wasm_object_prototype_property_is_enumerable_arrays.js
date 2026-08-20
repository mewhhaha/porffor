let propertyIsEnumerable = Object.prototype.propertyIsEnumerable;
let dynamicIndex = ["0"].join("");
let empty = [];
let nonempty = ["value"];
let holey = [, "value"];
let deleted = ["value"];
delete deleted[0];
let redefined = ["value"];
Object.defineProperty(redefined, dynamicIndex, {
  value: "value",
  writable: true,
  enumerable: false,
  configurable: true
});
let named = [];
Object.defineProperty(named, "visible", {
  value: true,
  writable: true,
  enumerable: true,
  configurable: true
});
Object.defineProperty(named, "hidden", {
  value: true,
  writable: true,
  enumerable: false,
  configurable: true
});
let visibleSymbol = Symbol("visible");
let hiddenSymbol = Symbol("hidden");
let sameDescriptionSymbol = Symbol("hidden");
let symbolNamed = [];
symbolNamed[visibleSymbol] = true;
Object.defineProperty(symbolNamed, hiddenSymbol, {
  value: true,
  writable: true,
  enumerable: false,
  configurable: true
});

function argumentsProperties(value) {
  let dynamicArgumentIndex = ["0"].join("");
  return propertyIsEnumerable.call(arguments, dynamicArgumentIndex)
    && !propertyIsEnumerable.call(arguments, "length");
}

function argumentsLengthDataEnumerable(enumerable) {
  Object.defineProperty(arguments, "length", {
    value: 1,
    writable: true,
    enumerable: enumerable,
    configurable: true
  });
  return propertyIsEnumerable.call(arguments, "length");
}

function argumentsLengthAccessorEnumerable(enumerable) {
  Object.defineProperty(arguments, "length", {
    get: function() { return 1; },
    enumerable: enumerable,
    configurable: true
  });
  return propertyIsEnumerable.call(arguments, "length");
}

function argumentsLengthPartialRedefinition() {
  Object.defineProperty(arguments, "length", { enumerable: true });
  Object.defineProperty(arguments, "length", { value: 1 });
  return propertyIsEnumerable.call(arguments, "length");
}

function argumentsLengthSetterOnlyEnumerable(enumerable) {
  Object.defineProperty(arguments, "length", {
    set: function(value) {},
    enumerable: enumerable,
    configurable: true
  });
  return propertyIsEnumerable.call(arguments, "length") === enumerable
    && arguments.length === undefined;
}

let enumerableProxyTrapCount = 0;
let enumerableProxy = new Proxy({}, {
  getOwnPropertyDescriptor: function(target, key) {
    enumerableProxyTrapCount++;
    return { value: true, writable: true, enumerable: true, configurable: true };
  }
});
let nonEnumerableProxyTrapCount = 0;
let nonEnumerableProxy = new Proxy({}, {
  getOwnPropertyDescriptor: function(target, key) {
    nonEnumerableProxyTrapCount++;
    return { value: true, writable: true, enumerable: false, configurable: true };
  }
});
let absentProxyTrapCount = 0;
let absentProxy = new Proxy({}, {
  getOwnPropertyDescriptor: function(target, key) {
    absentProxyTrapCount++;
    return undefined;
  }
});
let throwingProxyTrapCount = 0;
let throwingProxy = new Proxy({}, {
  getOwnPropertyDescriptor: function(target, key) {
    throwingProxyTrapCount++;
    throw "proxy descriptor throw";
  }
});
let arrayLengthUndefinedTrapCount = 0;
let arrayLengthUndefinedProxy = new Proxy([], {
  getOwnPropertyDescriptor: function(target, key) {
    arrayLengthUndefinedTrapCount++;
    return undefined;
  }
});
let arrayLengthConfigurableTrapCount = 0;
let arrayLengthConfigurableProxy = new Proxy([], {
  getOwnPropertyDescriptor: function(target, key) {
    arrayLengthConfigurableTrapCount++;
    return { value: 0, writable: true, enumerable: false, configurable: true };
  }
});
let arrayLengthValidTrapCount = 0;
let arrayLengthValidProxy = new Proxy([], {
  getOwnPropertyDescriptor: function(target, key) {
    arrayLengthValidTrapCount++;
    return { value: 0, writable: true, enumerable: false, configurable: false };
  }
});
let primitiveString = "A😀";

if (propertyIsEnumerable.call(empty, dynamicIndex)) throw "empty index";
if (!propertyIsEnumerable.call(nonempty, dynamicIndex)) throw "dense index";
if (propertyIsEnumerable.call(holey, dynamicIndex)) throw "hole index";
if (!propertyIsEnumerable.call(holey, "1")) throw "later dense index";
if (propertyIsEnumerable.call(deleted, dynamicIndex)) throw "deleted index";
if (propertyIsEnumerable.call(redefined, dynamicIndex)) throw "non-enumerable index";
if (propertyIsEnumerable.call(nonempty, "length")) throw "array length";
if (!propertyIsEnumerable.call(named, "visible")) throw "named enumerable";
if (propertyIsEnumerable.call(named, "hidden")) throw "named non-enumerable";
if (propertyIsEnumerable.call(named, "absent")) throw "named absent";
if (!propertyIsEnumerable.call(symbolNamed, visibleSymbol)) throw "symbol assignment";
if (propertyIsEnumerable.call(symbolNamed, hiddenSymbol)) throw "symbol non-enumerable";
if (propertyIsEnumerable.call(symbolNamed, sameDescriptionSymbol)) throw "symbol identity";
if (!argumentsProperties("value")) throw "arguments properties";
if (!argumentsLengthDataEnumerable(true)) throw "arguments data enumerable";
if (argumentsLengthDataEnumerable(false)) throw "arguments data non-enumerable";
if (!argumentsLengthAccessorEnumerable(true)) throw "arguments accessor enumerable";
if (argumentsLengthAccessorEnumerable(false)) throw "arguments accessor non-enumerable";
if (!argumentsLengthPartialRedefinition()) throw "arguments partial redefinition";
if (!argumentsLengthSetterOnlyEnumerable(true)) throw "arguments setter-only accessor";
if (!argumentsLengthSetterOnlyEnumerable(false)) throw "arguments setter-only non-enumerable";
if (!propertyIsEnumerable.call(enumerableProxy, "value")) throw "proxy enumerable";
if (enumerableProxyTrapCount !== 1) throw "proxy enumerable count";
if (propertyIsEnumerable.call(nonEnumerableProxy, "value")) throw "proxy non-enumerable";
if (nonEnumerableProxyTrapCount !== 1) throw "proxy non-enumerable count";
if (propertyIsEnumerable.call(absentProxy, "value")) throw "proxy absent";
if (absentProxyTrapCount !== 1) throw "proxy absent count";
try {
  propertyIsEnumerable.call(throwingProxy, "value");
  throw "proxy throw missing";
} catch (error) {
  if (error !== "proxy descriptor throw") throw error;
}
if (throwingProxyTrapCount !== 1) throw "proxy throw count";
try {
  propertyIsEnumerable.call(arrayLengthUndefinedProxy, "length");
  throw "array proxy missing undefined invariant";
} catch (error) {
  if (!(error instanceof TypeError)) throw error;
}
if (arrayLengthUndefinedTrapCount !== 1) throw "array proxy undefined count";
try {
  propertyIsEnumerable.call(arrayLengthConfigurableProxy, "length");
  throw "array proxy missing configurable invariant";
} catch (error) {
  if (!(error instanceof TypeError)) throw error;
}
if (arrayLengthConfigurableTrapCount !== 1) throw "array proxy configurable count";
if (propertyIsEnumerable.call(arrayLengthValidProxy, "length")) throw "array proxy valid descriptor";
if (arrayLengthValidTrapCount !== 1) throw "array proxy valid count";
if (!propertyIsEnumerable.call(primitiveString, "0")) throw "string index zero";
if (!propertyIsEnumerable.call(primitiveString, "1")) throw "string surrogate index";
if (!propertyIsEnumerable.call(primitiveString, "2")) throw "string final code unit";
if (propertyIsEnumerable.call(primitiveString, "3")) throw "string boundary";
if (propertyIsEnumerable.call(primitiveString, "4")) throw "string out of range";
if (propertyIsEnumerable.call(primitiveString, "length")) throw "string length";
true;
