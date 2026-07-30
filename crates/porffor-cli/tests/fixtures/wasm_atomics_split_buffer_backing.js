function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

var ordinary = new Uint8Array(new ArrayBuffer(8));
var sharedBuffer = new SharedArrayBuffer(8, { maxByteLength: 16 });
var shared = new Uint8Array(sharedBuffer);

ordinary[0] = 17;
shared[0] = 34;
Atomics.store(shared, 1, 51);

assertSame(ordinary[0], 17, "ordinary buffer remains distinct");
assertSame(shared[0], 34, "shared indexed read");
assertSame(shared[1], 51, "atomic write is visible to indexed read");

var view = new DataView(sharedBuffer);
view.setUint16(2, 0x1234);
assertSame(view.getUint16(2), 0x1234, "DataView shared backing");

var sliced = sharedBuffer.slice(0, 4);
assertSame(new Uint8Array(sliced)[0], 34, "SharedArrayBuffer slice");

sharedBuffer.grow(12);
shared[8] = 68;
assertSame(shared[8], 68, "SharedArrayBuffer grow");

var copied = new Uint8Array([85, 102]);
assertSame(copied[0], 85, "TypedArray source copy");
assertSame(ordinary[0], 17, "no cross-buffer corruption");

927;
