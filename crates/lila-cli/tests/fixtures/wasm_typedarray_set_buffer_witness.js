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

var detachedReceiver = new Uint8Array(1);
var detachedOffsetCoercions = 0;
__lilaDetachArrayBuffer(detachedReceiver.buffer);
assertErrorPrototype(function() {
  detachedReceiver.set([], {
    valueOf: function() {
      detachedOffsetCoercions++;
      return 0;
    }
  });
}, TypeError.prototype, "detached receiver entry");
assertSame(detachedOffsetCoercions, 0, "entry detach skips offset coercion");

var growBuffer = new ArrayBuffer(1, { maxByteLength: 3 });
var growTarget = new Uint8Array(growBuffer);
growTarget[0] = 7;
growTarget.set([8, 9], {
  valueOf: function() {
    growBuffer.resize(3);
    return 1;
  }
});
assertSame(growTarget.length, 3, "post-offset growth uses refreshed length");
assertSame(growTarget[0], 7, "post-offset growth prefix");
assertSame(growTarget[1], 8, "post-offset growth first write");
assertSame(growTarget[2], 9, "post-offset growth second write");

var shrinkBuffer = new ArrayBuffer(3, { maxByteLength: 3 });
var shrinkTarget = new Uint8Array(shrinkBuffer);
shrinkTarget[0] = 5;
assertErrorPrototype(function() {
  shrinkTarget.set([8, 9], {
    valueOf: function() {
      shrinkBuffer.resize(1);
      return 0;
    }
  });
}, RangeError.prototype, "post-offset shrink uses refreshed length");
assertSame(shrinkTarget[0], 5, "post-offset shrink writes nothing");

var offsetDetachBuffer = new ArrayBuffer(1);
var offsetDetachTarget = new Uint8Array(offsetDetachBuffer);
assertErrorPrototype(function() {
  offsetDetachTarget.set([], {
    valueOf: function() {
      __lilaDetachArrayBuffer(offsetDetachBuffer);
      return 0;
    }
  });
}, TypeError.prototype, "post-offset detachment");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var fixedTarget = new Uint8Array(fixedBuffer, 2, 2);
assertErrorPrototype(function() {
  fixedTarget.set([], {
    valueOf: function() {
      fixedBuffer.resize(1);
      return 0;
    }
  });
}, TypeError.prototype, "post-offset fixed out-of-bounds");

var detachedSource = new Uint8Array(1);
__lilaDetachArrayBuffer(detachedSource.buffer);
assertErrorPrototype(function() {
  new Uint8Array(1).set(detachedSource);
}, TypeError.prototype, "detached TypedArray source");

var outOfBoundsSourceBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var outOfBoundsSource = new Uint8Array(outOfBoundsSourceBuffer, 2, 2);
outOfBoundsSourceBuffer.resize(1);
assertErrorPrototype(function() {
  new Uint8Array(2).set(outOfBoundsSource);
}, TypeError.prototype, "out-of-bounds TypedArray source");

var oddByteSourceBuffer = new ArrayBuffer(3, { maxByteLength: 3 });
var oddByteSource = new Uint16Array(oddByteSourceBuffer);
oddByteSource[0] = 513;
var oddByteTarget = new Uint16Array([0, 17]);
oddByteTarget.set(oddByteSource);
assertSame(oddByteTarget[0], 513, "odd-byte source first element");
assertSame(oddByteTarget[1], 17, "odd-byte source length floor");

var other = __lilaCreateRealm().global;
var otherSet = other.Uint8Array.prototype.set;
assertErrorPrototype(function() {
  otherSet.call(new Uint8Array(0), [], -1);
}, other.RangeError.prototype, "borrowed set negative offset ToIndex");
assertErrorPrototype(function() {
  otherSet.call(new Uint8Array(1), new Uint8Array(2), 0);
}, other.RangeError.prototype, "borrowed set typed source exceeds target");
assertErrorPrototype(function() {
  otherSet.call(new Uint8Array(2), new Uint8Array(2), 1);
}, other.RangeError.prototype, "borrowed set typed source exceeds target suffix");
assertErrorPrototype(function() {
  otherSet.call(new Uint8Array(1), { length: 2 }, 0);
}, other.RangeError.prototype, "borrowed set array-like source exceeds target");
assertErrorPrototype(function() {
  otherSet.call(new Uint8Array(2), { length: 2 }, 1);
}, other.RangeError.prototype, "borrowed set array-like source exceeds target suffix");

true;
