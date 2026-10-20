let getter = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "byteLength").get;

__scriptwAssertThrows(TypeError, function () {
  getter();
});

__scriptwAssertThrows(TypeError, function () {
  getter.call({});
});

7;
