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

function assertThrowsSame(callback, expected, label) {
  try {
    callback();
  } catch (error) {
    assertSame(error, expected, label);
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
assertSame(detachedOffsetCoercions, 1, "detached target coerces offset first");

var offsetError = new Error("offset");
assertThrowsSame(function() {
  detachedReceiver.set(null, {
    valueOf: function() { throw offsetError; }
  });
}, offsetError, "offset error precedes detached target and null source");
assertErrorPrototype(function() {
  detachedReceiver.set([], -1);
}, RangeError.prototype, "negative offset precedes detached target");

var invalidReceiverOffsetCoercions = 0;
assertErrorPrototype(function() {
  Uint8Array.prototype.set.call({}, [], {
    valueOf: function() {
      invalidReceiverOffsetCoercions++;
      return 0;
    }
  });
}, TypeError.prototype, "invalid receiver precedes offset coercion");
assertSame(invalidReceiverOffsetCoercions, 0, "invalid receiver skips offset coercion");

var immutableTarget = new Uint8Array(new ArrayBuffer(1).transferToImmutable());
var immutableOffsetCoercions = 0;
assertErrorPrototype(function() {
  immutableTarget.set([], {
    valueOf: function() {
      immutableOffsetCoercions++;
      return 0;
    }
  });
}, TypeError.prototype, "immutable target precedes offset coercion");
assertSame(immutableOffsetCoercions, 0, "immutable target skips offset coercion");

var infinitySourceReads = 0;
assertErrorPrototype(function() {
  new Uint8Array(0).set({
    get length() {
      infinitySourceReads++;
      return 0;
    }
  }, Infinity);
}, RangeError.prototype, "infinite offset after array-like source length");
assertSame(infinitySourceReads, 1, "infinite offset reads source length first");
var lengthError = new Error("length");
assertThrowsSame(function() {
  new Uint8Array(0).set({
    get length() { throw lengthError; }
  }, Infinity);
}, lengthError, "source length error precedes infinite offset range error");
assertErrorPrototype(function() {
  detachedReceiver.set({
    get length() { throw lengthError; }
  }, Infinity);
}, TypeError.prototype, "detached target precedes array-like source length");

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
assertThrowsSame(function() {
  new Uint8Array(1).set(detachedSource, {
    valueOf: function() { throw offsetError; }
  });
}, offsetError, "offset error precedes detached source");
assertErrorPrototype(function() {
  new Uint8Array(1).set(detachedSource);
}, TypeError.prototype, "detached TypedArray source");
assertErrorPrototype(function() {
  new Uint8Array(1).set(detachedSource, Infinity);
}, TypeError.prototype, "detached TypedArray source precedes infinite offset range error");

var sourceDetachedByOffset = new Uint8Array(1);
assertErrorPrototype(function() {
  new Uint8Array(1).set(sourceDetachedByOffset, {
    valueOf: function() {
      __lilaDetachArrayBuffer(sourceDetachedByOffset.buffer);
      return 0;
    }
  });
}, TypeError.prototype, "source detached during offset coercion");

var targetDetachedBySourceLength = new Uint8Array(1);
var elementAfterDetachRead = false;
targetDetachedBySourceLength.set({
  get length() {
    __lilaDetachArrayBuffer(targetDetachedBySourceLength.buffer);
    return 1;
  },
  get 0() {
    elementAfterDetachRead = true;
    return 3;
  }
});
assertSame(elementAfterDetachRead, true, "array-like element read after target detach");

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
