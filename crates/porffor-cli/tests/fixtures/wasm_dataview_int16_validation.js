let buffer = new ArrayBuffer(6);
let view = new DataView(buffer, 1, 4);

view.setInt16(0, -1);
let signed = view.getInt16(0);
let unsigned = view.getUint16(0);

view.setInt16(2, -32768, true);
let littleSigned = view.getInt16(2, true);

__porfAssertThrows(TypeError, function () {
  DataView.prototype.getInt16.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  DataView.prototype.setInt16.call(1, 0, 1);
});

__porfAssertThrows(RangeError, function () {
  view.getInt16(3);
});

__porfAssertThrows(RangeError, function () {
  view.setInt16(-1, 1);
});

let detachedBuffer = new ArrayBuffer(2);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.getInt16(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setInt16(0, 1);
});

signed + unsigned + littleSigned;
