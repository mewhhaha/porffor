function callbackfn(val, idx, obj) {
  return obj instanceof Function;
}

function same(actual, expected, message) {
  if (actual !== true) throw "actual";
  if (expected !== true) throw "expected";
  if (actual === expected) return;
  throw message;
}

var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;

var testResult = Array.prototype.map.call(obj, callbackfn);
same(testResult[0], true, "testResult[0]");
same(testResult[1], true, "testResult[1]");
