let byteLengthBuffer = new ArrayBuffer(8);
let byteLengthView = new DataView(byteLengthBuffer, 2, 4);
__lilaDetachArrayBuffer(byteLengthBuffer);

__lilaAssertThrows(TypeError, function () {
  byteLengthView.byteLength;
});

let byteOffsetBuffer = new ArrayBuffer(8);
let byteOffsetView = new DataView(byteOffsetBuffer, 3, 2);
__lilaDetachArrayBuffer(byteOffsetView.buffer);

__lilaAssertThrows(TypeError, function () {
  byteOffsetView.byteOffset;
});

2;
