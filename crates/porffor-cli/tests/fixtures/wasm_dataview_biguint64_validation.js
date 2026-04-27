let buffer = new ArrayBuffer(40);
let view = new DataView(buffer, 1, 32);

let setResult = view.setBigUint64(0, 0xffffffffffffffffn);
let maxValue = view.getBigUint64(0);

view.setBigUint64(8, 0x0102030405060708n);
let bigEndian = view.getBigUint64(8);
let reversed = view.getBigUint64(8, true);

view.setBigUint64(16, 0x0102030405060708n, true);
let littleEndian = view.getBigUint64(16, true);
let customOffset = view.getBigUint64(16);

view.setBigUint64(24, -1n);
let wrappedNegative = view.getBigUint64(24);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.getBigUint64.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setBigUint64.call(1, 0, 1n);
});

__porfAssertThrows(RangeError, function () {
  view.getBigUint64(25);
});

__porfAssertThrows(RangeError, function () {
  view.setBigUint64(-1, 1n);
});

__porfAssertThrows(TypeError, function () {
  view.setBigUint64(0, 1);
});

let detachedBuffer = new ArrayBuffer(8);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.getBigUint64(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setBigUint64(0, 1n);
});

(setResult === undefined) +
  (maxValue === 0xffffffffffffffffn) +
  (bigEndian === 0x0102030405060708n) +
  (reversed === 0x0807060504030201n) +
  (littleEndian === 0x0102030405060708n) +
  (customOffset === 0x0807060504030201n) +
  (wrappedNegative === 0xffffffffffffffffn);
