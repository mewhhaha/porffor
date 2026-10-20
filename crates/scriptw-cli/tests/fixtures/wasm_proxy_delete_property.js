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

var handlerDispatchFailures = 0;
var handlerDispatchCalls = 0;
var dispatchTarget = {
  fromFunction: 1,
  fromArray: 2,
  fromArguments: 3,
  fromProxy: 4,
};
var dispatchSymbol = Symbol("delete handler dispatch");
dispatchTarget[dispatchSymbol] = 5;

function deleteTrap(expectedHandler, expectedKey) {
  return function(t, key) {
    handlerDispatchCalls++;
    if (this !== expectedHandler) handlerDispatchFailures++;
    if (t !== dispatchTarget) handlerDispatchFailures++;
    if (key !== expectedKey) handlerDispatchFailures++;
    return Reflect.deleteProperty(t, key);
  };
}

function functionHandler() {}
var functionHandlerPrototype = functionHandler.prototype;
var functionHandlerLookups = 0;
Object.defineProperty(functionHandler, "deleteProperty", {
  configurable: true,
  get: function() {
    functionHandlerLookups++;
    // Exact equality compares the tagged Function value. The prototype read is
    // an additional receiver-identity check, not the tag-retention witness.
    if (this !== functionHandler) handlerDispatchFailures++;
    if (this.prototype !== functionHandlerPrototype) handlerDispatchFailures++;
    return deleteTrap(functionHandler, "fromFunction");
  },
});
var functionHandlerProxy = new Proxy(dispatchTarget, functionHandler);
if (!delete functionHandlerProxy.fromFunction) handlerDispatchFailures++;
if (functionHandlerLookups !== 1) handlerDispatchFailures++;

var arrayHandler = [];
arrayHandler.deleteProperty = deleteTrap(arrayHandler, "fromArray");
if (!Reflect.deleteProperty(new Proxy(dispatchTarget, arrayHandler), "fromArray")) {
  handlerDispatchFailures++;
}

var argumentsHandler = (function() { return arguments; })();
argumentsHandler.deleteProperty = deleteTrap(argumentsHandler, "fromArguments");
var argumentsHandlerProxy = new Proxy(dispatchTarget, argumentsHandler);
if (!delete argumentsHandlerProxy.fromArguments) handlerDispatchFailures++;

var proxyHandler;
var proxyLookupHandler = Object.create({
  get: function(_target, key, receiver) {
    if (key !== "deleteProperty") handlerDispatchFailures++;
    if (receiver !== proxyHandler) handlerDispatchFailures++;
    return deleteTrap(proxyHandler, "fromProxy");
  },
});
proxyHandler = new Proxy({}, proxyLookupHandler);
if (!Reflect.deleteProperty(new Proxy(dispatchTarget, proxyHandler), "fromProxy")) {
  handlerDispatchFailures++;
}

var callableHandler;
var callableTrap = new Proxy(function() {}, {
  apply: function(_target, thisArg, args) {
    handlerDispatchCalls++;
    if (thisArg !== callableHandler) handlerDispatchFailures++;
    if (args[0] !== dispatchTarget) handlerDispatchFailures++;
    if (args[1] !== dispatchSymbol) handlerDispatchFailures++;
    return Reflect.deleteProperty(args[0], args[1]);
  },
});
callableHandler = { deleteProperty: callableTrap };
if (!Reflect.deleteProperty(new Proxy(dispatchTarget, callableHandler), dispatchSymbol)) {
  handlerDispatchFailures++;
}

if (handlerDispatchCalls !== 5) handlerDispatchFailures++;
if (Object.prototype.hasOwnProperty.call(dispatchTarget, "fromFunction")) {
  handlerDispatchFailures++;
}
if (Object.prototype.hasOwnProperty.call(dispatchTarget, "fromArray")) {
  handlerDispatchFailures++;
}
if (Object.prototype.hasOwnProperty.call(dispatchTarget, "fromArguments")) {
  handlerDispatchFailures++;
}
if (Object.prototype.hasOwnProperty.call(dispatchTarget, "fromProxy")) {
  handlerDispatchFailures++;
}
if (Object.prototype.hasOwnProperty.call(dispatchTarget, dispatchSymbol)) {
  handlerDispatchFailures++;
}

var lookupMarker = {};
var abruptLookupTarget = { keptAfterLookupThrow: 1 };
var abruptLookupHandler = new Proxy({}, {
  get: function(_target, key) {
    if (key === "deleteProperty") throw lookupMarker;
  },
});
var lookupThrowObserved = false;
try {
  Reflect.deleteProperty(
    new Proxy(abruptLookupTarget, abruptLookupHandler),
    "keptAfterLookupThrow"
  );
} catch (error) {
  lookupThrowObserved = error === lookupMarker;
}
if (!lookupThrowObserved) handlerDispatchFailures++;
if (!Object.prototype.hasOwnProperty.call(abruptLookupTarget, "keptAfterLookupThrow")) {
  handlerDispatchFailures++;
}

