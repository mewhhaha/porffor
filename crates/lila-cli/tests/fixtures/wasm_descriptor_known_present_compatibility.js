function throwsTypeError(callback) {
  try {
    callback();
    return false;
  } catch (error) {
    return error instanceof TypeError;
  }
}

function verifyDataCompatibility(receiver, key) {
  Object.defineProperty(receiver, key, {
    value: NaN,
    writable: false,
    enumerable: false,
    configurable: false
  });
  Object.defineProperty(receiver, key, { value: NaN });
  Object.defineProperty(receiver, key, {});

  let descriptor = Object.getOwnPropertyDescriptor(receiver, key);
  let zeroKey = key + "Zero";
  Object.defineProperty(receiver, zeroKey, {
    value: 0,
    writable: false,
    configurable: false
  });
  Object.defineProperty(receiver, zeroKey, { value: 0 });

  return Number.isNaN(descriptor.value)
    && descriptor.writable === false
    && descriptor.enumerable === false
    && descriptor.configurable === false
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { configurable: true });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { enumerable: true });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { writable: true });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { value: 1 });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { get: function() { return 1; } });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, zeroKey, { value: -0 });
    });
}

function verifyAccessorCompatibility(receiver, key) {
  function getter() {
    return 7;
  }
  function setter(value) {
    return value;
  }
  function otherGetter() {
    return 8;
  }
  function otherSetter(value) {
    return value + 1;
  }

  Object.defineProperty(receiver, key, {
    get: getter,
    set: setter,
    enumerable: false,
    configurable: false
  });
  Object.defineProperty(receiver, key, { get: getter });
  Object.defineProperty(receiver, key, { set: setter });
  Object.defineProperty(receiver, key, {});

  let descriptor = Object.getOwnPropertyDescriptor(receiver, key);
  return descriptor.get === getter
    && descriptor.set === setter
    && descriptor.enumerable === false
    && descriptor.configurable === false
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { get: otherGetter });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { set: otherSetter });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { enumerable: true });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { configurable: true });
    })
    && throwsTypeError(function() {
      Object.defineProperty(receiver, key, { value: 7 });
    });
}

function verifyConfigurableTransition(receiver, key) {
  function getter() {
    return 11;
  }

  Object.defineProperty(receiver, key, {
    value: 10,
    writable: false,
    enumerable: false,
    configurable: true
  });
  Object.defineProperty(receiver, key, {
    get: getter,
    enumerable: true,
    configurable: false
  });

  let descriptor = Object.getOwnPropertyDescriptor(receiver, key);
  return descriptor.get === getter
    && descriptor.set === undefined
    && descriptor.enumerable === true
    && descriptor.configurable === false;
}

function verifyMissingPropertyCreation(receiver, key) {
  Object.defineProperty(receiver, key, { value: 13 });
  let descriptor = Object.getOwnPropertyDescriptor(receiver, key);
  return descriptor.value === 13
    && descriptor.writable === false
    && descriptor.enumerable === false
    && descriptor.configurable === false;
}

let staticPrototypeConflictRejected = throwsTypeError(function() {
  class StaticPrototypeConflict {
    static ["prototype"] = 1;
  }
});

let ordinary = {};
let array = [];

staticPrototypeConflictRejected
  && verifyDataCompatibility(ordinary, "ordinaryData")
  && verifyAccessorCompatibility(ordinary, "ordinaryAccessor")
  && verifyConfigurableTransition(ordinary, "ordinaryConfigurable")
  && verifyMissingPropertyCreation(ordinary, "ordinaryMissing")
  && verifyDataCompatibility(array, "namedData")
  && verifyAccessorCompatibility(array, "namedAccessor")
  && verifyConfigurableTransition(array, "namedConfigurable")
  && verifyMissingPropertyCreation(array, "namedMissing")
  && array.length === 0;
