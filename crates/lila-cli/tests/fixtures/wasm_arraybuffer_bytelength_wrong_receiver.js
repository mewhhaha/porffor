let getter = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "byteLength").get;

__lilaAssertThrows(TypeError, function () {
  getter();
});

__lilaAssertThrows(TypeError, function () {
  getter.call({});
});

7;
