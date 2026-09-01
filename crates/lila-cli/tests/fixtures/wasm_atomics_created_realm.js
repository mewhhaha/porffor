function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertDataDescriptor(object, key, value, label) {
  var descriptor = Object.getOwnPropertyDescriptor(object, key);
  assertSame(descriptor.value, value, label + " value");
  assertSame(descriptor.writable, true, label + " writable");
  assertSame(descriptor.enumerable, false, label + " enumerable");
  assertSame(descriptor.configurable, true, label + " configurable");
}

function assertOtherError(callback, expectedPrototype, label) {
  try {
    callback();
  } catch (error) {
    assertSame(Object.getPrototypeOf(error), expectedPrototype, label);
    return;
  }
  throw label + " did not throw";
}

var other = __lilaCreateRealm().global;
var otherAtomics = other.Atomics;
var methodNames = [
  "add",
  "and",
  "compareExchange",
  "exchange",
  "load",
  "notify",
  "or",
  "pause",
  "store",
  "sub",
  "wait",
  "waitAsync",
  "xor",
  "isLockFree"
];
var methodLengths = [3, 3, 4, 3, 2, 3, 3, 0, 3, 3, 4, 4, 3, 1];

assertSame(otherAtomics === Atomics, false, "created realm Atomics identity");
assertSame(Object.getPrototypeOf(otherAtomics), other.Object.prototype, "Atomics object realm");
assertDataDescriptor(other, "Atomics", otherAtomics, "created realm Atomics global descriptor");

for (var i = 0; i < methodNames.length; i += 1) {
  var name = methodNames[i];
  var method = otherAtomics[name];
  assertSame(method === Atomics[name], false, name + " method realm identity");
  assertSame(Object.getPrototypeOf(method), other.Function.prototype, name + " function realm");
  assertSame(method.name, name, name + " name");
  assertSame(method.length, methodLengths[i], name + " length");
  assertDataDescriptor(otherAtomics, name, method, name + " descriptor");
}

var tagDescriptor = Object.getOwnPropertyDescriptor(otherAtomics, Symbol.toStringTag);
assertSame(tagDescriptor.value, "Atomics", "created realm Atomics toStringTag descriptor");
assertSame(tagDescriptor.writable, false, "tag writable");
assertSame(tagDescriptor.enumerable, false, "tag enumerable");
assertSame(tagDescriptor.configurable, true, "tag configurable");

var view = new Int32Array(new SharedArrayBuffer(4));
var borrowedAdd = otherAtomics.add;
assertSame(borrowedAdd(view, 0, 2), 0, "borrowed add result");
assertSame(view[0], 2, "borrowed add mutation");
assertOtherError(function() {
  borrowedAdd({}, 0, 1);
}, other.TypeError.prototype, "borrowed add TypeError realm");
assertOtherError(function() {
  borrowedAdd(view, 1, 1);
}, other.RangeError.prototype, "borrowed add RangeError realm");

true;
