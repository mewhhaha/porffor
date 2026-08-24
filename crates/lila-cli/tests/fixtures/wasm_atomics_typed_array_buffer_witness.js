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

var indexCoercions = 0;
var poisonedIndex = {
  valueOf() {
    indexCoercions = indexCoercions + 1;
    throw "index coerced";
  }
};
var poisonedValue = {
  valueOf() {
    throw "value coerced";
  }
};

var detachedBuffer = new ArrayBuffer(Int32Array.BYTES_PER_ELEMENT);
var detached = new Int32Array(detachedBuffer);
__lilaDetachArrayBuffer(detachedBuffer);

assertError(TypeError, function () {
  Atomics.add(detached, poisonedIndex, poisonedValue);
}, "detached add");
assertError(TypeError, function () {
  Atomics.notify(detached, poisonedIndex, poisonedValue);
}, "detached notify");
assertError(TypeError, function () {
  Atomics.wait(detached, poisonedIndex, poisonedValue, poisonedValue);
}, "detached wait");
assertError(TypeError, function () {
  Atomics.waitAsync(detached, poisonedIndex, poisonedValue, poisonedValue);
}, "detached waitAsync");
assertSame(indexCoercions, 0, "detached index coercions");

var fixedBuffer = new ArrayBuffer(8, { maxByteLength: 8 });
var fixed = new Int32Array(fixedBuffer, 4, 1);
fixedBuffer.resize(0);

assertError(TypeError, function () {
  Atomics.add(fixed, poisonedIndex, poisonedValue);
}, "out-of-bounds add");
assertError(TypeError, function () {
  Atomics.notify(fixed, poisonedIndex, poisonedValue);
}, "out-of-bounds notify");
assertSame(indexCoercions, 0, "out-of-bounds index coercions");

var addGrowBuffer = new ArrayBuffer(0, { maxByteLength: 4 });
var addGrowView = new Int32Array(addGrowBuffer);
var addGrowIndex = {
  valueOf() {
    addGrowBuffer.resize(4);
    return 0;
  }
};
assertError(RangeError, function () {
  Atomics.add(addGrowView, addGrowIndex, poisonedValue);
}, "add snapshots zero length");

var notifyGrowBuffer = new SharedArrayBuffer(0, { maxByteLength: 4 });
var notifyGrowView = new Int32Array(notifyGrowBuffer);
var notifyGrowIndex = {
  valueOf() {
    notifyGrowBuffer.grow(4);
    return 0;
  }
};
assertError(RangeError, function () {
  Atomics.notify(notifyGrowView, notifyGrowIndex, poisonedValue);
}, "notify snapshots zero length");

var waitGrowBuffer = new SharedArrayBuffer(0, { maxByteLength: 4 });
var waitGrowView = new Int32Array(waitGrowBuffer);
var waitGrowIndex = {
  valueOf() {
    waitGrowBuffer.grow(4);
    return 0;
  }
};
assertError(RangeError, function () {
  Atomics.wait(waitGrowView, waitGrowIndex, poisonedValue, poisonedValue);
}, "wait snapshots zero length");

var waitAsyncGrowBuffer = new SharedArrayBuffer(0, { maxByteLength: 4 });
var waitAsyncGrowView = new Int32Array(waitAsyncGrowBuffer);
var waitAsyncGrowIndex = {
  valueOf() {
    waitAsyncGrowBuffer.grow(4);
    return 0;
  }
};
assertError(RangeError, function () {
  Atomics.waitAsync(
    waitAsyncGrowView,
    waitAsyncGrowIndex,
    poisonedValue,
    poisonedValue
  );
}, "waitAsync snapshots zero length");

var oddBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
var odd = new Int32Array(oddBuffer);
oddBuffer.resize(5);
assertError(RangeError, function () {
  Atomics.add(odd, 1, poisonedValue);
}, "add floors partial element");

var oddSharedBuffer = new SharedArrayBuffer(4, { maxByteLength: 8 });
var oddShared = new Int32Array(oddSharedBuffer);
oddSharedBuffer.grow(5);
assertError(RangeError, function () {
  Atomics.notify(oddShared, 1, poisonedValue);
}, "notify floors partial element");
assertError(RangeError, function () {
  Atomics.wait(oddShared, 1, poisonedValue, poisonedValue);
}, "wait floors partial element");
assertError(RangeError, function () {
  Atomics.waitAsync(oddShared, 1, poisonedValue, poisonedValue);
}, "waitAsync floors partial element");

936;
