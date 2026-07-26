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
Object.defineProperty(numeric, "length", { value: 50 });
assertSame(numeric.sort(), numeric, "returns receiver");
assertSequence(numeric, [1, 2, 3, 11, 22, 111], "default numeric order");

var descending = new Int16Array([1, 2, 3, 4]);
descending.sort(function(a, b) { return b - a; });
assertSequence(descending, [4, 3, 2, 1], "custom comparator");

var stable = new Uint8Array([21, 12, 11, 22]);
stable.sort(function(a, b) { return a % 10 - b % 10; });
assertSequence(stable, [21, 11, 12, 22], "stable equal groups");

var coercionCount = 0;
var coercionSorted = new Uint8Array([3, 1, 2]);
coercionSorted.sort(function(a, b) {
  return {
    valueOf: function() {
      coercionCount++;
      return a - b;
    }
  };
});
assertSequence(coercionSorted, [1, 2, 3], "comparator result ToNumber");
assertSame(coercionCount > 0, true, "comparator result was coerced");

var floating = new Float64Array([NaN, 0, -0, 3, NaN, -2]);
floating.sort();
assertSame(floating[0], -2, "floating negative value");
assertSame(floating[1], -0, "negative zero sorts first");
assertSame(floating[2], 0, "positive zero sorts second");
assertSame(floating[3], 3, "floating positive value");
assertSame(floating[4], NaN, "first NaN sorts last");
assertSame(floating[5], NaN, "second NaN sorts last");

var signedBigInt = new BigInt64Array([3n, -2n, 1n]);
signedBigInt.sort();
assertSequence(signedBigInt, [-2n, 1n, 3n], "signed bigint order");
var unsignedBigInt = new BigUint64Array([18446744073709551615n, 0n, 9223372036854775808n]);
unsignedBigInt.sort();
assertSequence(
  unsignedBigInt,
  [0n, 9223372036854775808n, 18446744073709551615n],
  "unsigned bigint order"
);

var shared = new Uint8Array(new SharedArrayBuffer(3));
shared.set([8, 3, 5]);
assertSame(shared.sort(), shared, "shared receiver identity");
assertSequence(shared, [3, 5, 8], "shared source sorted");

var growBuffer = new ArrayBuffer(4, { maxByteLength: 6 });
var growTracking = new Uint8Array(growBuffer);
growTracking.set([4, 2, 3, 1]);
var grew = false;
growTracking.sort(function(a, b) {
  if (!grew) {
    grew = true;
    growBuffer.resize(6);
  }
  return a - b;
});
assertSequence(growTracking, [1, 2, 3, 4, 0, 0], "growth uses captured length");

var shrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var shrinkTracking = new Uint8Array(shrinkBuffer);
shrinkTracking.set([4, 3, 2, 1]);
var shrank = false;
shrinkTracking.sort(function(a, b) {
  if (!shrank) {
    shrank = true;
    shrinkBuffer.resize(2);
  }
  return a - b;
});
assertSame(shrinkTracking.length, 2, "tracking view shrank during comparator");
assertSame([4, 3, 2, 1].includes(shrinkTracking[0]), true, "tracking first original value");
assertSame([4, 3, 2, 1].includes(shrinkTracking[1]), true, "tracking second original value");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var fixed = new Uint8Array(fixedBuffer, 0, 4);
fixed.set([4, 3, 2, 1]);
var fixedShrank = false;
fixed.sort(function(a, b) {
  if (!fixedShrank) {
    fixedShrank = true;
    fixedBuffer.resize(2);
  }
  return a - b;
});
assertSequence(new Uint8Array(fixedBuffer), [4, 3], "out of bounds fixed view is not rewritten");

var detachedBuffer = new ArrayBuffer(4);
var detached = new Uint8Array(detachedBuffer);
var detachedCoercion = false;
detached.sort(function() {
  detachedBuffer.transfer();
  return {
    valueOf: function() {
      detachedCoercion = true;
      return 0;
    }
  };
});
assertSame(detachedCoercion, true, "ToNumber runs after comparator detaches receiver");
assertSame(detached.length, 0, "detached receiver remains detached");

var initiallyDetachedBuffer = new ArrayBuffer(2);
var initiallyDetached = new Uint8Array(initiallyDetachedBuffer);
initiallyDetachedBuffer.transfer();
assertThrows(TypeError, function() {
  initiallyDetached.sort();
}, "initially detached receiver");

var abrupt = new Uint8Array([3, 1, 2]);
var calls = 0;
function Abrupt() {}
assertThrows(Abrupt, function() {
  abrupt.sort(function() {
    calls++;
    throw new Abrupt();
  });
}, "abrupt comparator");
assertSame(calls, 1, "no comparison after abrupt completion");
assertSequence(abrupt, [3, 1, 2], "abrupt sort does not commit partial order");

var invalidComparators = [null, true, 1, "compare", {}, 1n];
for (var comparatorIndex = 0; comparatorIndex < invalidComparators.length; comparatorIndex++) {
  assertThrows(TypeError, function() {
    numeric.sort(invalidComparators[comparatorIndex]);
  }, "invalid comparator " + comparatorIndex);
}

var sort = Uint8Array.prototype.sort;
var invalidReceivers = [null, undefined, true, "abc", 12, Symbol(), [], {}, Uint8Array.prototype, 1n];
for (var invalidIndex = 0; invalidIndex < invalidReceivers.length; invalidIndex++) {
  assertThrows(TypeError, function() {
    sort.call(invalidReceivers[invalidIndex]);
  }, "invalid receiver " + invalidIndex);
}

true;
