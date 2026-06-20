var failures = 0;

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

var falseTarget = {};
var falseProxy = new Proxy(falseTarget, {
  defineProperty: function() {
    return 0;
  },
});
if (Reflect.defineProperty(falseProxy, "attr", { value: 1 }) !== false) failures |= 1;
if (falseTarget.attr !== undefined) failures |= 2;
if (!throwsTypeError(function() { Object.defineProperty(falseProxy, "attr", { value: 1 }); })) failures |= 4;

var fallbackTarget = {};
var fallbackProxy = new Proxy(fallbackTarget, {});
Object.defineProperty(fallbackProxy, "x", {
  configurable: true,
  enumerable: true,
  writable: true,
  value: 7,
});
if (fallbackTarget.x !== 7) failures |= 8;

var nestedTarget = {};
var nestedInner = new Proxy(nestedTarget, {});
var nestedOuterNull = new Proxy(nestedInner, {
  defineProperty: null,
});
if (Reflect.defineProperty(nestedOuterNull, "y", { value: 9 }) !== true) failures |= 16;
if (nestedTarget.y !== 9) failures |= 32;

var arrayTarget = [];
var arrayTargetProxy = new Proxy(arrayTarget, {});
var arrayOuterUndefined = new Proxy(arrayTargetProxy, {
  defineProperty: undefined,
});
Object.defineProperty(arrayOuterUndefined, "0", { value: 1 });
if (arrayTarget.length !== 1) failures |= 65536;
if (arrayTarget[0] !== 1) failures |= 131072;
if (!throwsTypeError(function() { Object.defineProperty(arrayOuterUndefined, "length", { get: function() {} }); })) failures |= 262144;

var stringTargetObject = new String("str");
var stringTargetProxy = new Proxy(stringTargetObject, {});
var stringOuterProxy = new Proxy(stringTargetProxy, {});
if (Reflect.defineProperty(stringOuterProxy, "4", { value: 4 }) !== true) failures |= 1024;
if (stringTargetObject[4] !== 4) failures |= 2048;
if (!throwsTypeError(function() { Object.defineProperty(stringOuterProxy, "0", { value: "x" }); })) failures |= 16384;

Object.preventExtensions(stringTargetObject);
if (Reflect.defineProperty(stringOuterProxy, "foo", { value: 5 }) !== false) failures |= 4096;

var functionTarget = function() {};
var functionTargetProxy = new Proxy(functionTarget, {});
var functionOuterProxy = new Proxy(functionTargetProxy, {});
Object.defineProperty(functionOuterProxy, "name", { value: "foo" });
if (functionTarget.name !== "foo") failures |= 8192;
if (!throwsTypeError(function() { Object.defineProperty(functionOuterProxy, "prototype", { set: function(_value) {} }); })) failures |= 32768;

var nonCallableProxy = new Proxy({}, {
  defineProperty: 1,
});
if (!throwsTypeError(function() { Object.defineProperty(nonCallableProxy, "z", { value: 1 }); })) failures |= 64;

var callTarget = {};
var handlerThis;
var callKey;
var callDescValue;
var callProxy = new Proxy(callTarget, {
  defineProperty: function(target, key, desc) {
    handlerThis = this;
    callKey = key;
    callDescValue = desc.value;
    return Reflect.defineProperty(target, key, desc);
  },
});
if (Reflect.defineProperty(callProxy, "called", { value: 11 }) !== true) failures |= 128;
if (handlerThis === undefined || callKey !== "called" || callDescValue !== 11) failures |= 256;
if (callTarget.called !== 11) failures |= 512;

var reflectInvariantCalls = 0;
var reflectInvariantProxy = new Proxy({}, {
  defineProperty: function(target, key, desc) {
    Object.defineProperty(target, key, {
      configurable: false,
      writable: true,
    });
    reflectInvariantCalls++;
    return true;
  },
});
if (!throwsTypeError(function() { Reflect.defineProperty(reflectInvariantProxy, "fixed", { writable: false }); })) failures |= 524288;
if (reflectInvariantCalls !== 1) failures |= 1048576;

var assignmentDesc;
var assignmentProxy = new Proxy({}, {
  defineProperty: function(target, key, desc) {
    assignmentDesc = desc;
    return true;
  },
});
assignmentProxy.assigned = 23;
if (assignmentDesc === undefined) failures |= 2097152;
if (Object.getPrototypeOf(assignmentDesc) !== Object.prototype) failures |= 4194304;
if (assignmentDesc.value !== 23) failures |= 8388608;
if (assignmentDesc.writable !== true) failures |= 16777216;
if (assignmentDesc.enumerable !== true) failures |= 33554432;
if (assignmentDesc.configurable !== true) failures |= 67108864;

var revokedAssignment = Proxy.revocable(Object.create(null), {});
revokedAssignment.revoke();
if (!throwsTypeError(function() { revokedAssignment.proxy.prop = null; })) failures |= 134217728;

failures === 0;
