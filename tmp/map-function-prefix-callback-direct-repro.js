function assert() {}

assert.x = function(a, b) {
  return true;
};

assert.y = function(actual, expected, message) {
  return;
};

function callbackfn(val, idx, obj) {
  return obj instanceof Function;
}

var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;

if (callbackfn(obj[0], 0, obj) !== true) throw "callback 0";
if (callbackfn(obj[1], 1, obj) !== true) throw "callback 1";
