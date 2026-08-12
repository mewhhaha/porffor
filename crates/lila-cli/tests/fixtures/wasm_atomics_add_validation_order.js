function assertTypeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof TypeError)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

var value = {
  valueOf() {
    throw "value coerced";
  }
};

var index = {
  valueOf() {
    throw "index coerced";
  }
};

var badConstructors = [Float64Array, Float32Array, Uint8ClampedArray];

for (var badArrayType of badConstructors) {
  var typedArray = new badArrayType(new SharedArrayBuffer(8));
  assertTypeError(function () {
    Atomics.add(typedArray, 0, value);
  }, badArrayType.name + " value");
  assertTypeError(function () {
    Atomics.add(typedArray, index, 0);
  }, badArrayType.name + " index");
}

345;
