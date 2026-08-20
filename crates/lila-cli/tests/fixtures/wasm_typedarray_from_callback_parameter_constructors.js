function isPrimitive(val) {
  return !val || (typeof val !== "object" && typeof val !== "function");
}

function makePassthrough(TA, primitiveOrIterable) {
  return primitiveOrIterable;
}

function makeArray(TA, primitiveOrIterable) {
  if (isPrimitive(primitiveOrIterable)) {
    var n = Number(primitiveOrIterable);
    if (!(n >= 0 && n < 9007199254740992)) return primitiveOrIterable;
    return Array.from({ length: n }, function () { return "0"; });
  }
  return Array.from(primitiveOrIterable);
}

function makeArrayLike(TA, primitiveOrIterable) {
  var arr = makeArray(TA, primitiveOrIterable);
  if (isPrimitive(arr)) return arr;
  var obj = { length: arr.length };
  if (arr.length > 0) obj[0] = arr[0];
  if (arr.length > 1) obj[1] = arr[1];
  return obj;
}

function makeTypedArray(TA, primitiveOrIterable) {
  var arr = makeArray(TA, primitiveOrIterable);
  if (isPrimitive(arr)) return arr;
  return new TA(arr);
}

function makeIterable(TA, primitiveOrIterable) {
  var arr = makeArray(TA, primitiveOrIterable);
  if (isPrimitive(arr)) return arr;
  if (typeof Symbol === "undefined") return arr;
  return arr;
}

function makeArrayBuffer(TA, primitiveOrIterable) {
  var arr = makeArray(TA, primitiveOrIterable);
  if (isPrimitive(arr)) return arr;
  return new TA(arr).buffer;
}

var typedArrayCtorArgFactories = [
  makePassthrough,
  makeArray,
  makeArrayLike,
  makeTypedArray,
  makeIterable,
  makeArrayBuffer,
];

function testWithAllTypedArrayConstructors(f, constructors) {
  var ctors = constructors || typedArrayConstructors;
  for (var k = 0; k < typedArrayCtorArgFactories.length; ++k) {
    var argFactory = typedArrayCtorArgFactories[k];
    for (var i = 0; i < ctors.length; ++i) {
      var constructor = ctors[i];
      var boundArgFactory = argFactory.bind(undefined, constructor);
      try {
        f(constructor, boundArgFactory);
      } catch (e) {
        throw e;
      }
    }
  }
}

function testWithTypedArrayConstructors(f, constructors) {
  var ctors = constructors || typedArrayConstructors;
  testWithAllTypedArrayConstructors(f, ctors);
}

var floatArrayConstructors = [Float64Array, Float32Array];
var nonClampedIntArrayConstructors = [
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array,
];
var intArrayConstructors =
  nonClampedIntArrayConstructors.concat([Uint8ClampedArray]);
var typedArrayConstructors = floatArrayConstructors.concat(intArrayConstructors);

function assertSameValue(actual, expected, message) {
  if (expected !== expected) {
    if (actual === actual) throw message;
    return;
  }
  if (actual !== expected) throw message;
}

testWithTypedArrayConstructors(function (TA, makeCtorArg) {
  var result = TA.from([NaN, undefined]);
  if (result.length !== 2) throw "float callback parameter length";
  if (result[0] === result[0]) throw "float callback parameter NaN";
  if (result[1] === result[1]) throw "float callback parameter undefined";
  assertSameValue(result[0], NaN, "float callback sameValue NaN");
  if (result.constructor !== TA) throw "float callback constructor";
  if (Object.getPrototypeOf(result) !== TA.prototype) throw "float callback prototype";

  var objectResult = TA.from({ 0: 7, length: 2 });
  if (objectResult.length !== 2) throw "float object callback length";
  if (objectResult[0] !== 7) throw "float object callback first";
  if (objectResult[1] === objectResult[1]) throw "float object callback missing";
}, floatArrayConstructors);

testWithTypedArrayConstructors(function (TA, makeCtorArg) {
  var result = TA.from([NaN, undefined]);
  if (result.length !== 2) throw "int callback parameter length";
  if (result[0] !== 0) throw "int callback parameter NaN";
  if (result[1] !== 0) throw "int callback parameter undefined";

  var objectResult = TA.from({ 0: 7, length: 2 });
  if (objectResult.length !== 2) throw "int object callback length";
  if (objectResult[0] !== 7) throw "int object callback first";
  if (objectResult[1] !== 0) throw "int object callback missing";
}, intArrayConstructors);

262;
