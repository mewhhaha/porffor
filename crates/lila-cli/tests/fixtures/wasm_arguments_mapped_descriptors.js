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
  && genericLengthUpdatePreservesAccessor(1);
