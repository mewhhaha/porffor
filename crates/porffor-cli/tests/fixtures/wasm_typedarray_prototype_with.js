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

var numeric = new Uint16Array([10, 20, 30]);
Object.defineProperty(numeric, "constructor", {
  get: function() {
    throw "constructor must not be read";
  }
});
var numericResult = numeric.with(-1, 99);
assertSequence(numericResult, [10, 20, 99], "numeric replacement");
assertSequence(numeric, [10, 20, 30], "numeric source unchanged");
assertSame(numericResult === numeric, false, "numeric result is distinct");
assertSame(
  Object.getPrototypeOf(numericResult),
  Uint16Array.prototype,
  "numeric result keeps intrinsic type"
);
Object.defineProperty(numeric, "length", { value: 20 });
assertSequence(numeric.with(-1.8, 7), [10, 20, 7], "negative fractional index");
assertSequence(numeric.with(-0.5, 6), [6, 20, 30], "negative fractional zero index");
assertSequence(numeric.with("1", 8), [10, 8, 30], "string index");
assertSequence(numeric.with(NaN, 9), [9, 20, 30], "NaN index");

var bigint = new BigInt64Array([1n, -2n, 3n]);
assertSequence(bigint.with(-2, 4n), [1n, 4n, 3n], "bigint replacement");
assertThrows(TypeError, function() {
  bigint.with(0, 4);
}, "bigint rejects number");
assertThrows(TypeError, function() {
  numeric.with(0, 4n);
}, "number rejects bigint");

var floating = new Float64Array([NaN, -0, Infinity]);
assertSequence(floating.with(2, -Infinity), [NaN, -0, -Infinity], "floating replacement");

var early = new Uint8Array([1, 2, 3]);
var order = [];
var observed = early.with({
  valueOf: function() {
    order.push("index");
    return 1;
  }
}, {
  valueOf: function() {
    order.push("value");
    early[0] = 5;
    return 6;
  }
});
assertSequence(order, ["index", "value"], "coercion order");
assertSequence(observed, [5, 6, 3], "replacement coerced before copy");
assertSequence(early, [5, 2, 3], "coercion mutation remains on source");

var boundsValueCoercions = 0;
assertThrows(RangeError, function() {
  numeric.with(100, {
    valueOf: function() {
      boundsValueCoercions++;
      return 1;
    }
  });
}, "positive out of bounds");
assertSame(boundsValueCoercions, 1, "value coerced before bounds rejection");
assertThrows(RangeError, function() {
  numeric.with(-4, 1);
}, "negative out of bounds");
assertThrows(RangeError, function() {
  numeric.with(Infinity, 1);
}, "infinite index");

var shared = new Uint8Array(new SharedArrayBuffer(3));
shared.set([7, 8, 9]);
var sharedResult = shared.with(1, 4);
assertSequence(sharedResult, [7, 4, 9], "shared source copy");
assertSequence(shared, [7, 8, 9], "shared source unchanged");

var growBuffer = new ArrayBuffer(2, { maxByteLength: 5 });
var growTracking = new Uint8Array(growBuffer);
growTracking.set([11, 22]);
var grownResult = growTracking.with(4, {
  valueOf: function() {
    growBuffer.resize(5);
    return 123;
  }
});
assertSequence(grownResult, [11, 22], "growth uses captured result length");
assertSame(growTracking.length, 5, "tracking source grew during coercion");

var emptyBuffer = new ArrayBuffer(0, { maxByteLength: 1 });
var emptyTracking = new Uint8Array(emptyBuffer);
var emptyResult = emptyTracking.with(0, {
  valueOf: function() {
    emptyBuffer.resize(1);
    return 1;
  }
});
assertSame(emptyResult.length, 0, "empty result keeps captured length");

var shrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var shrinkTracking = new Uint8Array(shrinkBuffer);
assertThrows(RangeError, function() {
  shrinkTracking.with(-1, {
    valueOf: function() {
      shrinkBuffer.resize(1);
      return 1;
    }
  });
}, "tracking index invalid after shrink");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var fixed = new Uint8Array(fixedBuffer, 1, 2);
assertThrows(RangeError, function() {
  fixed.with(0, {
    valueOf: function() {
      fixedBuffer.resize(2);
      return 1;
    }
  });
}, "fixed view out of bounds after coercion");

var detachedBuffer = new ArrayBuffer(2);
var detached = new Uint8Array(detachedBuffer);
detachedBuffer.transfer();
assertThrows(TypeError, function() {
  detached.with(0, 1);
}, "initially detached receiver");

var coercionDetachBuffer = new ArrayBuffer(2);
var coercionDetached = new Uint8Array(coercionDetachBuffer);
assertThrows(RangeError, function() {
  coercionDetached.with(0, {
    valueOf: function() {
      coercionDetachBuffer.transfer();
      return 1;
    }
  });
}, "receiver detached during coercion");

var withFunction = Uint8Array.prototype.with;
var invalidReceivers = [null, undefined, true, "abc", 12, Symbol(), [], {}, Uint8Array.prototype, 1n];
for (var invalidIndex = 0; invalidIndex < invalidReceivers.length; invalidIndex++) {
  assertThrows(TypeError, function() {
    withFunction.call(invalidReceivers[invalidIndex], 0, 0);
  }, "invalid receiver " + invalidIndex);
}

true;
