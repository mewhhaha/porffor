let typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
let map = typedArrayPrototype.map;

if (map === Array.prototype.map) throw "map intrinsic identity";
let descriptor = Object.getOwnPropertyDescriptor(typedArrayPrototype, "map");
if (descriptor === undefined) throw "map descriptor";
if (descriptor.value !== map) throw "map descriptor value";
if (descriptor.writable !== true) throw "map writable";
if (descriptor.enumerable !== false) throw "map enumerable";
if (descriptor.configurable !== true) throw "map configurable";
if (map.name !== "map") throw "map name";
if (map.length !== 1) throw "map length";

let sample = new Uint8Array([1, 2, 3]);
let callbackThis = {};
let callbackCount = 0;
let mapped = sample.map(function (value, index, array) {
  "use strict";
  if (this !== callbackThis) throw "callback this";
  if (array !== sample) throw "callback receiver";
  if (value !== sample[index]) throw "callback value";
  callbackCount = callbackCount + 1;
  return value + 256;
}, callbackThis);
if (!(mapped instanceof Uint8Array)) throw "default result type";
if (mapped === sample || mapped.buffer === sample.buffer) throw "result aliases source";
if (callbackCount !== 3) throw "callback count";
if (mapped.length !== 3 || mapped[0] !== 1 || mapped[1] !== 2 || mapped[2] !== 3) {
  throw "mapped conversion";
}

let order = [];
let speciesTarget = new Int16Array(5);
sample.constructor = {};
sample.constructor[Symbol.species] = function (length) {
  order.push("species");
  if (length !== 3) throw "species length";
  return speciesTarget;
};
let speciesResult = sample.map(function (value) {
  order.push("callback");
  return value + 10;
});
if (speciesResult !== speciesTarget) throw "custom species target";
if (order.length !== 4 || order[0] !== "species") throw "species ordering";
if (speciesTarget[0] !== 11 || speciesTarget[1] !== 12 || speciesTarget[2] !== 13) {
  throw "custom species values";
}
if (speciesTarget[3] !== 0 || speciesTarget[4] !== 0) throw "custom species length";

let bigint = new BigInt64Array([1n, -2n]);
let bigintResult = bigint.map(function (value) { return value + 3n; });
if (!(bigintResult instanceof BigInt64Array)) throw "bigint result type";
if (bigintResult[0] !== 4n || bigintResult[1] !== 1n) throw "bigint values";

let detached = new Uint8Array([4, 5, 6]);
let detachedValues = [];
let detachedResult = detached.map(function (value, index) {
  detachedValues.push(value);
  if (index === 0) __lilaDetachArrayBuffer(detached.buffer);
  return value === undefined ? 0 : value;
});
if (detachedValues.length !== 3) throw "detached callback count";
if (detachedValues[0] !== 4) throw "detached first value";
if (detachedValues[1] !== undefined || detachedValues[2] !== undefined) {
  throw "detached later values";
}
if (detachedResult[0] !== 4 || detachedResult[1] !== 0 || detachedResult[2] !== 0) {
  throw "detached mapped values";
}

let entryDetached = new Uint8Array([1]);
let entryDetachedCallbackCalled = false;
__lilaDetachArrayBuffer(entryDetached.buffer);
__lilaAssertThrows(TypeError, function () {
  entryDetached.map(function () { entryDetachedCallbackCalled = true; });
});
if (entryDetachedCallbackCalled) throw "detached entry callback";

let entryOobBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let entryOob = new Uint8Array(entryOobBuffer, 2, 2);
let entryOobCallbackCalled = false;
entryOobBuffer.resize(1);
__lilaAssertThrows(TypeError, function () {
  entryOob.map(function () { entryOobCallbackCalled = true; });
});
if (entryOobCallbackCalled) throw "out-of-bounds entry callback";

let shrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let shrinkSource = new Uint8Array(shrinkBuffer);
shrinkSource[0] = 1;
shrinkSource[1] = 2;
shrinkSource[2] = 3;
shrinkSource[3] = 4;
let shrinkValues = [];
let shrinkResult = shrinkSource.map(function (value, index) {
  shrinkValues.push(value);
  if (index === 1) shrinkBuffer.resize(2);
  return value === undefined ? 0 : value;
});
if (shrinkValues.length !== 4) throw "shrink snapshot length";
if (shrinkValues[2] !== undefined || shrinkValues[3] !== undefined) {
  throw "shrink current values";
}
if (shrinkResult[0] !== 1 || shrinkResult[1] !== 2 ||
    shrinkResult[2] !== 0 || shrinkResult[3] !== 0) {
  throw "shrink mapped values";
}

let tooSmall = new Uint8Array([1, 2]);
tooSmall.constructor = {};
tooSmall.constructor[Symbol.species] = function () { return new Uint8Array(1); };
__lilaAssertThrows(TypeError, function () {
  tooSmall.map(function (value) { return value; });
});

let wrongContentType = new Uint8Array([1]);
wrongContentType.constructor = {};
wrongContentType.constructor[Symbol.species] = function () { return new BigInt64Array(1); };
__lilaAssertThrows(TypeError, function () {
  wrongContentType.map(function (value) { return value; });
});

for (let invalidReceiver of [{}, [], Object.create(sample)]) {
  __lilaAssertThrows(TypeError, function () {
    map.call(invalidReceiver, function () {});
  });
}
__lilaAssertThrows(TypeError, function () { map.call(new Uint8Array(), {}); });

126;
