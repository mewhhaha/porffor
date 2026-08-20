function assertSame(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": expected " + expected + ", got " + actual;
  }
}

var i32 = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT));
var i64 = new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT));

var notified = Atomics.waitAsync(i32, 0, 0, 100);
assertSame(notified.async, true, "notified waiter async");
assertSame(Atomics.notify(i32, 0, 1), 1, "notification before deadline");
notified.value.then(function (outcome) {
  assertSame(outcome, "ok", "notification before deadline outcome");
  print("notified:" + outcome);
});

var microtaskView = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT));
var microtaskNotification = Atomics.waitAsync(microtaskView, 0, 0, 100);
Promise.resolve().then(function () {
  assertSame(Atomics.notify(microtaskView, 0, 1), 1, "microtask notification before deadline");
});
microtaskNotification.value.then(function (outcome) {
  assertSame(outcome, "ok", "microtask notification outcome");
  print("microtask:" + outcome);
});

var expired = Atomics.waitAsync(i32, 0, 0, 1);
var delay = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT));
assertSame(Atomics.wait(delay, 0, 0, 5), "timed-out", "deadline delay");
assertSame(Atomics.notify(i32, 0, 1), 0, "notification after deadline");
expired.value.then(function (outcome) {
  assertSame(outcome, "timed-out", "notification after deadline outcome");
  print("expired:" + outcome);
});

var i32Timeout = Atomics.waitAsync(i32, 0, 0, 2);
i32Timeout.value.then(function (outcome) {
  assertSame(outcome, "timed-out", "Int32Array finite timeout");
  print("i32:" + outcome);
});

var i64Timeout = Atomics.waitAsync(i64, 0, 0n, 2);
i64Timeout.value.then(function (outcome) {
  assertSame(outcome, "timed-out", "BigInt64Array finite timeout");
  print("i64:" + outcome);
});
