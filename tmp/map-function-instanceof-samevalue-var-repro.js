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
  if (actual === false) throw "actual false";
  if (actual === undefined) throw "actual undefined";
  if (expected !== true) throw "expected not true";
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
var first = testResult[0];
var second = testResult[1];
if (first !== true) throw "first precheck";
if (second !== true) throw "second precheck";
assert.sameValue(first, true, "first");
assert.sameValue(second, true, "second");
