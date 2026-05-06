function throwsTypeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof TypeError;
  }
  return false;
}

function withSpecies(ctor) {
  let array = [[2]];
  array.constructor = {
    get [Symbol.species]() {
      return ctor;
    }
  };
  return array.flat();
}

function NonExtensibleTarget() {
  Object.preventExtensions(this);
}

let nonExtensibleThrows = throwsTypeError(function () {
  withSpecies(NonExtensibleTarget);
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
  withSpecies(NonConfigurableTarget);
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

let overwritten = withSpecies(ConfigurableNonWritableTarget);
let overwrittenDesc = Object.getOwnPropertyDescriptor(overwritten, "0");

Object.isExtensible(new NonExtensibleTarget()) === false
  && nonExtensibleThrows
  && nonConfigurableThrows
  && overwrittenDesc.value === 2
  && overwrittenDesc.writable === true
  && overwrittenDesc.enumerable === true
  && overwrittenDesc.configurable === true;
