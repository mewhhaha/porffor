let buffer = new ArrayBuffer(4);
let view = new DataView(buffer);
view.setUint8(0, 11);
view.setUint8(1, 22);
view.setUint8(2, 33);
view.setUint8(3, 44);

let sliced = buffer.slice(1, -1);
let slicedView = new DataView(sliced);
if (sliced.byteLength !== 2) throw "slice length";
if (slicedView.getUint8(0) !== 22) throw "slice first byte";
if (slicedView.getUint8(1) !== 33) throw "slice second byte";

let sliceDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "slice");
if (typeof sliceDesc.value !== "function") throw "slice function";
if (sliceDesc.writable !== true) throw "slice writable";
if (sliceDesc.enumerable !== false) throw "slice enumerable";
if (sliceDesc.configurable !== true) throw "slice configurable";

let sliceLengthDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype.slice, "length");
if (ArrayBuffer.prototype.slice.length !== 2) throw "slice length value";
if (sliceLengthDesc.value !== 2) throw "slice length descriptor value";
if (sliceLengthDesc.writable !== false) throw "slice length writable";
if (sliceLengthDesc.enumerable !== false) throw "slice length enumerable";
if (sliceLengthDesc.configurable !== true) throw "slice length configurable";

let sliceNameDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype.slice, "name");
if (ArrayBuffer.prototype.slice.name !== "slice") throw "slice name value";
if (sliceNameDesc.value !== "slice") throw "slice name descriptor value";
if (sliceNameDesc.writable !== false) throw "slice name writable";
if (sliceNameDesc.enumerable !== false) throw "slice name enumerable";
if (sliceNameDesc.configurable !== true) throw "slice name configurable";

__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.slice.call(undefined);
});

__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.slice.call(null);
});

__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.slice.call(Symbol());
});

__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.slice.call({});
});

__lilaAssertThrows(TypeError, function () {
  ArrayBuffer.prototype.slice.call([]);
});

let clamped = buffer.slice(99, 100);
if (clamped.byteLength !== 0) throw "slice clamp";

let tagDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, Symbol.toStringTag);
tagDesc.value;

if (!Object.isExtensible(ArrayBuffer.prototype.slice)) throw "slice extensible";

let detachedDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "detached");
if (detachedDesc.set !== undefined) throw "detached setter";
if (buffer.detached !== false) throw "detached false";
__lilaDetachArrayBuffer(buffer);
if (buffer.detached !== true) throw "detached true";

let speciesObject = {};
speciesObject[Symbol.species] = function(length) {
  return new ArrayBuffer(10);
};
let speciesBuffer = new ArrayBuffer(8);
speciesBuffer.constructor = speciesObject;
let speciesResult = speciesBuffer.slice();
if (speciesResult.byteLength !== 10) throw "species larger";

let invalidSpecies = {};
invalidSpecies[Symbol.species] = function(length) {
  return {};
};
let invalidBuffer = new ArrayBuffer(8);
invalidBuffer.constructor = invalidSpecies;
let threw = false;
try {
  invalidBuffer.slice();
} catch (error) {
  threw = true;
}
if (!threw) throw "species invalid";

123;
