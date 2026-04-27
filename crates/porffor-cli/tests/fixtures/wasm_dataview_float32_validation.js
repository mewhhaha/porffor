let buffer = new ArrayBuffer(24);
let view = new DataView(buffer, 1, 16);

view.setFloat32(0, 1.5);
let big = view.getFloat32(0);

view.setFloat32(4, -2.25, true);
let little = view.getFloat32(4, true);

view.setFloat32(8, Infinity);
let positiveInfinity = view.getFloat32(8);

view.setFloat32(8, -Infinity, true);
let negativeInfinity = view.getFloat32(8, true);

view.setFloat32(12, NaN);
let nanValue = view.getFloat32(12);

view.setFloat32(0, -0);
let minusZero = view.getFloat32(0);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.getFloat32.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setFloat32.call(1, 0, 1);
});

__porfAssertThrows(RangeError, function () {
  view.getFloat32(13);
});

__porfAssertThrows(RangeError, function () {
  view.setFloat32(-1, 1);
});

let detachedBuffer = new ArrayBuffer(4);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.getFloat32(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setFloat32(0, 1);
});

(big == 1.5) +
  (little == -2.25) +
  (positiveInfinity == Infinity) +
  (negativeInfinity == -Infinity) +
  (nanValue != nanValue) +
  (1 / minusZero == -Infinity);
