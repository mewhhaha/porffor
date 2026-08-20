function assertSameValue(actual, expected, message) {
  if (expected !== expected) {
    if (actual === actual) throw message;
    return;
  }
  if (actual !== expected) throw message;
}

function argFactory(constructor) {
  return function () {
    return new constructor(1);
  };
}

var floatArrayConstructors = [Float64Array, Float32Array];
var intArrayConstructors = [
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array,
  Uint8ClampedArray,
];
var typedArrayConstructors = floatArrayConstructors.concat(intArrayConstructors);

if (typeof Float16Array !== "undefined") {
  floatArrayConstructors = floatArrayConstructors.concat([Float16Array]);
  typedArrayConstructors = typedArrayConstructors.concat([Float16Array]);
}

function testWithAllTypedArrayConstructors(f, constructors) {
  var ctors = constructors || typedArrayConstructors;
  for (var i = 0; i < ctors.length; i++) {
    var constructor = ctors[i];
    var boundArgFactory = argFactory.bind(undefined, constructor);
    f(constructor, boundArgFactory);
  }
}

function testWithTypedArrayConstructors(f, constructors) {
  testWithAllTypedArrayConstructors(f, constructors);
}

function isFloatConstructor(TA) {
  return TA === Float64Array || TA === Float32Array ||
    (typeof Float16Array !== "undefined" && TA === Float16Array);
}

function sharedCallback(TA, makeCtorArg) {
  var result = TA.from([NaN, undefined]);
  if (result.length !== 2) throw "shared callback length";
  if (isFloatConstructor(TA)) {
    assertSameValue(result[0], NaN, "shared callback float NaN");
    assertSameValue(result[1], NaN, "shared callback float undefined");
  } else {
    assertSameValue(result[0], 0, "shared callback int NaN");
    assertSameValue(result[1], 0, "shared callback int undefined");
  }

  var ctorArg = makeCtorArg()();
  if (ctorArg.length !== 1) throw "shared callback ctor arg length";
  var objectResult = TA.from({ 0: 7, length: 2 });
  assertSameValue(objectResult.length, 2, "shared callback object length");
  assertSameValue(objectResult[0], 7, "shared callback object first");
  if (isFloatConstructor(TA)) {
    assertSameValue(objectResult[1], NaN, "shared callback object missing float");
  } else {
    assertSameValue(objectResult[1], 0, "shared callback object missing int");
  }
}

testWithTypedArrayConstructors(sharedCallback, floatArrayConstructors);
testWithTypedArrayConstructors(sharedCallback, intArrayConstructors);

262;
