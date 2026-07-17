let directDeleteBuiltin = Array.prototype.map;
let reflectDeleteBuiltin = Array.prototype.filter;
let directDeleteNames = Object.getOwnPropertyNames(directDeleteBuiltin);
let proxy = new Proxy({ proxied: true }, {});
let hasOwnProperty = Object.prototype.hasOwnProperty;

function argumentsHasOwnProperties(first, second) {
  let values = arguments;
  Object.defineProperty(values, "named", {
    value: true,
    writable: true,
    enumerable: true,
    configurable: true
  });
  let hadLength = hasOwnProperty.call(values, "length");
  let hadFirst = hasOwnProperty.call(values, "0");
  let hadNamed = hasOwnProperty.call(values, "named");
  let deletedLength = delete values.length;
  let reflectedDeletedLength = Reflect.deleteProperty(values, "length");
  let prototype = Object.getPrototypeOf(values);
  prototype.length = "inherited length";
  let readsInheritedLength = values.length === "inherited length";
  delete prototype.length;
  return hadLength
    && hadFirst
    && hadNamed
    && deletedLength
    && reflectedDeletedLength
    && readsInheritedLength
    && Object.getOwnPropertyDescriptor(values, "length") === undefined
    && !hasOwnProperty.call(values, "length")
    && hasOwnProperty.call(values, "0")
    && hasOwnProperty.call(values, "named");
}

let nullishKeyCalls = 0;
let nullishKey = {
  toString: function() {
    nullishKeyCalls++;
    return "unused";
  }
};
let nullishCoercesKeyBeforeThrow = false;
try {
  hasOwnProperty.call(null, nullishKey);
} catch (error) {
  nullishCoercesKeyBeforeThrow = error instanceof TypeError && nullishKeyCalls === 1;
}
try {
  hasOwnProperty.call(undefined, nullishKey);
} catch (error) {
  nullishCoercesKeyBeforeThrow = nullishCoercesKeyBeforeThrow
    && error instanceof TypeError
    && nullishKeyCalls === 2;
}

let proxyKeyCalls = 0;
let proxyKey = {
  toString: function() {
    proxyKeyCalls++;
    return "value";
  }
};
let keyProxy = new Proxy({ value: true }, {});
let proxyCoercesKeyOnce = hasOwnProperty.call(keyProxy, proxyKey) && proxyKeyCalls === 1;

let abruptKeyCalls = 0;
let abruptKey = {
  toString: function() {
    abruptKeyCalls++;
    throw "abrupt key coercion";
  }
};
let proxyPropagatesKeyCoercion = false;
try {
  hasOwnProperty.call(keyProxy, abruptKey);
} catch (error) {
  proxyPropagatesKeyCoercion = error === "abrupt key coercion" && abruptKeyCalls === 1;
}

let stringValue = "abcdef";
let primitiveStringProperties = hasOwnProperty.call(stringValue, "length")
  && hasOwnProperty.call(stringValue, "0")
  && hasOwnProperty.call(stringValue, "1")
  && hasOwnProperty.call(stringValue, "5")
  && !hasOwnProperty.call(stringValue, "6");

let symbolKey = Symbol("own");
let symbolObject = {};
symbolObject[symbolKey] = true;
let symbolProperty = hasOwnProperty.call(symbolObject, symbolKey);
let proxySawSymbolKey = false;
let symbolProxy = new Proxy(symbolObject, {
  getOwnPropertyDescriptor: function(target, key) {
    proxySawSymbolKey = key === symbolKey;
    return Object.getOwnPropertyDescriptor(target, key);
  }
});
let symbolProxyProperty = hasOwnProperty.call(symbolProxy, symbolKey) && proxySawSymbolKey;

let wellKnownKeyObject = {};
wellKnownKeyObject["Symbol.iterator"] = "string";
wellKnownKeyObject[Symbol.iterator] = "symbol";
let wellKnownStringTrapKey = undefined;
let wellKnownSymbolTrapKey = undefined;
let wellKnownKeyProxy = new Proxy(wellKnownKeyObject, {
  getOwnPropertyDescriptor: function(target, key) {
    if (wellKnownStringTrapKey === undefined) {
      wellKnownStringTrapKey = key;
    } else {
      wellKnownSymbolTrapKey = key;
    }
    return Object.getOwnPropertyDescriptor(target, key);
  }
});
let preservesWellKnownPropertyKeyTags = hasOwnProperty.call(wellKnownKeyProxy, "Symbol.iterator")
  && hasOwnProperty.call(wellKnownKeyProxy, Symbol.iterator)
  && wellKnownStringTrapKey === "Symbol.iterator"
  && wellKnownSymbolTrapKey === Symbol.iterator;

directDeleteBuiltin.marker = true;
reflectDeleteBuiltin.marker = true;

let ok = directDeleteBuiltin.hasOwnProperty("length")
  && Object.prototype.hasOwnProperty.call(directDeleteBuiltin, "length")
  && Object.hasOwn(directDeleteBuiltin, "length")
  && directDeleteBuiltin.hasOwnProperty("name")
  && Object.prototype.hasOwnProperty.call(directDeleteBuiltin, "name")
  && Object.hasOwn(directDeleteBuiltin, "marker")
  && delete directDeleteBuiltin.length
  && Reflect.deleteProperty(reflectDeleteBuiltin, "length")
  && !directDeleteBuiltin.hasOwnProperty("length")
  && !Object.prototype.hasOwnProperty.call(directDeleteBuiltin, "length")
  && !Object.hasOwn(directDeleteBuiltin, "length")
  && !reflectDeleteBuiltin.hasOwnProperty("length")
  && !Object.prototype.hasOwnProperty.call(reflectDeleteBuiltin, "length")
  && !Object.hasOwn(reflectDeleteBuiltin, "length")
  && directDeleteBuiltin.hasOwnProperty("name")
  && Object.prototype.hasOwnProperty.call(directDeleteBuiltin, "name")
  && Object.hasOwn(directDeleteBuiltin, "name")
  && directDeleteNames.indexOf("length") !== -1
  && Object.getOwnPropertyNames(directDeleteBuiltin).indexOf("length") === -1
  && Object.getOwnPropertyNames(directDeleteBuiltin).indexOf("name") !== -1
  && proxy.hasOwnProperty("proxied")
  && Object.prototype.hasOwnProperty.call(proxy, "proxied")
  && !proxy.hasOwnProperty("missing")
  && directDeleteBuiltin.hasOwnProperty("marker")
  && delete directDeleteBuiltin.marker
  && !directDeleteBuiltin.hasOwnProperty("marker")
  && reflectDeleteBuiltin.hasOwnProperty("marker")
  && delete reflectDeleteBuiltin.marker
  && !reflectDeleteBuiltin.hasOwnProperty("marker")
  && nullishCoercesKeyBeforeThrow
  && proxyCoercesKeyOnce
  && proxyPropagatesKeyCoercion
  && argumentsHasOwnProperties(1, 2)
  && primitiveStringProperties
  && symbolProperty
  && symbolProxyProperty
  && preservesWellKnownPropertyKeyTags;

ok;
