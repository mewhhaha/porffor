function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertResultObject(result, expectedAsync, expectedValue, other, label) {
  assertSame(Object.getPrototypeOf(result), other.Object.prototype, label + " prototype");
  assertSame(Object.keys(result).join(","), "async,value", label + " key order");

  var asyncDescriptor = Object.getOwnPropertyDescriptor(result, "async");
  assertSame(asyncDescriptor.value, expectedAsync, label + " async value");
  assertSame(asyncDescriptor.writable, true, label + " async writable");
  assertSame(asyncDescriptor.enumerable, true, label + " async enumerable");
  assertSame(asyncDescriptor.configurable, true, label + " async configurable");

  var valueDescriptor = Object.getOwnPropertyDescriptor(result, "value");
  assertSame(valueDescriptor.value, expectedValue, label + " value");
  assertSame(valueDescriptor.writable, true, label + " value writable");
  assertSame(valueDescriptor.enumerable, true, label + " value enumerable");
  assertSame(valueDescriptor.configurable, true, label + " value configurable");
}

var other = __lilaCreateRealm().global;
var waitAsync = other.Atomics.waitAsync;
var view = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT));

view[0] = 1;
var notEqual = waitAsync(view, 0, 0, Infinity);
assertResultObject(notEqual, false, "not-equal", other, "not-equal result");

view[0] = 0;
var timedOut = waitAsync(view, 0, 0, 0);
assertResultObject(timedOut, false, "timed-out", other, "timeout-zero result");

var pending = waitAsync(view, 0, 0, Infinity);
assertResultObject(pending, true, pending.value, other, "async result");
assertSame(Object.getPrototypeOf(pending.value), other.Promise.prototype, "async Promise prototype");
assertSame(pending.value instanceof other.Promise, true, "async created-Realm Promise");
assertSame(pending.value instanceof Promise, false, "async not entry-Realm Promise");
assertSame(pending.value.then, other.Promise.prototype.then, "async Promise method Realm");

pending.value.then(function(outcome) {
  assertSame(outcome, "ok", "immediate notification outcome");
  print("created-waitAsync:" + outcome);
});
assertSame(other.Atomics.notify(view, 0, 1), 1, "immediate notification count");

true;
