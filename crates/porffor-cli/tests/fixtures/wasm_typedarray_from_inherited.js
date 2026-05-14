if (Uint8Array.from !== Float32Array.from) throw "TypedArray.from identity";
if (Uint8Array.hasOwnProperty("from")) throw "Uint8Array own from";
if (Float32Array.hasOwnProperty("from")) throw "Float32Array own from";

262;
