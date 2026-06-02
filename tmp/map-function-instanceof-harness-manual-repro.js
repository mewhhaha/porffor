function Test262Error(message) {
  this.message = message || "";
}

function assert(mustBeTrue, message) {
  if (mustBeTrue === true) {
    return;
  }
  throw new Test262Error(message);
}

assert._isSameValue = function(a, b) {
  if (a === b) {
    return a !== 0 || 1 / a === 1 / b;
  }
  return a !== a && b !== b;
};

assert.sameValue = function(actual, expected, message) {
  if (assert._isSameValue(actual, expected)) {
    return;
  }
  throw new Test262Error(message);
};

function callbackfn(val, idx, obj) {
  return obj instanceof Function;
}

var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;

var testResult = Array.prototype.map.call(obj, callbackfn);
if (testResult[0] !== true) throw "testResult[0]";
if (testResult[1] !== true) throw "testResult[1]";
if (assert._isSameValue(testResult[0], true) !== true) throw "sameValue0";
if (assert._isSameValue(testResult[1], true) !== true) throw "sameValue1";
