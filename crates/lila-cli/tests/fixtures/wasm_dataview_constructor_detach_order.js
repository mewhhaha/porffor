function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

function assertError(errorType, fn, label) {
  var threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof errorType)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

// DataView ( buffer, byteOffset, byteLength ): IsDetachedBuffer is checked
// right after ToIndex(byteOffset), before the offset is bounded by the
// buffer's byte length.
var offsetBuffer = new ArrayBuffer(0x1000);
assertError(TypeError, function () {
  new DataView(offsetBuffer, {
    valueOf() {
      __lilaDetachArrayBuffer(offsetBuffer);
      return 0x800;
    }
  });
}, "byteOffset coercion detaches");

var boundBuffer = new ArrayBuffer(8);
var lengthCoercions = 0;
assertError(RangeError, function () {
  new DataView(boundBuffer, 9, {
    valueOf() {
      lengthCoercions = lengthCoercions + 1;
      return 0;
    }
  });
}, "live offset bound");
assertSame(lengthCoercions, 0, "offset bound precedes byteLength coercion");

// A byteLength coercion that detaches is caught after the prototype lookup;
// the pre-prototype bound still uses the length read before the coercion.
var lengthBuffer = new ArrayBuffer(0x1000);
assertError(TypeError, function () {
  new DataView(lengthBuffer, 0x800, {
    valueOf() {
      __lilaDetachArrayBuffer(lengthBuffer);
      return 0x800;
    }
  });
}, "byteLength coercion detaches");
var staleBuffer = new ArrayBuffer(16);
assertError(RangeError, function () {
  new DataView(staleBuffer, 8, {
    valueOf() {
      __lilaDetachArrayBuffer(staleBuffer);
      return 9;
    }
  });
}, "stale byteLength bound");

// ToIndex rejects integers above 2^53 - 1 instead of truncating them.
var hugeBuffer = new ArrayBuffer(8);
assertError(RangeError, function () { new DataView(hugeBuffer, 1e300); }, "huge byteOffset");
assertError(RangeError, function () { new DataView(hugeBuffer, 0, 1e300); }, "huge byteLength");
assertSame(new DataView(hugeBuffer, -0.5).byteOffset, 0, "negative fraction offset");

// Resizing inside the NewTarget prototype lookup is re-observed.
var shrinkBuffer = new ArrayBuffer(8, { maxByteLength: 8 });
var shrinkTarget = function () {}.bind(null);
Object.defineProperty(shrinkTarget, "prototype", {
  get() {
    shrinkBuffer.resize(2);
    return DataView.prototype;
  }
});
assertError(RangeError, function () {
  Reflect.construct(DataView, [shrinkBuffer, 4], shrinkTarget);
}, "prototype lookup shrinks below offset");

var trackBuffer = new ArrayBuffer(8, { maxByteLength: 8 });
var trackTarget = function () {}.bind(null);
Object.defineProperty(trackTarget, "prototype", {
  get() {
    trackBuffer.resize(6);
    return DataView.prototype;
  }
});
var tracking = Reflect.construct(DataView, [trackBuffer, 4], trackTarget);
assertSame(tracking.byteLength, 2, "length-tracking view after shrink");
trackBuffer.resize(8);
assertSame(tracking.byteLength, 4, "length-tracking view after regrowth");

31;
