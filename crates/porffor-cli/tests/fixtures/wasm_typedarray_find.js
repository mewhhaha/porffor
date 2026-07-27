let typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
let find = typedArrayPrototype.find;
let findIndex = typedArrayPrototype.findIndex;
let findLast = typedArrayPrototype.findLast;
let findLastIndex = typedArrayPrototype.findLastIndex;

if (find === Array.prototype.find) throw "find intrinsic identity";
if (findIndex === Array.prototype.findIndex) throw "findIndex intrinsic identity";
if (findLast === Array.prototype.findLast) throw "findLast intrinsic identity";
if (findLastIndex === Array.prototype.findLastIndex) throw "findLastIndex intrinsic identity";
for (let methodName of ["find", "findIndex", "findLast", "findLastIndex"]) {
  let descriptor = Object.getOwnPropertyDescriptor(typedArrayPrototype, methodName);
  if (descriptor === undefined) throw methodName + " descriptor";
  if (descriptor.value !== typedArrayPrototype[methodName]) throw methodName + " value";
  if (descriptor.writable !== true) throw methodName + " writable";
  if (descriptor.enumerable !== false) throw methodName + " enumerable";
  if (descriptor.configurable !== true) throw methodName + " configurable";
  if (descriptor.value.name !== methodName) throw methodName + " name";
  if (descriptor.value.length !== 1) throw methodName + " length";
}

let sample = new Uint16Array([10, 20, 30]);
let callbackThis = {};
let callbackCount = 0;
let found = sample.find(function (value, index, array) {
  "use strict";
  if (this !== callbackThis) throw "thisArg";
  if (array !== sample) throw "callback array";
  if (value !== sample[index]) throw "callback value";
  callbackCount = callbackCount + 1;
  return value === 20;
}, callbackThis);
if (found !== 20 || callbackCount !== 2) throw "find result";
if (sample.findIndex(function (value) { return value === 30; }) !== 2) throw "findIndex result";
if (sample.findLast(function (value) { return value < 30; }) !== 20) throw "findLast result";
if (sample.findLastIndex(function (value) { return value < 30; }) !== 1) throw "findLastIndex result";
if (sample.find(function () { return false; }) !== undefined) throw "find miss";
if (sample.findIndex(function () { return 0; }) !== -1) throw "findIndex miss";
if (sample.findLast(function () { return false; }) !== undefined) throw "findLast miss";
if (sample.findLastIndex(function () { return 0; }) !== -1) throw "findLastIndex miss";

let mutable = new Uint8Array([1, 2, 3]);
let changed = mutable.find(function (value, index, array) {
  if (index === 0) array[1] = 9;
  return value === 9;
});
if (changed !== 9) throw "current view value";

let bigint = new BigInt64Array([1n, -2n, 3n]);
if (bigint.find(function (value) { return value < 0n; }) !== -2n) throw "bigint find";
if (bigint.find(function (value) { return value == -2; }) !== -2n) {
  throw "bigint loose number find";
}
if (bigint.findIndex(function (value) { return value === 3n; }) !== 2) throw "bigint findIndex";
if (bigint.findLast(function (value) { return value > 0n; }) !== 3n) throw "bigint findLast";
if (bigint.findLastIndex(function (value) { return value > 0n; }) !== 2) throw "bigint findLastIndex";

