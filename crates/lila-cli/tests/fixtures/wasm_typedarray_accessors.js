let typedArrayPrototype = Object.getPrototypeOf(Int8Array).prototype;
if (typedArrayPrototype === undefined) throw "typedarray prototype missing";
let concretePrototypeParent = Object.getPrototypeOf(Int8Array.prototype);
if (concretePrototypeParent === typedArrayPrototype) {} else throw "concrete prototype chain";

let byteLengthDesc = Object.getOwnPropertyDescriptor(typedArrayPrototype, "byteLength");
let byteOffsetDesc = Object.getOwnPropertyDescriptor(typedArrayPrototype, "byteOffset");
let lengthDesc = Object.getOwnPropertyDescriptor(typedArrayPrototype, "length");
let toStringTagDesc = Object.getOwnPropertyDescriptor(typedArrayPrototype, Symbol.toStringTag);

if (byteLengthDesc === undefined) throw "byteLength descriptor missing";
if (byteLengthDesc.set !== undefined) throw "byteLength setter";
if (byteOffsetDesc.set !== undefined) throw "byteOffset setter";
if (lengthDesc.set !== undefined) throw "length setter";
if (byteLengthDesc.value !== undefined) throw "byteLength data descriptor";
if (typeof byteLengthDesc.get !== "function") throw "byteLength getter";
if (typeof byteOffsetDesc.get !== "function") throw "byteOffset getter";
if (typeof lengthDesc.get !== "function") throw "length getter";
if (toStringTagDesc.set !== undefined) throw "toStringTag setter";
if (typeof toStringTagDesc.get !== "function") throw "toStringTag getter";
if (byteLengthDesc.enumerable !== false) throw "byteLength enumerable";
if (byteOffsetDesc.enumerable !== false) throw "byteOffset enumerable";
if (lengthDesc.enumerable !== false) throw "length enumerable";
if (byteLengthDesc.configurable !== true) throw "byteLength configurable";
if (byteOffsetDesc.configurable !== true) throw "byteOffset configurable";
if (lengthDesc.configurable !== true) throw "length configurable";
if (toStringTagDesc.enumerable !== false) throw "toStringTag enumerable";
if (toStringTagDesc.configurable !== true) throw "toStringTag configurable";

if (Object.getOwnPropertyDescriptor(byteLengthDesc.get, "length").value !== 0) throw "byteLength getter length";
if (Object.getOwnPropertyDescriptor(byteOffsetDesc.get, "length").value !== 0) throw "byteOffset getter length";
if (Object.getOwnPropertyDescriptor(lengthDesc.get, "length").value !== 0) throw "length getter length";
if (Object.getOwnPropertyDescriptor(toStringTagDesc.get, "length").value !== 0) throw "toStringTag getter length";
if (Object.getOwnPropertyDescriptor(byteLengthDesc.get, "name").value !== "get byteLength") throw "byteLength getter name";
if (Object.getOwnPropertyDescriptor(byteOffsetDesc.get, "name").value !== "get byteOffset") throw "byteOffset getter name";
if (Object.getOwnPropertyDescriptor(lengthDesc.get, "name").value !== "get length") throw "length getter name";
if (Object.getOwnPropertyDescriptor(toStringTagDesc.get, "name").value !== "get [Symbol.toStringTag]") throw "toStringTag getter name";
let dynamicLengthKey = "length";
let dynamicNameKey = "name";
if (Object.getOwnPropertyDescriptor(byteLengthDesc.get, dynamicLengthKey).value !== 0) throw "byteLength getter dynamic length";
if (Object.getOwnPropertyDescriptor(byteOffsetDesc.get, dynamicNameKey).value !== "get byteOffset") throw "byteOffset getter dynamic name";
if (Object.getOwnPropertyDescriptor(lengthDesc.get, dynamicNameKey).value !== "get length") throw "length getter dynamic name";

