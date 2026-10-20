function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

// The pinned Error Stack Accessor proposal stores captured text internally.
// Error.prototype exposes it through its non-enumerable getter and setter.
var err = new Error("boom");
assertSame(typeof err.stack, "string", "typeof stack");
assertSame(err.stack, "Error: boom", "stack value");
assertSame(err.hasOwnProperty("stack"), false, "constructor leaves stack internal");
var descriptor = Object.getOwnPropertyDescriptor(Error.prototype, "stack");
assertSame(typeof descriptor.get, "function", "prototype stack getter");
assertSame(typeof descriptor.set, "function", "prototype stack setter");
assertSame(descriptor.enumerable, false, "non-enumerable prototype accessor");
assertSame(descriptor.configurable, true, "configurable prototype accessor");
assertSame(Object.keys(err).indexOf("stack"), -1, "non-enumerable stack");

var bare = new TypeError();
assertSame(bare.stack, "TypeError", "bare stack folds to the name");
assertSame(bare.stack.trim(), "TypeError", "stack trims");

var range = new RangeError(42);
assertSame(range.stack, "RangeError: 42", "coerced message");

err.stack = "override";
assertSame(err.stack, "override", "setter installs own value");
var own = Object.getOwnPropertyDescriptor(err, "stack");
assertSame(own.value, "override", "own data value");
assertSame(own.writable, true, "own data is writable");
assertSame(own.enumerable, true, "setter uses ordinary data definition");
assertSame(own.configurable, true, "own data is configurable");
delete err.stack;
assertSame(err.stack, "Error: boom", "deletion reveals original captured text");

348;
