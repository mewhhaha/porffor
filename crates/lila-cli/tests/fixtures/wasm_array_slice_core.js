function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let index = 0; index < actual.length; index++) {
    if (actual[index] !== expected[index]) return false;
  }
  return true;
}

function hasOwn(object, key) {
  return Object.prototype.hasOwnProperty.call(object, key);
}

let bounds = [0, 1, 2, 3, 4];
let defaultEnd = bounds.slice(1, undefined);
let negativeBounds = bounds.slice(-4, -1);
let nanBounds = bounds.slice(NaN, NaN);
let infiniteBounds = bounds.slice(-Infinity, Infinity);

let generic = { 0: "zero", 2: "two", length: 3 };
let genericResult = Array.prototype.slice.call(generic);

let inherited = ["zero", , "two"];
Array.prototype[1] = "inherited";
let inheritedResult = inherited.slice();
delete Array.prototype[1];

let coercionOrder = "";
let coercionSource = {
  0: "zero",
  1: "one",
  2: "two",
  get length() {
    coercionOrder = coercionOrder + "length";
    return 3;
  }
};
let coercionResult = Array.prototype.slice.call(
  coercionSource,
  {
    valueOf: function () {
      coercionOrder = coercionOrder + "start";
      return 1;
    }
  },
  {
    valueOf: function () {
      coercionOrder = coercionOrder + "end";
      return 2;
    }
  }
);

let lastIndex = "9007199254740990";
let safeIntegerSource = { length: 9007199254740991 };
safeIntegerSource[lastIndex] = "last";
let safeIntegerResult = Array.prototype.slice.call(safeIntegerSource, 9007199254740990);

sameArray(defaultEnd, [1, 2, 3, 4])
  && sameArray(negativeBounds, [1, 2, 3])
  && nanBounds.length === 0
  && sameArray(infiniteBounds, [0, 1, 2, 3, 4])
  && genericResult.length === 3
  && genericResult[0] === "zero"
  && !hasOwn(genericResult, "1")
  && genericResult[2] === "two"
  && inheritedResult.length === 3
  && inheritedResult[0] === "zero"
  && inheritedResult[1] === "inherited"
  && hasOwn(inheritedResult, "1")
  && inheritedResult[2] === "two"
  && sameArray(coercionResult, ["one"])
  && coercionOrder === "lengthstartend"
  && sameArray(Array.prototype.slice.call("cat", 1), ["a", "t"])
  && safeIntegerResult.length === 1
  && safeIntegerResult[0] === "last";
