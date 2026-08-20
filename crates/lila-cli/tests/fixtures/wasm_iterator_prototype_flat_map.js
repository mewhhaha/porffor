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

function assertThrowsConstructor(callback, constructor, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    threw = error instanceof constructor;
  }
  if (!threw) {
    throw label;
  }
}

var values = [0, 1, 2, 3];
var index = 0;
var mapperCalls = 0;
var outerReturnCalls = 0;
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
    outerReturnCalls = outerReturnCalls + 1;
    return {};
  },
};

var helper = iterator.flatMap(function (value, count) {
  "use strict";
  assertSameValue(this, undefined, "mapper this");
  assertSameValue(count, value, "mapper index");
  mapperCalls = mapperCalls + 1;
  var result = [];
  var i = 0;
  while (i < value) {
    result.push(value);
    i = i + 1;
  }
  return result;
});
assertSameValue(index, 0, "flatMap creation is lazy");
assertSameValue(mapperCalls, 0, "mapper not called at creation");

var step = helper.next();
assertSameValue(step.done, false, "first done");
assertSameValue(step.value, 1, "first flattened value");
step = helper.next();
assertSameValue(step.done, false, "second done");
assertSameValue(step.value, 2, "second flattened value");
step = helper.next();
assertSameValue(step.done, false, "third done");
assertSameValue(step.value, 2, "third flattened value");
step = helper.next();
assertSameValue(step.done, false, "fourth done");
assertSameValue(step.value, 3, "fourth flattened value");
step = helper.next();
assertSameValue(step.done, false, "fifth done");
assertSameValue(step.value, 3, "fifth flattened value");
step = helper.next();
assertSameValue(step.done, false, "sixth done");
assertSameValue(step.value, 3, "sixth flattened value");
step = helper.next();
assertSameValue(step.done, true, "exhausted done");
assertSameValue(step.value, undefined, "exhausted value");
assertSameValue(outerReturnCalls, 0, "ordinary exhaustion does not close");
assertSameValue(mapperCalls, 4, "mapper call count");

var plainIndex = 0;
var plainSource = {
  __proto__: Iterator.prototype,
  next: function () {
    if (plainIndex > 1) {
      return { done: true, value: undefined };
    }
    var value = plainIndex;
    plainIndex = plainIndex + 1;
    return { done: false, value: value };
  },
};
var plainHelper = plainSource.flatMap(function (value) {
  var innerIndex = 0;
  return {
    next: function () {
      if (innerIndex >= value + 1) {
        return { done: true, value: undefined };
      }
      innerIndex = innerIndex + 1;
      return { done: false, value: value };
    },
  };
});
assertSameValue(plainHelper.next().value, 0, "plain first");
assertSameValue(plainHelper.next().value, 1, "plain second");
assertSameValue(plainHelper.next().value, 1, "plain third");
assertSameValue(plainHelper.next().done, true, "plain done");

var invalidReturnCalls = 0;
var invalidIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    return { done: false, value: 1 };
  },
  return: function () {
    invalidReturnCalls = invalidReturnCalls + 1;
    return {};
  },
};
var invalidHelper = invalidIterator.flatMap(function () {
  return 5;
});
assertThrowsTypeError(function () {
  invalidHelper.next();
}, "primitive mapper result");
assertSameValue(invalidReturnCalls, 1, "primitive closes outer");

function InnerNextSentinel() {}
var innerNextClosed = false;
var innerNextSource = {
  __proto__: Iterator.prototype,
  next: function () {
    return { done: false, value: 0 };
  },
  return: function () {
    innerNextClosed = true;
    return { done: true };
  },
};
var innerNextHelper = innerNextSource.flatMap(function () {
  return {
    next: function () {
      throw new InnerNextSentinel();
    },
  };
});
assertThrowsConstructor(function () {
  innerNextHelper.next();
}, InnerNextSentinel, "inner next throw is preserved");
assertSameValue(innerNextClosed, true, "inner next throw closes outer");

function InnerDoneSentinel() {}
var innerDoneClosed = false;
var innerDoneSource = {
  __proto__: Iterator.prototype,
  next: function () {
    return { done: false, value: 0 };
  },
  return: function () {
    innerDoneClosed = true;
    return { done: true };
  },
};
var innerDoneHelper = innerDoneSource.flatMap(function () {
  return {
    next: function () {
      return {
        get done() {
          throw new InnerDoneSentinel();
        },
      };
    },
  };
});
assertThrowsConstructor(function () {
  innerDoneHelper.next();
}, InnerDoneSentinel, "inner done throw is preserved");
assertSameValue(innerDoneClosed, true, "inner done throw closes outer");

function InnerValueSentinel() {}
var innerValueClosed = false;
var innerValueSource = {
  __proto__: Iterator.prototype,
  next: function () {
    return { done: false, value: 0 };
  },
  return: function () {
    innerValueClosed = true;
    return { done: true };
  },
};
var innerValueHelper = innerValueSource.flatMap(function () {
  return {
    next: function () {
      return {
        done: false,
        get value() {
          throw new InnerValueSentinel();
        },
      };
    },
  };
});
assertThrowsConstructor(function () {
  innerValueHelper.next();
}, InnerValueSentinel, "inner value throw is preserved");
assertSameValue(innerValueClosed, true, "inner value throw closes outer");

function MapperSentinel() {}
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
var throwingHelper = throwingIterator.flatMap(function () {
  throw new MapperSentinel();
});
var matchedMapperThrow = false;
try {
  throwingHelper.next();
} catch (error) {
  matchedMapperThrow = error instanceof MapperSentinel;
}
assertSameValue(matchedMapperThrow, true, "mapper throw is preserved");
assertSameValue(throwingReturnCalls, 1, "mapper throw closes outer");

var innerReturnCalls = 0;
var outerCloseCalls = 0;
var closeIterator = {
  __proto__: Iterator.prototype,
  next: function () {
    return { done: false, value: 1 };
  },
  return: function () {
    outerCloseCalls = outerCloseCalls + 1;
    return {};
  },
};
var closeHelper = closeIterator.flatMap(function () {
  return {
    next: function () {
      return { done: false, value: 9 };
    },
    return: function () {
      innerReturnCalls = innerReturnCalls + 1;
      return {};
    },
  };
});
step = closeHelper.next();
assertSameValue(step.value, 9, "inner yielded value");
closeHelper.return();
assertSameValue(innerReturnCalls, 1, "inner return forwarded");
assertSameValue(outerCloseCalls, 1, "outer return forwarded");
closeHelper.return();
assertSameValue(innerReturnCalls, 1, "inner return not repeated");
assertSameValue(outerCloseCalls, 1, "outer return not repeated");

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
reentrantHelper = reentrantSource.flatMap(function () {
  reentrantMapperCount = reentrantMapperCount + 1;
  reentrantHelper.next();
  return [1];
});
assertThrowsTypeError(function () {
  reentrantHelper.next();
}, "reentrant helper next");
assertSameValue(reentrantEnterCount, 1, "reentrant enter count");
assertSameValue(reentrantMapperCount, 1, "reentrant mapper count");

true;
