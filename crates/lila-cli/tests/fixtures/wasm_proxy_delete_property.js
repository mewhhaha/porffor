var failures = 0;

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

var target = { foo: 1 };
var seenTarget = null;
var seenKey = null;
var seenHandler = null;
var handler = {
  deleteProperty: function(t, key) {
    seenTarget = t;
    seenKey = key;
    seenHandler = this;
    return delete t[key];
  },
};
var proxy = new Proxy(target, handler);
if (!delete proxy.foo) failures |= 1;
if (Object.prototype.hasOwnProperty.call(target, "foo")) failures |= 2;
if (seenTarget !== target) failures |= 4;
if (seenKey !== "foo") failures |= 8;
if (seenHandler !== handler) failures |= 16;

var falseProxy = new Proxy({ bar: 1 }, {
  deleteProperty: function() {
    return 0;
  },
});
if (Reflect.deleteProperty(falseProxy, "bar") !== false) failures |= 32;
if (!throwsTypeError(function() {
  "use strict";
  delete falseProxy.bar;
})) failures |= 64;

var fixed = {};
Object.defineProperty(fixed, "locked", {
  value: 1,
  configurable: false,
});
var fixedProxy = new Proxy(fixed, {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() { delete fixedProxy.locked; })) failures |= 128;

var nonExtensible = { kept: 1 };
Object.preventExtensions(nonExtensible);
var nonExtensibleProxy = new Proxy(nonExtensible, {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() { Reflect.deleteProperty(nonExtensibleProxy, "kept"); })) {
  failures |= 256;
}

var array = [1];
var arrayProxy = new Proxy(new Proxy(array, {}), {
  deleteProperty: undefined,
});
if (!delete arrayProxy[0]) failures |= 512;
if (array.hasOwnProperty("0")) failures |= 1024;
if (Reflect.deleteProperty(arrayProxy, "length") !== false) failures |= 2048;

var nestedCalls = 0;
var nestedTarget = new Proxy({}, {
  deleteProperty: function(_target, key) {
    nestedCalls++;
    return key === "ok";
  },
});
var nestedProxy = new Proxy(nestedTarget, {
  deleteProperty: null,
});
if (!delete nestedProxy.ok) failures |= 4096;
if (!throwsTypeError(function() {
  "use strict";
  delete nestedProxy.nope;
})) failures |= 8192;
if (nestedCalls !== 2) failures |= 16384;

var stringProxy = new Proxy(new Proxy(new String("str"), {}), {
  deleteProperty: null,
});
if (Reflect.deleteProperty(stringProxy, "length") !== false) failures |= 32768;
if (!throwsTypeError(function() {
  "use strict";
  delete stringProxy[0];
})) failures |= 65536;

var reProxy = new Proxy(new Proxy(/(?:)/g, {}), {
  deleteProperty: null,
});
if (!delete reProxy.foo) failures |= 131072;
if (!throwsTypeError(function() {
  "use strict";
  delete reProxy.lastIndex;
})) failures |= 262144;

var funcProxy = new Proxy(new Proxy(function() {}, {}), {});
if (!delete funcProxy.length) failures |= 524288;
if (!throwsTypeError(function() {
  "use strict";
  delete funcProxy.prototype;
})) failures |= 1048576;

var revoked = Proxy.revocable({}, {});
revoked.revoke();
if (!throwsTypeError(function() { delete revoked.proxy.foo; })) failures |= 2097152;

var arrayNamed = [];
Object.defineProperty(arrayNamed, "named", {
  value: 1,
  configurable: false,
});
var arrayNamedProxy = new Proxy(arrayNamed, {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() {
  Reflect.deleteProperty(arrayNamedProxy, "named");
})) failures |= 4194304;

var arraySymbol = Symbol("array delete invariant");
var arrayWithSymbol = [];
Object.defineProperty(arrayWithSymbol, arraySymbol, {
  value: 1,
  configurable: false,
});
var arraySymbolProxy = new Proxy(arrayWithSymbol, {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() {
  Reflect.deleteProperty(arraySymbolProxy, arraySymbol);
})) failures |= 8388608;

var nonExtensibleArray = [];
nonExtensibleArray.configurable = 1;
Object.preventExtensions(nonExtensibleArray);
var nonExtensibleArrayProxy = new Proxy(nonExtensibleArray, {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() {
  Reflect.deleteProperty(nonExtensibleArrayProxy, "configurable");
})) failures |= 16777216;

var boxedStringProxy = new Proxy(new String("xy"), {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() {
  Reflect.deleteProperty(boxedStringProxy, "0");
})) failures |= 33554432;

function PrototypeTarget() {}
var prototypeProxy = new Proxy(PrototypeTarget, {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() {
  Reflect.deleteProperty(prototypeProxy, "prototype");
})) failures |= 67108864;

var allFalseArguments = (function() { return arguments; })(1);
Object.defineProperty(allFalseArguments, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: false,
});
var allFalseArgumentsProxy = new Proxy(allFalseArguments, {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() {
  Reflect.deleteProperty(allFalseArgumentsProxy, "length");
})) failures |= 134217728;

var fixedArguments = (function() { return arguments; })(1);
Object.defineProperty(fixedArguments, "0", { configurable: false });
var fixedArgumentsProxy = new Proxy(fixedArguments, {
  deleteProperty: function() {
    return true;
  },
});
if (!throwsTypeError(function() {
  Reflect.deleteProperty(fixedArgumentsProxy, "0");
})) failures |= 268435456;

var absentTarget = [];
Object.preventExtensions(absentTarget);
var absentProxy = new Proxy(absentTarget, {
  deleteProperty: function() {
    return true;
  },
});
if (Reflect.deleteProperty(absentProxy, "missing") !== true) failures |= 536870912;

failures === 0;
