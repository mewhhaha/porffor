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

var values = ["a", "b", "c", "d"];
var index = 0;
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

var helper = iterator.take(2);
assertSameValue(index, 0, "take creation is lazy");

var directStep = iterator.next();
assertSameValue(directStep.done, false, "direct done");
assertSameValue(directStep.value, "a", "direct value");

var step = helper.next();
assertSameValue(step.done, false, "first taken done");
assertSameValue(step.value, "b", "first taken value");

step = helper.next();
assertSameValue(step.done, false, "second taken done");
assertSameValue(step.value, "c", "second taken value");

step = helper.next();
assertSameValue(step.done, true, "limit done");
assertSameValue(step.value, undefined, "limit value");
assertSameValue(returnCalls, 1, "limit close");

var shortIndex = 0;
var shortReturnCalls = 0;
var shortIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    shortIndex = shortIndex + 1;
    if (shortIndex >= 2) {
      return { done: true, value: undefined };
    }
    return { done: false, value: "only" };
  },
  return: function () {
    shortReturnCalls = shortReturnCalls + 1;
    return {};
  },
};
var shortHelper = shortIterator.take(5);
step = shortHelper.next();
assertSameValue(step.done, false, "short first done");
assertSameValue(step.value, "only", "short first value");
step = shortHelper.next();
assertSameValue(step.done, true, "short exhausted done");
assertSameValue(step.value, undefined, "short exhausted value");
assertSameValue(shortIndex, 2, "short next calls");
assertSameValue(shortReturnCalls, 0, "short exhaustion does not close");

var zeroIndex = 0;
var zeroReturnCalls = 0;
var zeroIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    zeroIndex = zeroIndex + 1;
    return { done: false, value: 1 };
  },
  return: function () {
    zeroReturnCalls = zeroReturnCalls + 1;
    return {};
  },
};
var zeroHelper = zeroIterator.take(0);
assertSameValue(zeroIndex, 0, "zero creation lazy");
step = zeroHelper.next();
assertSameValue(step.done, true, "zero done");
assertSameValue(step.value, undefined, "zero value");
assertSameValue(zeroIndex, 0, "zero did not advance");
assertSameValue(zeroReturnCalls, 1, "zero close");

var returnIndex = 0;
var explicitReturnCalls = 0;
var returnIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    returnIndex = returnIndex + 1;
    return { done: false, value: returnIndex };
  },
  return: function () {
    explicitReturnCalls = explicitReturnCalls + 1;
    return {};
  },
};
var returnHelper = returnIterator.take(3);
step = returnHelper.return();
assertSameValue(step.done, true, "return done");
assertSameValue(step.value, undefined, "return value");
assertSameValue(explicitReturnCalls, 1, "return close");
step = returnHelper.next();
assertSameValue(step.done, true, "after return done");
assertSameValue(explicitReturnCalls, 1, "after return no extra close");

var badNextHelper = Iterator.prototype.take.call({ next: 0 }, 1);
assertThrowsTypeError(function () {
  badNextHelper.next();
}, "bad next throws from helper");

var missingClosed = false;
var missingIterator = {
  __proto__: Iterator.prototype,
  get next() {
    throw "next should not be read";
  },
  return: function () {
    missingClosed = true;
    return {};
  },
};
var missingThrew = false;
try {
  missingIterator.take();
} catch (error) {
  missingThrew = error instanceof RangeError;
}
if (!missingThrew) {
  throw "missing limit throws";
}
assertSameValue(missingClosed, true, "missing limit closes");

var nanClosed = false;
var nanIterator = {
  __proto__: Iterator.prototype,
  get next() {
    throw "next should not be read";
  },
  return: function () {
    nanClosed = true;
    return {};
  },
};
var nanThrew = false;
try {
  nanIterator.take(NaN);
} catch (error) {
  nanThrew = error instanceof RangeError;
}
if (!nanThrew) {
  throw "nan limit throws";
}
assertSameValue(nanClosed, true, "nan limit closes");

var negativeClosed = false;
var negativeIterator = {
  __proto__: Iterator.prototype,
  get next() {
    throw "next should not be read";
  },
  return: function () {
    negativeClosed = true;
    return {};
  },
};
var negativeThrew = false;
try {
  negativeIterator.take(-1);
} catch (error) {
  negativeThrew = error instanceof RangeError;
}
if (!negativeThrew) {
  throw "negative limit throws";
}
assertSameValue(negativeClosed, true, "negative limit closes");

var valueOfClosed = false;
var valueOfIterator = {
  __proto__: Iterator.prototype,
  get next() {
    throw "next should not be read";
  },
  return: function () {
    valueOfClosed = true;
    return {};
  },
};
function TakeValueOfSentinel() {}
var valueOfThrew = false;
var valueOfMatched = false;
try {
  valueOfIterator.take({
    get valueOf() {
      throw new TakeValueOfSentinel();
    },
  });
} catch (error) {
  valueOfThrew = true;
  valueOfMatched = error instanceof TakeValueOfSentinel;
}
if (!valueOfThrew) {
  throw "valueOf limit throws";
}
if (!valueOfMatched) {
  throw "valueOf limit constructor";
}
assertSameValue(valueOfClosed, true, "valueOf limit closes");

var reentrantEnterCount = 0;
var reentrantHelper;
var reentrantSource = {
  __proto__: Iterator.prototype,
  next: function () {
    reentrantEnterCount = reentrantEnterCount + 1;
    reentrantHelper.next();
    return { done: false, value: undefined };
  },
};

reentrantHelper = reentrantSource.take(100);
assertThrowsTypeError(function () {
  reentrantHelper.next();
}, "reentrant helper next");
assertSameValue(reentrantEnterCount, 1, "reentrant enter count");

true;
