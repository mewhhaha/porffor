let source = new Int8Array(10);
source.$TypedArrayViewedArrayBuffer = new ArrayBuffer(1);
source.$TypedArrayByteOffset = 0;
source.$TypedArrayByteLength = 0;
source.$TypedArrayBytesPerElement = 0;
source.$TypedArrayLengthTracking = true;
let cloned = new Int8Array(source);

if (cloned.length !== 10) throw "cloned length";
if (cloned === source) throw "same instance";
if (Object.getPrototypeOf(cloned) !== Int8Array.prototype) throw "prototype";

let spoofedSource = {
  0: 7,
  1: 8,
  length: 2,
  $TypedArrayViewedArrayBuffer: new ArrayBuffer(8),
  $TypedArrayByteOffset: 0,
  $TypedArrayByteLength: 8,
  $TypedArrayBytesPerElement: 1,
  $TypedArrayLengthTracking: false
};
let fromSpoofedSource = new Uint8Array(spoofedSource);
if (
  fromSpoofedSource.length !== 2
  || fromSpoofedSource[0] !== 7
  || fromSpoofedSource[1] !== 8
) {
  throw "spoofed source";
}

123;
