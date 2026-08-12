function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

var stringMatch = "102030".match(/0./);
check(stringMatch[0], "02", "string zero-any match");
check(stringMatch.length, 1, "string zero-any length");
check(stringMatch.index, 1, "string zero-any index");
check(stringMatch.input, "102030", "string zero-any input");

var n = 10203040506070809000;
check(String(n), "10203040506070809000", "large integer decimal string");

Number.prototype.match = String.prototype.match;
var numberMatch = n.match(/0./);
check(numberMatch[0], "02", "number borrowed match");
check(numberMatch.length, 1, "number borrowed length");
check(numberMatch.index, 1, "number borrowed index");
check(numberMatch.input, String(n), "number borrowed input");

var re = /0./;
re.lastIndex = 0;
var numberMatchWithLastIndex = n.match(re);
check(numberMatchWithLastIndex[0], "02", "number borrowed lastIndex match");
check(numberMatchWithLastIndex.index, 1, "number borrowed lastIndex index");

true;
