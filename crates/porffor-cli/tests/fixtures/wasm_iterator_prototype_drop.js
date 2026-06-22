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

var helper = iterator.drop(2);
assertSameValue(index, 0, "drop creation is lazy");

var directStep = iterator.next();
assertSameValue(directStep.done, false, "direct done");
assertSameValue(directStep.value, "a", "direct value");

var step = helper.next();
assertSameValue(step.done, false, "first dropped-helper done");
assertSameValue(step.value, "d", "first dropped-helper value");
assertSameValue(returnCalls, 0, "drop does not close after skip");

step = helper.next();
assertSameValue(step.done, true, "drop exhausted done");
assertSameValue(step.value, undefined, "drop exhausted value");
assertSameValue(returnCalls, 0, "drop exhaustion no return");

var zeroIndex = 0;
var zeroIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    zeroIndex = zeroIndex + 1;
    return { done: false, value: zeroIndex };
  },
  return: function () {
    throw "zero return should not run";
  },
};
step = zeroIterator.drop(0).next();
assertSameValue(step.done, false, "zero done");
assertSameValue(step.value, 1, "zero value");

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
var returnHelper = returnIterator.drop(3);
step = returnHelper.return();
assertSameValue(step.done, true, "return done");
assertSameValue(step.value, undefined, "return value");
assertSameValue(explicitReturnCalls, 1, "return close");
step = returnHelper.next();
assertSameValue(step.done, true, "after return done");
assertSameValue(explicitReturnCalls, 1, "after return no extra close");

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
  missingIterator.drop();
} catch (error) {
  missingThrew = error instanceof RangeError;
}
if (!missingThrew) {
  throw "missing limit throws";
}
assertSameValue(missingClosed, true, "missing limit closes");

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
function DropValueOfSentinel() {}
var valueOfThrew = false;
var valueOfMatched = false;
try {
  valueOfIterator.drop({
    get valueOf() {
      throw new DropValueOfSentinel();
    },
  });
} catch (error) {
  valueOfThrew = true;
  valueOfMatched = error instanceof DropValueOfSentinel;
}
if (!valueOfThrew) {
  throw "valueOf limit throws";
}
if (!valueOfMatched) {
  throw "valueOf limit constructor";
}
assertSameValue(valueOfClosed, true, "valueOf limit closes");

var badNextHelper = Iterator.prototype.drop.call({ next: 0 }, 1);
assertThrowsTypeError(function () {
  badNextHelper.next();
}, "bad next throws from helper");

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

reentrantHelper = reentrantSource.drop(100);
assertThrowsTypeError(function () {
  reentrantHelper.next();
}, "reentrant helper next");
assertSameValue(reentrantEnterCount, 1, "reentrant enter count");

true;
