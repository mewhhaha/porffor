if ("isView" in SharedArrayBuffer) throw "SharedArrayBuffer.isView";

let arrayBufferOnlyMethods = [
  "resize",
  "transfer",
  "transferToFixedLength",
  "transferToImmutable",
  "sliceToImmutable"
];
for (let index = 0; index < arrayBufferOnlyMethods.length; index++) {
  let name = arrayBufferOnlyMethods[index];
  if (Object.prototype.hasOwnProperty.call(SharedArrayBuffer.prototype, name)) {
    throw "SharedArrayBuffer.prototype." + name;
  }
}

if (typeof SharedArrayBuffer.prototype.grow !== "function") throw "grow";
if (typeof SharedArrayBuffer.prototype.slice !== "function") throw "slice";
if (SharedArrayBuffer[Symbol.species] !== SharedArrayBuffer) throw "species";

123;
