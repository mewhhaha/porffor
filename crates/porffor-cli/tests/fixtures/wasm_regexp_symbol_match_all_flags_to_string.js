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
var flagsGetterCalls = 0;
var flagsToStringCalls = 0;
Object.defineProperty(regexp, "flags", {
  get: function () {
    flagsGetterCalls = flagsGetterCalls + 1;
    return {
      toString: function () {
        flagsToStringCalls = flagsToStringCalls + 1;
        return "g";
      }
    };
  }
});

var input = "a*b";
var iterator = RegExp.prototype[Symbol.matchAll].call(regexp, input);
check(flagsGetterCalls, 1, "flags getter count");
check(flagsToStringCalls, 1, "flags toString count");
checkMatch(iterator.next(), "a", 0, input, "first");
checkMatch(iterator.next(), "b", 2, input, "second");
check(iterator.next().done, true, "final done");

true;
