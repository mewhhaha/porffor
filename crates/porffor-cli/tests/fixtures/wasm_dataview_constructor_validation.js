__porfAssertThrows(TypeError, function () {
  new DataView(1);
});

__porfAssertThrows(TypeError, function () {
  new DataView({});
});

let buffer = new ArrayBuffer(8);

__porfAssertThrows(RangeError, function () {
  new DataView(buffer, -1);
});

__porfAssertThrows(RangeError, function () {
  new DataView(buffer, 9);
});

__porfAssertThrows(RangeError, function () {
  new DataView(buffer, 4, 5);
});

let view = new DataView(buffer, 2, 3);
view.byteOffset + view.byteLength;
