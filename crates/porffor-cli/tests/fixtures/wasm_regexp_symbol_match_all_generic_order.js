function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

var sequence = "";
var receiver = {
  get flags() {
    sequence = sequence + "f";
    return {
      toString: function () {
        sequence = sequence + "s";
        return "";
      }
    };
  },
  get [Symbol.match]() {
    sequence = sequence + "m";
    return false;
  }
};

RegExp.prototype[Symbol.matchAll].call(receiver, {
  toString: function () {
    sequence = sequence + "a";
  }
});

check(sequence, "afsm", "order");
true;
