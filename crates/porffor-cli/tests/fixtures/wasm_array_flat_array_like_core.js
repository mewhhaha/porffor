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

function throwsAny(fn) {
  try {
    fn();
  } catch (err) {
    return true;
  }
  return false;
}

function flatArguments() {
  return [].flat.call(arguments);
}

let flat = Array.prototype.flat;
let nestedObject = { value: 2 };
let nestedArgumentsArray = [3];
let trueFlat = flat.call(true);
let falseFlat = flat.call(false);
let normal = [1, [2, nestedArgumentsArray]];
let depthZero = [1, [2]];
let nestedArrayLike = { length: 1, 0: 2 };

sameArray(flatArguments([1], [2, nestedArgumentsArray], nestedObject), [1, 2, nestedArgumentsArray, nestedObject])
  && sameArray(flat.call({ length: 1, 0: [1] }), [1])
  && sameArray(flat.call({ length: undefined, 0: [1] }), [])
  && sameArray(flat.call({ length: NaN, 0: [1] }), [])
  && sameArray(flat.call({ length: -1, 0: [1] }), [])
  && sameArray(trueFlat, [])
  && sameArray(falseFlat, [])
  && sameArray(depthZero.flat({}), depthZero)
  && sameArray(normal.flat("1"), [1, 2, nestedArgumentsArray])
  && sameArray(normal.flat(2), [1, 2, 3])
  && sameArray(flat.call({ length: 2, 0: [1], 1: nestedArrayLike }), [1, nestedArrayLike])
  && throwsAny(function () { [].flat(Symbol()); })
  && throwsAny(function () { [].flat(Object.create(null)); });
