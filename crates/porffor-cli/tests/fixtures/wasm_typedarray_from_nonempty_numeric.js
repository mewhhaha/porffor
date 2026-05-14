let bytes = Uint8Array.from([42, 43, 42]);
if (bytes.length !== 3) throw "Uint8Array.from length";
if (bytes[0] !== 42 || bytes[1] !== 43 || bytes[2] !== 42) {
  throw "Uint8Array.from values";
}
if (bytes.constructor !== Uint8Array) throw "Uint8Array.from constructor";
if (Object.getPrototypeOf(bytes) !== Uint8Array.prototype) throw "Uint8Array.from prototype";

let clamped = Uint8ClampedArray.from([42, 43, 42]);
if (clamped[0] !== 42 || clamped[1] !== 43 || clamped[2] !== 42) {
  throw "Uint8ClampedArray.from values";
}

let floats = Float32Array.from([42, 43, 42]);
if (floats.length !== 3) throw "Float32Array.from length";
if (floats[0] !== 42 || floats[1] !== 43 || floats[2] !== 42) {
  throw "Float32Array.from values";
}

262;
