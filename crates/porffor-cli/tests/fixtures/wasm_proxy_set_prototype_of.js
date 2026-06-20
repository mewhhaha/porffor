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

var reflectSetPrototypeOfDesc = Object.getOwnPropertyDescriptor(Reflect, "setPrototypeOf");
if (typeof Reflect.setPrototypeOf !== "function") failures |= 32768;
if (reflectSetPrototypeOfDesc.value !== Reflect.setPrototypeOf) failures |= 65536;
if (reflectSetPrototypeOfDesc.writable !== true) failures |= 131072;
if (reflectSetPrototypeOfDesc.enumerable !== false) failures |= 262144;
if (reflectSetPrototypeOfDesc.configurable !== true) failures |= 524288;

var reflectSetPrototypeOfLengthDesc = Object.getOwnPropertyDescriptor(Reflect.setPrototypeOf, "length");
if (Reflect.setPrototypeOf.length !== 2) failures |= 1048576;
if (reflectSetPrototypeOfLengthDesc.value !== 2) failures |= 2097152;
if (reflectSetPrototypeOfLengthDesc.writable !== false) failures |= 4194304;
if (reflectSetPrototypeOfLengthDesc.enumerable !== false) failures |= 8388608;
if (reflectSetPrototypeOfLengthDesc.configurable !== true) failures |= 16777216;

var reflectSetPrototypeOfNameDesc = Object.getOwnPropertyDescriptor(Reflect.setPrototypeOf, "name");
if (Reflect.setPrototypeOf.name !== "setPrototypeOf") failures |= 33554432;
if (reflectSetPrototypeOfNameDesc.value !== "setPrototypeOf") failures |= 67108864;
if (reflectSetPrototypeOfNameDesc.writable !== false) failures |= 134217728;
if (reflectSetPrototypeOfNameDesc.enumerable !== false) failures |= 268435456;
if (reflectSetPrototypeOfNameDesc.configurable !== true) failures |= 536870912;

var seenTarget = null;
var seenProto = null;
var seenHandler = null;
var handler = {
  setPrototypeOf: function(t, p) {
    seenTarget = t;
    seenProto = p;
    seenHandler = this;
    Object.setPrototypeOf(t, p);
    return true;
  },
};
var proxy = new Proxy(target, handler);

if (Object.setPrototypeOf(proxy, proto) !== proxy) failures |= 1;
if (Object.getPrototypeOf(target) !== proto) failures |= 2;
if (seenTarget !== target) failures |= 4;
if (seenProto !== proto) failures |= 8;
if (seenHandler !== handler) failures |= 16;

var falseProxy = new Proxy({}, {
  setPrototypeOf: function() {
    return 0;
  },
});
if (Reflect.setPrototypeOf(falseProxy, proto) !== false) failures |= 32;
if (!throwsTypeError(function() { Object.setPrototypeOf(falseProxy, proto); })) failures |= 64;

var fixedProto = { fixed: true };
var fixedTarget = Object.create(fixedProto);
var fixedProxy = new Proxy(fixedTarget, {
  setPrototypeOf: function() {
    return true;
  },
});
Object.preventExtensions(fixedTarget);
if (Reflect.setPrototypeOf(fixedProxy, fixedProto) !== true) failures |= 128;
if (!throwsTypeError(function() { Reflect.setPrototypeOf(fixedProxy, {}); })) failures |= 256;

var nestedTarget = {};
var innerProxy = new Proxy(nestedTarget, {
  setPrototypeOf: function(t, p) {
    Object.setPrototypeOf(t, p);
    return true;
  },
});
var outerProxy = new Proxy(innerProxy, {
  setPrototypeOf: null,
});
var nestedProto = { nested: true };
if (Object.setPrototypeOf(outerProxy, nestedProto) !== outerProxy) failures |= 512;
if (Object.getPrototypeOf(nestedTarget) !== nestedProto) failures |= 1024;

var cycleTarget = {};
var directCycleProto = Object.create(cycleTarget);
if (!throwsTypeError(function() { Object.setPrototypeOf(cycleTarget, directCycleProto); })) failures |= 16384;
Object.setPrototypeOf(cycleTarget, null);
var cycleInnerProxy = new Proxy(cycleTarget, {});
var cycleOuterProxy = new Proxy(cycleInnerProxy, {
  setPrototypeOf: null,
});
Object.setPrototypeOf(cycleOuterProxy, null);
var cycleProto = Object.create(cycleTarget);
if (Object.getPrototypeOf(cycleProto) !== cycleTarget) failures |= 8192;
if (!throwsTypeError(function() { Object.setPrototypeOf(cycleOuterProxy, cycleProto); })) failures |= 2048;

var revoked = Proxy.revocable({}, {});
revoked.revoke();
if (!throwsTypeError(function() { Object.setPrototypeOf(revoked.proxy, null); })) failures |= 4096;

failures === 0;
