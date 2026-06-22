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

function checkThrowsConstructor(callback, constructor, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    threw = error instanceof constructor;
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

function ValueSentinel() {}
var closed = false;
var throwingValueIterator = {
  next: function () {
    return {
      done: false,
      get value() {
        throw new ValueSentinel();
      },
    };
  },
  return: function () {
    closed = true;
    return {};
  },
};
checkThrowsConstructor(function () {
  Iterator.prototype.toArray.call(throwingValueIterator);
}, ValueSentinel, "value getter throw");
check(closed, false, "value getter throw does not close iterator");

true;
