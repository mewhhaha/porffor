function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function expectEntryTypeError(action, expectedMessage, label) {
  var caught;
  try {
    action();
  } catch (error) {
    caught = error;
  }

  assert(caught !== undefined, label + " did not throw");
  assert(
    Object.getPrototypeOf(caught) === TypeError.prototype,
    label + " did not throw the entry TypeError"
  );
  assert(caught.message === expectedMessage, label + " wrong message");
}

expectEntryTypeError(
  function () {
    for (var value of null) {
      throw new Error("nullish for-of entered its body");
    }
  },
  "for-of target is not iterable",
  "nullish source"
);

var nonCallableIterator = {};
nonCallableIterator[Symbol.iterator] = 0;
expectEntryTypeError(
  function () {
    for (var value of nonCallableIterator) {
      throw new Error("non-callable iterator for-of entered its body");
    }
  },
  "for-of target is not iterable",
  "non-callable iterator method"
);

var primitiveIteratorResult = {};
primitiveIteratorResult[Symbol.iterator] = function () {
  return 0;
};
expectEntryTypeError(
  function () {
    for (var value of primitiveIteratorResult) {
      throw new Error("primitive iterator result for-of entered its body");
    }
  },
  "for-of iterator method must return object",
  "primitive iterator result"
);

var nonCallableNext = {};
nonCallableNext[Symbol.iterator] = function () {
  return { next: 0 };
};
expectEntryTypeError(
  function () {
    for (var value of nonCallableNext) {
      throw new Error("non-callable next for-of entered its body");
    }
  },
  "for-of iterator next must be callable",
  "non-callable next"
);

var primitiveNextResult = {};
primitiveNextResult[Symbol.iterator] = function () {
  return {
    next: function () {
      return 0;
    },
  };
};
expectEntryTypeError(
  function () {
    for (var value of primitiveNextResult) {
      throw new Error("primitive next result for-of entered its body");
    }
  },
  "for-of iterator next result must be object",
  "primitive next result"
);

var validIteratorCalls = 0;
var validNextCalls = 0;
var validIterable = {};
validIterable[Symbol.iterator] = function () {
  validIteratorCalls += 1;
  var done = false;
  return {
    next: function () {
      validNextCalls += 1;
      if (done) return { done: true };
      done = true;
      return { value: 42, done: false };
    },
  };
};
var validValues = "";
for (var value of validIterable) {
  validValues += value;
}
assert(validValues === "42", "valid for-of value");
assert(validIteratorCalls === 1, "valid iterator method call count");
assert(validNextCalls === 2, "valid next call count");

true;
