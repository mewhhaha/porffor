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

function detachingIndex(buffer) {
  return {
    valueOf() {
      buffer.transfer();
      return 0;
    }
  };
}

function detachingValue(buffer) {
  return {
    valueOf() {
      buffer.transfer();
      return 1;
    }
  };
}

function freshInt32() {
  return new Int32Array(new ArrayBuffer(4));
}

// Index coercion detaches after the entry check: RevalidateAtomicAccess
// must still throw.
var cases = [
  ["load", function (ta, index) { Atomics.load(ta, index); }],
  ["store", function (ta, index) { Atomics.store(ta, index, 0); }],
  ["add", function (ta, index) { Atomics.add(ta, index, 0); }],
  ["exchange", function (ta, index) { Atomics.exchange(ta, index, 0); }],
  ["compareExchange", function (ta, index) { Atomics.compareExchange(ta, index, 0, 0); }]
];
for (var [name, run] of cases) {
  var ta = freshInt32();
  var buffer = ta.buffer;
  assertTypeError(function () { run(ta, detachingIndex(buffer)); }, name + " index");
}

// Value coercion detaches: same re-check.
var valueCases = [
  ["store", function (ta, value) { Atomics.store(ta, 0, value); }],
  ["add", function (ta, value) { Atomics.add(ta, 0, value); }],
  ["exchange", function (ta, value) { Atomics.exchange(ta, 0, value); }]
];
for (var [name, run] of valueCases) {
  var ta = freshInt32();
  var buffer = ta.buffer;
  assertTypeError(function () { run(ta, detachingValue(buffer)); }, name + " value");
}

// The compareExchange replacement is coerced last; detaching there throws too.
var ta = freshInt32();
var buffer = ta.buffer;
assertTypeError(
  function () { Atomics.compareExchange(ta, 0, 0, detachingValue(buffer)); },
  "compareExchange replacement"
);

// notify intentionally does not revalidate: detaching mid-coercion returns 0.
ta = freshInt32();
buffer = ta.buffer;
if (Atomics.notify(ta, detachingIndex(buffer)) !== 0) throw "notify should return 0";

347;