let numericConstructors = [
  Float64Array,
  Float32Array,
  Int32Array,
  Uint32Array,
  Int16Array,
  Uint16Array,
  Int8Array,
  Uint8Array,
  Uint8ClampedArray
];
if (typeof Float16Array !== "undefined") {
  numericConstructors = numericConstructors.concat([Float16Array]);
}
for (let NumericArray of numericConstructors) {
  let values = new NumericArray([1, 2, 3]);
  if (values.find(function (value) { return value === 2; }) !== 2) {
    throw NumericArray.name + " find";
  }
  if (values.findIndex(function (value) { return value === 3; }) !== 2) {
    throw NumericArray.name + " findIndex";
  }
  if (values.findLast(function (value) { return value < 3; }) !== 2) {
    throw NumericArray.name + " findLast";
  }
  if (values.findLastIndex(function (value) { return value < 3; }) !== 1) {
    throw NumericArray.name + " findLastIndex";
  }
}
let unsignedBigint = new BigUint64Array([1n, 2n, 3n]);
if (unsignedBigint.find(function (value) { return value === 2n; }) !== 2n) {
  throw "BigUint64Array find";
}
if (unsignedBigint.find(function (value) { return value == 2; }) !== 2n) {
  throw "BigUint64Array loose number find";
}
if (unsignedBigint.findLastIndex(function (value) { return value < 3n; }) !== 1) {
  throw "BigUint64Array findLastIndex";
}
if (9007199254740993n == 9007199254740992) throw "rounded loose bigint equality";
if (!(9007199254740992n == 9007199254740992)) throw "exact loose bigint equality";
if (1n == 1.5 || 1n == Infinity || 1n == NaN) throw "non-integer loose bigint equality";
if (!(0n == -0) || !(-0 == 0n)) throw "negative zero loose bigint equality";
if (!(-9223372036854775808n == -9223372036854775808)) {
  throw "minimum i64 loose bigint equality";
}
if (!(9223372036854774784n == 9223372036854774784)) {
  throw "below i64 upper boundary loose bigint equality";
}
if (!(9223372036854775808n == 9223372036854775808)
    || !(9223372036854775808 == 9223372036854775808n)) {
  throw "heap bigint loose number symmetry";
}
if (9223372036854775809n == 9223372036854775808) {
  throw "heap bigint rounded inequality";
}
if (!(18446744073709551616n == 18446744073709551616)
    || !(-18446744073709551616 == -18446744073709551616n)) {
  throw "multi-limb bigint loose number equality";
}

let wideUnsignedBigint = new BigUint64Array([
  9223372036854775808n,
  9223372036854777856n
]);
if (wideUnsignedBigint.find(function (value) {
  return value == 9223372036854775808;
}) !== 9223372036854775808n) {
  throw "BigUint64Array 2^63 loose number find";
}
if (wideUnsignedBigint.find(function (value) {
  return 9223372036854777856 == value;
}) !== 9223372036854777856n) {
  throw "BigUint64Array 2^63 plus 2048 loose number find";
}

function abstractlyEqual(left, right) {
  return left == right;
}
if (!abstractlyEqual(1n, true) || !abstractlyEqual(true, 1n)
    || !abstractlyEqual(0n, false) || !abstractlyEqual(false, 0n)
    || abstractlyEqual(2n, true) || abstractlyEqual(true, 2n)) {
  throw "dynamic boolean bigint loose equality";
}

for (let invalidReceiver of [{}, [], Object.create(sample)]) {
  __porfAssertThrows(TypeError, function () { find.call(invalidReceiver, function () {}); });
  __porfAssertThrows(TypeError, function () { findIndex.call(invalidReceiver, function () {}); });
  __porfAssertThrows(TypeError, function () { findLast.call(invalidReceiver, function () {}); });
  __porfAssertThrows(TypeError, function () { findLastIndex.call(invalidReceiver, function () {}); });
}
__porfAssertThrows(TypeError, function () { find.call(sample, 1); });
__porfAssertThrows(TypeError, function () { findIndex.call(sample, {}); });
__porfAssertThrows(TypeError, function () { findLast.call(sample, 1); });
__porfAssertThrows(TypeError, function () { findLastIndex.call(sample, {}); });

let privateState = new Uint8Array([3, 4]);
privateState.$TypedArrayViewedArrayBuffer = new ArrayBuffer(1);
for (let property of [
  "$TypedArrayByteOffset",
  "$TypedArrayByteLength",
  "$TypedArrayBytesPerElement",
  "$TypedArrayElementKind",
  "$TypedArrayLengthTracking",
  "length"
]) {
  Object.defineProperty(privateState, property, {
    get: function () {
      throw property + " must not be read";
    }
  });
}
if (privateState.find(function (value) { return value === 4; }) !== 4) {
  throw "private header find";
}
if (privateState.findIndex(function (value) { return value === 4; }) !== 1) {
  throw "private header findIndex";
}
if (privateState.findLast(function (value) { return value === 3; }) !== 3) {
  throw "private header findLast";
}
if (privateState.findLastIndex(function (value) { return value === 3; }) !== 0) {
  throw "private header findLastIndex";
}

let proxyCalls = 0;
let proxyThis = {};
let callableProxy = new Proxy(function () {}, {
  apply: function (target, thisArg, argumentsList) {
    proxyCalls = proxyCalls + 1;
    if (thisArg !== proxyThis) throw "proxy thisArg";
    if (argumentsList[1] !== proxyCalls - 1) throw "proxy index";
    if (argumentsList[2] !== sample) throw "proxy receiver";
    return argumentsList[0] === 20;
  }
});
if (sample.findIndex(callableProxy, proxyThis) !== 1 || proxyCalls !== 2) {
  throw "callable proxy";
}

