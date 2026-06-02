function assert() {}

assert.x = function(a, b) {
  return true;
};

assert.y = function(actual, expected, message) {
  return;
};

var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;

if (obj[0] !== 11) throw "read 0";
if (obj[1] !== 9) throw "read 1";
