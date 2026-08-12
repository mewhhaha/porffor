function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
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

var numeric = new Uint16Array([1, 2, 65535]);
var numericResult = numeric.toReversed();
assertSame(numericResult[0], 65535, "numeric first value");
assertSame(numericResult[1], 2, "numeric second value");
assertSame(numericResult[2], 1, "numeric third value");
assertSame(numeric[0], 1, "numeric source remains unchanged");
assertSame(numericResult === numeric, false, "numeric result is distinct");
assertSame(
  Object.getPrototypeOf(numericResult),
  Uint16Array.prototype,
  "numeric result keeps intrinsic type"
);

var bigint = new BigInt64Array([1n, -2n, 3n]);
var bigintResult = bigint.toReversed();
assertSame(bigintResult[0], 3n, "bigint first value");
assertSame(bigintResult[1], -2n, "bigint second value");
assertSame(bigintResult[2], 1n, "bigint third value");

var floating = new Float64Array([NaN, -0, Infinity]);
var floatingResult = floating.toReversed();
assertSame(floatingResult[0], Infinity, "floating infinity");
assertSame(floatingResult[1], -0, "floating negative zero");
assertSame(floatingResult[2], NaN, "floating NaN");

var ignoresConstructor = new Uint8Array([4, 5]);
Object.defineProperty(ignoresConstructor, "constructor", {
  get: function() {
    throw "constructor must not be read";
  }
});
Object.defineProperty(ignoresConstructor, "length", { value: 20 });
var ignoredResult = ignoresConstructor.toReversed();
assertSame(ignoredResult.length, 2, "internal length is used");
assertSame(ignoredResult[0], 5, "constructor getter is ignored");

var resizableBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
var fixed = new Uint8Array(resizableBuffer, 0, 3);
var tracking = new Uint8Array(resizableBuffer, 1);
fixed.set([1, 2, 3]);
new Uint8Array(resizableBuffer)[3] = 4;
var trackingResult = tracking.toReversed();
assertSame(trackingResult.length, 3, "tracking initial length");
assertSame(trackingResult[0], 4, "tracking initial reverse");
assertSame(trackingResult[2], 2, "tracking initial tail");
resizableBuffer.resize(6);
trackingResult = tracking.toReversed();
assertSame(trackingResult.length, 5, "tracking grown length");
assertSame(trackingResult[0], 0, "tracking grown zero fill");
assertSame(trackingResult[2], 4, "tracking grown existing value");
assertSame(fixed.toReversed().length, 3, "fixed length after grow");
resizableBuffer.resize(2);
assertSame(tracking.toReversed()[0], 2, "tracking value after shrink");
var outOfBoundsThrew = false;
try {
  fixed.toReversed();
} catch (error) {
  outOfBoundsThrew = error instanceof TypeError;
}
assertSame(outOfBoundsThrew, true, "fixed out of bounds after shrink");

var detachedBuffer = new ArrayBuffer(2);
var detached = new Uint8Array(detachedBuffer);
detachedBuffer.transfer();
var detachedThrew = false;
try {
  detached.toReversed();
} catch (error) {
  detachedThrew = error instanceof TypeError;
}
assertSame(detachedThrew, true, "detached receiver");

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
  assertThrows(TypeError, function() {
    Uint8Array.prototype.toReversed.call(invalidReceivers[invalidIndex]);
  }, "invalid receiver " + invalidIndex);
}

true;
