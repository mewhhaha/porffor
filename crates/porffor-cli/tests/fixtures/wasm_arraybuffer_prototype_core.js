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

let clamped = buffer.slice(99, 100);
if (clamped.byteLength !== 0) throw "slice clamp";

let tagDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, Symbol.toStringTag);
tagDesc.value;

if (!Object.isExtensible(ArrayBuffer.prototype.slice)) throw "slice extensible";

let detachedDesc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "detached");
if (detachedDesc.set !== undefined) throw "detached setter";
if (buffer.detached !== false) throw "detached false";
__porfDetachArrayBuffer(buffer);
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
