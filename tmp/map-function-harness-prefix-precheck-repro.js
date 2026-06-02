function assert() {}

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
  throw message;
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
