function check(condition, message) {
  if (!condition) throw message;
}

var closeCalls = 0;
var iterator = {
  next: function() { return { done: false, value: 1 }; },
  return: function() {
    closeCalls = closeCalls + 1;
    return {};
  },
};
iterator[Symbol.iterator] = function() { return iterator; };

outer: for (let outerValue of [0]) {
  for (let value of iterator) {
    continue outer;
  }
}

check(closeCalls === 1, "outer labelled continue IteratorClose");
true;
