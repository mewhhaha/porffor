let buffer = new ArrayBuffer(48);
let view = new DataView(buffer, 1, 32);

view.setFloat64(0, 1.5);
let big = view.getFloat64(0);

view.setFloat64(8, -2.25, true);
let little = view.getFloat64(8, true);

view.setFloat64(16, Infinity);
let positiveInfinity = view.getFloat64(16);

view.setFloat64(16, -Infinity, true);
let negativeInfinity = view.getFloat64(16, true);

view.setFloat64(24, NaN);
let nanValue = view.getFloat64(24);

view.setFloat64(0, -0);
let minusZero = view.getFloat64(0);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.getFloat64.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setFloat64.call(1, 0, 1);
});

__porfAssertThrows(RangeError, function () {
  view.getFloat64(25);
});

__porfAssertThrows(RangeError, function () {
  view.setFloat64(-1, 1);
});

let detachedBuffer = new ArrayBuffer(8);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.getFloat64(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setFloat64(0, 1);
});

(big == 1.5) +
  (little == -2.25) +
  (positiveInfinity == Infinity) +
  (negativeInfinity == -Infinity) +
  (nanValue != nanValue) +
  (1 / minusZero == -Infinity);
