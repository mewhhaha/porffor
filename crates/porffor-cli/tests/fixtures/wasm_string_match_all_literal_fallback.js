function check(actual, expected) {
  if (actual !== expected) {
    throw "check failed";
  }
}

function checkMatch(match, value, index, input) {
  check(match[0], value);
  check(match.length, 1);
  check(match.index, index);
  check(match.input, input);
}

var commaMatches = Array.from("a,b,c".matchAll(","));
check(commaMatches.length, 2);
checkMatch(commaMatches[0], ",", 1, "a,b,c");
checkMatch(commaMatches[1], ",", 3, "a,b,c");

var numberMatches = Array.from("a1b1c".matchAll(1));
check(numberMatches.length, 2);
checkMatch(numberMatches[0], "1", 1, "a1b1c");
checkMatch(numberMatches[1], "1", 3, "a1b1c");

var calls = 0;
var overrideArg;
RegExp.prototype[Symbol.matchAll] = function(string) {
  calls++;
  overrideArg = string;
};

var receiver = {
  [Symbol.toPrimitive]: function() {
    calls++;
    return "abc";
  }
};
String.prototype.matchAll.call(receiver, null);
check(overrideArg, "abc");
check(calls, 2);

true;
