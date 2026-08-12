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

var odd = new Uint16Array([1, 2, 3, 4, 5]);
var oddResult = odd.reverse();
assertSame(oddResult, odd, "odd result identity");
assertSequence(odd, [5, 4, 3, 2, 1], "odd reverse");

var even = new Int8Array([-1, 2, -3, 4]);
assertSame(even.reverse(), even, "even result identity");
assertSequence(even, [4, -3, 2, -1], "even reverse");

var bigint = new BigInt64Array([1n, -2n, 3n, -4n]);
assertSame(bigint.reverse(), bigint, "bigint result identity");
assertSequence(bigint, [-4n, 3n, -2n, 1n], "bigint reverse");

var floating = new Float64Array([NaN, -0, Infinity]);
floating.reverse();
assertSequence(floating, [Infinity, -0, NaN], "floating reverse");

var ignoresLength = new Uint8Array([6, 7]);
Object.defineProperty(ignoresLength, "length", {
  get: function() {
    throw "length must not be read";
  }
});
ignoresLength.label = "preserved";
ignoresLength.reverse();
assertSame(ignoresLength[0], 7, "internal length first value");
assertSame(ignoresLength[1], 6, "internal length second value");
assertSame(ignoresLength.label, "preserved", "non-index property");

var shared = new Uint8Array(new SharedArrayBuffer(4));
shared.set([8, 9, 10, 11]);
assertSame(shared.reverse(), shared, "shared result identity");
assertSequence(shared, [11, 10, 9, 8], "shared reverse");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
var fixed = new Uint8Array(fixedBuffer, 0, 4);
fixed.set([1, 2, 3, 4]);
fixed.reverse();
assertSequence(fixed, [4, 3, 2, 1], "fixed initial reverse");
fixedBuffer.resize(6);
assertSame(fixed.length, 4, "fixed length after grow");
fixed.reverse();
assertSequence(fixed, [1, 2, 3, 4], "fixed reverse after grow");
fixedBuffer.resize(3);
assertThrowsTypeError(function() {
  fixed.reverse();
}, "fixed out of bounds after shrink");

var trackingBuffer = new ArrayBuffer(5, { maxByteLength: 8 });
var tracking = new Uint8Array(trackingBuffer, 1);
tracking.set([1, 2, 3, 4]);
tracking.reverse();
assertSequence(tracking, [4, 3, 2, 1], "tracking initial reverse");
trackingBuffer.resize(7);
tracking.set([1, 2, 3, 4, 5, 6]);
tracking.reverse();
assertSequence(tracking, [6, 5, 4, 3, 2, 1], "tracking grown reverse");
trackingBuffer.resize(3);
tracking.reverse();
assertSequence(tracking, [5, 6], "tracking shrunk reverse");

var detachedBuffer = new ArrayBuffer(2);
var detached = new Uint8Array(detachedBuffer);
detachedBuffer.transfer();
assertThrowsTypeError(function() {
  detached.reverse();
}, "detached receiver");

var reverse = Uint8Array.prototype.reverse;
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
    reverse.call(invalidReceivers[invalidIndex]);
  }, "invalid receiver " + invalidIndex);
}

true;
