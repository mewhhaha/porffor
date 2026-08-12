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

if (typeof Atomics.wait !== "function") throw "wait function";
if (Atomics.wait.length !== 4) throw "wait length";
if (Atomics.wait.name !== "wait") throw "wait name";

var i32 = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4));
i32[0] = 1;
assertSame(Atomics.wait(i32, 0, 0), "not-equal", "i32 not equal");
i32[0] = 0;
assertSame(Atomics.wait(i32, 0, 0, 0), "timed-out", "i32 zero timeout");
assertSame(Atomics.wait(i32, 0, 0, -1), "timed-out", "i32 negative timeout");
assertSame(Atomics.wait(i32, 0, 0, false), "timed-out", "i32 false timeout");
assertSame(Atomics.wait(i32, 0, 0, null), "timed-out", "i32 null timeout");
assertSame(Atomics.wait(i32, 0, 0, 1), "timed-out", "i32 positive timeout");

var indexCoerced = 0;
var index = {
  valueOf() {
    indexCoerced = indexCoerced + 1;
    return 0;
  }
};
assertSame(Atomics.wait(i32, index, 0, 0), "timed-out", "index object");
assertSame(indexCoerced, 1, "index coerced");

var valueCoerced = 0;
var value = {
  valueOf() {
    valueCoerced = valueCoerced + 1;
    return 0;
  }
};
assertSame(Atomics.wait(i32, 0, value, 0), "timed-out", "value object");
assertSame(valueCoerced, 1, "value coerced");

var timeoutCoerced = 0;
var timeout = {
  valueOf() {
    timeoutCoerced = timeoutCoerced + 1;
    return 0;
  }
};
assertSame(Atomics.wait(i32, 0, 0, timeout), "timed-out", "timeout object");
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
assertSame(Atomics.wait(i32, 0, 0, timeoutString), "timed-out", "timeout toString");
assertSame(Atomics.wait(i32, 0, 0, timeoutPrimitive), "timed-out", "timeout toPrimitive");

var i64 = new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 2));
i64[0] = 1n;
assertSame(Atomics.wait(i64, 0, 0n, 0), "not-equal", "i64 not equal");
i64[0] = 0n;
assertSame(Atomics.wait(i64, 0, 0n, 0), "timed-out", "i64 timed out");
assertSame(Atomics.wait(i64, 0, 0n, 1), "timed-out", "i64 positive timeout");

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
    assertSame(Atomics.wait(view, indexes[i], zero), "not-equal", label + " index " + i);
  }
}

var i32Window = new Int32Array(new SharedArrayBuffer(128), 32, 20);
assertSame(Atomics.wait(i32Window, 0, 0, 0), "timed-out", "i32 window timed out");
assertSame(Atomics.wait(i32Window, 0, 37, 0), "not-equal", "i32 window not equal");
assertGoodIndexes(i32Window, 0, 37, "i32 window");

var poisoned = {
  valueOf() {
    throw "should not coerce";
  }
};

assertTypeError(function () {
  Atomics.wait(new Int32Array(new ArrayBuffer(Int32Array.BYTES_PER_ELEMENT)), poisoned, poisoned, poisoned);
}, "local Int32Array");
assertTypeError(function () {
  Atomics.wait(new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT)), poisoned, poisoned, poisoned);
}, "Int16Array");
assertTypeError(function () {
  Atomics.wait(new BigUint64Array(new SharedArrayBuffer(BigUint64Array.BYTES_PER_ELEMENT)), poisoned, poisoned, poisoned);
}, "BigUint64Array");
assertTypeError(function () {
  Atomics.wait({}, poisoned, poisoned, poisoned);
}, "plain object");
assertTypeError(function () {
  Atomics.wait(i32, Symbol("index"), poisoned, poisoned);
}, "symbol index");
assertTypeError(function () {
  Atomics.wait(i32, 0, Symbol("value"), poisoned);
}, "symbol value");
assertTypeError(function () {
  Atomics.wait(i32, 0, 0, Symbol("timeout"));
}, "symbol timeout");

var timeoutPoisoned = {
  valueOf() {
    throw "timeout poisoned";
  }
};
try {
  Atomics.wait(i32, 0, 0, timeoutPoisoned);
  throw "timeout poison missing";
} catch (error) {
  if (error !== "timeout poisoned") throw "timeout poison wrong error";
}

901;
