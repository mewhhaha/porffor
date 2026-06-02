function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

function throwsTypeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof TypeError;
  }
  return false;
}

function throwsSentinel(fn) {
  let sentinel = { thrown: true };
  try {
    fn(sentinel);
  } catch (err) {
    return err === sentinel;
  }
  return false;
}

function keepEven(value, index, source) {
  return value % 2 === 0 && source.length === 4 && index >= 0;
}

function keepThis(value) {
  return this.keep === value;
}

let descriptor = Object.getOwnPropertyDescriptor(Array.prototype, "filter");
let filter = Array.prototype.filter;
let sparse = [1, 2, 3];
delete sparse[1];
let callbackCount = 0;
let sparseResult = sparse.filter(function (value) {
  callbackCount = callbackCount + 1;
  return value > 1;
});
let arrayLikeReceiver = { length: 3, 0: 4, 1: 5, 2: 6 };

typeof filter === "function"
  && filter.name === "filter"
  && filter.length === 1
  && descriptor.value === filter
  && descriptor.writable === true
  && descriptor.enumerable === false
  && descriptor.configurable === true
  && sameArray([1, 2, 3, 4].filter(keepEven), [2, 4])
  && sameArray([1, 2, 3].filter(keepThis, { keep: 2 }), [2])
  && sameArray(filter.call(arrayLikeReceiver, function (value) { return value >= 5; }), [5, 6])
  && sameArray(filter.call(true, function () { return true; }), [])
  && sparseResult.length === 1
  && sparseResult[0] === 3
  && callbackCount === 2
  && throwsTypeError(function () { filter.call(null, function (value) { return value; }); })
  && throwsTypeError(function () { filter.call(undefined, function (value) { return value; }); })
  && throwsTypeError(function () { [1].filter(); })
  && throwsTypeError(function () { [1].filter(null); })
  && throwsTypeError(function () { [1].filter(1); })
  && throwsSentinel(function (sentinel) { [1].filter(function () { throw sentinel; }); });
