function check(condition, message) {
  if (!condition) throw message;
}

function makeIterator(first, second, closeState) {
  var position = 0;
  var iterator = {
    next: function() {
      position += 1;
      if (position === 1) return { done: false, value: first };
      if (position === 2) return { done: false, value: second };
      return { done: true };
    },
    return: function() {
      closeState.calls = closeState.calls + 1;
      return {};
    },
  };
  iterator[Symbol.iterator] = function() { return iterator; };
  return iterator;
}

var firstClose = { calls: 0 };
for (let value of makeIterator(1, 2, firstClose)) {}
var secondClose = { calls: 0 };
var caughtOriginal;
try {
  for (let value of makeIterator(1, 2, secondClose)) {
    throw 37;
  }
} catch (error) {
  caughtOriginal = error;
}

check(
  firstClose.calls === 0 && secondClose.calls === 1 && caughtOriginal === 37,
  "sequential iterator factories",
);
true;
