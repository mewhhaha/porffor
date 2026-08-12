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

function keepThis(value) {
  return this.keep === value;
}

let descriptor = Object.getOwnPropertyDescriptor(Array.prototype, "some");
let some = Array.prototype.some;
let sparse = [1, 2, 3];
delete sparse[1];
let sparseCount = 0;
let sparseResult = sparse.some(function (value, index, source) {
  sparseCount = sparseCount + 1;
  return value === 3 && index === 2 && source === sparse;
});
let shortCircuitCount = 0;
let shortCircuitResult = [0, 1, 2].some(function (value) {
  shortCircuitCount = shortCircuitCount + 1;
  return value > 0;
});
let emptyCount = 0;
let emptyResult = [].some(function () {
  emptyCount = emptyCount + 1;
  return true;
});
let arrayLikeReceiver = { length: 3, 0: 4, 1: 5, 2: 6 };
let argumentCheckArray = [7];
let argumentCheck = argumentCheckArray.some(function (value, index, source) {
  return value === 7 && index === 0 && source === argumentCheckArray;
});

typeof some === "function"
  && some.name === "some"
  && some.length === 1
  && descriptor.value === some
  && descriptor.writable === true
  && descriptor.enumerable === false
  && descriptor.configurable === true
  && [1, 2, 3].some(function (value) { return value > 2; }) === true
  && [1, 2, 3].some(function (value) { return value > 3; }) === false
  && shortCircuitResult === true
  && shortCircuitCount === 2
  && emptyResult === false
  && emptyCount === 0
  && [2].some(keepThis, { keep: 2 }) === true
  && some.call(arrayLikeReceiver, function (value) { return value === 5; }) === true
  && some.call(arrayLikeReceiver, function (value) { return value > 7; }) === false
  && some.call(true, function () { return true; }) === false
  && sparseResult === true
  && sparseCount === 2
  && argumentCheck === true
  && throwsTypeError(function () { some.call(null, function (value) { return value; }); })
  && throwsTypeError(function () { some.call(undefined, function (value) { return value; }); })
  && throwsTypeError(function () { [1].some(); })
  && throwsTypeError(function () { [1].some(null); })
  && throwsTypeError(function () { [1].some(1); })
  && throwsSentinel(function (sentinel) { [1].some(function () { throw sentinel; }); });
