function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertSequence(actual, expected, label) {
  assertSame(actual.length, expected.length, label + " length");
  for (var index = 0; index < expected.length; index++) {
    assertSame(actual[index], expected[index], label + " value " + index);
  }
}

function assertThrows(expectedConstructor, callback, label) {
  try {
    callback();
  } catch (error) {
    assertSame(error.constructor, expectedConstructor, label + " constructor");
    return;
  }
  throw label + " did not throw";
}

var numeric = new Uint16Array([111, 3, 22, 2, 11, 1]);
Object.defineProperty(numeric, "constructor", {
  get: function() {
    throw "constructor must not be read";
  }
});
Object.defineProperty(numeric, "length", { value: 50 });
var numericResult = numeric.toSorted();
assertSequence(numericResult, [1, 2, 3, 11, 22, 111], "default numeric order");
assertSequence(numeric, [111, 3, 22, 2, 11, 1], "numeric source unchanged");
assertSame(numericResult === numeric, false, "numeric result is distinct");
assertSame(Object.getPrototypeOf(numericResult), Uint16Array.prototype, "same intrinsic type");

assertSequence(
  new Int16Array([1, 2, 3, 4]).toSorted(function(a, b) { return b - a; }),
  [4, 3, 2, 1],
  "custom comparator"
);

var stable = new Uint8Array([21, 12, 11, 22]);
assertSequence(
  stable.toSorted(function(a, b) { return a % 10 - b % 10; }),
  [21, 11, 12, 22],
  "stable equal groups"
);

var coercionCount = 0;
var coercionSorted = new Uint8Array([3, 1, 2]).toSorted(function(a, b) {
  return {
    valueOf: function() {
      coercionCount++;
      return a - b;
    }
  };
});
assertSequence(coercionSorted, [1, 2, 3], "comparator result ToNumber");
assertSame(coercionCount > 0, true, "comparator result was coerced");

var floating = new Float64Array([NaN, 0, -0, 3, NaN, -2]).toSorted();
assertSame(floating[0], -2, "floating negative value");
assertSame(floating[1], -0, "negative zero sorts first");
assertSame(floating[2], 0, "positive zero sorts second");
assertSame(floating[3], 3, "floating positive value");
assertSame(floating[4], NaN, "first NaN sorts last");
assertSame(floating[5], NaN, "second NaN sorts last");

assertSequence(new BigInt64Array([3n, -2n, 1n]).toSorted(), [-2n, 1n, 3n], "signed bigint order");
assertSequence(
  new BigUint64Array([18446744073709551615n, 0n, 9223372036854775808n]).toSorted(),
  [0n, 9223372036854775808n, 18446744073709551615n],
  "unsigned bigint order"
);

var shared = new Uint8Array(new SharedArrayBuffer(3));
shared.set([8, 3, 5]);
assertSequence(shared.toSorted(), [3, 5, 8], "shared source");
assertSequence(shared, [8, 3, 5], "shared source unchanged");

var resizableBuffer = new ArrayBuffer(3, { maxByteLength: 6 });
var tracking = new Uint8Array(resizableBuffer);
tracking.set([3, 1, 2]);
var resized = false;
var capturedCopy = tracking.toSorted(function(a, b) {
  if (!resized) {
    resized = true;
    resizableBuffer.resize(1);
  }
  return a - b;
});
assertSequence(capturedCopy, [1, 2, 3], "source copied before comparator");
assertSame(tracking.length, 1, "comparator resize is observable on source");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
var fixed = new Uint8Array(fixedBuffer, 0, 4);
fixed.set([4, 2, 3, 1]);
assertSequence(fixed.toSorted(), [1, 2, 3, 4], "fixed resizable view");
fixedBuffer.resize(2);
assertThrows(TypeError, function() {
  fixed.toSorted();
}, "out of bounds fixed view");

var detachedBuffer = new ArrayBuffer(2);
var detached = new Uint8Array(detachedBuffer);
detachedBuffer.transfer();
assertThrows(TypeError, function() {
  detached.toSorted();
}, "detached receiver");

var calls = 0;
function Abrupt() {}
assertThrows(Abrupt, function() {
  new Uint8Array([3, 1, 2]).toSorted(function() {
    calls++;
    throw new Abrupt();
  });
}, "abrupt comparator");
assertSame(calls, 1, "no comparison after abrupt completion");

var invalidComparators = [null, true, 1, "compare", {}, 1n];
for (var comparatorIndex = 0; comparatorIndex < invalidComparators.length; comparatorIndex++) {
  assertThrows(TypeError, function() {
    numeric.toSorted(invalidComparators[comparatorIndex]);
  }, "invalid comparator " + comparatorIndex);
}

var toSorted = Uint8Array.prototype.toSorted;
var invalidReceivers = [null, undefined, true, "abc", 12, Symbol(), [], {}, Uint8Array.prototype, 1n];
for (var invalidIndex = 0; invalidIndex < invalidReceivers.length; invalidIndex++) {
  assertThrows(TypeError, function() {
    toSorted.call(invalidReceivers[invalidIndex]);
  }, "invalid receiver " + invalidIndex);
}

true;
