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

var input = "a*b";
var receiver = {
  toString: function () {
    return input;
  }
};
var iterator = /\w/g[Symbol.matchAll](receiver);
checkMatch(iterator.next(), "a", 0, input, "first");
checkMatch(iterator.next(), "b", 2, input, "second");

var finalResult = iterator.next();
check(finalResult.done, true, "final done");
check(finalResult.value, undefined, "final value");

true;
