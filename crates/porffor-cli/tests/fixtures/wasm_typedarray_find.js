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
if (bigint.findIndex(function (value) { return value === 3n; }) !== 2) throw "bigint findIndex";
if (bigint.findLast(function (value) { return value > 0n; }) !== 3n) throw "bigint findLast";
if (bigint.findLastIndex(function (value) { return value > 0n; }) !== 2) throw "bigint findLastIndex";

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

let marker = {};
let abrupt = false;
try {
  sample.find(function () { throw marker; });
} catch (error) {
  abrupt = error === marker;
}
if (!abrupt) throw "callback abrupt";

124;
