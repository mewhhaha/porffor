let view = new DataView(new ArrayBuffer(4), 1, 2);
let get = DataView.prototype.getInt8;
let set = DataView.prototype.setInt8;

set.call(view, 0, 255);
view.setInt8(1, -2);

__porfAssertThrows(TypeError, function () {
  get.call({}, 0);
});

__porfAssertThrows(TypeError, function () {
  set.call({}, 0, 1);
});

__porfAssertThrows(RangeError, function () {
  view.getInt8(2);
});

__porfAssertThrows(RangeError, function () {
  view.setInt8(2, 1);
});

let detachedBuffer = new ArrayBuffer(1);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(TypeError, function () {
  detachedView.getInt8(0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setInt8(0, 1);
});

view.getInt8(0) * 10 + view.getInt8(1);
