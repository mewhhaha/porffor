let typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
let filter = typedArrayPrototype.filter;

if (filter === Array.prototype.filter) throw "filter intrinsic identity";
let descriptor = Object.getOwnPropertyDescriptor(typedArrayPrototype, "filter");
if (descriptor === undefined) throw "filter descriptor";
if (descriptor.value !== filter) throw "filter descriptor value";
if (descriptor.writable !== true) throw "filter writable";
if (descriptor.enumerable !== false) throw "filter enumerable";
if (descriptor.configurable !== true) throw "filter configurable";
if (filter.name !== "filter") throw "filter name";
if (filter.length !== 1) throw "filter length";

let sample = new Uint8Array([1, 2, 3, 4]);
let callbackThis = {};
let callbackCount = 0;
let filtered = sample.filter(function (value, index, array) {
  "use strict";
  if (this !== callbackThis) throw "callback this";
  if (array !== sample) throw "callback receiver";
  if (value !== sample[index]) throw "callback value";
  callbackCount = callbackCount + 1;
  return value % 2 !== 0;
}, callbackThis);
if (!(filtered instanceof Uint8Array)) throw "default result type";
if (filtered === sample || filtered.buffer === sample.buffer) throw "result aliases source";
if (callbackCount !== 4) throw "callback count";
if (filtered.length !== 2 || filtered[0] !== 1 || filtered[1] !== 3) {
  throw "filtered values";
}

let originalSource = new Uint8Array([8]);
let originalResult = originalSource.filter(function (value) {
  originalSource[0] = 99;
  return true;
});
if (originalResult[0] !== 8) throw "selected original value";

let order = [];
let speciesSource = new Uint8Array([5, 6, 7]);
let speciesTarget = new Int16Array(4);
speciesSource.constructor = {};
speciesSource.constructor[Symbol.species] = function (length) {
  order.push("species");
  if (length !== 2) throw "species length";
  return speciesTarget;
};
let speciesResult = speciesSource.filter(function (value) {
  order.push("callback");
  return value !== 6;
});
if (speciesResult !== speciesTarget) throw "custom species target";
if (order.length !== 4 || order[0] !== "callback" || order[2] !== "callback" ||
    order[3] !== "species") {
  throw "species ordering";
}
if (speciesTarget[0] !== 5 || speciesTarget[1] !== 7 ||
    speciesTarget[2] !== 0 || speciesTarget[3] !== 0) {
  throw "custom species values";
}

let lateSpeciesSource = new Uint8Array([10, 11]);
let lateSpeciesTarget = new Uint16Array(1);
let lateSpeciesResult = lateSpeciesSource.filter(function (value, index) {
  if (index === 0) {
    lateSpeciesSource.constructor = {};
    lateSpeciesSource.constructor[Symbol.species] = function (length) {
      if (length !== 1) throw "late species length";
      return lateSpeciesTarget;
    };
  }
  return value === 11;
});
if (lateSpeciesResult !== lateSpeciesTarget || lateSpeciesTarget[0] !== 11) {
  throw "late species lookup";
}

let bigint = new BigInt64Array([1n, -2n, 3n]);
let bigintResult = bigint.filter(function (value) { return value > 0n; });
if (!(bigintResult instanceof BigInt64Array)) throw "bigint result type";
if (bigintResult.length !== 2 || bigintResult[0] !== 1n || bigintResult[1] !== 3n) {
  throw "bigint values";
}

let entryDetached = new Uint8Array([1]);
let entryDetachedCallbackCalled = false;
__lilaDetachArrayBuffer(entryDetached.buffer);
__lilaAssertThrows(TypeError, function () {
  entryDetached.filter(function () { entryDetachedCallbackCalled = true; });
});
if (entryDetachedCallbackCalled) throw "detached entry callback";

let entryOobBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let entryOob = new Uint8Array(entryOobBuffer, 2, 2);
let entryOobCallbackCalled = false;
entryOobBuffer.resize(1);
__lilaAssertThrows(TypeError, function () {
  entryOob.filter(function () { entryOobCallbackCalled = true; });
});
if (entryOobCallbackCalled) throw "out-of-bounds entry callback";

let shrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let shrinkSource = new Uint8Array(shrinkBuffer);
shrinkSource[0] = 1;
shrinkSource[1] = 2;
shrinkSource[2] = 3;
shrinkSource[3] = 4;
let shrinkValues = [];
let shrinkResult = shrinkSource.filter(function (value, index) {
  shrinkValues.push(value);
  if (index === 1) shrinkBuffer.resize(2);
  return value !== undefined;
});
if (shrinkValues.length !== 4) throw "shrink snapshot length";
if (shrinkValues[2] !== undefined || shrinkValues[3] !== undefined) {
  throw "shrink current values";
}
if (shrinkResult.length !== 2 || shrinkResult[0] !== 1 || shrinkResult[1] !== 2) {
  throw "shrink filtered values";
}

let speciesReads = 0;
let abruptSource = new Uint8Array([1]);
Object.defineProperty(abruptSource, "constructor", {
  get: function () {
    speciesReads = speciesReads + 1;
    return Uint8Array;
  }
});
__lilaAssertThrows(Error, function () {
  abruptSource.filter(function () { throw new Error("callback abrupt"); });
});
if (speciesReads !== 0) throw "species read after abrupt callback";

let tooSmall = new Uint8Array([1, 2]);
tooSmall.constructor = {};
tooSmall.constructor[Symbol.species] = function () { return new Uint8Array(1); };
__lilaAssertThrows(TypeError, function () {
  tooSmall.filter(function () { return true; });
});

let wrongContentType = new Uint8Array([1]);
wrongContentType.constructor = {};
wrongContentType.constructor[Symbol.species] = function () { return new BigInt64Array(1); };
__lilaAssertThrows(TypeError, function () {
  wrongContentType.filter(function () { return true; });
});

for (let invalidReceiver of [{}, [], Object.create(sample)]) {
  __lilaAssertThrows(TypeError, function () {
    filter.call(invalidReceiver, function () {});
  });
}
__lilaAssertThrows(TypeError, function () { filter.call(new Uint8Array(), {}); });

127;
