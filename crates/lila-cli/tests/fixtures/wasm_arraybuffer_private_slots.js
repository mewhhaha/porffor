function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

function assertTypeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof TypeError)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

let arrayBufferByteLength = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype,
  "byteLength"
).get;
let arrayBufferDetached = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype,
  "detached"
).get;
let arrayBufferMaxByteLength = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype,
  "maxByteLength"
).get;
let arrayBufferResizable = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype,
  "resizable"
).get;
let sharedByteLength = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype,
  "byteLength"
).get;
let sharedMaxByteLength = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype,
  "maxByteLength"
).get;
let sharedGrowable = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype,
  "growable"
).get;

let spoofedBuffer = {
  $ArrayBufferDataPtr: 0,
  $ArrayBufferByteLength: 8,
  "$ArrayBuffer.maxByteLength": 8,
  "$ArrayBuffer.resizable": true,
  "$ArrayBuffer.shared": false,
  "$ArrayBuffer.immutable": false
};
assertTypeError(function () {
  arrayBufferByteLength.call(spoofedBuffer);
}, "spoofed byteLength");
assertTypeError(function () {
  arrayBufferDetached.call(spoofedBuffer);
}, "spoofed detached");
assertTypeError(function () {
  arrayBufferMaxByteLength.call(spoofedBuffer);
}, "spoofed maxByteLength");
assertTypeError(function () {
  arrayBufferResizable.call(spoofedBuffer);
}, "spoofed resizable");
assertTypeError(function () {
  ArrayBuffer.prototype.slice.call(spoofedBuffer, 0, 1);
}, "spoofed slice");
assertTypeError(function () {
  ArrayBuffer.prototype.sliceToImmutable.call(spoofedBuffer, 0, 1);
}, "spoofed immutable slice");

let spoofedShared = {
  $ArrayBufferDataPtr: 0,
  $ArrayBufferByteLength: 8,
  "$ArrayBuffer.maxByteLength": 8,
  "$ArrayBuffer.resizable": true,
  "$ArrayBuffer.shared": true
};
assertTypeError(function () {
  sharedByteLength.call(spoofedShared);
}, "spoofed shared byteLength");
assertTypeError(function () {
  sharedMaxByteLength.call(spoofedShared);
}, "spoofed shared maxByteLength");
assertTypeError(function () {
  sharedGrowable.call(spoofedShared);
}, "spoofed shared growable");
assertTypeError(function () {
  SharedArrayBuffer.prototype.slice.call(spoofedShared, 0, 1);
}, "spoofed shared slice");

let spoofedSpecies = {
  [Symbol.species]: function () {
    return spoofedBuffer;
  }
};
let speciesSource = new ArrayBuffer(2);
speciesSource.constructor = spoofedSpecies;
assertTypeError(function () {
  speciesSource.slice();
}, "spoofed species result");

let spoofedSharedSpecies = {
  [Symbol.species]: function () {
    return spoofedShared;
  }
};
let sharedSpeciesSource = new SharedArrayBuffer(2);
sharedSpeciesSource.constructor = spoofedSharedSpecies;
assertTypeError(function () {
  sharedSpeciesSource.slice();
}, "spoofed shared species result");

let slotReads = 0;
function poisonSlot(object, key, value) {
  Object.defineProperty(object, key, {
    configurable: true,
    get() {
      slotReads = slotReads + 1;
      return value;
    }
  });
}

let buffer = new ArrayBuffer(4, { maxByteLength: 8 });
for (let internalName of [
  "$ArrayBufferDataPtr",
  "$ArrayBufferByteLength",
  "$ArrayBuffer.maxByteLength",
  "$ArrayBuffer.resizable",
  "$ArrayBuffer.shared",
  "$ArrayBuffer.immutable"
]) {
  if (Object.hasOwn(buffer, internalName)) throw internalName + " exposed";
}
let bytes = new Uint8Array(buffer);
bytes[0] = 17;
bytes[1] = 34;
poisonSlot(buffer, "$ArrayBufferDataPtr", 0);
poisonSlot(buffer, "$ArrayBufferByteLength", 0);
poisonSlot(buffer, "$ArrayBuffer.maxByteLength", 0);
poisonSlot(buffer, "$ArrayBuffer.resizable", false);
poisonSlot(buffer, "$ArrayBuffer.shared", true);
poisonSlot(buffer, "$ArrayBuffer.immutable", true);

assertSame(buffer.byteLength, 4, "private byteLength");
assertSame(buffer.maxByteLength, 8, "private maxByteLength");
assertSame(buffer.resizable, true, "private resizable");
assertSame(buffer.detached, false, "private detached");
let sliced = buffer.slice(0, 2);
assertSame(sliced.byteLength, 2, "private slice length");
let slicedBytes = new Uint8Array(sliced);
assertSame(slicedBytes[0], 17, "private slice byte zero");
assertSame(slicedBytes[1], 34, "private slice byte one");
let immutable = buffer.sliceToImmutable(1, 2);
assertSame(immutable.byteLength, 1, "private immutable slice length");

let shared = new SharedArrayBuffer(4, { maxByteLength: 8 });
for (let internalName of [
  "$ArrayBufferDataPtr",
  "$ArrayBufferByteLength",
  "$ArrayBuffer.maxByteLength",
  "$ArrayBuffer.resizable",
  "$ArrayBuffer.shared",
  "$ArrayBuffer.immutable"
]) {
  if (Object.hasOwn(shared, internalName)) throw "shared " + internalName + " exposed";
}
let sharedBytes = new Uint8Array(shared);
sharedBytes[0] = 51;
poisonSlot(shared, "$ArrayBufferDataPtr", 0);
poisonSlot(shared, "$ArrayBufferByteLength", 0);
poisonSlot(shared, "$ArrayBuffer.maxByteLength", 0);
poisonSlot(shared, "$ArrayBuffer.resizable", false);
poisonSlot(shared, "$ArrayBuffer.shared", false);

assertSame(shared.byteLength, 4, "private shared byteLength");
assertSame(shared.maxByteLength, 8, "private shared maxByteLength");
assertSame(shared.growable, true, "private shared growable");
let sharedSlice = shared.slice(0, 1);
assertSame(sharedSlice.byteLength, 1, "private shared slice length");
assertSame(new Uint8Array(sharedSlice)[0], 51, "private shared slice byte");
assertSame(slotReads, 0, "private slot reads");

941;
