function check(condition, message) {
  if (!condition) throw message;
}

var closeCalls = 0;
var caughtOriginal;
var iterator = {
  next: function() { return { done: false, value: 1 }; },
  return: function() {
    closeCalls = closeCalls + 1;
    throw "close replacement";
  },
};
iterator[Symbol.iterator] = function() { return iterator; };

try {
  for (let value of iterator) {
    throw "body original";
  }
} catch (error) {
  caughtOriginal = error;
}

check(
  closeCalls === 1 && caughtOriginal === "body original",
  "IteratorClose preserves incoming throw",
);
true;
