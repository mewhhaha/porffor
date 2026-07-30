let typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
let every = typedArrayPrototype.every;
let some = typedArrayPrototype.some;

if (every === Array.prototype.every) throw "every intrinsic identity";
if (some === Array.prototype.some) throw "some intrinsic identity";
for (let methodName of ["every", "some"]) {
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
let allPositive = sample.every(function (value, index, array) {
  "use strict";
  if (this !== callbackThis) throw "thisArg";
  if (array !== sample) throw "callback array";
  if (value !== sample[index]) throw "callback value";
  callbackCount = callbackCount + 1;
  return value > 0;
}, callbackThis);
if (!allPositive || callbackCount !== 3) throw "every result";

callbackCount = 0;
let hasTwenty = sample.some(function (value) {
  callbackCount = callbackCount + 1;
  return value === 20;
});
if (!hasTwenty || callbackCount !== 2) throw "some result";
if (sample.every(function (value) { return value < 20; })) throw "every short circuit";
if (sample.some(function (value) { return value > 30; })) throw "some miss";
if (!new Uint8Array().every(function () { throw "empty every callback"; })) throw "empty every";
if (new Uint8Array().some(function () { throw "empty some callback"; })) throw "empty some";

let bigint = new BigInt64Array([1n, -2n, 3n]);
if (bigint.every(function (value) { return typeof value === "bigint"; }) !== true) {
  throw "bigint every";
}
if (bigint.some(function (value) { return value < 0n; }) !== true) throw "bigint some";

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
  let values = new NumericArray([0, 1, 2]);
  if (!values.every(function (value) { return typeof value === "number"; })) {
    throw NumericArray.name + " every";
  }
  if (!values.some(function (value) { return value === 2; })) {
    throw NumericArray.name + " some";
  }
}
let unsignedBigint = new BigUint64Array([0n, 1n, 2n]);
if (!unsignedBigint.every(function (value) { return typeof value === "bigint"; })) {
  throw "BigUint64Array every";
}
if (!unsignedBigint.some(function (value) { return value === 2n; })) {
  throw "BigUint64Array some";
}

for (let invalidReceiver of [{}, [], Object.create(sample)]) {
  __porfAssertThrows(TypeError, function () { every.call(invalidReceiver, function () {}); });
  __porfAssertThrows(TypeError, function () { some.call(invalidReceiver, function () {}); });
}
__porfAssertThrows(TypeError, function () { every.call(sample, 1); });
__porfAssertThrows(TypeError, function () { some.call(sample, {}); });

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
if (!privateState.every(function (value, index) { return value === index + 3; })) {
  throw "private header every";
}
if (!privateState.some(function (value) { return value === 4; })) {
  throw "private header some";
}

let genericSlotReads = 0;
let spoofedGeneric = {
  0: 3,
  1: 4,
  length: 2,
  get $TypedArrayByteLength() {
    genericSlotReads = genericSlotReads + 1;
    return 16;
  },
  get $TypedArrayViewedArrayBuffer() {
    genericSlotReads = genericSlotReads + 1;
    return new ArrayBuffer(16);
  }
};
if (!Array.prototype.every.call(spoofedGeneric, function (value) {
  return value > 0;
})) {
  throw "generic every result";
}
if (!Array.prototype.some.call(spoofedGeneric, function (value) {
  return value === 4;
})) {
  throw "generic some result";
}
if (genericSlotReads !== 0) throw "generic spoofed slots observed";

let proxyCalls = 0;
let callableProxy = new Proxy(function () {}, {
  apply: function (target, thisArg, argumentsList) {
    proxyCalls = proxyCalls + 1;
    if (argumentsList[1] !== proxyCalls - 1) throw "proxy index";
    if (argumentsList[2] !== sample) throw "proxy receiver";
    return argumentsList[0] === 20;
  }
});
if (!sample.some(callableProxy) || proxyCalls !== 2) throw "callable proxy";

let mutable = new Uint8Array([1, 2, 3]);
if (!mutable.every(function (value, index, array) {
  if (index === 0) array[1] = 9;
  return index !== 1 || value === 9;
})) throw "current view value";

let detached = new Uint8Array([1, 2, 3]);
let detachedValues = [];
let detachedResult = detached.some(function (value, index) {
  detachedValues.push(value);
  if (index === 0) __porfDetachArrayBuffer(detached.buffer);
  return false;
});
if (detachedResult !== false || detachedValues.length !== 3) throw "detached callback count";
if (detachedValues[0] !== 1) throw "detached first";
if (detachedValues[1] !== undefined || detachedValues[2] !== undefined) throw "detached later";
__porfAssertThrows(TypeError, function () { every.call(detached, function () {}); });

let shrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let shrinkView = new Uint8Array(shrinkBuffer, 0, 4);
shrinkView[0] = 1;
shrinkView[1] = 2;
shrinkView[2] = 3;
shrinkView[3] = 4;
let shrinkValues = [];
let shrinkResult = shrinkView.every(function (value, index) {
  shrinkValues.push(value);
  if (index === 1) shrinkBuffer.resize(3);
  return true;
});
if (!shrinkResult || shrinkValues.length !== 4) throw "shrink snapshot length";
if (shrinkValues[2] !== undefined || shrinkValues[3] !== undefined) throw "shrink current view";
__porfAssertThrows(TypeError, function () { some.call(shrinkView, function () {}); });

let growBuffer = new ArrayBuffer(2, { maxByteLength: 4 });
let growView = new Uint8Array(growBuffer);
growView[0] = 1;
growView[1] = 2;
let growValues = [];
if (!growView.every(function (value, index) {
  growValues.push(value);
  if (index === 0) {
    growBuffer.resize(4);
    growView[2] = 3;
    growView[3] = 4;
  }
  return true;
})) throw "grow result";
if (growValues.length !== 2 || growValues[0] !== 1 || growValues[1] !== 2) {
  throw "grow snapshot length";
}

let nonInteger = new Uint8Array([7, 8]);
Object.defineProperty(nonInteger, "1.5x", {
  get: function () {
    throw "non-integer property";
  }
});
nonInteger.extra = 9;
let integerCount = 0;
nonInteger.every(function (value, index) {
  integerCount = integerCount + 1;
  return value === index + 7;
});
if (integerCount !== 2) throw "non-integer iteration";

let marker = {};
let abrupt = false;
try {
  sample.every(function () { throw marker; });
} catch (error) {
  abrupt = error === marker;
}
if (!abrupt) throw "callback abrupt";

125;
