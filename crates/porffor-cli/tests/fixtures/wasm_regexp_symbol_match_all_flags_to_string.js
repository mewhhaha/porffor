function check(actual, expected, label) {
  if (actual !== expected) {
    throw label;
  }
}

function checkMatch(result, value, index, input, label) {
  check(result.done, false, label + " done");
  check(result.value[0], value, label + " value");
  check(result.value.index, index, label + " index");
  check(result.value.input, input, label + " input");
}

var regexp = /\w/;
Object.defineProperty(regexp, "flags", {
  value: {
    toString: function () {
      return "g";
    }
  }
});

var input = "a*b";
var iterator = regexp[Symbol.matchAll](input);
checkMatch(iterator.next(), "a", 0, input, "first");
checkMatch(iterator.next(), "b", 2, input, "second");
check(iterator.next().done, true, "final done");

true;
