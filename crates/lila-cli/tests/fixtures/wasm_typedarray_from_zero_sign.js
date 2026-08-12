let floats = Float32Array.from([-0, +0]);
if (floats[0] !== 0) throw "Float32Array converts -0 value";
if (1 / floats[0] !== -Infinity) throw "Float32Array preserves -0";
if (1 / floats[1] !== Infinity) throw "Float32Array preserves +0";

let ints = Int32Array.from([-0, +0]);
if (1 / ints[0] !== Infinity) throw "Int32Array converts -0";
if (1 / ints[1] !== Infinity) throw "Int32Array converts +0";

262;
