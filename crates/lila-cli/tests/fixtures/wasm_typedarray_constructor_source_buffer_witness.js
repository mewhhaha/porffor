function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertErrorPrototype(callback, expectedPrototype, label) {
  try {
    callback();
  } catch (error) {
    assertSame(Object.getPrototypeOf(error), expectedPrototype, label);
    return;
  }
  throw label + " did not throw";
}

var detachedSource = new Uint8Array(1);
__lilaDetachArrayBuffer(detachedSource.buffer);
assertErrorPrototype(function () {
  new Uint8Array(detachedSource);
}, TypeError.prototype, "constructor detached source");

var outOfBoundsBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var outOfBoundsSource = new Uint8Array(outOfBoundsBuffer, 2, 2);
outOfBoundsBuffer.resize(1);
assertErrorPrototype(function () {
  new Uint8Array(outOfBoundsSource);
}, TypeError.prototype, "constructor out-of-bounds source");

var oddByteBuffer = new ArrayBuffer(5, { maxByteLength: 5 });
var oddByteSource = new Uint16Array(oddByteBuffer);
oddByteSource[0] = 513;
oddByteSource[1] = 1027;
var oddByteClone = new Uint16Array(oddByteSource);
assertSame(oddByteClone.length, 2, "odd-byte source length snapshot");
assertSame(oddByteClone[0], 513, "odd-byte source first element");
assertSame(oddByteClone[1], 1027, "odd-byte source second element");
assertSame(Object.getPrototypeOf(oddByteClone), Uint16Array.prototype, "target prototype");

var regrownBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var regrownSource = new Uint16Array(regrownBuffer, 2, 1);
regrownSource[0] = 1541;
regrownBuffer.resize(1);
regrownBuffer.resize(4);
var regrownClone = new Uint16Array(regrownSource);
assertSame(regrownClone.length, 1, "fixed source regrowth");
assertSame(regrownClone[0], 0, "fixed source regrown contents");

var throwSlotGets = 0;
var throwSlotSource = new Uint8Array([17, 29]);
throwSlotSource.__proto__ = {
  get $LilaGeneratorThrow() {
    throwSlotGets = throwSlotGets + 1;
    throw "TypedArray source observed generator throw slot";
  },
};
var throwSlotClone = new Uint8Array(throwSlotSource);
assertSame(throwSlotGets, 0, "TypedArray source skips generator throw slot");
assertSame(throwSlotClone[0], 17, "throw-slot clone first element");
assertSame(throwSlotClone[1], 29, "throw-slot clone second element");

assertErrorPrototype(function () {
  new BigInt64Array(new Uint8Array([1]));
}, TypeError.prototype, "Number source into BigInt target");
assertErrorPrototype(function () {
  new Uint8Array(new BigInt64Array([1n]));
}, TypeError.prototype, "BigInt source into Number target");

true;
