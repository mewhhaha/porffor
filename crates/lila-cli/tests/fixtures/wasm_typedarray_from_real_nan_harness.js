function assert(mustBeTrue, message) {
  if (mustBeTrue === true) return;
  throw message;
}

assert._isSameValue = function (a, b) {
  if (a === b) {
    return a !== 0 || 1 / a === 1 / b;
  }
  return a !== a && b !== b;
};

assert.sameValue = function (actual, expected, message) {
  if (assert._isSameValue(actual, expected)) return;
  throw message;
};

var floatArrayConstructors = [
  Float64Array,
  Float32Array
];
var nonClampedIntArrayConstructors = [
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array
];
var intArrayConstructors = nonClampedIntArrayConstructors.concat([Uint8ClampedArray]);

function testWithTypedArrayConstructors(f, selected) {
  var passthrough = function (value) { return value; };
  if (selected === floatArrayConstructors) {
    f(Float64Array, passthrough);
    f(Float32Array, passthrough);
    return;
  }
  if (selected === intArrayConstructors) {
    f(Int32Array, passthrough);
    f(Int16Array, passthrough);
    f(Int8Array, passthrough);
    f(Uint32Array, passthrough);
    f(Uint16Array, passthrough);
    f(Uint8Array, passthrough);
    f(Uint8ClampedArray, passthrough);
    return;
  }
  f(Float64Array, passthrough);
  f(Float32Array, passthrough);
}

testWithTypedArrayConstructors(function (TA, makeCtorArg) {
  var result = TA.from([NaN, undefined]);
  assert.sameValue(result.length, 2, "float length");
  assert.sameValue(result[0], NaN, "float first NaN");
  assert.sameValue(result[1], NaN, "float second NaN");
  assert.sameValue(result.constructor, TA, "float constructor");
  assert.sameValue(Object.getPrototypeOf(result), TA.prototype, "float prototype");
}, floatArrayConstructors);

testWithTypedArrayConstructors(function (TA, makeCtorArg) {
  var result = TA.from([NaN, undefined]);
  assert.sameValue(result.length, 2, "int length");
  assert.sameValue(result[0], 0, "int first zero");
  assert.sameValue(result[1], 0, "int second zero");
  assert.sameValue(result.constructor, TA, "int constructor");
  assert.sameValue(Object.getPrototypeOf(result), TA.prototype, "int prototype");
}, intArrayConstructors);

262;
