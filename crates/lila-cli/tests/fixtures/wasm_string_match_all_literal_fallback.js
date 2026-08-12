function check(actual, expected, label) {
  if (actual !== expected) {
    throw label;
  }
}

function checkMatch(match, value, index, input) {
  check(match[0], value, "match value");
  check(match.length, 1, "match length");
  check(match.index, index, "match index");
  check(match.input, input, "match input");
}

var commaMatches = Array.from("a,b,c".matchAll(","));
check(commaMatches.length, 2, "comma length");
checkMatch(commaMatches[0], ",", 1, "a,b,c");
checkMatch(commaMatches[1], ",", 3, "a,b,c");

var numberMatches = Array.from("a1b1c".matchAll(1));
check(numberMatches.length, 2, "number length");
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
check(overrideArg, "abc", "override arg");
check(calls, 2, "override calls");

var matchReads = 0;
var customResult = {};
var customMatcher = {
  get [Symbol.match]() {
    matchReads++;
    return false;
  },
  [Symbol.matchAll]: function() {
    return customResult;
  }
};
check("x".matchAll(customMatcher), customResult, "custom result");
check(matchReads, 1, "match reads");

true;
