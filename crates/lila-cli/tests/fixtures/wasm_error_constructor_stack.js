function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

// Constructed errors carry an own `stack` string whose value is the
// `Error.prototype.toString` composition (frames are not captured).
var err = new Error("boom");
assertSame(typeof err.stack, "string", "typeof stack");
assertSame(err.stack, "Error: boom", "stack value");
assertSame(err.hasOwnProperty("stack"), true, "own stack");
assertSame(Object.keys(err).indexOf("stack"), -1, "non-enumerable stack");

var bare = new TypeError();
assertSame(bare.stack, "TypeError", "bare stack folds to the name");
assertSame(bare.stack.trim(), "TypeError", "stack trims");

var range = new RangeError(42);
assertSame(range.stack, "RangeError: 42", "coerced message");

348;
