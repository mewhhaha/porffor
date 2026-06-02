function assert() {}

assert._isSameValue = function(a, b) {
  if (a === b) {
    return a !== 0;
  }
  return false;
};

assert.sameValue = function(actual, expected) {
  if (assert._isSameValue(actual, expected)) return;
  throw "fail";
};

assert.sameValue(true, true);
