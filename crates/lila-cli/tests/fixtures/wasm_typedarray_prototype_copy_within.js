function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertSequence(actual, expected, label) {
  assertSame(actual.length, expected.length, label + " length");
  for (var index = 0; index < expected.length; index++) {
    assertSame(actual[index], expected[index], label + " value " + index);
  }
}

function assertThrowsTypeError(callback, label) {
  try {
    callback();
  } catch (error) {
    assertSame(error.constructor, TypeError, label + " constructor");
    return;
  }
  throw label + " did not throw";
}

var typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
var copyWithin = typedArrayPrototype.copyWithin;
var descriptor = Object.getOwnPropertyDescriptor(
  typedArrayPrototype,
  "copyWithin"
);
assertSame(descriptor.value, copyWithin, "descriptor value");
assertSame(descriptor.writable, true, "descriptor writable");
assertSame(descriptor.enumerable, false, "descriptor enumerable");
assertSame(descriptor.configurable, true, "descriptor configurable");
assertSame(copyWithin.name, "copyWithin", "function name");
assertSame(copyWithin.length, 2, "function length");

var backward = new Uint8Array([1, 2, 3, 4]);
assertSame(backward.copyWithin(1, 0, 3), backward, "backward identity");
assertSequence(backward, [1, 1, 2, 3], "backward overlap");

var forward = new Uint8Array([1, 2, 3, 4]);
forward.copyWithin(0, 1);
assertSequence(forward, [2, 3, 4, 4], "forward overlap");

var offsetBuffer = new ArrayBuffer(8);
var offsetBacking = new Uint16Array(offsetBuffer);
offsetBacking.set([10, 20, 30, 40]);
var offsetView = new Uint16Array(offsetBuffer, 2, 3);
offsetView.copyWithin(2, 0);
assertSequence(offsetBacking, [10, 20, 30, 20], "byte offset");

var bitBuffer = new ArrayBuffer(16);
var bitWords = new Uint32Array(bitBuffer);
bitWords[0] = 2141266757;
bitWords[1] = 4290772993;
new Float32Array(bitBuffer).copyWithin(2, 0, 2);
assertSame(bitWords[2], 2141266757, "first NaN payload");
assertSame(bitWords[3], 4290772993, "second NaN payload");

var bigint = new BigInt64Array([1n, -2n, 3n, -4n]);
bigint.copyWithin(1, 0, 3);
assertSequence(bigint, [1n, 1n, -2n, 3n], "bigint overlap");

var order = [];
var ordered = new Uint8Array([1, 2, 3, 4]);
ordered.copyWithin(
  {
    valueOf: function() {
      order.push("target");
      return 1;
    }
  },
  {
    valueOf: function() {
      order.push("start");
      return 0;
    }
  },
  {
    valueOf: function() {
      order.push("end");
      return 2;
    }
  }
);
assertSame(order.join(","), "target,start,end", "coercion order");
assertSequence(ordered, [1, 1, 2, 4], "coerced indexes");

var ignoresLength = new Uint8Array([5, 6]);
Object.defineProperty(ignoresLength, "length", {
  get: function() {
    throw "length must not be read";
  }
});
ignoresLength.copyWithin(1, 0);
assertSequence(ignoresLength, [5, 5], "internal length");

var trackingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
var tracking = new Uint8Array(trackingBuffer);
tracking.set([0, 1, 2, 3]);
tracking.copyWithin(
  {
    valueOf: function() {
      trackingBuffer.resize(3);
      return 2;
    }
  },
  0
);
assertSequence(tracking, [0, 1, 0], "tracking shrink");

trackingBuffer.resize(4);
tracking.set([0, 1, 2, 3]);
tracking.copyWithin(
  {
    valueOf: function() {
      trackingBuffer.resize(6);
      tracking[4] = 4;
      tracking[5] = 5;
      return 0;
    }
  },
  2
);
assertSequence(tracking, [2, 3, 2, 3, 4, 5], "tracking grow");

trackingBuffer.resize(4);
var fixed = new Uint8Array(trackingBuffer, 0, 4);
assertThrowsTypeError(function() {
  fixed.copyWithin(
    {
      valueOf: function() {
        trackingBuffer.resize(2);
        return 0;
      }
    },
    1
  );
}, "fixed shrink during coercion");

var detachedBuffer = new ArrayBuffer(4);
var detached = new Uint8Array(detachedBuffer);
detachedBuffer.transfer();
assertThrowsTypeError(function() {
  detached.copyWithin(
    {
      valueOf: function() {
        throw "target must not be coerced";
      }
    },
    0
  );
}, "detached receiver");

var detachDuringCoercionBuffer = new ArrayBuffer(4);
var detachDuringCoercion = new Uint8Array(detachDuringCoercionBuffer);
assertThrowsTypeError(function() {
  detachDuringCoercion.copyWithin(0, {
    valueOf: function() {
      detachDuringCoercionBuffer.transfer();
      return 1;
    }
  });
}, "detach during coercion");

var invalidReceivers = [
  null,
  undefined,
  true,
  "abc",
  12,
  Symbol(),
  [1, 2, 3],
  { 0: 1, length: 1 },
  Uint8Array.prototype,
  1n
];
for (var invalidIndex = 0; invalidIndex < invalidReceivers.length; invalidIndex++) {
  assertThrowsTypeError(function() {
    copyWithin.call(invalidReceivers[invalidIndex], 0, 0);
  }, "invalid receiver " + invalidIndex);
}

true;
