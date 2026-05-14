var floatArrayConstructors = [Float64Array, Float32Array];
var intArrayConstructors = [
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array,
  Uint8ClampedArray
];
var typedArrayConstructors = floatArrayConstructors.concat(intArrayConstructors);
var TypedArray = Object.getPrototypeOf(Int8Array);

function testWithTypedArrayConstructors(callback) {
  for (var i = 0; i < typedArrayConstructors.length; i = i + 1) {
    callback(typedArrayConstructors[i]);
  }
}

testWithTypedArrayConstructors(function(TypedArray) {
  let values = [0, {
    valueOf() {
      values.length = 0;
      return 100;
    }
  }, 2];
  let array = new TypedArray(values);
  if (array.length !== 3) throw "length";
  if (array[0] !== 0) throw "zero";
  if (array[1] !== 100) throw "one";
  if (array[2] !== 2) throw "two";
});

123;
