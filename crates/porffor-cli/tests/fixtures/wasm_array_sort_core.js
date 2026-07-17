function hasOwn(object, key) {
  return Object.prototype.hasOwnProperty.call(object, key);
}

let defaultValues = [10, 2, 1];
let defaultResult = defaultValues.sort();
let defaultStrings = ["z", "a", "m"];
defaultStrings.sort();

let sparse = [undefined, , 2];
sparse.sort();

let stable = [
  { key: 1, name: "first" },
  { key: 0, name: "middle" },
  { key: 1, name: "last" }
];
stable.sort(function (left, right) {
  return left.key - right.key;
});

let sawNegative = false;
let sawPositive = false;
let comparatorThisIsUndefined = true;
let compared = [3, 1, 2];
compared.sort(function (left, right) {
  "use strict";
  comparatorThisIsUndefined = comparatorThisIsUndefined && this === undefined;
  let result = left - right;
  sawNegative = sawNegative || result < 0;
  sawPositive = sawPositive || result > 0;
  return result;
});

let nanResult = [1, 0];
nanResult.sort(function () {
  return NaN;
});

let proxyCompared = [2, 1];
proxyCompared.sort(new Proxy(function (left, right) {
  return left - right;
}, {}));

let toNumberCalled = false;
let coerced = [2, 1];
coerced.sort(function (left, right) {
  return {
    valueOf: function () {
      toNumberCalled = true;
      return left - right;
    }
  };
});

let utf16 = ["\uE000", "\u{10000}", "\uD800"];
utf16.sort();

defaultResult === defaultValues
  && defaultValues[0] === 1
  && defaultValues[1] === 10
  && defaultValues[2] === 2
  && defaultStrings[0] === "a"
  && defaultStrings[1] === "m"
  && defaultStrings[2] === "z"
  && sparse[0] === 2
  && sparse[1] === undefined
  && hasOwn(sparse, "1")
  && !hasOwn(sparse, "2")
  && sparse.length === 3
  && stable[0].name === "middle"
  && stable[1].name === "first"
  && stable[2].name === "last"
  && compared[0] === 1
  && compared[1] === 2
  && compared[2] === 3
  && sawNegative
  && sawPositive
  && comparatorThisIsUndefined
  && nanResult[0] === 1
  && nanResult[1] === 0
  && proxyCompared[0] === 1
  && proxyCompared[1] === 2
  && coerced[0] === 1
  && coerced[1] === 2
  && toNumberCalled
  && utf16[0] === "\uD800"
  && utf16[1] === "\u{10000}"
  && utf16[2] === "\uE000";
