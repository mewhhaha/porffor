function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

function checkThrowsTypeError(callback, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    threw = error instanceof TypeError;
  }
  check(threw, true, label);
}

checkThrowsTypeError(function () {
  Iterator.prototype.toArray.call(0);
}, "primitive receiver");

checkThrowsTypeError(function () {
  Iterator.prototype.toArray.call({ next: 0 });
}, "non-callable next");

var i = 0;
var iterator = {
  next: function () {
    i = i + 1;
    if (i < 3) {
      return { value: i, done: false };
    }
    return { done: true };
  },
};

var values = Iterator.prototype.toArray.call(iterator);
check(values.length, 2, "length");
check(values[0], 1, "first");
check(values[1], 2, "second");

true;
