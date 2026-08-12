let typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
let includes = typedArrayPrototype.includes;
let indexOf = typedArrayPrototype.indexOf;
let lastIndexOf = typedArrayPrototype.lastIndexOf;

if (includes === Array.prototype.includes) throw "includes intrinsic identity";
if (indexOf === Array.prototype.indexOf) throw "indexOf intrinsic identity";
if (lastIndexOf === Array.prototype.lastIndexOf) throw "lastIndexOf intrinsic identity";
for (let methodName of ["includes", "indexOf", "lastIndexOf"]) {
  let descriptor = Object.getOwnPropertyDescriptor(typedArrayPrototype, methodName);
  if (descriptor === undefined) throw methodName + " descriptor";
  if (descriptor.value !== typedArrayPrototype[methodName]) throw methodName + " value";
  if (descriptor.writable !== true) throw methodName + " writable";
  if (descriptor.enumerable !== false) throw methodName + " enumerable";
  if (descriptor.configurable !== true) throw methodName + " configurable";
  if (descriptor.value.name !== methodName) throw methodName + " name";
  if (descriptor.value.length !== 1) throw methodName + " length";
}

let floats = new Float64Array([NaN, -0, 2, 2]);
if (!floats.includes(NaN)) throw "includes SameValueZero NaN";
if (floats.indexOf(NaN) !== -1) throw "indexOf strict NaN";
if (floats.lastIndexOf(NaN) !== -1) throw "lastIndexOf strict NaN";
if (!floats.includes(0) || floats.indexOf(0) !== 1 || floats.lastIndexOf(0) !== 1) {
  throw "signed zero equality";
}
if (floats.includes(2, 3) !== true) throw "includes positive fromIndex";
if (floats.includes(2, 4) !== false) throw "includes length fromIndex";
if (floats.includes(2, -2) !== true) throw "includes negative fromIndex";
if (floats.indexOf(2, -1) !== 3) throw "indexOf negative fromIndex";
if (floats.indexOf(2, Infinity) !== -1) throw "indexOf positive infinity";
if (floats.indexOf(2, -Infinity) !== 2) throw "indexOf negative infinity";
if (floats.lastIndexOf(2) !== 3) throw "lastIndexOf default";
if (floats.lastIndexOf(2, undefined) !== -1) throw "lastIndexOf explicit undefined";
if (floats.lastIndexOf(2, Infinity) !== 3) throw "lastIndexOf positive infinity";
if (floats.lastIndexOf(2, -Infinity) !== -1) throw "lastIndexOf negative infinity";

let internalLength = new Uint8Array([4, 5, 6]);
Object.defineProperty(internalLength, "length", {
  get: function () {
    throw "observable length";
  },
});
if (!internalLength.includes(5) || internalLength.indexOf(6) !== 2) {
  throw "internal typed array length";
}

let bigint = new BigInt64Array([1n, -2n, 1n]);
if (!bigint.includes(-2n)) throw "bigint includes";
if (bigint.indexOf(1n) !== 0) throw "bigint indexOf";
if (bigint.lastIndexOf(1n) !== 2) throw "bigint lastIndexOf";
if (bigint.includes(1) || bigint.indexOf(1) !== -1 || bigint.lastIndexOf(1) !== -1) {
  throw "bigint search type";
}
let bigUnsigned = new BigUint64Array([18446744073709551615n]);
if (!bigUnsigned.includes(18446744073709551615n)) throw "big uint includes";
if (bigUnsigned.indexOf(18446744073709551615n) !== 0) throw "big uint indexOf";

let emptyCoercions = 0;
let unusedFromIndex = {
  valueOf: function () {
    emptyCoercions = emptyCoercions + 1;
    return 0;
  },
};
if (new Uint8Array().includes(0, unusedFromIndex)) throw "empty includes";
if (new Uint8Array().indexOf(0, unusedFromIndex) !== -1) throw "empty indexOf";
if (new Uint8Array().lastIndexOf(0, unusedFromIndex) !== -1) throw "empty lastIndexOf";
if (emptyCoercions !== 0) throw "empty fromIndex coercion";

let searchElement = {
  valueOf: function () {
    throw "search element coercion";
  },
};
if (new Uint8Array([1]).includes(searchElement)) throw "includes search coercion";
if (new Uint8Array([1]).indexOf(searchElement) !== -1) throw "indexOf search coercion";

for (let invalidReceiver of [{}, [], Object.create(new Uint8Array([1]))]) {
  __lilaAssertThrows(TypeError, function () { includes.call(invalidReceiver, 1); });
  __lilaAssertThrows(TypeError, function () { indexOf.call(invalidReceiver, 1); });
  __lilaAssertThrows(TypeError, function () { lastIndexOf.call(invalidReceiver, 1); });
}
let invalidCoercions = 0;
let invalidFromIndex = {
  valueOf: function () {
    invalidCoercions = invalidCoercions + 1;
    return 0;
  },
};
__lilaAssertThrows(TypeError, function () { includes.call({}, 1, invalidFromIndex); });
if (invalidCoercions !== 0) throw "receiver validation order";

let growBuffer = new ArrayBuffer(2, { maxByteLength: 4 });
let growView = new Uint8Array(growBuffer);
growView[0] = 1;
growView[1] = 2;
let growFromIndex = {
  valueOf: function () {
    growBuffer.resize(4);
    growView[2] = 9;
    growView[3] = 9;
    return 0;
  },
};
if (growView.includes(9, growFromIndex)) throw "includes grow snapshot";

