let buffer = new ArrayBuffer(12);
let view = new DataView(buffer, 1, 8);

view.setUint32(0, 0x12345678);
let big = view.getUint32(0);
let little = view.getUint32(0, true);

view.setUint32(4, -1, true);
let wrapped = view.getUint32(4, true);

view.setUint32(4, 4294967296);
let zero = view.getUint32(4);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.getUint32.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setUint32.call(1, 0, 1);
});

__porfAssertThrows(RangeError, function () {
  view.getUint32(5);
});

__porfAssertThrows(RangeError, function () {
  view.setUint32(-1, 1);
});

let detachedBuffer = new ArrayBuffer(4);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.getUint32(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setUint32(0, 1);
});

big + little + wrapped + zero;
