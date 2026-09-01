function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertErrorPrototype(callback, expectedPrototype, label) {
  try {
    callback();
  } catch (error) {
    assertSame(Object.getPrototypeOf(error), expectedPrototype, label + " prototype");
    return;
  }
  throw label + " did not throw";
}

var withFunction = Uint8Array.prototype.with;

var detached = new Uint8Array(1);
var coercions = 0;
__lilaDetachArrayBuffer(detached.buffer);
assertErrorPrototype(function() {
  withFunction.call(detached, {
    valueOf: function() {
      coercions++;
      return 0;
    }
  }, {
    valueOf: function() {
      coercions++;
      return 1;
    }
  });
}, TypeError.prototype, "detached receiver error realm");
assertSame(coercions, 0, "detached receiver skips coercion");

var outOfBoundsBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var outOfBounds = new Uint8Array(outOfBoundsBuffer, 2, 2);
outOfBoundsBuffer.resize(1);
assertErrorPrototype(function() {
  withFunction.call(outOfBounds, 0, 1);
}, TypeError.prototype, "out-of-bounds receiver error realm");

var oddByteBuffer = new ArrayBuffer(3, { maxByteLength: 3 });
var oddByteTracking = new Uint16Array(oddByteBuffer);
oddByteTracking[0] = 11;
var oddByteResult = oddByteTracking.with(0, 22);
assertSame(oddByteResult.length, 1, "odd-byte tracking length floor");
assertSame(oddByteResult[0], 22, "odd-byte tracking replacement");

var other = __lilaCreateRealm().global;
var otherWith = other.Uint8Array.prototype.with;
assertErrorPrototype(function() {
  otherWith.call(new Uint8Array(1), 1, 0);
}, other.RangeError.prototype, "borrowed with out-of-range error realm");

true;
