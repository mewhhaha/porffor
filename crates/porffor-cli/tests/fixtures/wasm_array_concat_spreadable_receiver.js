function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

let receiver = {
  length: 3,
  0: "r0",
  2: "r2",
  constructor: {
    get [Symbol.species]() {
      throw "non-array receiver must ignore species";
    }
  }
};
receiver[Symbol.isConcatSpreadable] = true;

let sparseArg = {
  length: 3,
  0: "a0",
  2: "a2"
};
sparseArg[Symbol.isConcatSpreadable] = true;

let result = Array.prototype.concat.call(receiver, sparseArg, "tail");

sameArray(result, ["r0", undefined, "r2", "a0", undefined, "a2", "tail"])
  && Object.getPrototypeOf(result) === Array.prototype
  && Object.prototype.hasOwnProperty.call(result, "1") === false
  && Object.prototype.hasOwnProperty.call(result, "4") === false;
