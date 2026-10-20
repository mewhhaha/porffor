let byteLengthBuffer = new ArrayBuffer(8);
let byteLengthView = new DataView(byteLengthBuffer, 2, 4);
__scriptwDetachArrayBuffer(byteLengthBuffer);

__scriptwAssertThrows(TypeError, function () {
  byteLengthView.byteLength;
});

let byteOffsetBuffer = new ArrayBuffer(8);
let byteOffsetView = new DataView(byteOffsetBuffer, 3, 2);
__scriptwDetachArrayBuffer(byteOffsetView.buffer);

__scriptwAssertThrows(TypeError, function () {
  byteOffsetView.byteOffset;
});

2;