let fixedBuffer = new ArrayBuffer(24);
let fixed = new Uint16Array(fixedBuffer, 4);
let fixedPrototype = Object.getPrototypeOf(fixed);
if (fixedPrototype === Uint16Array.prototype) {} else throw "instance concrete prototype";
if (Object.getOwnPropertyDescriptor(fixed, "byteLength") !== undefined) throw "instance own byteLength";
if (Object.getOwnPropertyDescriptor(fixed, "byteOffset") !== undefined) throw "instance own byteOffset";
if (Object.getOwnPropertyDescriptor(fixed, "length") !== undefined) throw "instance own length";
if (Object.getOwnPropertyDescriptor(Uint16Array.prototype, "byteLength") !== undefined) throw "concrete own byteLength";
if (Object.getOwnPropertyDescriptor(Uint16Array.prototype, "byteOffset") !== undefined) throw "concrete own byteOffset";
if (Object.getOwnPropertyDescriptor(Uint16Array.prototype, "length") !== undefined) throw "concrete own length";
if (Object.getOwnPropertyDescriptor({}, "byteLength") !== undefined) throw "plain own byteLength";
if (fixed.byteOffset !== 4) throw "fixed byteOffset";
if (fixed.byteLength !== 20) throw "fixed byteLength";
if (fixed.length !== 10) throw "fixed length";
if (byteLengthDesc.get.call(fixed) !== 20) throw "call byteLength";
if (byteOffsetDesc.get.call(fixed) !== 4) throw "call byteOffset";
if (lengthDesc.get.call(fixed) !== 10) throw "call length";
if (toStringTagDesc.get.call(fixed) !== "Uint16Array") throw "call toStringTag";
if (fixed[Symbol.toStringTag] !== "Uint16Array") throw "inherited toStringTag";
if (new BigInt64Array()[Symbol.toStringTag] !== "BigInt64Array") throw "bigint toStringTag";
if (toStringTagDesc.get.call({}) !== undefined) throw "plain toStringTag";
if (toStringTagDesc.get.call(undefined) !== undefined) throw "undefined toStringTag";

let ordinaryLengthReads = 0;
let spoofedSlotReads = 0;
let ordinaryLength = {
  get length() {
    ordinaryLengthReads = ordinaryLengthReads + 1;
    return 17;
  },
  get $TypedArrayByteLength() {
    spoofedSlotReads = spoofedSlotReads + 1;
    return 8;
  }
};
if (ordinaryLength.length !== 17) throw "ordinary spoofed length";
if (ordinaryLengthReads !== 1) throw "ordinary length Get";
if (spoofedSlotReads !== 0) throw "ordinary spoofed slot observed";

function functionLength(first, second) {}
Object.defineProperty(functionLength, "$TypedArrayByteLength", {
  get() {
    spoofedSlotReads = spoofedSlotReads + 1;
    return 8;
  }
});
if (functionLength.length !== 2) throw "function spoofed length";
if (spoofedSlotReads !== 0) throw "function spoofed slot observed";

let proxyLengthReads = "";
let proxiedLength = new Proxy({
  length: 23,
  $TypedArrayByteLength: 8
}, {
  get(target, key, receiver) {
    proxyLengthReads = proxyLengthReads + String(key) + ",";
    return Reflect.get(target, key, receiver);
  }
});
if (proxiedLength.length !== 23) throw "proxy spoofed length";
if (proxyLengthReads !== "length,") throw "proxy length Get";

let brandedLength = new Uint8Array(6);
for (let internalName of [
  "$TypedArrayViewedArrayBuffer",
  "$TypedArrayByteOffset",
  "$TypedArrayByteLength",
  "$TypedArrayBytesPerElement",
  "$TypedArrayElementKind",
  "$TypedArrayLengthTracking"
]) {
  if (Object.hasOwn(brandedLength, internalName)) throw internalName + " exposed";
}
Object.defineProperty(brandedLength, "$TypedArrayByteLength", {
  get() {
    spoofedSlotReads = spoofedSlotReads + 1;
    return 64;
  }
});
if (brandedLength.length !== 6) throw "branded spoofed length";
if (spoofedSlotReads !== 0) throw "branded spoofed slot observed";

