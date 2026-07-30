const intrinsicToString = BigInt.prototype.toString;
const intrinsicValueOf = BigInt.prototype.valueOf;
let toStringGets = 0;
let valueOfGets = 0;
let toStringFunction = function () {
  return `${intrinsicToString.call(this)}s`;
};
let valueOfFunction = function () {
  return intrinsicValueOf.call(this) * 2n;
};

Object.defineProperty(BigInt.prototype, "toString", {
  get() {
    ++toStringGets;
    return toStringFunction;
  },
});
Object.defineProperty(BigInt.prototype, "valueOf", {
  get() {
    ++valueOfGets;
    return valueOfFunction;
  },
});

let dateThrows = false;
let defaultResult = Object(3n) + 1n;
let stringResult = `${Object(3n)}`;
let propertyResult = { "3s": 7 }[Object(3n)];

toStringFunction = undefined;
valueOfFunction = null;
try {
  new Date(Object(3n));
} catch (error) {
  dateThrows = error instanceof TypeError;
}

(defaultResult === 7n) +
  (stringResult === "3s") +
  (propertyResult === 7) +
  (toStringGets === 3) +
  (valueOfGets === 2) +
  dateThrows;
