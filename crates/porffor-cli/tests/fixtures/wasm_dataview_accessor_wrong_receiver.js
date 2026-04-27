let desc = Object.getOwnPropertyDescriptor(DataView.prototype, "byteLength");

__porfAssertThrows(TypeError, function () {
  desc.get.call({});
});

__porfAssertThrows(TypeError, function () {
  desc.get.call(1);
});

let view = new DataView(new ArrayBuffer(6), 1, 4);
desc.get.call(view);
