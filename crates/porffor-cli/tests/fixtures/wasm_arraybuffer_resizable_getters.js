let maxDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "maxByteLength");
if (maxDesc.set !== undefined) throw "maxByteLength setter";

let resizableDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "resizable");
if (resizableDesc.set !== undefined) throw "resizable setter";

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
  resizableDesc.get.call(1);
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call([]);
});

__porfAssertThrows(TypeError, function () {
  resizableDesc.get.call(new DataView(new ArrayBuffer(1)));
});

123;
