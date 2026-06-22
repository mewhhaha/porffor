function assertSameValue(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

function assertThrowsTypeError(callback, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    threw = error instanceof TypeError;
  }
  if (!threw) {
    throw label;
  }
}

var values = [1, 0, 2, 0, 3, 0, 4];
var index = 0;
var predicateCalls = 0;
var returnCalls = 0;
var iterator = {
  __proto__: Iterator.prototype,
  next: function () {
    if (index >= values.length) {
      return { done: true, value: undefined };
    }
    var value = values[index];
    index = index + 1;
    return { done: false, value: value };
  },
  return: function () {
    returnCalls = returnCalls + 1;
    return {};
  },
};

var helper = iterator.filter(function (value, count) {
  "use strict";
  assertSameValue(this, undefined, "predicate this");
  predicateCalls = predicateCalls + 1;
  return value !== 0 && count < 6;
});
assertSameValue(index, 0, "filter creation is lazy");
assertSameValue(predicateCalls, 0, "predicate not called at creation");

var step = helper.next();
assertSameValue(step.done, false, "first done");
assertSameValue(step.value, 1, "first filtered value");
step = helper.next();
assertSameValue(step.done, false, "second done");
assertSameValue(step.value, 2, "second filtered value");
step = helper.next();
assertSameValue(step.done, false, "third done");
assertSameValue(step.value, 3, "third filtered value");
step = helper.next();
assertSameValue(step.done, true, "exhausted done");
assertSameValue(step.value, undefined, "exhausted value");
assertSameValue(returnCalls, 0, "ordinary exhaustion does not close");
assertSameValue(predicateCalls, 7, "predicate call count includes skipped values");

var truthyIndex = 0;
var truthyValues = [0, "", "keep"];
var truthyIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    if (truthyIndex >= truthyValues.length) {
      return { done: true, value: undefined };
    }
    var value = truthyValues[truthyIndex];
    truthyIndex = truthyIndex + 1;
    return { done: false, value: value };
  },
};
var truthyHelper = truthyIterator.filter(function (value) {
  return value;
});
step = truthyHelper.next();
assertSameValue(step.done, false, "truthy done");
assertSameValue(step.value, "keep", "truthy value");
assertSameValue(truthyIndex, 3, "falsy values skipped before yield");

var explicitReturnCalls = 0;
var returnIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    return { done: false, value: 1 };
  },
  return: function () {
    explicitReturnCalls = explicitReturnCalls + 1;
    return {};
  },
};
var returnHelper = returnIterator.filter(function () {
  return true;
});
step = returnHelper.return();
assertSameValue(step.done, true, "return done");
assertSameValue(step.value, undefined, "return value");
assertSameValue(explicitReturnCalls, 1, "return close");
step = returnHelper.next();
assertSameValue(step.done, true, "after return done");
assertSameValue(explicitReturnCalls, 1, "after return no extra close");

var invalidClosed = false;
var invalidIterator = {
  __proto__: Iterator.prototype,
  get next() {
    throw "next should not be read";
  },
  return: function () {
    invalidClosed = true;
    return {};
  },
};
assertThrowsTypeError(function () {
  invalidIterator.filter();
}, "missing predicate throws");
assertSameValue(invalidClosed, true, "missing predicate closes");

var objectClosed = false;
var objectIterator = {
  __proto__: Iterator.prototype,
  get next() {
    throw "next should not be read";
  },
  return: function () {
    objectClosed = true;
    return {};
  },
};
assertThrowsTypeError(function () {
  objectIterator.filter({});
}, "object predicate throws");
assertSameValue(objectClosed, true, "object predicate closes");

var badNextHelper = Iterator.prototype.filter.call({ next: 0 }, function () {
  return true;
});
assertThrowsTypeError(function () {
  badNextHelper.next();
}, "bad next throws from helper");

function PredicateSentinel() {}
function ReturnSentinel() {}
var throwingReturnCalls = 0;
var throwingIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    return { done: false, value: 1 };
  },
  return: function () {
    throwingReturnCalls = throwingReturnCalls + 1;
    throw new ReturnSentinel();
  },
};
var throwingHelper = throwingIterator.filter(function () {
  throw new PredicateSentinel();
});
var matchedPredicateThrow = false;
try {
  throwingHelper.next();
} catch (error) {
  matchedPredicateThrow = error instanceof PredicateSentinel;
}
assertSameValue(matchedPredicateThrow, true, "predicate throw is preserved");
assertSameValue(throwingReturnCalls, 1, "predicate throw closes");

var reentrantEnterCount = 0;
var reentrantPredicateCount = 0;
var reentrantHelper;
var reentrantSource = {
  __proto__: Iterator.prototype,
  next: function () {
    reentrantEnterCount = reentrantEnterCount + 1;
    return { done: false, value: undefined };
  },
  return: function () {
    return {};
  },
};

reentrantHelper = reentrantSource.filter(function () {
  reentrantPredicateCount = reentrantPredicateCount + 1;
  reentrantHelper.next();
  return true;
});
assertThrowsTypeError(function () {
  reentrantHelper.next();
}, "reentrant helper next");
assertSameValue(reentrantEnterCount, 1, "reentrant enter count");
assertSameValue(reentrantPredicateCount, 1, "reentrant predicate count");

true;
