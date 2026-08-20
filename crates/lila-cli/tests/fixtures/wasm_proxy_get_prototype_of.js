var failures = 0;

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

var proto = { marker: 1 };
var target = {};
var seenTarget = null;
var seenHandler = null;
var handler = {
  getPrototypeOf: function(t) {
    seenTarget = t;
    seenHandler = this;
    return proto;
  },
};
var proxy = new Proxy(target, handler);

if (Object.getPrototypeOf(proxy) !== proto) failures |= 1;
if (seenTarget !== target) failures |= 2;
if (seenHandler !== handler) failures |= 4;

var targetProto = { stable: true };
var fixedTarget = Object.create(targetProto);
var fixedProxy = new Proxy(fixedTarget, {
  getPrototypeOf: function() {
    return {};
  },
});
Object.preventExtensions(fixedTarget);
if (!throwsTypeError(function() { Object.getPrototypeOf(fixedProxy); })) failures |= 8;

var primitiveProxy = new Proxy({}, {
  getPrototypeOf: function() {
    return 1;
  },
});
if (!throwsTypeError(function() { Object.getPrototypeOf(primitiveProxy); })) failures |= 16;

var innerProto = { inner: true };
var innerTarget = Object.create(innerProto);
var innerProxy = new Proxy(innerTarget, {
  getPrototypeOf: function() {
    return innerProto;
  },
});
var outerProxy = new Proxy(innerProxy, {
  getPrototypeOf: undefined,
});
if (Object.getPrototypeOf(outerProxy) !== innerProto) failures |= 32;

var revoked = Proxy.revocable({}, {});
revoked.revoke();
if (!throwsTypeError(function() { Object.getPrototypeOf(revoked.proxy); })) failures |= 64;

function Custom() {}

var instanceProxy = new Proxy({}, {
  getPrototypeOf: function() {
    return Custom.prototype;
  },
});
if (!(instanceProxy instanceof Custom)) failures |= 128;

var instanceTarget = {};
var badInstanceProxy = new Proxy(instanceTarget, {
  getPrototypeOf: function() {
    return Custom.prototype;
  },
});
Object.preventExtensions(instanceTarget);
if (!throwsTypeError(function() { return badInstanceProxy instanceof Custom; })) failures |= 256;

var taggedProto = { tagged: true };
var taggedCalls = [];

function taggedGetPrototypeOf(label, expectedHandler) {
  return function() {
    taggedCalls.push(label + ":" + (this === expectedHandler));
    return taggedProto;
  };
}

function functionHandler() {}
functionHandler.getPrototypeOf = taggedGetPrototypeOf("function", functionHandler);
if (Object.getPrototypeOf(new Proxy({}, functionHandler)) !== taggedProto) failures |= 512;

var arrayHandler = [];
arrayHandler.getPrototypeOf = taggedGetPrototypeOf("array", arrayHandler);
if (Reflect.getPrototypeOf(new Proxy({}, arrayHandler)) !== taggedProto) failures |= 1024;

var argumentsHandler = (function() { return arguments; })();
argumentsHandler.getPrototypeOf = taggedGetPrototypeOf("arguments", argumentsHandler);
if (Object.getPrototypeOf(new Proxy({}, argumentsHandler)) !== taggedProto) failures |= 2048;

var nestedHandler;
var nestedLookupHandler = Object.create({
  get: function(_target, key) {
    taggedCalls.push("nested-get:" + key);
    return taggedGetPrototypeOf("nested", nestedHandler);
  },
});
nestedHandler = new Proxy({}, nestedLookupHandler);
if (Reflect.getPrototypeOf(new Proxy({}, nestedHandler)) !== taggedProto) failures |= 4096;

var abruptMarker = {};
var abruptLookupHandler = {};
Object.defineProperty(abruptLookupHandler, "get", {
  get: function() {
    throw abruptMarker;
  },
});
var abruptHandler = new Proxy({}, abruptLookupHandler);
var abruptObserved = false;
try {
  Object.getPrototypeOf(new Proxy({}, abruptHandler));
} catch (error) {
  abruptObserved = error === abruptMarker;
}
if (!abruptObserved) failures |= 8192;

var expectedTaggedCalls =
  "function:true,array:true,arguments:true,nested-get:getPrototypeOf,nested:true";
if (taggedCalls.join(",") !== expectedTaggedCalls) failures |= 16384;

failures === 0;
