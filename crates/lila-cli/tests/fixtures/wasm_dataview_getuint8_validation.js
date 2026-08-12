let view = new DataView(new ArrayBuffer(4), 1, 2);
let get = DataView.prototype.getUint8;
view.setUint8(0, 4);
view.setUint8(1, 5);

__lilaAssertThrows(TypeError, function () {
  get.call({}, 0);
});

__lilaAssertThrows(TypeError, function () {
  get.call(1, 0);
});

__lilaAssertThrows(RangeError, function () {
  view.getUint8(2);
});

__lilaAssertThrows(RangeError, function () {
  view.getUint8(-1);
});

let detachedBuffer = new ArrayBuffer(1);
let detachedView = new DataView(detachedBuffer);
__lilaDetachArrayBuffer(detachedBuffer);

__lilaAssertThrows(TypeError, function () {
  detachedView.getUint8(0);
});

view.getUint8(0) + view.getUint8(1) + view.byteOffset + view.byteLength;
