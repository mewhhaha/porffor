function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var sum = Math.sumPrecise;

assert(sum([1, 2, 3]) === 6, "runtime array");

function* numbers() {
  yield 1;
  yield 2;
}
assert(sum(numbers()) === 3, "runtime generator");

var overridden = [4];
overridden[Symbol.iterator] = numbers;
assert(sum(overridden) === 3, "overridden array iterator");

var custom = {};
custom[Symbol.iterator] = function () {
  var index = 0;
  return {
    next: function () {
      index += 1;
      return index <= 2
        ? { value: index, done: false }
        : { value: undefined, done: true };
    },
  };
};
assert(sum(custom) === 3, "custom iterable");

function sumArguments() {
  return sum(arguments);
}
assert(sumArguments(2, 3, 4) === 9, "arguments iterable");

assert(1 / sum([]) === -Infinity, "empty is negative zero");
assert(1 / sum([-0, -0]) === -Infinity, "all negative zero");
assert(1 / sum([-0, 0]) === Infinity, "positive zero enters finite state");
assert(sum([1e30, 0.1, -1e30]) === 0.1, "exact cancellation");
assert(
  sum([
    8.98846567431158e307,
    8.988465674311579e307,
    -1.7976931348623157e308,
  ]) === 9.9792015476736e291,
  "adversarial round-to-nearest-even",
);
assert(sum([8.98846567431158e307, 8.98846567431158e307]) === Infinity, "overflow");

var nan = sum([NaN]);
assert(nan !== nan, "NaN");
assert(sum([Infinity, Infinity]) === Infinity, "positive infinity");
assert(sum([-Infinity]) === -Infinity, "negative infinity");
nan = sum([Infinity, -Infinity]);
assert(nan !== nan, "opposite infinities");

var coercions = 0;
var nonNumber = {
  valueOf: function () {
    coercions += 1;
    return 1;
  },
  toString: function () {
    coercions += 1;
    return "1";
  },
};

function expectTypeError(action, label) {
  var threw = false;
  try {
    action();
  } catch (error) {
    threw = true;
    assert(error instanceof TypeError, label + " TypeError");
  }
  assert(threw, label + " did not throw");
}

expectTypeError(function () { sum([nonNumber]); }, "object element");
expectTypeError(function () { sum([0n]); }, "BigInt element");
expectTypeError(function () { sum([NaN, nonNumber]); }, "after NaN");
expectTypeError(
  function () { sum([-Infinity, Infinity, nonNumber]); },
  "after opposite infinities",
);
assert(coercions === 0, "non-number values are not coerced");

var nextCalls = 0;
var returnCalls = 0;
var closing = {};
closing[Symbol.iterator] = function () {
  return {
    next: function () {
      nextCalls += 1;
      return { value: nonNumber, done: false };
    },
    return: function () {
      returnCalls += 1;
      return {};
    },
  };
};
expectTypeError(function () { sum(closing); }, "close on non-number");
assert(nextCalls === 1 && returnCalls === 1, "next and return exactly once");

var closeMarker = {};
var preserving = {};
preserving[Symbol.iterator] = function () {
  return {
    next: function () {
      return { value: nonNumber, done: false };
    },
    return: function () {
      throw closeMarker;
    },
  };
};
var preservedTypeError = false;
try {
  sum(preserving);
} catch (error) {
  preservedTypeError = error instanceof TypeError && error !== closeMarker;
}
assert(preservedTypeError, "close preserves algorithm-created TypeError");

var abruptMarker = {};
var abruptReturnCalls = 0;
var abrupt = {};
abrupt[Symbol.iterator] = function () {
  return {
    next: function () {
      throw abruptMarker;
    },
    return: function () {
      abruptReturnCalls += 1;
      return {};
    },
  };
};
var abruptIdentity = false;
try {
  sum(abrupt);
} catch (error) {
  abruptIdentity = error === abruptMarker;
}
assert(abruptIdentity && abruptReturnCalls === 0, "next abrupt propagates without close");

var other = __lilaCreateRealm().global;

function expectOtherTypeError(action, expectedMessage, label) {
  var threw = false;
  try {
    action();
  } catch (error) {
    threw = true;
    assert(error instanceof other.TypeError, label + " defining realm");
    assert(!(error instanceof TypeError), label + " not entry realm");
    assert(error.message === expectedMessage, label + " message");
  }
  assert(threw, label + " did not throw");
}

expectOtherTypeError(
  function () { other.Math.sumPrecise([nonNumber]); },
  "Math.sumPrecise non-number element",
  "created-realm element",
);
expectOtherTypeError(
  function () { other.Math.sumPrecise({}); },
  "Math.sumPrecise input is not iterable",
  "created-realm non-iterable",
);

var badMethod = {};
badMethod[Symbol.iterator] = 0;
expectOtherTypeError(
  function () { other.Math.sumPrecise(badMethod); },
  "Math.sumPrecise input is not iterable",
  "created-realm iterator method",
);

var badIterator = {};
badIterator[Symbol.iterator] = function () {
  return 0;
};
expectOtherTypeError(
  function () { other.Math.sumPrecise(badIterator); },
  "Math.sumPrecise iterator method must return an object",
  "created-realm iterator result",
);

var badNext = {};
badNext[Symbol.iterator] = function () {
  return { next: 0 };
};
expectOtherTypeError(
  function () { other.Math.sumPrecise(badNext); },
  "Math.sumPrecise iterator next method is not callable",
  "created-realm next",
);

var badNextResult = {};
badNextResult[Symbol.iterator] = function () {
  return {
    next: function () {
      return 0;
    },
  };
};
expectOtherTypeError(
  function () { other.Math.sumPrecise(badNextResult); },
  "Math.sumPrecise iterator next result must be an object",
  "created-realm next result",
);

var otherStringNextCalls = 0;
other.String.prototype[other.Symbol.iterator] = function () {
  var done = false;
  return {
    next: function () {
      otherStringNextCalls += 1;
      if (done) {
        return { value: undefined, done: true };
      }
      done = true;
      return { value: 7, done: false };
    },
  };
};
assert(
  other.Math.sumPrecise("uses defining-realm String.prototype") === 7,
  "created-realm primitive iterator prototype",
);
assert(otherStringNextCalls === 2, "created-realm primitive iterator steps");

true;
