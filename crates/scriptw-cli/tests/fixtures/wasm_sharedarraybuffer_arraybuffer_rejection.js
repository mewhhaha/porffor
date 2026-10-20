let sab = new SharedArrayBuffer(4);

let byteLength = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "byteLength");
let detached = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "detached");
let maxByteLength = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "maxByteLength");
let resizable = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "resizable");

__scriptwAssertThrows(TypeError, function () {
  byteLength.get.call(sab);
});
__scriptwAssertThrows(TypeError, function () {
  Object.defineProperties(sab, { byteLength: byteLength });
  sab.byteLength;
});
__scriptwAssertThrows(TypeError, function () {
  detached.get.call(sab);
});
__scriptwAssertThrows(TypeError, function () {
  Object.defineProperties(sab, { detached: detached });
  sab.detached;
});
__scriptwAssertThrows(TypeError, function () {
  maxByteLength.get.call(sab);
});
__scriptwAssertThrows(TypeError, function () {
  Object.defineProperties(sab, { maxByteLength: maxByteLength });
  sab.maxByteLength;
});
__scriptwAssertThrows(TypeError, function () {
  resizable.get.call(sab);
});
__scriptwAssertThrows(TypeError, function () {
  Object.defineProperties(sab, { resizable: resizable });
  sab.resizable;
});

__scriptwAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.resize.call(sab, 0);
});
__scriptwAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.slice.call(sab, 0);
});
__scriptwAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.transfer.call(sab);
});
__scriptwAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.transferToFixedLength.call(sab);
});

123;
