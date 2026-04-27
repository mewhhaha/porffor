let buffer = new ArrayBuffer(32);
let view = new DataView(buffer, 1, 24);

view.setBigUint64(0, 0x0102030405060708n);
let bigEndian = view.getBigUint64(0);
let reversed = view.getBigUint64(0, true);

view.setBigInt64(8, -1n, true);
let signedLittle = view.getBigInt64(8, true);
let unsignedLittle = view.getBigUint64(8, true);

view.setBigUint64(16, BigInt(255));
let customOffset = view.getBigUint64(16);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.getBigInt64.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setBigInt64.call(1, 0, 1n);
});

__porfAssertThrows(RangeError, function () {
  view.getBigUint64(17);
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
  detachedView.getBigInt64(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setBigInt64(0, 1n);
});

(bigEndian === 0x0102030405060708n) +
  (reversed === 0x0807060504030201n) +
  (signedLittle === -1n) +
  (unsignedLittle === 0xffffffffffffffffn) +
  (customOffset === 255n);
