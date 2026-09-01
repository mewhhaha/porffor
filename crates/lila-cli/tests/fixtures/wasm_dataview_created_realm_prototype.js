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

function assertOtherRangeError(callback, label) {
  try {
    callback();
  } catch (error) {
    assertSame(Object.getPrototypeOf(error), other.RangeError.prototype, label);
    return;
  }
  throw label + " did not throw";
}

var other = __lilaCreateRealm().global;
var otherPrototype = other.DataView.prototype;
var getter = otherPrototype.getUint16;
var setter = otherPrototype.setUint32;

assertSame(getter === DataView.prototype.getUint16, false, "getter method realm identity");
assertSame(setter === DataView.prototype.setUint32, false, "setter method realm identity");
assertSame(Object.getPrototypeOf(getter), other.Function.prototype, "getter function realm");
assertSame(Object.getPrototypeOf(setter), other.Function.prototype, "setter function realm");
assertDataDescriptor(otherPrototype, "getUint16", getter, "getter descriptor");
assertDataDescriptor(otherPrototype, "setUint32", setter, "setter descriptor");
assertSame(getter.name, "getUint16", "getter name");
assertSame(getter.length, 1, "getter length");
assertSame(setter.name, "setUint32", "setter name");
assertSame(setter.length, 2, "setter length");

var bufferDescriptor = Object.getOwnPropertyDescriptor(otherPrototype, "buffer");
assertSame(typeof bufferDescriptor.get, "function", "buffer getter");
assertSame(bufferDescriptor.set, undefined, "buffer setter");
assertSame(bufferDescriptor.enumerable, false, "buffer enumerable");
assertSame(bufferDescriptor.configurable, true, "buffer configurable");

var tagDescriptor = Object.getOwnPropertyDescriptor(otherPrototype, Symbol.toStringTag);
assertSame(tagDescriptor.value, "DataView", "created realm toStringTag descriptor");
assertSame(tagDescriptor.writable, false, "tag writable");
assertSame(tagDescriptor.enumerable, false, "tag enumerable");
assertSame(tagDescriptor.configurable, true, "tag configurable");

var view = new DataView(new ArrayBuffer(4));
setter.call(view, 0, 0x01020304);
assertSame(getter.call(view, 0), 0x0102, "borrowed method result");
assertOtherRangeError(function() {
  getter.call(view, 3);
}, "borrowed getter positive bound");
assertOtherRangeError(function() {
  setter.call(view, 1, 1);
}, "borrowed setter positive bound");

true;
