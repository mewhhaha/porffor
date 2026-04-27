let getter = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "byteLength").get;

__porfAssertThrows(TypeError, function () {
  getter();
});

__porfAssertThrows(TypeError, function () {
  getter.call({});
});

7;
