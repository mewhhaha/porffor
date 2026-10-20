let floatCtors = [Float64Array, Float32Array];
for (let i = 0; i < floatCtors.length; i++) {
  let TA = floatCtors[i];
  let result = TA.from([NaN, undefined, -0, +0]);
  if (result.length !== 4) throw "float constructor array length";
  if (result[0] === result[0]) throw "float constructor array NaN";
  if (result[1] === result[1]) throw "float constructor array undefined";
  if (1 / result[2] !== -Infinity) throw "float constructor array preserves -0";
  if (1 / result[3] !== Infinity) throw "float constructor array preserves +0";
}

let intCtors = [Int32Array, Uint8Array].concat([Uint8ClampedArray]);
for (let i = 0; i < intCtors.length; i++) {
  let TA = intCtors[i];
  let result = TA.from({ 0: 42, 2: 44, length: 4 });
  if (result.length !== 4) throw "int constructor array length";
  if (result[0] !== 42) throw "int constructor array first value";
  if (result[1] !== 0) throw "int constructor array missing value";
  if (result[2] !== 44) throw "int constructor array third value";
  if (result[3] !== 0) throw "int constructor array final missing value";

  let zeros = TA.from([-0, +0]);
  if (1 / zeros[0] !== Infinity) throw "int constructor array converts -0";
  if (1 / zeros[1] !== Infinity) throw "int constructor array converts +0";
}

262;
