function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

function assertTypeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof TypeError)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

function assertResult(actual, asyncValue, value, label) {
  if (actual === null || typeof actual !== "object") throw label + " result object";
  assertSame(actual.async, asyncValue, label + " async");
  assertSame(actual.value, value, label + " value");
}

if (typeof Atomics.waitAsync !== "function") throw "waitAsync function";
if (Atomics.waitAsync.length !== 4) throw "waitAsync length";
if (Atomics.waitAsync.name !== "waitAsync") throw "waitAsync name";

var desc = Object.getOwnPropertyDescriptor(Atomics, "waitAsync");
if (desc === undefined) throw "waitAsync descriptor missing";
if (desc.value !== Atomics.waitAsync) throw "waitAsync descriptor value";
if (desc.writable !== true) throw "waitAsync writable";
if (desc.enumerable !== false) throw "waitAsync enumerable";
if (desc.configurable !== true) throw "waitAsync configurable";

var i32 = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4));
i32[0] = 1;
assertResult(Atomics.waitAsync(i32, 0, 0), false, "not-equal", "i32 not equal");
i32[0] = 0;
assertResult(Atomics.waitAsync(i32, 0, 0, 0), false, "timed-out", "i32 zero timeout");
assertResult(Atomics.waitAsync(i32, 0, 0, -1), false, "timed-out", "i32 negative timeout");
assertResult(Atomics.waitAsync(i32, 0, 0, false), false, "timed-out", "i32 false timeout");
assertResult(Atomics.waitAsync(i32, 0, 0, null), false, "timed-out", "i32 null timeout");

var indexCoerced = 0;
var index = {
  valueOf() {
    indexCoerced = indexCoerced + 1;
    return 0;
  }
};
assertResult(Atomics.waitAsync(i32, index, 0, 0), false, "timed-out", "index object");
assertSame(indexCoerced, 1, "index coerced");

var valueCoerced = 0;
var value = {
  valueOf() {
    valueCoerced = valueCoerced + 1;
    return 0;
  }
};
assertResult(Atomics.waitAsync(i32, 0, value, 0), false, "timed-out", "value object");
assertSame(valueCoerced, 1, "value coerced");

var timeoutCoerced = 0;
var timeout = {
  valueOf() {
    timeoutCoerced = timeoutCoerced + 1;
    return 0;
  }
};
assertResult(Atomics.waitAsync(i32, 0, 0, timeout), false, "timed-out", "timeout object");
assertSame(timeoutCoerced, 1, "timeout coerced");

var timeoutString = {
  toString() {
    return "0";
  }
};
var timeoutPrimitive = {
  [Symbol.toPrimitive]() {
    return 0;
  }
};
assertResult(Atomics.waitAsync(i32, 0, 0, timeoutString), false, "timed-out", "timeout toString");
assertResult(Atomics.waitAsync(i32, 0, 0, timeoutPrimitive), false, "timed-out", "timeout toPrimitive");

var i64 = new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 2));
i64[0] = 1n;
assertResult(Atomics.waitAsync(i64, 0, 0n, 0), false, "not-equal", "i64 not equal");
i64[0] = 0n;
assertResult(Atomics.waitAsync(i64, 0, 0n, 0), false, "timed-out", "i64 timed out");
assertResult(Atomics.waitAsync(i64, 0, 0n, timeout), false, "timed-out", "i64 timeout object");
assertResult(Atomics.waitAsync(i64, 0, 0n, timeoutString), false, "timed-out", "i64 timeout toString");
assertResult(Atomics.waitAsync(i64, 0, 0n, timeoutPrimitive), false, "timed-out", "i64 timeout toPrimitive");

function clearView(view, zero) {
  for (var i = 0; i < view.length; i++) {
    view[i] = zero;
  }
}

function assertGoodIndexes(view, zero, stored, label) {
  var indexes = [
    0 / -1,
    "-0",
    view.length - 1,
    {
      valueOf() {
        return 0;
      }
    },
    {
      valueOf: false,
      toString() {
        return "0";
      }
    }
  ];

  for (var i = 0; i < indexes.length; i++) {
    clearView(view, zero);
    Atomics.store(view, indexes[i], stored);
    assertResult(Atomics.waitAsync(view, indexes[i], zero), false, "not-equal", label + " index " + i);
  }
}