var trapCallMarker = {};
var trapCallCount = 0;
var abruptTrapTarget = {};
Object.defineProperty(abruptTrapTarget, "fixedAfterTrapThrow", {
  value: 1,
  configurable: false,
});
Object.preventExtensions(abruptTrapTarget);
var abruptTrapHandler;
var abruptCallableTrap = new Proxy(function() {}, {
  apply: function(_target, thisArg, args) {
    trapCallCount++;
    if (thisArg !== abruptTrapHandler) handlerDispatchFailures++;
    if (args[0] !== abruptTrapTarget) handlerDispatchFailures++;
    if (args[1] !== "fixedAfterTrapThrow") handlerDispatchFailures++;
    throw trapCallMarker;
  },
});
abruptTrapHandler = { deleteProperty: abruptCallableTrap };
var trapCallThrowObserved = false;
try {
  Reflect.deleteProperty(
    new Proxy(abruptTrapTarget, abruptTrapHandler),
    "fixedAfterTrapThrow"
  );
} catch (error) {
  trapCallThrowObserved = error === trapCallMarker;
}
if (!trapCallThrowObserved) handlerDispatchFailures++;
if (trapCallCount !== 1) handlerDispatchFailures++;
if (!Object.prototype.hasOwnProperty.call(abruptTrapTarget, "fixedAfterTrapThrow")) {
  handlerDispatchFailures++;
}

var absentLookupCalls = 0;
var forwardedDeleteCalls = 0;
var forwardedTarget = { forwarded: 1 };
var nestedDeleteTarget = new Proxy(forwardedTarget, {
  deleteProperty: function(t, key) {
    forwardedDeleteCalls++;
    return Reflect.deleteProperty(t, key);
  },
});
var absentProxyHandler = new Proxy({}, {
  get: function(_target, key) {
    if (key !== "deleteProperty") handlerDispatchFailures++;
    absentLookupCalls++;
    return undefined;
  },
});
if (!Reflect.deleteProperty(new Proxy(nestedDeleteTarget, absentProxyHandler), "forwarded")) {
  handlerDispatchFailures++;
}
if (absentLookupCalls !== 1) handlerDispatchFailures++;
if (forwardedDeleteCalls !== 1) handlerDispatchFailures++;
if (Object.prototype.hasOwnProperty.call(forwardedTarget, "forwarded")) {
  handlerDispatchFailures++;
}

var deepDeleteOrder = 0;
function nullishDeleteHandler(marker, trap) {
  var handler = {};
  Object.defineProperty(handler, "deleteProperty", {
    get: function() {
      deepDeleteOrder = deepDeleteOrder * 10 + marker;
      return trap;
    },
  });
  return handler;
}

var deepDeleteCalls = 0;
var deepDeleteTarget = { forwardedAcrossSixProxies: 1 };
var deepDeleteProxy = new Proxy(deepDeleteTarget, {
  deleteProperty: function(target, key) {
    deepDeleteOrder = deepDeleteOrder * 10 + 7;
    deepDeleteCalls++;
    return Reflect.deleteProperty(target, key);
  },
});
deepDeleteProxy = new Proxy(deepDeleteProxy, nullishDeleteHandler(1, undefined));
deepDeleteProxy = new Proxy(deepDeleteProxy, nullishDeleteHandler(2, null));
deepDeleteProxy = new Proxy(deepDeleteProxy, nullishDeleteHandler(3, undefined));
deepDeleteProxy = new Proxy(deepDeleteProxy, nullishDeleteHandler(4, null));
deepDeleteProxy = new Proxy(deepDeleteProxy, nullishDeleteHandler(5, undefined));
deepDeleteProxy = new Proxy(deepDeleteProxy, nullishDeleteHandler(6, null));
if (!Reflect.deleteProperty(deepDeleteProxy, "forwardedAcrossSixProxies")) {
  handlerDispatchFailures++;
}
if (deepDeleteOrder !== 6543217) handlerDispatchFailures++;
if (deepDeleteCalls !== 1) handlerDispatchFailures++;
if (Object.prototype.hasOwnProperty.call(
  deepDeleteTarget,
  "forwardedAcrossSixProxies"
)) {
  handlerDispatchFailures++;
}

failures === 0 && handlerDispatchFailures === 0;
