function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

function same(value) {
  return value;
}

let boundResult = [0, 0].flatMap(function () {
  return this;
}.bind([1, 2]));

let obj1 = { length: 1, 0: "a" };
let obj2 = new Int32Array(2);
let obj3 = { get length() { throw 99; } };
let arrayLike = {
  length: 4,
  0: obj1,
  1: obj2,
  2: obj3,
  get 3() { return arrayLike; }
};
let arrayLikeResult = Array.prototype.flatMap.call(arrayLike, same);

let typed = new Int32Array([1, 0, 42]);
let typedResult = Array.prototype.flatMap.call(typed, same);

sameArray(boundResult, [1, 2, 1, 2])
  && sameArray(arrayLikeResult, [obj1, obj2, obj3, arrayLike])
  && arrayLikeResult[0] === obj1
  && arrayLikeResult[1] === obj2
  && arrayLikeResult[2] === obj3
  && arrayLikeResult[3] === arrayLike
  && sameArray(typedResult, [1, 0, 42])
  && !(typedResult instanceof Int32Array);
