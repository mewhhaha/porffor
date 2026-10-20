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

let spoofedSlotReads = 0;
let fakeBuffer = {
  $ArrayBufferDataPtr: 0,
  "$ArrayBuffer.shared": true
};
let fakeView = {
  $TypedArrayViewedArrayBuffer: fakeBuffer,
  get $TypedArrayElementKind() {
    spoofedSlotReads = spoofedSlotReads + 1;
    return 5;
  }
};

assertTypeError(function () {
  Atomics.add(fakeView, 0, 1);
}, "spoofed add");
assertTypeError(function () {
  Atomics.notify(fakeView, 0, 0);
}, "spoofed notify");
assertTypeError(function () {
  Atomics.wait(fakeView, 0, 0, 0);
}, "spoofed wait");
assertTypeError(function () {
  Atomics.waitAsync(fakeView, 0, 0, 0);
}, "spoofed waitAsync");
assertSame(spoofedSlotReads, 0, "spoofed object slot reads");

let sharedBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 2);
let sharedView = new Int32Array(sharedBuffer);
sharedView[0] = 0;
sharedView.$TypedArrayViewedArrayBuffer = fakeBuffer;
assertSame(
  sharedView.$TypedArrayViewedArrayBuffer,
  fakeBuffer,
  "viewed buffer spoof installed"
);

function poisonSlot(object, key, value) {
  Object.defineProperty(object, key, {
    configurable: true,
    get() {
      spoofedSlotReads = spoofedSlotReads + 1;
      return value;
    }
  });
}

poisonSlot(sharedView, "$TypedArrayByteOffset", 128);
poisonSlot(sharedView, "$TypedArrayByteLength", 0);
poisonSlot(sharedView, "$TypedArrayBytesPerElement", 8);
poisonSlot(sharedView, "$TypedArrayElementKind", 5);
poisonSlot(sharedView, "$TypedArrayLengthTracking", true);
poisonSlot(sharedBuffer, "$ArrayBufferDataPtr", 0);
poisonSlot(sharedBuffer, "$ArrayBufferByteLength", 0);
poisonSlot(sharedBuffer, "$ArrayBuffer.shared", false);

assertSame(Atomics.add(sharedView, 0, 2), 0, "private add result");
assertSame(Atomics.load(sharedView, 0), 2, "private load result");
assertSame(Atomics.notify(sharedView, 0, 0), 0, "private notify result");
assertSame(Atomics.wait(sharedView, 0, 2, 0), "timed-out", "private wait result");

let asyncResult = Atomics.waitAsync(sharedView, 0, 2, 0);
assertSame(asyncResult.async, false, "private waitAsync async");
assertSame(asyncResult.value, "timed-out", "private waitAsync value");
assertSame(spoofedSlotReads, 0, "genuine view slot reads");

926;
