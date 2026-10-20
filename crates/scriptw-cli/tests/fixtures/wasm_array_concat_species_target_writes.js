function throwsTypeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof TypeError;
  }
  return false;
}

function withSpecies(ctor, firstArg) {
  let array = [];
  array.constructor = {
    get [Symbol.species]() {
      return ctor;
    }
  };
  return array.concat(firstArg);
}

function NonExtensibleTarget() {
  Object.preventExtensions(this);
}

let nonExtensibleNonSpreadThrows = throwsTypeError(function () {
  withSpecies(NonExtensibleTarget, 24);
});
let nonExtensibleSpreadThrows = throwsTypeError(function () {
  withSpecies(NonExtensibleTarget, [24]);
});

function NonConfigurableTarget() {
  Object.defineProperty(this, "0", {
    value: 17,
    writable: true,
    enumerable: true,
    configurable: false
  });
}

let nonConfigurableNonSpreadThrows = throwsTypeError(function () {
  withSpecies(NonConfigurableTarget, 24);
});
let nonConfigurableSpreadThrows = throwsTypeError(function () {
  withSpecies(NonConfigurableTarget, [24]);
});

function ConfigurableNonWritableTarget() {
  Object.defineProperty(this, "0", {
    value: 17,
    writable: false,
    enumerable: false,
    configurable: true
  });
}

let overwrittenNonSpread = withSpecies(ConfigurableNonWritableTarget, 24);
let overwrittenNonSpreadDesc = Object.getOwnPropertyDescriptor(overwrittenNonSpread, "0");
let overwrittenSpread = withSpecies(ConfigurableNonWritableTarget, [24]);
let overwrittenSpreadDesc = Object.getOwnPropertyDescriptor(overwrittenSpread, "0");

Object.isExtensible(new NonExtensibleTarget()) === false
  && nonExtensibleNonSpreadThrows
  && nonExtensibleSpreadThrows
  && nonConfigurableNonSpreadThrows
  && nonConfigurableSpreadThrows
  && overwrittenNonSpreadDesc.value === 24
  && overwrittenNonSpreadDesc.writable === true
  && overwrittenNonSpreadDesc.enumerable === true
  && overwrittenNonSpreadDesc.configurable === true
  && overwrittenNonSpread.length === 1
  && overwrittenSpreadDesc.value === 24
  && overwrittenSpreadDesc.writable === true
  && overwrittenSpreadDesc.enumerable === true
  && overwrittenSpreadDesc.configurable === true
  && overwrittenSpread.length === 1;
