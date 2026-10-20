function freezesMappedValue(a) {
  Object.defineProperty(arguments, "0", { configurable: false });
  arguments[0] = 2;
  Object.defineProperty(arguments, "0", { writable: false });
  const descriptor = Object.getOwnPropertyDescriptor(arguments, "0");
  a = 3;
  arguments[0] = 4;
  return descriptor.value === 2
    && descriptor.writable === false
    && descriptor.enumerable === true
    && descriptor.configurable === false
    && a === 3
    && arguments[0] === 2;
}

function detachesAccessor(a) {
  let assigned;
  let receiverMatches;
  const args = arguments;
  Object.defineProperty(arguments, "0", {
    set(value) { assigned = value; receiverMatches = this === args; },
    enumerable: true,
    configurable: true,
  });
  arguments[0] = 2;
  a = 3;
  return assigned === 2 && receiverMatches && a === 3 && arguments[0] === undefined;
}

function detachesDeletedIndex(a) {
  const deleted = delete arguments[0];
  a = 2;
  return deleted && a === 2 && arguments[0] === undefined;
}

function rejectedRedefinitionKeepsMapping(a) {
  Object.defineProperty(arguments, "0", { configurable: false });
  let threw = false;
  try {
    Object.defineProperty(arguments, "0", { configurable: true });
  } catch (error) {
    threw = error instanceof TypeError;
  }
  a = 2;
  return threw && arguments[0] === 2;
}

function accessorConversionDoesNotCallGetter(a) {
  let calls = 0;
  Object.defineProperty(arguments, "0", {
    get() { calls += 1; return a; },
    configurable: true,
  });
  Object.defineProperty(arguments, "0", { writable: true });
  return calls === 0 && arguments[0] === undefined;
}

function indexedWriteDoesNotChangeArgumentsLength(a) {
  arguments[5] = 9;
  return arguments.length === 1 && arguments[5] === 9;
}

let escapedArguments;

function redefinesDeletedIndexAsAccessor(a) {
  const args = arguments;
  escapedArguments = args;
  let assigned;
  const getter = function () { return 10; };
  const setter = function (value) {
    assigned = value;
    escapedArguments.setterEffect = value;
  };
  delete args[0];
  Object.defineProperties(args, {
    "0": {
      get: getter,
      set: setter,
      enumerable: true,
      configurable: true,
    },
  });
  args[0] = 20;
  const descriptor = Object.getOwnPropertyDescriptor(args, "0");
  return args[0] === 10
    && assigned === 20
    && args.setterEffect === 20
    && descriptor.get === getter
    && descriptor.set === setter
    && descriptor.enumerable === true
    && descriptor.configurable === true;
}

function updatesDetachedAccessor(a) {
  const args = arguments;
  const firstGetter = function () { return 10; };
  const secondGetter = function () { return 20; };
  Object.defineProperty(args, "0", {
    get: firstGetter,
    enumerable: true,
    configurable: true,
  });
  Object.defineProperties(args, {
    "0": {
      get: secondGetter,
      enumerable: false,
      configurable: false,
    },
  });
  const descriptor = Object.getOwnPropertyDescriptor(args, "0");
  return descriptor.get === secondGetter
    && descriptor.set === undefined
    && descriptor.enumerable === false
    && descriptor.configurable === false;
}

function preservesNonzeroMappedSlot(a, b, c) {
  Object.defineProperty(arguments, "1", { enumerable: false });
  b = 7;
  const bindingStillMapped = arguments[1] === 7;
  Object.defineProperty(arguments, "1", { value: 9 });
  const valueUpdatedBinding = b === 9;
  Object.defineProperty(arguments, "1", { writable: false });
  b = 11;
  return bindingStillMapped
    && valueUpdatedBinding
    && arguments[1] === 9
    && Object.getOwnPropertyDescriptor(arguments, "1").writable === false;
}

function writeDynamicNamed(target, key, value) {
  target[key] = value;
}

function honorsOwnNamedSetter(a) {
  const args = arguments;
  let assigned;
  const setter = function (value) { assigned = value; };
  Object.defineProperty(args, "ownNamed", {
    set: setter,
    configurable: true,
  });
  writeDynamicNamed(args, "ownNamed", 17);
  const descriptor = Object.getOwnPropertyDescriptor(args, "ownNamed");
  return assigned === 17
    && descriptor.set === setter
    && descriptor.get === undefined;
}

function honorsInheritedNamedSetter(a) {
  const args = arguments;
  const prototype = {};
  let receiver;
  let assigned;
  Object.defineProperty(prototype, "inheritedNamed", {
    set(value) { receiver = this; assigned = value; },
    configurable: true,
  });
  Object.setPrototypeOf(args, prototype);
  writeDynamicNamed(args, "inheritedNamed", 23);
  return receiver === args
    && assigned === 23
    && Object.getOwnPropertyDescriptor(args, "inheritedNamed") === undefined;
}

