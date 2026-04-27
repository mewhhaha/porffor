let buffer = new ArrayBuffer(6);
let view = new DataView(buffer, 1, 4);

view.setUint8(0);
view.setUint8(1, -1);
view.setUint8(2, 256);
view.setUint8(3, Infinity);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setUint8.call({}, 0, 1);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setUint8.call(1, 0, 1);
});

__porfAssertThrows(RangeError, function () {
  view.setUint8(4, 1);
});

__porfAssertThrows(RangeError, function () {
  view.setUint8(-1, 1);
});

let detachedBuffer = new ArrayBuffer(1);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.setUint8(0, 1);
});

view.getUint8(0) + view.getUint8(1) + view.getUint8(2) + view.getUint8(3);
