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

function throwsSentinel(fn) {
  let sentinel = { thrown: true };
  try {
    fn(sentinel);
  } catch (err) {
    return err === sentinel;
  }
  return false;
}

function mapper(value) {
  return value;
}

function speciesResult(speciesValue) {
  let called = false;
  let arr = [[42, 1], [42, 2]];
  arr.constructor = {
    get [Symbol.species]() {
      if (called) throw "species read twice";
      called = true;
      return speciesValue;
    }
  };
  let actual = arr.flatMap(mapper);
  return sameArray(actual, [42, 1, 42, 2]) && Object.getPrototypeOf(actual) === Array.prototype && called === true;
}

function speciesThrowsTypeError(speciesValue) {
  let called = false;
  let arr = [[42, 1], [42, 2]];
  arr.constructor = {
    get [Symbol.species]() {
      if (called) throw "species read twice";
      called = true;
      return speciesValue;
    }
  };
  return throwsTypeError(function () { arr.flatMap(mapper); }) && called === true;
}

let noSpecies = [[42, 1], [42, 2]];
noSpecies.constructor = {};

sameArray(noSpecies.flatMap(mapper), [42, 1, 42, 2])
  && Object.getPrototypeOf(noSpecies.flatMap(mapper)) === Array.prototype
  && speciesResult(null)
  && speciesResult(undefined)
  && speciesThrowsTypeError(0)
  && speciesThrowsTypeError("")
  && speciesThrowsTypeError(false)
  && speciesThrowsTypeError({})
  && speciesThrowsTypeError([])
  && speciesThrowsTypeError(Symbol())
  && throwsSentinel(function (sentinel) {
    let arr = [[42, 1]];
    arr.constructor = {
      get [Symbol.species]() {
        throw sentinel;
      }
    };
    arr.flatMap(mapper);
  });
