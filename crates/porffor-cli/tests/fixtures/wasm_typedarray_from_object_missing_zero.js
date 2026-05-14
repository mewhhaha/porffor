let floats = Float32Array.from({ length: 4, 0: 42, 2: 44 });
if (floats.length !== 4) throw "Float32Array object length";
if (floats[0] !== 42 || floats[2] !== 44) throw "Float32Array object values";
if (floats[1] === floats[1]) throw "Float32Array missing should be NaN";
if (floats[3] === floats[3]) throw "Float32Array trailing missing should be NaN";

let ints = Int32Array.from({ length: 4, 0: 42, 2: 44 });
if (ints.length !== 4) throw "Int32Array object length";
if (ints[0] !== 42 || ints[1] !== 0 || ints[2] !== 44 || ints[3] !== 0) {
  throw "Int32Array object values";
}

262;
