function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

let objectValue = { marker: 1 };
let sparse = [1, 2];
delete sparse[1];
let sparseResult = sparse.concat([3]);
let zeroSource = [1, 2];
let zeroArg = zeroSource.concat();
let arrayArg = [1].concat([2, 3]);
let nonArrayArg = [1].concat(objectValue);
let multipleArgs = [1].concat([2], 3, [4, 5]);
let descriptor = Object.getOwnPropertyDescriptor(Array.prototype, "concat");
let concat = Array.prototype.concat;

typeof concat === "function"
  && concat.name === "concat"
  && concat.length === 1
  && descriptor.value === concat
  && descriptor.writable === true
  && descriptor.enumerable === false
  && descriptor.configurable === true
  && sameArray(zeroArg, [1, 2])
  && zeroArg !== zeroSource
  && sameArray(arrayArg, [1, 2, 3])
  && sameArray(nonArrayArg, [1, objectValue])
  && sameArray(multipleArgs, [1, 2, 3, 4, 5])
  && sparseResult.length === 3
  && sparseResult[0] === 1
  && sparseResult[2] === 3
  && Object.prototype.hasOwnProperty.call(sparseResult, "1") === false;
