var nextCount = 0;
var returnCount = 0;
var thrown = {};
var target = {};

Object.defineProperty(target, "poisoned", {
  set: function(value) {
    throw thrown;
  }
});

var iterable = {};
iterable[Symbol.iterator] = function() {
  return {
    next: function() {
      nextCount++;
      return { done: true };
    },
    return: function() {
      returnCount++;
      return {};
    }
  };
};

var caught;
try {
  [...target.poisoned] = iterable;
} catch (error) {
  caught = error;
}

caught === thrown && nextCount === 1 && returnCount === 0;