function honorsNonWritableNamedProperty(a) {
  const args = arguments;
  Object.defineProperty(args, "fixedNamed", {
    value: 29,
    writable: false,
    configurable: true,
  });
  writeDynamicNamed(args, "fixedNamed", 31);
  return args.fixedNamed === 29;
}

function rejectsAbsentIndexOnNonExtensibleArguments(a) {
  const args = arguments;
  delete args[0];
  Object.preventExtensions(args);
  let threw = false;
  try {
    Object.defineProperty(args, "0", { value: 37 });
  } catch (error) {
    threw = error instanceof TypeError;
  }
  return threw
    && Object.getOwnPropertyDescriptor(args, "0") === undefined
    && args[0] === undefined;
}

function honorsInheritedIndexSetterAfterDelete(a) {
  const args = arguments;
  const prototype = {};
  let receiver;
  let assigned;
  delete args[0];
  Object.defineProperty(prototype, "0", {
    set(value) { receiver = this; assigned = value; },
    configurable: true,
  });
  Object.setPrototypeOf(args, prototype);
  args[0] = 41;
  return receiver === args
    && assigned === 41
    && Object.getOwnPropertyDescriptor(args, "0") === undefined;
}

function rejectsAbsentIndexAssignmentOnNonExtensibleArguments(a) {
  const args = arguments;
  delete args[0];
  Object.preventExtensions(args);
  args[0] = 43;
  return Object.getOwnPropertyDescriptor(args, "0") === undefined
    && args[0] === undefined;
}

function honorsArgumentsPrototypeIndexedDescriptors() {
  let receiver;
  let assigned;
  const setterPrototype = (function (a) {
    Object.defineProperty(arguments, "0", {
      set(value) { receiver = this; assigned = value; },
      configurable: true,
    });
    return arguments;
  }(1));
  const setterChild = (function (a) {
    delete arguments[0];
    return arguments;
  }(1));
  Object.setPrototypeOf(setterChild, setterPrototype);
  setterChild[0] = 47;

  const fixedPrototype = (function (a) {
    Object.defineProperty(arguments, "0", {
      value: 53,
      writable: false,
      configurable: true,
    });
    return arguments;
  }(1));
  const fixedChild = (function (a) {
    delete arguments[0];
    return arguments;
  }(1));
  Object.setPrototypeOf(fixedChild, fixedPrototype);
  fixedChild[0] = 59;

  return receiver === setterChild
    && assigned === 47
    && Object.getPrototypeOf(setterChild) === setterPrototype
    && Object.getOwnPropertyDescriptor(setterChild, "0") === undefined
    && fixedPrototype[0] === 53
    && Object.getPrototypeOf(fixedChild) === fixedPrototype
    && Object.getOwnPropertyDescriptor(fixedChild, "0") === undefined;
}

function genericLengthUpdatePreservesAccessor(a) {
  const args = arguments;
  let value = 7;
  const getter = function () { return value; };
  const setter = function (next) { value = next; };
  const replacementGetter = function () { return value + 1; };
  Object.defineProperty(args, "length", {
    get: getter,
    set: setter,
    configurable: true,
  });
  Object.defineProperty(args, "length", { get: replacementGetter });
  Object.defineProperty(args, "length", { enumerable: true });
  const descriptor = Object.getOwnPropertyDescriptor(args, "length");
  args.length = 11;
  return descriptor.get === replacementGetter
    && descriptor.set === setter
    && !("value" in descriptor)
    && !("writable" in descriptor)
    && descriptor.enumerable === true
    && descriptor.configurable === true
    && args.length === 12;
}

freezesMappedValue(1)
  && detachesAccessor(1)
  && detachesDeletedIndex(1)
  && rejectedRedefinitionKeepsMapping(1)
  && accessorConversionDoesNotCallGetter(1)
  && indexedWriteDoesNotChangeArgumentsLength(1)
  && redefinesDeletedIndexAsAccessor(1)
  && updatesDetachedAccessor(1)
  && preservesNonzeroMappedSlot(1, 2, 3)
  && honorsOwnNamedSetter(1)
  && honorsInheritedNamedSetter(1)
  && honorsNonWritableNamedProperty(1)
  && rejectsAbsentIndexOnNonExtensibleArguments(1)
  && honorsInheritedIndexSetterAfterDelete(1)
  && rejectsAbsentIndexAssignmentOnNonExtensibleArguments(1)
  && honorsArgumentsPrototypeIndexedDescriptors()
  && genericLengthUpdatePreservesAccessor(1);
