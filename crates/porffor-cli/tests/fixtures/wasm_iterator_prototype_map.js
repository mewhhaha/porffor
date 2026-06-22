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

var values = ["a", "b", "c"];
var index = 0;
var mapperCalls = 0;
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

var helper = iterator.map(function (value, count) {
  mapperCalls = mapperCalls + 1;
  return value + ":" + count;
});
assertSameValue(index, 0, "map creation is lazy");
assertSameValue(mapperCalls, 0, "mapper not called at creation");

var step = helper.next();
assertSameValue(step.done, false, "first done");
assertSameValue(step.value, "a:0", "first mapped value");
step = helper.next();
assertSameValue(step.done, false, "second done");
assertSameValue(step.value, "b:1", "second mapped value");
step = helper.next();
assertSameValue(step.done, false, "third done");
assertSameValue(step.value, "c:2", "third mapped value");
step = helper.next();
assertSameValue(step.done, true, "exhausted done");
assertSameValue(step.value, undefined, "exhausted value");
assertSameValue(returnCalls, 0, "ordinary exhaustion does not close");
assertSameValue(mapperCalls, 3, "mapper call count");

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
var returnHelper = returnIterator.map(function (value) {
  return value + 1;
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
  invalidIterator.map();
}, "missing mapper throws");
assertSameValue(invalidClosed, true, "missing mapper closes");

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
  objectIterator.map({});
}, "object mapper throws");
assertSameValue(objectClosed, true, "object mapper closes");

var badNextHelper = Iterator.prototype.map.call({ next: 0 }, function (value) {
  return value;
});
assertThrowsTypeError(function () {
  badNextHelper.next();
}, "bad next throws from helper");

function MapSentinel() {}
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
var throwingHelper = throwingIterator.map(function () {
  throw new MapSentinel();
});
var matchedMapThrow = false;
try {
  throwingHelper.next();
} catch (error) {
  matchedMapThrow = error instanceof MapSentinel;
}
assertSameValue(matchedMapThrow, true, "mapper throw is preserved");
assertSameValue(throwingReturnCalls, 1, "mapper throw closes");

var reentrantEnterCount = 0;
var reentrantMapperCount = 0;
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

reentrantHelper = reentrantSource.map(function () {
  reentrantMapperCount = reentrantMapperCount + 1;
  reentrantHelper.next();
});
assertThrowsTypeError(function () {
  reentrantHelper.next();
}, "reentrant helper next");
assertSameValue(reentrantEnterCount, 1, "reentrant enter count");
assertSameValue(reentrantMapperCount, 1, "reentrant mapper count");

true;
