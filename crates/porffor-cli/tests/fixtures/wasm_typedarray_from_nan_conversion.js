let float32 = Float32Array.from([NaN, undefined]);
if (float32.length !== 2) throw "Float32Array length";
if (float32[0] === float32[0]) throw "Float32Array NaN";
if (float32[1] === float32[1]) throw "Float32Array undefined";

let float64 = Float64Array.from([NaN, undefined]);
if (float64.length !== 2) throw "Float64Array length";
if (float64[0] === float64[0]) throw "Float64Array NaN";
if (float64[1] === float64[1]) throw "Float64Array undefined";

let int8 = Int8Array.from([NaN, undefined]);
if (int8[0] !== 0 || int8[1] !== 0) throw "Int8Array conversion";

let uint8 = Uint8Array.from([NaN, undefined]);
if (uint8[0] !== 0 || uint8[1] !== 0) throw "Uint8Array conversion";

let int16 = Int16Array.from([NaN, undefined]);
if (int16[0] !== 0 || int16[1] !== 0) throw "Int16Array conversion";

let uint16 = Uint16Array.from([NaN, undefined]);
if (uint16[0] !== 0 || uint16[1] !== 0) throw "Uint16Array conversion";

let int32 = Int32Array.from([NaN, undefined]);
if (int32[0] !== 0 || int32[1] !== 0) throw "Int32Array conversion";

let uint32 = Uint32Array.from([NaN, undefined]);
if (uint32[0] !== 0 || uint32[1] !== 0) throw "Uint32Array conversion";

let clamped = Uint8ClampedArray.from([NaN, undefined]);
if (clamped[0] !== 0 || clamped[1] !== 0) throw "Uint8ClampedArray conversion";

if (typeof Float16Array !== "undefined") throw "Float16Array exposed";

262;
