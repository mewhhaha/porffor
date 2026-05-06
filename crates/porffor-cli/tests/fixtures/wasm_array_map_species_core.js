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
  return value + 1;
}

let capturedThisValue;
let capturedNewTarget;
let capturedLength = -1;
let capturedArgumentsLength = -1;
let capturedArgument0 = -1;
let capturedCallCount = 0;

function CapturingSpecies(length) {
  capturedThisValue = this;
  capturedNewTarget = new.target;
  capturedLength = length;
  capturedArgumentsLength = arguments.length;
  capturedArgument0 = arguments[0];
  capturedCallCount = capturedCallCount + 1;
  this.lengthValue = length;
}

function speciesResult(speciesValue) {
  let called = false;
  let arr = [41, 42];
  arr.constructor = {
    get [Symbol.species]() {
      if (called) throw "species read twice";
      called = true;
      return speciesValue;
    }
  };
  let actual = arr.map(mapper);
  return sameArray(actual, [42, 43]) && Object.getPrototypeOf(actual) === Array.prototype && called === true;
}

function speciesThrowsTypeError(speciesValue) {
  let called = false;
  let arr = [41, 42];
  arr.constructor = {
    get [Symbol.species]() {
      if (called) throw "species read twice";
      called = true;
      return speciesValue;
    }
  };
  return throwsTypeError(function () { arr.map(mapper); }) && called === true;
}

let custom = [41, 42];
custom.constructor = {
  get [Symbol.species]() {
    return CapturingSpecies;
  }
};
let customResult = custom.map(mapper);
let customLengthDesc = Object.getOwnPropertyDescriptor(customResult, "lengthValue");
let customZeroDesc = Object.getOwnPropertyDescriptor(customResult, "0");
let customOneDesc = Object.getOwnPropertyDescriptor(customResult, "1");

let noSpecies = [41, 42];
noSpecies.constructor = {};

sameArray(noSpecies.map(mapper), [42, 43])
  && Object.getPrototypeOf(noSpecies.map(mapper)) === Array.prototype
  && customResult instanceof CapturingSpecies
  && capturedThisValue === customResult
  && capturedNewTarget === CapturingSpecies
  && capturedLength === 2
  && capturedArgumentsLength === 1
  && capturedArgument0 === 2
  && capturedCallCount === 1
  && Object.getPrototypeOf(capturedThisValue) === CapturingSpecies.prototype
  && customLengthDesc.value === 2
  && customZeroDesc.value === 42
  && customOneDesc.value === 43
  && speciesResult(null)
  && speciesResult(undefined)
  && speciesThrowsTypeError(0)
  && speciesThrowsTypeError("")
  && speciesThrowsTypeError(false)
  && speciesThrowsTypeError({})
  && speciesThrowsTypeError([])
  && speciesThrowsTypeError(Symbol())
  && throwsSentinel(function (sentinel) {
    let arr = [41];
    arr.constructor = {
      get [Symbol.species]() {
        throw sentinel;
      }
    };
    arr.map(mapper);
  });
