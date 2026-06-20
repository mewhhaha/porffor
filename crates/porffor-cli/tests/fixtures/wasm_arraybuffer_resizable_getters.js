let maxDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "maxByteLength");
if (maxDesc.set !== undefined) throw "maxByteLength setter";

let resizableDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "resizable");
if (resizableDesc.set !== undefined) throw "resizable setter";
if (typeof resizableDesc.get !== "function") throw "resizable getter";
if (resizableDesc.get.length !== 0) throw "resizable getter length";
if (resizableDesc.get.name !== "get resizable") throw "resizable getter name";
if (resizableDesc.enumerable !== false) throw "resizable enumerable";
if (resizableDesc.configurable !== true) throw "resizable configurable";

let resizableLengthDesc = Object.getOwnPropertyDescriptor(resizableDesc.get, "length");
if (resizableLengthDesc.value !== 0) throw "resizable length value";
if (resizableLengthDesc.writable !== false) throw "resizable length writable";
if (resizableLengthDesc.enumerable !== false) throw "resizable length enumerable";
if (resizableLengthDesc.configurable !== true) throw "resizable length configurable";

let resizableNameDesc = Object.getOwnPropertyDescriptor(resizableDesc.get, "name");
if (resizableNameDesc.value !== "get resizable") throw "resizable name value";
if (resizableNameDesc.writable !== false) throw "resizable name writable";
if (resizableNameDesc.enumerable !== false) throw "resizable name enumerable";
if (resizableNameDesc.configurable !== true) throw "resizable name configurable";

let fixed = new ArrayBuffer(8);
if (fixed.maxByteLength !== 8) throw "fixed maxByteLength";
if (fixed.resizable !== false) throw "fixed resizable";

let growable = new ArrayBuffer(2, { maxByteLength: 10 });
if (growable.maxByteLength !== 10) throw "growable maxByteLength";
if (growable.resizable !== true) throw "growable resizable";

__porfDetachArrayBuffer(growable);
if (growable.maxByteLength !== 0) throw "detached maxByteLength";
if (growable.resizable !== true) throw "detached resizable";

__porfAssertThrows(TypeError, function () {
  maxDesc.get.call(undefined);
});

__porfAssertThrows(TypeError, function () {
  maxDesc.get.call(1);
});

__porfAssertThrows(TypeError, function () {
  maxDesc.get.call([]);
});

__porfAssertThrows(TypeError, function () {
  maxDesc.get.call(new DataView(new ArrayBuffer(1)));
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call(undefined);
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call(null);
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call(1);
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call("1");
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call(true);
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call(false);
});

let resizableSymbolReceiver = Symbol("s");
__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call(resizableSymbolReceiver);
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call([]);
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call(new DataView(new ArrayBuffer(1)));
});

123;
