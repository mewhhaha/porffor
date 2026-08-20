// A non-empty `v`-mode class is exactly one union, one homogeneous
// intersection chain, or one homogeneous subtraction chain.

function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function computedPatternThrowsSyntaxError(pattern, flags) {
  try {
    new RegExp(pattern, flags);
  } catch (error) {
    return error.name === "SyntaxError";
  }
  return false;
}

function computedPatternConstructs(pattern, flags) {
  try {
    new RegExp(pattern, flags);
    return true;
  } catch (_) {
    return false;
  }
}

// Static constructor arguments exercise `try_lower_static_regexp_compilation`.
var staticErrorName = "no-throw";
try {
  new RegExp("[a&&b--c]", "v");
} catch (error) {
  staticErrorName = error.name;
}
check(staticErrorName, "SyntaxError", "mixed static class-set operators");

staticErrorName = "no-throw";
try {
  new RegExp("[a&&-]", "v");
} catch (error) {
  staticErrorName = error.name;
}
check(staticErrorName, "SyntaxError", "raw static class-set syntax");

staticErrorName = "no-throw";
try {
  new RegExp("[\\q{a]", "v");
} catch (error) {
  staticErrorName = error.name;
}
check(staticErrorName, "SyntaxError", "unclosed static class string");

staticErrorName = "no-throw";
try {
  new RegExp("[\\q{a}&&]", "v");
} catch (error) {
  staticErrorName = error.name;
}
check(staticErrorName, "SyntaxError", "class string with missing outer operand");

staticErrorName = "no-throw";
try {
  new RegExp("[\\01]", "v");
} catch (error) {
  staticErrorName = error.name;
}
check(staticErrorName, "SyntaxError", "class-set zero escape decimal lookahead");

staticErrorName = "no-throw";
try {
  new RegExp("[\\q{a}])", "v");
} catch (error) {
  staticErrorName = error.name;
}
check(staticErrorName, "SyntaxError", "class string before stray parenthesis");

// Array element reads keep the arguments dynamic and exercise the emitted
// runtime pattern table. Before the shape parser was closed, every row below
// was accepted and installed as an ordinary range program.
var invalidPatterns = [
  "[a--b&&c]",
  "[ab&&c]",
  "[a&&bc]",
  "[&&a]",
  "[a&&]",
  "[--a]",
  "[a--]",
  "[a&&&b]",
  "[a&&&]",
  "[a&&-]",
  "[a---]",
  "[!!]",
  "[\\q]",
  "[\\q{a]",
  "[\\q{a!!b}]",
  "[\\q{a}",
  "[\\q{a}&&]",
  "[\\q{a}-b]",
  "[a-\\q{b}]",
  "[\\q{a}!!]",
  "[\\q{a}])",
  "[\\q{a}](",
  "[\\q{a}]\\k<missing>",
  "(?:[\\q{a}]|)*)",
  "(?:[\\q{a}]|)*(",
  "(?:[\\q{a}]|)*\\k<missing>",
  "[^\\q{ab}]",
  "[^\\q{}]",
  "[\\01]",
  "[\\q{\\01}]",
];
var flags = ["v"];
for (var index = 0; index < invalidPatterns.length; index += 1) {
  check(
    computedPatternThrowsSyntaxError(invalidPatterns[index], flags[0]),
    true,
    "computed invalid class-set expression " + index,
  );
}

// Legal siblings prevent a blanket rejection of every set operator from
// satisfying the negative half of the fixture.
var validPatterns = ["[[a-c]&&[b-d]&&[c-e]]", "[[a-c]--b--c]"];
var intersection = new RegExp(validPatterns[0], flags[0]);
check(intersection.test("c"), true, "homogeneous intersection accepts c");
check(intersection.test("b"), false, "homogeneous intersection rejects b");

var subtraction = new RegExp(validPatterns[1], flags[0]);
check(subtraction.test("a"), true, "homogeneous subtraction keeps a");
check(subtraction.test("b"), false, "homogeneous subtraction removes b");
check(subtraction.test("c"), false, "homogeneous subtraction removes c");

check(new RegExp("[abc]", "v").test("b"), true, "ordinary union remains valid");
check(new RegExp("[\\0a]", "v").test("a"), true, "valid zero escape remains live");

var escapedOperand = new RegExp("[a&&\\&]", "v");
check(escapedOperand.test("a"), false, "escaped ampersand intersection rejects a");
check(escapedOperand.test("&"), false, "escaped ampersand intersection rejects ampersand");

var escapedIntersection = new RegExp("[\\&&&\\&]", "v");
check(escapedIntersection.test("&"), true, "escaped ampersands remain operands");

// Legal class strings remain an explicit matcher capability gap, but their
// construction must not be relabelled as a SyntaxError or rejected outright.
var legalUnsupportedPatterns = [
  "[\\q{a|b}]",
  "[\\q{|a||b|}]",
  "[\\q{a}&&a]",
  "[^\\q{a}]",
  "[^\\q{ab}&&a]",
  "[^a--\\q{ab}]",
  "[\\q{\\0a}]",
  "[\\q{a}]b",
  "([\\q{a}])",
  "(?:[\\q{a}]|)*",
];
for (index = 0; index < legalUnsupportedPatterns.length; index += 1) {
  check(
    computedPatternConstructs(legalUnsupportedPatterns[index], flags[0]),
    true,
    "computed legal class string " + index,
  );
}

true;
