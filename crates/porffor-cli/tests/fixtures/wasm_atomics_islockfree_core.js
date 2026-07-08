if (typeof Atomics !== "object") throw "Atomics object";
if (Atomics !== globalThis.Atomics) throw "global binding";
if (typeof Atomics.isLockFree !== "function") throw "isLockFree function";
if (Atomics.isLockFree.length !== 1) throw "isLockFree length";
if (Atomics.isLockFree.name !== "isLockFree") throw "isLockFree name";

let desc = Object.getOwnPropertyDescriptor(Atomics, "isLockFree");
if (desc.enumerable !== false) throw "isLockFree enumerable";
if (desc.writable !== true) throw "isLockFree writable";
if (desc.configurable !== true) throw "isLockFree configurable";

function verifyProperty(obj, name, expected) {
  let originalDesc = Object.getOwnPropertyDescriptor(obj, name);
  if (originalDesc === undefined) throw "descriptor exists";
  if (expected.value !== undefined) {
    if (originalDesc.value !== expected.value) throw "descriptor value";
    if (obj[name] !== expected.value) throw "property value";
  }
  if (expected.writable !== undefined && originalDesc.writable !== expected.writable) throw "descriptor writable";
  if (expected.enumerable !== undefined && originalDesc.enumerable !== expected.enumerable) throw "descriptor enumerable";
  if (expected.configurable !== undefined && originalDesc.configurable !== expected.configurable) throw "descriptor configurable";
}

verifyProperty(Atomics, "isLockFree", {
  enumerable: false,
  writable: true,
  configurable: true
});
verifyProperty(Atomics.isLockFree, "length", {
  value: 1,
  enumerable: false,
  writable: false,
  configurable: true
});
verifyProperty(Atomics.isLockFree, "name", {
  value: "isLockFree",
  enumerable: false,
  writable: false,
  configurable: true
});

for (let key in Atomics) {
  if (key === "isLockFree") throw "isLockFree for-in";
}

if (Atomics.isLockFree(4) !== true) throw "4-byte lock-free";
if (Atomics.isLockFree(4.9) !== true) throw "ToInteger 4.9";
if (Atomics.isLockFree("4") !== true) throw "string size";
if (Atomics.isLockFree(true) !== false) throw "boolean size";
if (Atomics.isLockFree(0) !== false) throw "zero size";
if (Atomics.isLockFree(3) !== false) throw "3-byte size";
if (Atomics.isLockFree(5) !== false) throw "5-byte size";
if (BigInt64Array.BYTES_PER_ELEMENT !== 8) throw "BigInt64Array bytes";
if (BigUint64Array.BYTES_PER_ELEMENT !== 8) throw "BigUint64Array bytes";
if (Atomics.isLockFree(BigInt64Array.BYTES_PER_ELEMENT) !== Atomics.isLockFree(BigInt64Array.BYTES_PER_ELEMENT)) throw "BigInt64Array lock-free stable";
if (Atomics.isLockFree(BigUint64Array.BYTES_PER_ELEMENT) !== Atomics.isLockFree(BigUint64Array.BYTES_PER_ELEMENT)) throw "BigUint64Array lock-free stable";
if (Atomics.isLockFree(NaN) !== false) throw "NaN size";
if (Atomics.isLockFree(Infinity) !== false) throw "infinity size";

let valueOfCount = 0;
let objectSize = {
  valueOf: function () {
    valueOfCount = valueOfCount + 1;
    return 4;
  }
};
if (Atomics.isLockFree(objectSize) !== true) throw "object size";
if (valueOfCount !== 1) throw "object coercion count";

__porfAssertThrows(TypeError, function () {
  Atomics.isLockFree(4n);
});

let f = Atomics.isLockFree;
for (let key in f) {
  if (key === "length" || key === "name") throw "function metadata for-in";
}
f.length = "unlikelyValue";
if (f.length !== 1) throw "length writable";
f.name = "unlikelyValue";
if (f.name !== "isLockFree") throw "name writable";
if (delete f.length !== true) throw "length delete result";
if (Object.getOwnPropertyDescriptor(f, "length") !== undefined) throw "length configurable";
if (delete f.name !== true) throw "name delete result";
if (Object.getOwnPropertyDescriptor(f, "name") !== undefined) throw "name configurable";

123;
