var failures = 0;

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

var target = { foo: 1, bar: 2 };
Object.defineProperty(target, "hidden", {
  value: 3,
  enumerable: false
});

var handlerThis;
var trapTarget;
var proxy = new Proxy(target, {
  ownKeys: function(targetArg) {
    handlerThis = this;
    trapTarget = targetArg;
    return ["foo", "hidden", "bar"];
  }
});

var keys = Object.keys(proxy);

if (keys.length !== 2) failures |= 1;
if (keys[0] !== "foo") failures |= 2;
if (keys[1] !== "bar") failures |= 4;
if (handlerThis === undefined) failures |= 8;
if (trapTarget !== target) failures |= 16;

var duplicateProxy = new Proxy({}, {
  ownKeys: function() {
    return ["dup", "dup"];
  }
});
if (!throwsTypeError(function() { Object.keys(duplicateProxy); })) failures |= 32;

var invalidEntryProxy = new Proxy({}, {
  ownKeys: function() {
    return [1];
  }
});
if (!throwsTypeError(function() { Object.keys(invalidEntryProxy); })) failures |= 64;

var invalidResultProxy = new Proxy({}, {
  ownKeys: function() {
    return undefined;
  }
});
if (!throwsTypeError(function() { Object.keys(invalidResultProxy); })) failures |= 128;

var nestedTarget = new Proxy({ a: 1, b: 2 }, {});
var nestedProxy = new Proxy(nestedTarget, {
  ownKeys: null
});
var nestedKeys = Object.keys(nestedProxy);
if (nestedKeys.length !== 2) failures |= 256;
if (nestedKeys[0] !== "a") failures |= 512;
if (nestedKeys[1] !== "b") failures |= 1024;

var symbolKey = Symbol();
var symbolTarget = {};
symbolTarget[symbolKey] = 4;
var symbolHandlerThis;
var symbolTrapTarget;
var symbolProxy = new Proxy(symbolTarget, {
  ownKeys: function(targetArg) {
    symbolHandlerThis = this;
    symbolTrapTarget = targetArg;
    return ["ignored", symbolKey];
  }
});
var symbolKeys = Object.getOwnPropertySymbols(symbolProxy);
if (symbolKeys.length !== 1) failures |= 2048;
if (symbolKeys[0] !== symbolKey) failures |= 4096;
if (symbolHandlerThis === undefined) failures |= 8192;
if (symbolTrapTarget !== symbolTarget) failures |= 16384;

var reflectSymbol = Symbol();
var reflectTrapResult = [reflectSymbol, "length", "foo", "0"];
var reflectTarget = new Proxy([], {
  ownKeys: function() {
    return reflectTrapResult;
  }
});
var reflectProxy = new Proxy(reflectTarget, {
  ownKeys: undefined
});
var reflectKeys = Reflect.ownKeys(reflectProxy);
if (reflectKeys.length !== 4) failures |= 32768;
if (reflectKeys[0] !== reflectSymbol) failures |= 65536;
if (reflectKeys[1] !== "length") failures |= 131072;
if (reflectKeys[2] !== "foo") failures |= 262144;
if (reflectKeys[3] !== "0") failures |= 524288;

var boxedSymbol = Symbol();
var boxedString = new String("str");
boxedString[boxedSymbol] = 5;
var boxedKeys = Reflect.ownKeys(new Proxy(new Proxy(boxedString, {}), {}));
if (boxedKeys.length !== 5) failures |= 1048576;
if (boxedKeys[0] !== "0") failures |= 2097152;
if (boxedKeys[1] !== "1") failures |= 4194304;
if (boxedKeys[2] !== "2") failures |= 8388608;
if (boxedKeys[3] !== "length") failures |= 16777216;
if (boxedKeys[4] !== boxedSymbol) failures |= 33554432;

failures === 0;
