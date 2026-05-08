let bytes = Uint8Array.from([]);
if (bytes.length !== 0) throw "Uint8Array.from empty length";
if (bytes.constructor !== Uint8Array) throw "Uint8Array.from empty constructor";
if (Object.getPrototypeOf(bytes) !== Uint8Array.prototype) {
  throw "Uint8Array.from empty prototype";
}

let floats = Float32Array.from([]);
if (floats.length !== 0) throw "Float32Array.from empty length";
if (floats.constructor !== Float32Array) throw "Float32Array.from empty constructor";
if (Object.getPrototypeOf(floats) !== Float32Array.prototype) {
  throw "Float32Array.from empty prototype";
}

262;
