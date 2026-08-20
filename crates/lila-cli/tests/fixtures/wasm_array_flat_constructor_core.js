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

function flatWithConstructor(value) {
  let array = [1, [2]];
  array.constructor = value;
  return array.flat();
}

let defaultArray = [1, [2]];
let undefinedConstructor = [1, [2]];
undefinedConstructor.constructor = undefined;
let objectConstructor = [1, [2]];
objectConstructor.constructor = {};
let arrayLike = { length: 1, 0: [3] };

throwsTypeError(function () { flatWithConstructor(null); })
  && throwsTypeError(function () { flatWithConstructor(1); })
  && throwsTypeError(function () { flatWithConstructor("string"); })
  && throwsTypeError(function () { flatWithConstructor(true); })
  && sameArray(defaultArray.flat(), [1, 2])
  && sameArray(undefinedConstructor.flat(), [1, 2])
  && sameArray(objectConstructor.flat(), [1, 2])
  && sameArray(Array.prototype.flat.call(arrayLike), [3]);