let growLastBuffer = new ArrayBuffer(2, { maxByteLength: 4 });
let growLastView = new Uint8Array(growLastBuffer);
let growLastFromIndex = {
  valueOf: function () {
    growLastBuffer.resize(4);
    growLastView[2] = 9;
    return Infinity;
  },
};
if (growLastView.lastIndexOf(9, growLastFromIndex) !== -1) {
  throw "lastIndexOf grow snapshot";
}

let shrinkIncludesBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let shrinkIncludesView = new Uint8Array(shrinkIncludesBuffer, 0, 4);
let shrinkIncludesFromIndex = {
  valueOf: function () {
    shrinkIncludesBuffer.resize(2);
    return 0;
  },
};
if (!shrinkIncludesView.includes(undefined, shrinkIncludesFromIndex)) {
  throw "includes shrink undefined";
}

let shrinkIndexBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let shrinkIndexView = new Uint8Array(shrinkIndexBuffer, 0, 4);
let shrinkIndexFromIndex = {
  valueOf: function () {
    shrinkIndexBuffer.resize(2);
    return 0;
  },
};
if (shrinkIndexView.indexOf(undefined, shrinkIndexFromIndex) !== -1) {
  throw "indexOf shrink undefined";
}

let detachIncludes = new Uint8Array([1, 2]);
let detachIncludesFromIndex = {
  valueOf: function () {
    __lilaDetachArrayBuffer(detachIncludes.buffer);
    return 0;
  },
};
if (!detachIncludes.includes(undefined, detachIncludesFromIndex)) {
  throw "includes detached undefined";
}

let detachIndex = new Uint8Array([1, 2]);
let detachIndexFromIndex = {
  valueOf: function () {
    __lilaDetachArrayBuffer(detachIndex.buffer);
    return 0;
  },
};
if (detachIndex.indexOf(undefined, detachIndexFromIndex) !== -1) {
  throw "indexOf detached undefined";
}

let detachLast = new Uint8Array([1, 2]);
let detachLastFromIndex = {
  valueOf: function () {
    __lilaDetachArrayBuffer(detachLast.buffer);
    return Infinity;
  },
};
if (detachLast.lastIndexOf(undefined, detachLastFromIndex) !== -1) {
  throw "lastIndexOf detached undefined";
}

let initiallyDetached = new Uint8Array([1]);
__lilaDetachArrayBuffer(initiallyDetached.buffer);
__lilaAssertThrows(TypeError, function () { includes.call(initiallyDetached, 1); });
__lilaAssertThrows(TypeError, function () { indexOf.call(initiallyDetached, 1); });
__lilaAssertThrows(TypeError, function () { lastIndexOf.call(initiallyDetached, 1); });

function runOddUint16Search(method) {
  let buffer = new ArrayBuffer(4, { maxByteLength: 4 });
  let view = new Uint16Array(buffer);
  view[1] = 0x5678;
  let fromIndex = {
    valueOf: function () {
      buffer.resize(3);
      return 1;
    },
  };
  return method.call(view, undefined, fromIndex);
}
if (runOddUint16Search(includes) !== true) throw "typed includes odd-byte floor";
if (runOddUint16Search(indexOf) !== -1) throw "typed indexOf odd-byte floor";
if (runOddUint16Search(lastIndexOf) !== -1) throw "typed lastIndexOf odd-byte floor";
if (runOddUint16Search(Array.prototype.includes) !== true) {
  throw "array includes odd-byte floor";
}
if (runOddUint16Search(Array.prototype.indexOf) !== -1) {
  throw "array indexOf odd-byte floor";
}
if (runOddUint16Search(Array.prototype.lastIndexOf) !== -1) {
  throw "array lastIndexOf odd-byte floor";
}

let fixedRegrowBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let fixedRegrowView = new Uint8Array(fixedRegrowBuffer, 0, 4);
fixedRegrowView[2] = 9;
fixedRegrowBuffer.resize(2);
let fixedIndexCalls = 0;
let fixedIndex = {
  valueOf: function () {
    fixedIndexCalls = fixedIndexCalls + 1;
    return 0;
  },
};
if (Array.prototype.indexOf.call(fixedRegrowView, 9, fixedIndex) !== -1) {
  throw "array indexOf fixed out of bounds";
}
if (Array.prototype.lastIndexOf.call(fixedRegrowView, 9, fixedIndex) !== -1) {
  throw "array lastIndexOf fixed out of bounds";
}
if (fixedIndexCalls !== 0) throw "array fixed out-of-bounds length";
__lilaAssertThrows(TypeError, function () { includes.call(fixedRegrowView, 9); });
fixedRegrowBuffer.resize(4);
fixedRegrowView[2] = 9;
if (!includes.call(fixedRegrowView, 9)) throw "typed includes fixed regrow";
if (indexOf.call(fixedRegrowView, 9) !== 2) throw "typed indexOf fixed regrow";
if (lastIndexOf.call(fixedRegrowView, 9) !== 2) throw "typed lastIndexOf fixed regrow";
if (Array.prototype.indexOf.call(fixedRegrowView, 9) !== 2) {
  throw "array indexOf fixed regrow";
}
if (Array.prototype.lastIndexOf.call(fixedRegrowView, 9) !== 2) {
  throw "array lastIndexOf fixed regrow";
}

132;
