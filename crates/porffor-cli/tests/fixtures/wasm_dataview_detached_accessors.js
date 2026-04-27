let byteLengthBuffer = new ArrayBuffer(8);
let byteLengthView = new DataView(byteLengthBuffer, 2, 4);
__porfDetachArrayBuffer(byteLengthBuffer);

__porfAssertThrows(TypeError, function () {
  byteLengthView.byteLength;
});

let byteOffsetBuffer = new ArrayBuffer(8);
let byteOffsetView = new DataView(byteOffsetBuffer, 3, 2);
__porfDetachArrayBuffer(byteOffsetView.buffer);

__porfAssertThrows(TypeError, function () {
  byteOffsetView.byteOffset;
});

2;