let inheritedTypedArray = {};
Object.setPrototypeOf(inheritedTypedArray, fixed);
__lilaAssertThrows(TypeError, function () {
  byteLengthDesc.get.call(inheritedTypedArray);
});
__lilaAssertThrows(TypeError, function () {
  byteOffsetDesc.get.call(inheritedTypedArray);
});
__lilaAssertThrows(TypeError, function () {
  lengthDesc.get.call(inheritedTypedArray);
});

let sized = new Float32Array(3);
if (sized.byteOffset !== 0) throw "sized byteOffset";
if (sized.byteLength !== 12) throw "sized byteLength";
if (sized.length !== 3) throw "sized length";
if (Float32Array.BYTES_PER_ELEMENT !== 4) throw "bytes per element";
if (4 * Float32Array.BYTES_PER_ELEMENT !== 16) throw "bytes per element expression";
function bytesPerElementProduct(TA) {
  return 4 * TA.BYTES_PER_ELEMENT;
}
if (bytesPerElementProduct(Int8Array) !== 4) throw "bytes per element parameter";

let detachedBuffer = new ArrayBuffer(8);
let detached = new Uint32Array(detachedBuffer);
__lilaDetachArrayBuffer(detachedBuffer);
if (detached.byteOffset !== 0) throw "detached byteOffset";
if (detached.byteLength !== 0) throw "detached byteLength";
if (detached.length !== 0) throw "detached length";
if (detached[Symbol.toStringTag] !== "Uint32Array") throw "detached toStringTag";

__lilaAssertThrows(TypeError, function () {
  byteLengthDesc.get.call({});
});
__lilaAssertThrows(TypeError, function () {
  byteOffsetDesc.get.call(undefined);
});
__lilaAssertThrows(TypeError, function () {
  lengthDesc.get.call(new DataView(new ArrayBuffer(4)));
});

let resizable = new ArrayBuffer(16, { maxByteLength: 32 });
let auto = new Uint8Array(resizable, 4);
resizable.resize(24);
if (auto.byteOffset !== 4) throw "resize byteOffset";
if (auto.byteLength !== 20) throw "resize byteLength";
if (auto.length !== 20) throw "resize length";

let fixedResizable = new ArrayBuffer(16, { maxByteLength: 40 });
let fixedView = new Uint8Array(fixedResizable, 8, 8);
fixedResizable.resize(10);
if (fixedView.byteOffset !== 0) throw "fixed resized out byteOffset";
if (fixedView.byteLength !== 0) throw "fixed resized out byteLength";
if (fixedView.length !== 0) throw "fixed resized out length";
fixedResizable.resize(16);
if (fixedView.byteOffset !== 8) throw "fixed resized in byteOffset";
if (fixedView.byteLength !== 8) throw "fixed resized in byteLength";
if (fixedView.length !== 8) throw "fixed resized in length";

let trackingBuffer = new ArrayBuffer(20, { maxByteLength: 40 });
let trackingView = new Uint16Array(trackingBuffer, 4);
if (trackingView.byteOffset !== 4) throw "tracking initial byteOffset";
if (trackingView.byteLength !== 16) throw "tracking initial byteLength";
if (trackingView.length !== 8) throw "tracking initial length";
trackingBuffer.resize(10);
if (trackingView.byteOffset !== 4) throw "tracking shrink byteOffset";
if (trackingView.byteLength !== 6) throw "tracking shrink byteLength";
if (trackingView.length !== 3) throw "tracking shrink length";
trackingBuffer.resize(2);
if (trackingView.byteOffset !== 0) throw "tracking beyond byteOffset";
if (trackingView.byteLength !== 0) throw "tracking beyond byteLength";
if (trackingView.length !== 0) throw "tracking beyond length";

let spoofedTypedArray = {
  0: "ordinary value",
  subarray: Uint8Array.prototype.subarray
};
if (spoofedTypedArray[0] !== "ordinary value") throw "spoofed typed array inference";

123;
