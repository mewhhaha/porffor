function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

function assertError(errorType, fn, label) {
  var threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof errorType)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

function detaching(ta, result) {
  return {
    valueOf() {
      __lilaDetachArrayBuffer(ta.buffer);
      return result;
    }
  };
}

// RevalidateAtomicAccess: a coercion that detaches the backing buffer after
// ValidateAtomicAccess turns every integer operation into a TypeError.
var constructors = [Int8Array, Uint8Array, Int16Array, Uint16Array, Int32Array, Uint32Array];
var checks = 0;
for (var i = 0; i < constructors.length; i++) {
  var TA = constructors[i];
  var ta;
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.load(ta, detaching(ta, 0)); }, TA.name + " load index detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.store(ta, detaching(ta, 0), 1); }, TA.name + " store index detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.store(ta, 0, detaching(ta, 1)); }, TA.name + " store value detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.add(ta, 0, detaching(ta, 1)); }, TA.name + " add value detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.sub(ta, 0, detaching(ta, 1)); }, TA.name + " sub value detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.and(ta, 0, detaching(ta, 1)); }, TA.name + " and value detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.or(ta, 0, detaching(ta, 1)); }, TA.name + " or value detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.xor(ta, 0, detaching(ta, 1)); }, TA.name + " xor value detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.exchange(ta, 0, detaching(ta, 1)); }, TA.name + " exchange value detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.compareExchange(ta, 0, detaching(ta, 0), 1); }, TA.name + " compareExchange expected detach");
  ta = new TA(1);
  assertError(TypeError, function () { Atomics.compareExchange(ta, 0, 0, detaching(ta, 1)); }, TA.name + " compareExchange replacement detach");
  checks = checks + 11;
}

var bigTa = new BigInt64Array(1);
assertError(TypeError, function () { Atomics.add(bigTa, 0, detaching(bigTa, 1n)); }, "BigInt64Array add value detach");
var bigUnsigned = new BigUint64Array(1);
assertError(TypeError, function () { Atomics.store(bigUnsigned, 0, detaching(bigUnsigned, 1n)); }, "BigUint64Array store value detach");

// Both compareExchange values are coerced before the revalidation.
var order = [];
var orderTa = new Int32Array(1);
assertError(TypeError, function () {
  Atomics.compareExchange(orderTa, 0, {
    valueOf() {
      order.push("expected");
      __lilaDetachArrayBuffer(orderTa.buffer);
      return 0;
    }
  }, {
    valueOf() {
      order.push("replacement");
      return 1;
    }
  });
}, "compareExchange coerces both values first");
assertSame(order.join(), "expected,replacement", "compareExchange coercion order");

// Shrinking a length-tracking view below the element is a RangeError, also for
// a partial trailing element; a fixed-length view pushed out of bounds is a
// TypeError.
var shrinkBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
var shrinkView = new Int32Array(shrinkBuffer);
assertError(RangeError, function () {
  Atomics.store(shrinkView, 1, { valueOf() { shrinkBuffer.resize(4); return 7; } });
}, "tracking view shrink");
assertSame(Atomics.load(shrinkView, 0), 0, "rejected store wrote nothing");
var partialBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
var partialView = new Int32Array(partialBuffer);
assertError(RangeError, function () {
  Atomics.add(partialView, 1, { valueOf() { partialBuffer.resize(5); return 7; } });
}, "tracking view partial element");
var fixedBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
var fixedView = new Int32Array(fixedBuffer, 0, 2);
assertError(TypeError, function () {
  Atomics.exchange(fixedView, 0, { valueOf() { fixedBuffer.resize(4); return 7; } });
}, "fixed view out of bounds");

// Growing during the coercion keeps the access valid against the live buffer.
var growBuffer = new ArrayBuffer(4, { maxByteLength: 16 });
var growView = new Int32Array(growBuffer);
assertSame(Atomics.store(growView, 0, { valueOf() { growBuffer.resize(16); return 5; } }), 5, "grow store result");
assertSame(Atomics.add(growView, 3, 2), 0, "grown element");
assertSame(growView[0] + growView[3], 7, "grow writes visible");

// Atomics.notify on a non-shared buffer returns +0 even after its count
// coercion detaches the buffer.
var notifyBuffer = new ArrayBuffer(4);
var notifyView = new Int32Array(notifyBuffer);
assertSame(Atomics.notify(notifyView, 0, {
  valueOf() {
    __lilaDetachArrayBuffer(notifyBuffer);
    return 1;
  }
}), 0, "non-shared notify after count detach");

checks + 1;
