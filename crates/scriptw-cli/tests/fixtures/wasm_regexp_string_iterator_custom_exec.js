function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

var regexp = /./g;
var input = "abc";
var iterator = regexp[Symbol.matchAll](input);
var callCount = 0;
var callArg0;
var first = ["ab"];

RegExp.prototype.exec = function () {
  callCount = callCount + 1;
  callArg0 = arguments[0];
  return callCount === 1 ? first : null;
};

var firstResult = iterator.next();
check(firstResult.done, false, "first done");
check(callCount, 1, "first call count");
check(firstResult.value[0], "ab", "first value");
check(callArg0, input, "call arg");

var secondResult = iterator.next();
check(secondResult.done, true, "second done");
check(secondResult.value, undefined, "second value");
check(callCount, 2, "call count");
true;