let emptyCalls = 0;
if (new Uint8Array().find(function () { emptyCalls = emptyCalls + 1; }) !== undefined) {
  throw "empty find result";
}
if (new Uint8Array().findLastIndex(function () { emptyCalls = emptyCalls + 1; }) !== -1) {
  throw "empty findLastIndex result";
}
if (emptyCalls !== 0) throw "empty predicate calls";

let detached = new Uint8Array([1, 2, 3]);
let detachedValues = [];
let detachedResult = detached.find(function (value, index) {
  detachedValues.push(value);
  if (index === 0) __porfDetachArrayBuffer(detached.buffer);
  return false;
});
if (detachedResult !== undefined) throw "detached result";
if (detachedValues.length !== 3) throw "detached callback count";
if (detachedValues[0] !== 1) throw "detached first";
if (detachedValues[1] !== undefined || detachedValues[2] !== undefined) throw "detached later";
__porfAssertThrows(TypeError, function () { find.call(detached, function () {}); });

let growBuffer = new ArrayBuffer(2, { maxByteLength: 4 });
let growView = new Uint8Array(growBuffer);
growView[0] = 1;
growView[1] = 2;
let growCalls = 0;
let growResult = growView.find(function () {
  growCalls = growCalls + 1;
  if (growCalls === 1) growBuffer.resize(4);
  return false;
});
if (growResult !== undefined || growCalls !== 2) throw "grow snapshot length";

let shrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let shrinkView = new Uint8Array(shrinkBuffer, 0, 4);
shrinkView[0] = 1;
shrinkView[1] = 2;
shrinkView[2] = 3;
shrinkView[3] = 4;
let shrinkValues = [];
let shrinkResult = shrinkView.findIndex(function (value, index) {
  shrinkValues.push(value);
  if (index === 1) shrinkBuffer.resize(3);
  return false;
});
if (shrinkResult !== -1 || shrinkValues.length !== 4) throw "shrink snapshot length";
if (shrinkValues[2] !== undefined || shrinkValues[3] !== undefined) throw "shrink current view";
__porfAssertThrows(TypeError, function () { find.call(shrinkView, function () {}); });

let reverseBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let reverseView = new Uint8Array(reverseBuffer);
reverseView[0] = 1;
reverseView[1] = 2;
reverseView[2] = 3;
reverseView[3] = 4;
let reverseIndices = [];
let reverseResult = reverseView.findLast(function (value, index) {
  reverseIndices.push(index);
  if (index === 3) reverseBuffer.resize(2);
  return value === 2;
});
if (reverseResult !== 2) throw "findLast shrink result";
if (reverseIndices.length !== 3) throw "findLast reverse count";
if (reverseIndices[0] !== 3 || reverseIndices[1] !== 2 || reverseIndices[2] !== 1) {
  throw "findLast reverse order";
}

let reverseGrowBuffer = new ArrayBuffer(2, { maxByteLength: 4 });
let reverseGrow = new Uint8Array(reverseGrowBuffer);
reverseGrow[0] = 1;
reverseGrow[1] = 2;
let reverseGrowIndices = [];
reverseGrow.findLastIndex(function (value, index) {
  reverseGrowIndices.push(index);
  if (index === 1) {
    reverseGrowBuffer.resize(4);
    reverseGrow[2] = 3;
    reverseGrow[3] = 4;
  }
  return false;
});
if (
  reverseGrowIndices.length !== 2 ||
  reverseGrowIndices[0] !== 1 ||
  reverseGrowIndices[1] !== 0
) {
  throw "findLast grow snapshot";
}

let reverseDetach = new Uint8Array([1, 2, 3]);
let reverseDetachedIndex = reverseDetach.findLastIndex(function (value, index) {
  if (index === 2) {
    __porfDetachArrayBuffer(reverseDetach.buffer);
    return false;
  }
  return value === undefined;
});
if (reverseDetachedIndex !== 1) throw "findLast detach current value";

let marker = {};
let abrupt = false;
try {
  sample.find(function () { throw marker; });
} catch (error) {
  abrupt = error === marker;
}
if (!abrupt) throw "callback abrupt";

124;
