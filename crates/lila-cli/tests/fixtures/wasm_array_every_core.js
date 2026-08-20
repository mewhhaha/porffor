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

let descriptor = Object.getOwnPropertyDescriptor(Array.prototype, "every");
let every = Array.prototype.every;
let sparse = [1, 2, 3];
delete sparse[1];
let sparseCount = 0;
let sparseResult = sparse.every(function (value, index, source) {
  sparseCount = sparseCount + 1;
  return value > 0 && index !== 1 && source === sparse;
});
let shortCircuitCount = 0;
let shortCircuitResult = [0, 1, 2].every(function (value) {
  shortCircuitCount = shortCircuitCount + 1;
  return value < 1;
});
let arrayLikeReceiver = { length: 3, 0: 4, 1: 5, 2: 6 };
let argumentCheckArray = [7];
let argumentCheck = argumentCheckArray.every(function (value, index, source) {
  return value === 7 && index === 0 && source === argumentCheckArray;
});

typeof every === "function"
  && every.name === "every"
  && every.length === 1
  && descriptor.value === every
  && descriptor.writable === true
  && descriptor.enumerable === false
  && descriptor.configurable === true
  && [1, 2, 3].every(function (value) { return value > 0; }) === true
  && shortCircuitResult === false
  && shortCircuitCount === 2
  && [2].every(keepThis, { keep: 2 }) === true
  && every.call(arrayLikeReceiver, function (value) { return value >= 4; }) === true
  && every.call(arrayLikeReceiver, function (value) { return value >= 5; }) === false
  && every.call(true, function () { return false; }) === true
  && sparseResult === true
  && sparseCount === 2
  && argumentCheck === true
  && throwsTypeError(function () { every.call(null, function (value) { return value; }); })
  && throwsTypeError(function () { every.call(undefined, function (value) { return value; }); })
  && throwsTypeError(function () { [1].every(); })
  && throwsTypeError(function () { [1].every(null); })
  && throwsTypeError(function () { [1].every(1); })
  && throwsSentinel(function (sentinel) { [1].every(function () { throw sentinel; }); });
