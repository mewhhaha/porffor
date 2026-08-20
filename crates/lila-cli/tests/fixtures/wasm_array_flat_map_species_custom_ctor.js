function hasOwn(object, key) {
  return Object.prototype.hasOwnProperty.call(object, key);
}

function throwsTypeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof TypeError;
  }
  return false;
}

function withSpecies(ctor, mapper) {
  let array = [[11]];
  array.constructor = {
    get [Symbol.species]() {
      return ctor;
    }
  };
  return array.flatMap(mapper);
}

let callCount = 0;
let capturedLength = -1;
let capturedNewTarget = null;

function Species(length) {
  capturedLength = length;
  capturedNewTarget = new.target;
  callCount = callCount + 1;
}

let source = [[1, 2], [3, 4]];
source.constructor = {
  get [Symbol.species]() {
    return Species;
  }
};

let result = source.flatMap(function (value) {
  return value;
});

let desc0 = Object.getOwnPropertyDescriptor(result, "0");
let desc1 = Object.getOwnPropertyDescriptor(result, "1");
let desc2 = Object.getOwnPropertyDescriptor(result, "2");
let desc3 = Object.getOwnPropertyDescriptor(result, "3");

function Poisoned() {
  throw "poisoned species";
}

let poisoned = [[1]];
poisoned.constructor = {
  get [Symbol.species]() {
    return Poisoned;
  }
};

let propagated = false;
try {
  poisoned.flatMap(function (value) {
    throw "mapper should not run";
  });
} catch (err) {
  propagated = err === "poisoned species";
}

function NonExtensibleTarget() {
  Object.preventExtensions(this);
}

let nonExtensibleThrows = throwsTypeError(function () {
  withSpecies(NonExtensibleTarget, function (value) {
    return value;
  });
});

function NonConfigurableTarget() {
  Object.defineProperty(this, "0", {
    value: 17,
    writable: true,
    enumerable: true,
    configurable: false
  });
}

let nonConfigurableThrows = throwsTypeError(function () {
  withSpecies(NonConfigurableTarget, function () {
    return [23];
  });
});

function ConfigurableNonWritableTarget() {
  let target = new Array(0);
  if (target.length !== 0) throw "array length constructor";
  Object.defineProperty(target, "0", {
    value: 17,
    writable: false,
    enumerable: false,
    configurable: true
  });
  return target;
}

let overwritten = withSpecies(ConfigurableNonWritableTarget, function () {
  return [2];
});
let overwrittenDesc = Object.getOwnPropertyDescriptor(overwritten, "0");

result instanceof Species
  && callCount === 1
  && capturedLength === 0
  && capturedNewTarget === Species
  && Object.getPrototypeOf(result) === Species.prototype
  && desc0.value === 1
  && desc0.writable === true
  && desc0.enumerable === true
  && desc0.configurable === true
  && desc1.value === 2
  && desc1.writable === true
  && desc1.enumerable === true
  && desc1.configurable === true
  && desc2.value === 3
  && desc2.writable === true
  && desc2.enumerable === true
  && desc2.configurable === true
  && desc3.value === 4
  && desc3.writable === true
  && desc3.enumerable === true
  && desc3.configurable === true
  && !hasOwn(result, "length")
  && propagated
  && Object.isExtensible(result) === true
  && Object.isExtensible(new NonExtensibleTarget()) === false
  && nonExtensibleThrows
  && nonConfigurableThrows
  && overwrittenDesc.value === 2
  && overwrittenDesc.writable === true
  && overwrittenDesc.enumerable === true
  && overwrittenDesc.configurable === true;
