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

function assertRangeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof RangeError)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

function assertThrowsValue(fn, expected, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (error !== expected) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

if (typeof Atomics.notify !== "function") throw "notify function";
if (Atomics.notify.length !== 3) throw "notify length";
if (Atomics.notify.name !== "notify") throw "notify name";

var shared = new Int32Array(new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4));
var local = new Int32Array(new ArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4));

assertSame(Atomics.notify(shared, 0), 0, "shared missing count");
assertSame(Atomics.notify(shared, 0, undefined), 0, "shared undefined count");
assertSame(Atomics.notify(shared, 0, 1), 0, "shared count");
assertSame(Atomics.notify(shared, 0, -3), 0, "shared negative count");
assertSame(Atomics.notify(shared, 0, Number.POSITIVE_INFINITY), 0, "shared infinite count");
assertSame(Atomics.notify(shared, 0, "33"), 0, "shared string count");
assertSame(Atomics.notify(shared, 0, { valueOf: 8 }), 0, "shared object count");
assertSame(Atomics.notify(shared, 0, NaN), 0, "shared nan count");
assertSame(Atomics.notify(shared, 0, 0 / 0), 0, "shared arithmetic nan count");
assertSame(Atomics.notify(local, 0, 1), 0, "local count");

var indexCoerced = 0;
var index = {
  valueOf() {
    indexCoerced = indexCoerced + 1;
    return 0;
  }
};
assertSame(Atomics.notify(local, index, 0), 0, "index object");
assertSame(indexCoerced, 1, "index coerced");

var countCoerced = 0;
var count = {
  valueOf() {
    countCoerced = countCoerced + 1;
    return 0;
  }
};
assertSame(Atomics.notify(local, 0, count), 0, "count object");
assertSame(countCoerced, 1, "count coerced");

var indexPoisoned = {
  valueOf() {
    throw "index poisoned";
  }
};
var indexPrimitivePoisoned = {
  [Symbol.toPrimitive]() {
    throw "index primitive poisoned";
  }
};
var countPoisoned = {
  valueOf() {
    throw "count poisoned";
  }
};

var poisoned = {
  valueOf() {
    throw "should not coerce";
  }
};

assertThrowsValue(function () {
  Atomics.notify(shared, indexPoisoned, poisoned);
}, "index poisoned", "index poisoned");
assertThrowsValue(function () {
  Atomics.notify(shared, indexPrimitivePoisoned, poisoned);
}, "index primitive poisoned", "index primitive poisoned");
assertThrowsValue(function () {
  Atomics.notify(shared, 0, countPoisoned);
}, "count poisoned", "count poisoned");
assertRangeError(function () {
  Atomics.notify(shared, -1, poisoned);
}, "negative index");
assertRangeError(function () {
  Atomics.notify(shared, -Infinity, poisoned);
}, "negative infinity index");

assertTypeError(function () {
  Atomics.notify(new Int16Array(new SharedArrayBuffer(Int16Array.BYTES_PER_ELEMENT * 2)), poisoned, poisoned);
}, "Int16Array");
assertTypeError(function () {
  Atomics.notify(new Uint32Array(new SharedArrayBuffer(Uint32Array.BYTES_PER_ELEMENT * 2)), poisoned, poisoned);
}, "Uint32Array");
assertTypeError(function () {
  Atomics.notify({}, poisoned, poisoned);
}, "plain object");
assertTypeError(function () {
  Atomics.notify(shared, 0, Symbol("count"));
}, "symbol count");
assertTypeError(function () {
  Atomics.notify(shared, Symbol("index"), poisoned);
}, "symbol index");

890;
