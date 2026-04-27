let buffer = new ArrayBuffer(24);
let view = new DataView(buffer, 1, 18);

view.setFloat16(0, 1.5);
let big = view.getFloat16(0);

view.setFloat16(2, -2.25, true);
let little = view.getFloat16(2, true);

view.setFloat16(4, Infinity);
let positiveInfinity = view.getFloat16(4);

view.setFloat16(4, -Infinity, true);
let negativeInfinity = view.getFloat16(4, true);

view.setFloat16(6, NaN);
let nanValue = view.getFloat16(6);

view.setFloat16(8, -0);
let minusZero = view.getFloat16(8);

view.setFloat16(10, 0.000000059604644775390625);
let subnormal = view.getFloat16(10);

view.setFloat16(12, 65504);
let normal = view.getFloat16(12);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.getFloat16.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setFloat16.call(1, 0, 1);
});

__porfAssertThrows(RangeError, function () {
  view.getFloat16(17);
});

__porfAssertThrows(RangeError, function () {
  view.setFloat16(-1, 1);
});

let detachedBuffer = new ArrayBuffer(4);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.getFloat16(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setFloat16(0, 1);
});

(big == 1.5) +
  (little == -2.25) +
  (positiveInfinity == Infinity) +
  (negativeInfinity == -Infinity) +
  (nanValue != nanValue) +
  (1 / minusZero == -Infinity) +
  (subnormal == 0.000000059604644775390625) +
  (normal == 65504);
