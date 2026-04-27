let buffer = new ArrayBuffer(12);
let view = new DataView(buffer, 1, 8);

view.setInt32(0, -1);
let signed = view.getInt32(0);
let unsigned = view.getUint32(0);

view.setInt32(4, -2147483648, true);
let littleSigned = view.getInt32(4, true);
let littleUnsigned = view.getUint32(4, true);

view.setInt32(4, 2147483648);
let bigSigned = view.getInt32(4);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.getInt32.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setInt32.call(1, 0, 1);
});

__porfAssertThrows(RangeError, function () {
  view.getInt32(5);
});

__porfAssertThrows(RangeError, function () {
  view.setInt32(-1, 1);
});

let detachedBuffer = new ArrayBuffer(4);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.getInt32(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setInt32(0, 1);
});

signed + unsigned + littleSigned + littleUnsigned + bigSigned;
