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

function returnFromIteratorLoop() {
  for (let value of iterator) {
    return "returned";
  }
}

check(returnFromIteratorLoop() === "returned" && closeCalls === 1, "return IteratorClose");
true;
