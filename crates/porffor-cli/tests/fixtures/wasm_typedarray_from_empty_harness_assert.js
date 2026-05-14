let assert = {
  sameValue: function (actual, expected, message) {
    if (actual !== expected) throw message;
  }
};

let uint8 = Uint8Array.from([]);
assert.sameValue(uint8.length, 0, "Uint8Array empty length");
assert.sameValue(uint8.constructor, Uint8Array, "Uint8Array constructor identity");

let float32 = Float32Array.from([]);
assert.sameValue(float32.length, 0, "Float32Array empty length");
assert.sameValue(float32.constructor, Float32Array, "Float32Array constructor identity");

262;
