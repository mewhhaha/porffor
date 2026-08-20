let sab = new SharedArrayBuffer(4);

let byteLength = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "byteLength");
let detached = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "detached");
let maxByteLength = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "maxByteLength");
let resizable = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "resizable");

__lilaAssertThrows(TypeError, function () {
  byteLength.get.call(sab);
});
__lilaAssertThrows(TypeError, function () {
  Object.defineProperties(sab, { byteLength: byteLength });
  sab.byteLength;
});
__lilaAssertThrows(TypeError, function () {
  detached.get.call(sab);
});
__lilaAssertThrows(TypeError, function () {
  Object.defineProperties(sab, { detached: detached });
  sab.detached;
});
__lilaAssertThrows(TypeError, function () {
  maxByteLength.get.call(sab);
});
__lilaAssertThrows(TypeError, function () {
  Object.defineProperties(sab, { maxByteLength: maxByteLength });
  sab.maxByteLength;
});
__lilaAssertThrows(TypeError, function () {
  resizable.get.call(sab);
});
__lilaAssertThrows(TypeError, function () {
  Object.defineProperties(sab, { resizable: resizable });
  sab.resizable;
});

__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.resize.call(sab, 0);
});
__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.slice.call(sab, 0);
});
__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.transfer.call(sab);
});
__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.transferToFixedLength.call(sab);
});

123;
