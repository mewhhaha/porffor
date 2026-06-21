function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function checkMatch(match, value, index, input, label) {
  check(match[0], value, label + " value");
  check(match.length, 1, label + " length");
  check(match.index, index, label + " index");
  check(match.input, input, label + " input");
}

var commaMatches = Array.from("a,b,c".matchAll(","));
check(commaMatches.length, 2, "comma match count");
checkMatch(commaMatches[0], ",", 1, "a,b,c", "comma first");
checkMatch(commaMatches[1], ",", 3, "a,b,c", "comma second");

var numberMatches = Array.from("a1b1c".matchAll(1));
check(numberMatches.length, 2, "number match count");
checkMatch(numberMatches[0], "1", 1, "a1b1c", "number first");
checkMatch(numberMatches[1], "1", 3, "a1b1c", "number second");

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
check(overrideArg, "abc", "override arg");
check(calls, 2, "override calls");

true;
