var ok = true;

var exhaustedState = { next: 0, closed: 0 };
var exhaustedIterable = {};
exhaustedIterable[Symbol.iterator] = function() {
  return {
    next: function() {
      exhaustedState.next++;
      if (exhaustedState.next === 1) return { value: undefined, done: false };
      if (exhaustedState.next === 2) return { value: null, done: false };
      if (exhaustedState.next === 3) return { value: 9, done: false };
      return { value: 100, done: true };
    },
    return: function() {
      exhaustedState.closed++;
      return {};
    }
  };
};
var first, second, rest;
var assignmentResult = [first = 4, second = 5, ...rest] = exhaustedIterable;
ok = ok && first === 4 && second === null && rest.length === 1 && rest[0] === 9;
ok = ok && assignmentResult === exhaustedIterable && exhaustedState.next === 4;
ok = ok && exhaustedState.closed === 0;

var closeState = { next: 0, closed: 0, thisCorrect: false, argumentCount: -1 };
var closeIterator = {
  next: function() {
    closeState.next++;
    return { value: 1, done: false };
  },
  return: function() {
    closeState.closed++;
    closeState.thisCorrect = this === closeIterator;
    closeState.argumentCount = arguments.length;
    return {};
  }
};
var closeIterable = {};
closeIterable[Symbol.iterator] = function() { return closeIterator; };
[] = closeIterable;
ok = ok && closeState.next === 0 && closeState.closed === 1;
ok = ok && closeState.thisCorrect && closeState.argumentCount === 0;

var ownIteratorGetterHits = 0;
var ownIteratorCalls = 0;
var ownIteratorArray = [10];
var ownArrayIterator = function() {
  ownIteratorCalls++;
  var done = false;
  return {
    next: function() {
      if (done) return { value: undefined, done: true };
      done = true;
      return { value: 11, done: false };
    },
    return: function() { return {}; }
  };
};
Object.defineProperty(ownIteratorArray, Symbol.iterator, {
  get: function() {
    ownIteratorGetterHits++;
    return ownArrayIterator;
  }
});
var overriddenValue;
[overriddenValue] = ownIteratorArray;
ok = ok && overriddenValue === 11;
ok = ok && ownIteratorGetterHits === 1 && ownIteratorCalls === 1;

var originalError = {};
var closeError = {};
var abruptCloseCount = 0;
var abruptIterator = {
  next: function() { return { value: undefined, done: false }; },
  return: function() {
    abruptCloseCount++;
    throw closeError;
  }
};
var abruptIterable = {};
abruptIterable[Symbol.iterator] = function() { return abruptIterator; };
function throwOriginalError() { throw originalError; }
var caughtDefaultError;
try {
  var [defaulted = throwOriginalError()] = abruptIterable;
} catch (error) {
  caughtDefaultError = error;
}
ok = ok && caughtDefaultError === originalError && abruptCloseCount === 1;

var caughtCloseError;
try {
  [] = abruptIterable;
} catch (error) {
  caughtCloseError = error;
}
ok = ok && caughtCloseError === closeError && abruptCloseCount === 2;

var order = [];
function orderedSource() {
  var iterator = {
    next: function() {
      order.push(5);
      return {
        get done() {
          order.push(6);
          return true;
        }
      };
    }
  };
  var iterable = {};
  iterable[Symbol.iterator] = function() {
    order.push(2);
    return iterator;
  };
  order.push(1);
  return iterable;
}
function orderedTarget() {
  order.push(3);
  return {
    set value(value) { order.push(8); }
  };
}
function orderedKey() {
  order.push(4);
  return {
    toString: function() {
      order.push(7);
      return "value";
    }
  };
}
([orderedTarget()[orderedKey()]] = orderedSource());
ok = ok && order.length === 8;
for (var orderIndex = 0; orderIndex < order.length; orderIndex += 1) {
  ok = ok && order[orderIndex] === orderIndex + 1;
}

var forOfClosures = [];
for (let [loopValue] of [[1], [2]]) {
  forOfClosures.push(function() { return loopValue; });
}
ok = ok && forOfClosures[0]() === 1 && forOfClosures[1]() === 2;

var forInClosures = [];
for (let [firstCharacter] in { i: 1 }) {
  forInClosures.push(function() { return firstCharacter; });
}
ok = ok && forInClosures[0]() === "i";

var assignedLoopValue = 0;
for ([assignedLoopValue] of [[7]]) {}
ok = ok && assignedLoopValue === 7;

let [stringFirst, ...stringRest] = "ab";
ok = ok && stringFirst === "a" && stringRest.length === 1 && stringRest[0] === "b";

let [[nestedValue = 13] = []] = [[]];
ok = ok && nestedValue === 13;

function makeArrayPatternAssigner() {
  let capturedValue = 0;
  return function(iterable) {
    [capturedValue] = iterable;
    return capturedValue;
  };
}
var assignCapturedValue = makeArrayPatternAssigner();
ok = ok && assignCapturedValue([12]) === 12;

var restTarget = {};
var restKey = "values";
[...restTarget[restKey]] = [14, 15];
ok = ok && restTarget.values.length === 2;
ok = ok && restTarget.values[0] === 14 && restTarget.values[1] === 15;

ok;
