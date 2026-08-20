function check(condition, message) {
  if (!condition) throw message;
}

var closeCalls = 0;
var closeFinally = false;
var caughtClose;
var iterator = {
  next: function() { return { done: false, value: 1 }; },
  return: function() {
    closeCalls = closeCalls + 1;
    throw "close";
  },
};
iterator[Symbol.iterator] = function() { return iterator; };

try {
  try {
    for (let value of iterator) {
      break;
    }
  } finally {
    closeFinally = true;
  }
} catch (error) {
  caughtClose = error;
}

check(closeCalls === 1 && caughtClose === "close" && closeFinally, "IteratorClose outer routing");
true;