var i32Window = new Int32Array(new SharedArrayBuffer(128), 32, 20);
assertResult(Atomics.waitAsync(i32Window, 0, 0, 0), false, "timed-out", "i32 window timed out");
assertResult(Atomics.waitAsync(i32Window, 0, 37, 0), false, "not-equal", "i32 window not equal");
assertGoodIndexes(i32Window, 0, 37, "i32 window");

var i64Window = new BigInt64Array(new SharedArrayBuffer(256), 32, 20);
assertResult(Atomics.waitAsync(i64Window, 0, 0n, 0), false, "timed-out", "i64 window timed out");
assertResult(Atomics.waitAsync(i64Window, 0, 37n, 0), false, "not-equal", "i64 window not equal");
assertGoodIndexes(i64Window, 0n, 37n, "i64 window");

var poisoned = {
  valueOf() {
    throw "should not coerce";
  }
};

assertTypeError(function () {
  Atomics.waitAsync(new Int32Array(new ArrayBuffer(Int32Array.BYTES_PER_ELEMENT)), poisoned, poisoned, poisoned);
}, "local Int32Array");
assertTypeError(function () {
  Atomics.waitAsync(new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT)), poisoned, poisoned, poisoned);
}, "Int16Array");
assertTypeError(function () {
  Atomics.waitAsync(new BigUint64Array(new SharedArrayBuffer(BigUint64Array.BYTES_PER_ELEMENT)), poisoned, poisoned, poisoned);
}, "BigUint64Array");
assertTypeError(function () {
  Atomics.waitAsync({}, poisoned, poisoned, poisoned);
}, "plain object");
var positiveTimeoutResult = Atomics.waitAsync(i32, 0, 0, 1);
assertSame(positiveTimeoutResult.async, true, "positive timeout async");
assertSame(positiveTimeoutResult.value instanceof Promise, true, "positive timeout promise");
Atomics.add(i32, 0, 1);
assertSame(Atomics.notify(i32, 0, 1), 1, "positive timeout notify count");
positiveTimeoutResult.value.then(function (outcome) {
  assertSame(outcome, "ok", "positive timeout outcome");
});

Atomics.store(i32, 0, 0);
var firstWaiter = Atomics.waitAsync(i32, 0, 0);
var secondWaiter = Atomics.waitAsync(i32, 0, 0);
assertSame(Atomics.notify(i32, 0, 0), 0, "zero notify count");
Atomics.store(i32, 0, 1);
assertSame(Atomics.notify(i32, 0), 2, "default notify count");
var waiterOrder = 0;
firstWaiter.value.then(function (outcome) {
  assertSame(outcome, "ok", "first waiter outcome");
  assertSame(waiterOrder, 0, "first waiter order");
  waiterOrder = 1;
});
secondWaiter.value.then(function (outcome) {
  assertSame(outcome, "ok", "second waiter outcome");
  assertSame(waiterOrder, 1, "second waiter order");
});

assertTypeError(function () {
  Atomics.waitAsync(i32, Symbol("index"), poisoned, poisoned);
}, "symbol index");
assertTypeError(function () {
  Atomics.waitAsync(i32, 0, Symbol("value"), poisoned);
}, "symbol value");
assertTypeError(function () {
  Atomics.waitAsync(i32, 0, 0, Symbol("timeout"));
}, "symbol timeout");
assertTypeError(function () {
  Atomics.waitAsync(i64, Symbol("index"), poisoned, poisoned);
}, "i64 symbol index");
assertTypeError(function () {
  Atomics.waitAsync(i64, 0, Symbol("value"), poisoned);
}, "i64 symbol value");
assertTypeError(function () {
  Atomics.waitAsync(i64, 0, 0n, Symbol("timeout"));
}, "i64 symbol timeout");

var timeoutPoisoned = {
  valueOf() {
    throw "timeout poisoned";
  }
};
try {
  Atomics.waitAsync(i32, 0, 0, timeoutPoisoned);
  throw "timeout poison missing";
} catch (error) {
  if (error !== "timeout poisoned") throw "timeout poison wrong error";
}
try {
  Atomics.waitAsync(i64, 0, 0n, timeoutPoisoned);
  throw "i64 timeout poison missing";
} catch (error) {
  if (error !== "timeout poisoned") throw "i64 timeout poison wrong error";
}

913;
