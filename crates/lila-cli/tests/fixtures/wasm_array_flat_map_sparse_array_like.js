function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

function pair(value) {
  return [39, value * 2];
}

let sparse = {
  length: 3,
  0: 1,
  2: 21,
  get 3() { throw 12; }
};
let sparseResult = Array.prototype.flatMap.call(sparse, pair);

let emptyLength = {
  length: undefined,
  get 0() { throw 13; }
};
let emptyLengthResult = Array.prototype.flatMap.call(emptyLength, pair);

let lengthReads = 0;
let lengthOnce = {
  get length() {
    lengthReads = lengthReads + 1;
    if (lengthReads === 1) return 2;
    throw 14;
  },
  0: 21,
  1: 19.5,
  get 2() { throw 15; }
};
let lengthOnceResult = Array.prototype.flatMap.call(lengthOnce, pair);

let highIndex = {
  length: 10001
};
highIndex[10000] = 7;
let highIndexResult = Array.prototype.flatMap.call(highIndex, pair);

sameArray(sparseResult, [39, 2, 39, 42])
  && sameArray(emptyLengthResult, [])
  && lengthReads === 1
  && sameArray(lengthOnceResult, [39, 42, 39, 39])
  && sameArray(highIndexResult, [39, 14]);
