var ok = true;

var preparedTargets = 0;
var nonCallableClosed = 0;
var targetObject = {};
function prepareTarget() {
  preparedTargets++;
  return targetObject;
}
var nonCallableIterable = {};
nonCallableIterable[Symbol.iterator] = function() {
  return {
    next: 0,
    return: function() {
      nonCallableClosed++;
      return {};
    }
  };
};
try {
  [prepareTarget().value] = nonCallableIterable;
  ok = false;
} catch (error) {
  ok = ok && error.name === "TypeError";
}
ok = ok && preparedTargets === 1 && nonCallableClosed === 0;

function assertStepFailureDoesNotClose(resultFactory, expectedError) {
  var closed = 0;
  var iterable = {};
  iterable[Symbol.iterator] = function() {
    return {
      next: resultFactory,
      return: function() {
        closed++;
        return {};
      }
    };
  };
  var caught;
  try {
    var [value] = iterable;
  } catch (error) {
    caught = error;
  }
  return caught === expectedError && closed === 0;
}

var nextError = {};
ok = ok && assertStepFailureDoesNotClose(function() { throw nextError; }, nextError);

var nonObjectResultErrorName;
var nonObjectResultClosed = 0;
var nonObjectResultIterable = {};
nonObjectResultIterable[Symbol.iterator] = function() {
  return {
    next: function() { return 1; },
    return: function() {
      nonObjectResultClosed++;
      return {};
    }
  };
};
try {
  var [nonObjectResultValue] = nonObjectResultIterable;
} catch (error) {
  nonObjectResultErrorName = error.name;
}
ok = ok && nonObjectResultErrorName === "TypeError" && nonObjectResultClosed === 0;

var doneError = {};
ok = ok && assertStepFailureDoesNotClose(function() {
  return {
    get done() { throw doneError; }
  };
}, doneError);

var valueError = {};
ok = ok && assertStepFailureDoesNotClose(function() {
  return {
    done: false,
    get value() { throw valueError; }
  };
}, valueError);

var elisionValueGets = 0;
var elisionClosed = 0;
var elisionIterable = {};
elisionIterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        done: false,
        get value() {
          elisionValueGets++;
          return 1;
        }
      };
    },
    return: function() {
      elisionClosed++;
      return {};
    }
  };
};
[,] = elisionIterable;
ok = ok && elisionValueGets === 0 && elisionClosed === 1;

const immutableElement = 0;
var immutableElementClosed = 0;
var immutableElementIterable = {};
immutableElementIterable[Symbol.iterator] = function() {
  return {
    next: function() { return { value: 1, done: false }; },
    return: function() {
      immutableElementClosed++;
      return {};
    }
  };
};
try {
  [immutableElement] = immutableElementIterable;
  ok = false;
} catch (error) {
  ok = ok && error.name === "TypeError";
}
ok = ok && immutableElementClosed === 1;

const immutableRest = [];
var immutableRestClosed = 0;
var immutableRestStep = 0;
var immutableRestIterable = {};
immutableRestIterable[Symbol.iterator] = function() {
  return {
    next: function() {
      immutableRestStep++;
      if (immutableRestStep === 1) return { value: 1, done: false };
      return { value: undefined, done: true };
    },
    return: function() {
      immutableRestClosed++;
      return {};
    }
  };
};
try {
  [...immutableRest] = immutableRestIterable;
  ok = false;
} catch (error) {
  ok = ok && error.name === "TypeError";
}
ok = ok && immutableRestClosed === 0 && immutableRestStep === 2;

ok;
