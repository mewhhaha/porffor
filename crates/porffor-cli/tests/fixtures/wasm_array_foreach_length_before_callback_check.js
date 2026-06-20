function catchesTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error.name === "TypeError";
  }
  return false;
}

var firstAccessed = false;
var first = {
  0: 11,
  1: 12
};

Object.defineProperty(first, "length", {
  get: function() {
    firstAccessed = true;
    return 2;
  },
  configurable: true
});

var firstThrew = catchesTypeError(function() {
  Array.prototype.forEach.call(first, null);
});

var secondAccessed = false;
var second = {
  0: 11,
  1: 12
};

Object.defineProperty(second, "length", {
  get: function() {
    return {
      toString: function() {
        secondAccessed = true;
        return "2";
      }
    };
  },
  configurable: true
});

var secondThrew = catchesTypeError(function() {
  Array.prototype.forEach.call(second, null);
});

firstThrew && firstAccessed && secondThrew && secondAccessed;
