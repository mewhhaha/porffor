let typedArrayConstructors = [
  Float64Array,
  Float32Array,
  Int32Array,
  Uint32Array,
  Uint8Array,
  Uint8ClampedArray,
];
let floatArrayConstructors = [Float64Array, Float32Array];
let intArrayConstructors = [Int32Array, Uint32Array, Uint8Array, Uint8ClampedArray];

if (typeof Float16Array !== "undefined") {
  floatArrayConstructors = floatArrayConstructors.concat([Float16Array]);
  typedArrayConstructors = typedArrayConstructors.concat([Float16Array]);
}

function testWithAllTypedArrayConstructors(f, constructors) {
  let ctors = constructors || typedArrayConstructors;
  for (let i = 0; i < ctors.length; i++) {
    f(ctors[i]);
  }
}

function testWithTypedArrayConstructors(f, constructors) {
  testWithAllTypedArrayConstructors(f, constructors);
}

testWithTypedArrayConstructors(function (TA) {
  let result = TA.from([NaN]);
  if (result.length !== 1) throw "float helper length";
  if (result[0] === result[0]) throw "float helper NaN";
}, floatArrayConstructors);

testWithTypedArrayConstructors(function (TA) {
  let result = TA.from([NaN]);
  if (result.length !== 1) throw "int helper length";
  if (result[0] !== 0) throw "int helper NaN conversion";
}, intArrayConstructors);

262;
