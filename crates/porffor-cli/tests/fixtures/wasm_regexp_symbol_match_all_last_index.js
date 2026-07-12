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

var regexp = /./g;
var originalLastIndex = {
  valueOf: function () {
    return 2;
  }
};
regexp.lastIndex = originalLastIndex;
var input = "abcd";
var iterator = regexp[Symbol.matchAll](input);
check(regexp.lastIndex === originalLastIndex, true, "original lastIndex preserved");
regexp.lastIndex = 0;

checkMatch(iterator.next(), "c", 2, input, "first");
checkMatch(iterator.next(), "d", 3, input, "second");
check(iterator.next().done, true, "final done");

true;
