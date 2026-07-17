function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let index = 0; index < actual.length; index++) {
    if (actual[index] !== expected[index]) return false;
  }
  return true;
}

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error.constructor === TypeError;
  }
  return false;
}

function withSpecies(species) {
  let source = ["zero", "one"];
  source.constructor = {
    get [Symbol.species]() {
      return species;
    }
  };
  return source.slice();
}

function CustomTarget(length) {
  this.constructedLength = length;
}

function NonExtensibleTarget() {
  this.length = 0;
  Object.preventExtensions(this);
}

function NonConfigurableTarget() {
  Object.defineProperty(this, "0", {
    value: "old",
    writable: true,
    enumerable: true,
    configurable: false
  });
}

function ConfigurableTarget() {
  Object.defineProperty(this, "0", {
    value: "old",
    writable: false,
    enumerable: false,
    configurable: true
  });
}

function NonWritableLengthTarget() {
  Object.defineProperty(this, "length", {
    value: 0,
    writable: false,
    enumerable: false,
    configurable: true
  });
}

let custom = withSpecies(CustomTarget);
let fallbackFromNull = withSpecies(null);
let fallbackFromUndefined = withSpecies(undefined);
let overwritten = withSpecies(ConfigurableTarget);
let overwrittenDescriptor = Object.getOwnPropertyDescriptor(overwritten, "0");

let invalidConstructorSource = [0];
invalidConstructorSource.constructor = 1;

let sentinel = { sentinel: true };
let abruptSpecies = [0];
abruptSpecies.constructor = {
  get [Symbol.species]() {
    throw sentinel;
  }
};
let abruptSpeciesPreserved = false;
try {
  abruptSpecies.slice();
} catch (error) {
  abruptSpeciesPreserved = error === sentinel;
}

let genericSpeciesIgnored = Array.prototype.slice.call({
  0: "generic",
  length: 1,
  constructor: {
    get [Symbol.species]() {
      throw "non-array species should not run";
    }
  }
});

custom.constructedLength === 2
  && custom.length === 2
  && custom[0] === "zero"
  && custom[1] === "one"
  && sameArray(fallbackFromNull, ["zero", "one"])
  && sameArray(fallbackFromUndefined, ["zero", "one"])
  && throwsTypeError(function () { withSpecies({}); })
  && throwsTypeError(function () { invalidConstructorSource.slice(); })
  && abruptSpeciesPreserved
  && throwsTypeError(function () { withSpecies(NonExtensibleTarget); })
  && throwsTypeError(function () { withSpecies(NonConfigurableTarget); })
  && throwsTypeError(function () { withSpecies(NonWritableLengthTarget); })
  && overwrittenDescriptor.value === "zero"
  && overwrittenDescriptor.writable === true
  && overwrittenDescriptor.enumerable === true
  && overwrittenDescriptor.configurable === true
  && overwritten.length === 2
  && sameArray(genericSpeciesIgnored, ["generic"]);
