function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function expectOtherRealmTypeError(action, label) {
  var caught;
  try {
    action();
  } catch (error) {
    caught = error;
  }

  assert(caught !== undefined, label + " did not throw");
  assert(caught instanceof other.TypeError, label + " defining realm");
  assert(!(caught instanceof TypeError), label + " not entry realm");
}

var other = __lilaCreateRealm().global;
var nonCallableNextCalls = 0;
var nonCallablePredicateCalls = 0;
var nonCallableReturn = {
  __proto__: other.Iterator.prototype,
  next: function () {
    nonCallableNextCalls += 1;
    return { value: 1, done: false };
  },
  return: 0,
};

expectOtherRealmTypeError(
  function () {
    other.Iterator.prototype.some.call(nonCallableReturn, function () {
      nonCallablePredicateCalls += 1;
      return true;
    });
  },
  "non-callable return"
);
assert(nonCallableNextCalls === 1, "non-callable return next count");
assert(nonCallablePredicateCalls === 1, "non-callable return predicate count");

var primitiveNextCalls = 0;
var primitivePredicateCalls = 0;
var primitiveReturnCalls = 0;
var primitiveReturn = {
  __proto__: other.Iterator.prototype,
  next: function () {
    primitiveNextCalls += 1;
    return { value: 2, done: false };
  },
  return: function () {
    primitiveReturnCalls += 1;
    return 0;
  },
};

expectOtherRealmTypeError(
  function () {
    other.Iterator.prototype.some.call(primitiveReturn, function () {
      primitivePredicateCalls += 1;
      return true;
    });
  },
  "primitive return result"
);
assert(primitiveNextCalls === 1, "primitive return next count");
assert(primitivePredicateCalls === 1, "primitive return predicate count");
assert(primitiveReturnCalls === 1, "primitive return call count");

var validNextCalls = 0;
var validPredicateCalls = 0;
var validReturnCalls = 0;
var validReturn = {
  __proto__: other.Iterator.prototype,
  next: function () {
    validNextCalls += 1;
    return { value: 3, done: false };
  },
  return: function () {
    validReturnCalls += 1;
    return {};
  },
};
var validResult = other.Iterator.prototype.some.call(
  validReturn,
  function () {
    validPredicateCalls += 1;
    return true;
  }
);
assert(validResult === true, "valid close result");
assert(validNextCalls === 1, "valid close next count");
assert(validPredicateCalls === 1, "valid close predicate count");
assert(validReturnCalls === 1, "valid close return count");

true;
