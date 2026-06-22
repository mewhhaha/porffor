function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

var callCount = 0;
var callArg0;
var callArg1;
var regexp = /\d/u;
regexp.constructor = {
  [Symbol.species]: function () {
    callCount = callCount + 1;
    callArg0 = arguments[0];
    callArg1 = arguments[1];
    return /\w/g;
  }
};

var input = "a*b";
var iterator = regexp[Symbol.matchAll](input);
check(callCount, 1, "call count");
check(callArg0 === regexp, true, "arg0");
check(callArg1, "u", "arg1");

var first = iterator.next();
check(first.done, false, "first done");
check(first.value[0], "a", "first value");
check(first.value.index, 0, "first index");
check(first.value.input, input, "first input");

var second = iterator.next();
check(second.done, true, "second done");
true;
