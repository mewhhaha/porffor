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

let capturedThisValue;
let capturedNewTarget;
let capturedArgumentsLength = -1;
let capturedArgument0 = -1;
let capturedCallCount = 0;

function CapturingSpecies(length) {
  capturedThisValue = this;
  capturedNewTarget = new.target;
  capturedArgumentsLength = arguments.length;
  capturedArgument0 = arguments[0];
  capturedCallCount = capturedCallCount + 1;
  this.lengthValue = length;
}

function speciesResult(speciesValue) {
  let called = false;
  let arr = [41];
  arr.constructor = {
    get [Symbol.species]() {
      if (called) throw "species read twice";
      called = true;
      return speciesValue;
    }
  };
  let actual = arr.concat([42]);
  return sameArray(actual, [41, 42]) && called === true;
}

function speciesThrowsTypeError(speciesValue) {
  let called = false;
  let arr = [41];
  arr.constructor = {
    get [Symbol.species]() {
      if (called) throw "species read twice";
      called = true;
      return speciesValue;
    }
  };
  return throwsTypeError(function () { arr.concat([42]); }) && called === true;
}

let custom = [41];
custom.constructor = {
  get [Symbol.species]() {
    return CapturingSpecies;
  }
};
let customResult = custom.concat([42]);
let customLengthDesc = Object.getOwnPropertyDescriptor(customResult, "lengthValue");
let customZeroDesc = Object.getOwnPropertyDescriptor(customResult, "0");
let customOneDesc = Object.getOwnPropertyDescriptor(customResult, "1");

let noSpecies = [41];
noSpecies.constructor = {};

sameArray(noSpecies.concat([42]), [41, 42])
  && Object.getPrototypeOf(noSpecies.concat([42])) === Array.prototype
  && customResult instanceof CapturingSpecies
  && capturedThisValue === customResult
  && capturedNewTarget === CapturingSpecies
  && capturedArgumentsLength === 1
  && capturedArgument0 === 0
  && capturedCallCount === 1
  && Object.getPrototypeOf(capturedThisValue) === CapturingSpecies.prototype
  && customLengthDesc.value === 0
  && customZeroDesc.value === 41
  && customOneDesc.value === 42
  && customResult.length === 2
  && speciesResult(null)
  && speciesResult(undefined)
  && speciesThrowsTypeError(0)
  && speciesThrowsTypeError("")
  && speciesThrowsTypeError(false)
  && speciesThrowsTypeError({})
  && speciesThrowsTypeError([]);
