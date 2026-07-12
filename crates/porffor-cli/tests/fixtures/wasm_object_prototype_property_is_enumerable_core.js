let propertyIsEnumerable = Object.prototype.propertyIsEnumerable;
let target = {};
let key = Symbol("same-description");
target[key] = 1;

let primitiveCalls = 0;
let primitiveKey = {
  [Symbol.toPrimitive]() {
    primitiveCalls++;
    return key;
  },
};

let toStringCalls = 0;
let valueOfSuppressed = false;
let toStringKey = {
  toString() {
    toStringCalls++;
    return key;
  },
  valueOf() {
    valueOfSuppressed = true;
    throw new Error("valueOf must be suppressed");
  },
};

let valueOfCalls = 0;
let valueOfKey = {
  toString: null,
  valueOf() {
    valueOfCalls++;
    return key;
  },
};

let abrupt = {};
let abruptKey = {
  [Symbol.toPrimitive]() {
    throw abrupt;
  },
};
let abruptPropagated = false;
try {
  propertyIsEnumerable.call(target, abruptKey);
} catch (error) {
  abruptPropagated = error === abrupt;
}

let nullReceiverOrdering = false;
let nullReceiverKey = {
  [Symbol.toPrimitive]() {
    throw abrupt;
  },
};
try {
  propertyIsEnumerable.call(null, nullReceiverKey);
} catch (error) {
  nullReceiverOrdering = error === abrupt;
}

let other = __porfCreateRealm().global;
let otherReceiverError = false;
try {
  other.Object.prototype.propertyIsEnumerable.call(null, "x");
} catch (error) {
  otherReceiverError = error instanceof other.TypeError
    && !(error instanceof TypeError);
}

propertyIsEnumerable.call(target, primitiveKey)
  && propertyIsEnumerable.call(target, toStringKey)
  && propertyIsEnumerable.call(target, valueOfKey)
  && primitiveCalls === 1
  && toStringCalls === 1
  && !valueOfSuppressed
  && valueOfCalls === 1
  && !propertyIsEnumerable.call(target, Symbol("same-description"))
  && !propertyIsEnumerable.call(target, "same-description")
  && abruptPropagated
  && nullReceiverOrdering
  && otherReceiverError;
