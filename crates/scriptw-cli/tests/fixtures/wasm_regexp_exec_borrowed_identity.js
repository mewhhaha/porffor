function check(actual, expected, label) {
  if (actual !== expected) {
    throw label;
  }
}

var builtinExec = RegExp.prototype.exec;
Object.prototype.exec = builtinExec;

var initialVariable = /x/;
var initialVariableMatch;
try {
  initialVariableMatch = initialVariable.exec("x");
} catch (error) {
  throw "initial variable exec threw TypeError: " + (error instanceof TypeError);
}
check(initialVariableMatch[0], "x", "initial variable exec matches");
var initialLiteralMatch;
try {
  initialLiteralMatch = (/x/).exec("x");
} catch (error) {
  throw "initial literal exec threw TypeError: " + (error instanceof TypeError);
}
check(initialLiteralMatch[0], "x", "initial literal exec matches");

var orderedAlternativeMatch = /ll|l/.exec("null");
check(orderedAlternativeMatch[0], "ll", "exec prefers the first matching alternative");
check(orderedAlternativeMatch.index, 2, "ordered alternative match index");
var secondAlternativeMatch = /no|yes/.exec("yes");
check(secondAlternativeMatch[0], "yes", "exec tries the second alternative");
var shorterAlternativeMatch = /long|x/.exec("x");
check(shorterAlternativeMatch[0], "x", "exec tries a shorter second alternative");
var globalAlternative = /a|b/g;
check(globalAlternative.exec("ba")[0], "b", "global exec matches the second alternative");
check(globalAlternative.lastIndex, 1, "global alternative updates lastIndex");
check(globalAlternative.exec("ba")[0], "a", "global exec resumes at lastIndex");
check(globalAlternative.lastIndex, 2, "global alternative advances lastIndex again");
var stickyAlternative = /a|b/y;
stickyAlternative.lastIndex = 1;
var stickyAlternativeMatch = stickyAlternative.exec("xb");
check(stickyAlternativeMatch[0], "b", "sticky exec matches at lastIndex");
check(stickyAlternativeMatch.index, 1, "sticky alternative match index");
check(stickyAlternative.lastIndex, 2, "sticky alternative updates lastIndex");

var escapedDotMatch = /\.14/.exec({ toString: function () { return Math.PI; } });
if (escapedDotMatch === null) throw "escaped dot did not match";
check(escapedDotMatch[0], ".14", "escaped dot matches a literal dot");
check(escapedDotMatch.index, 1, "escaped dot match index");
check(escapedDotMatch.input, String(Math.PI), "escaped dot match input");
var escapedPipeMatch = /a\|b/.exec("a|b");
if (escapedPipeMatch === null) throw "escaped pipe did not match";
check(escapedPipeMatch[0], "a|b", "escaped pipe matches a literal pipe");
check(escapedPipeMatch.index, 0, "escaped pipe exact-length match index");

var ignoreCaseMatch = /LS/i.exec({ toString: function () { return false; } });
if (ignoreCaseMatch === null) throw "ignoreCase literal did not match";
check(ignoreCaseMatch[0], "ls", "ignoreCase preserves the input match case");
check(ignoreCaseMatch.index, 2, "ignoreCase literal match index");
check(ignoreCaseMatch.input, "false", "ignoreCase literal match input");
var ignoreCaseAlternativeMatch = /foo|bar/i.exec("BAR");
check(ignoreCaseAlternativeMatch[0], "BAR", "ignoreCase tries the second alternative");
var ignoreCaseGlobal = /ls/ig;
check(ignoreCaseGlobal.exec("xxLS")[0], "LS", "ignoreCase global matches input case");
check(ignoreCaseGlobal.lastIndex, 4, "ignoreCase global updates lastIndex");

function borrowedExecThrowsBoolean() {
  var threw = false;
  try {
    false.exec("x");
  } catch (error) {
    threw = error instanceof TypeError;
  }
  return threw;
}
function borrowedExecThrowsString() {
  var threw = false;
  try {
    "x".exec("x");
  } catch (error) {
    threw = error instanceof TypeError;
  }
  return threw;
}
function borrowedExecThrowsNumber() {
  var threw = false;
  try {
    1..exec("x");
  } catch (error) {
    threw = error instanceof TypeError;
  }
  return threw;
}
check(borrowedExecThrowsBoolean(), true, "borrowed exec rejects boolean receiver");
check(borrowedExecThrowsString(), true, "borrowed exec rejects string receiver");
check(borrowedExecThrowsNumber(), true, "borrowed exec rejects number receiver");

var toStringCalls = 0;
var argumentWithThrowingToString = {
  toString: function () {
    toStringCalls = toStringCalls + 1;
    throw "input toString should not run";
  }
};
var incompatibleReceiverThrows = false;
try {
  ({}).exec(argumentWithThrowingToString);
} catch (error) {
  incompatibleReceiverThrows = error instanceof TypeError;
}
check(incompatibleReceiverThrows, true, "exec validates receiver before input coercion");
check(toStringCalls, 0, "incompatible exec receiver does not coerce input");

var literalExecCalls = 0;
RegExp.prototype.exec = function (input) {
  literalExecCalls = literalExecCalls + 1;
  return "custom " + input;
};
var literalExecResult;
try {
  literalExecResult = (/x/).exec("ok");
} catch (error) {
  throw "literal override threw TypeError: " + (error instanceof TypeError);
}
check(literalExecResult, "custom ok", "literal exec observes prototype override");
check(literalExecCalls, 1, "literal exec invokes prototype override once");
RegExp.prototype.exec = builtinExec;

Object.prototype.exec = function (value) {
  return value + "!";
};
check(({}).exec("x"), "x!", "custom Object.prototype.exec is callable");

var match;
try {
  match = (/x/).exec("x");
} catch (error) {
  throw "restored literal exec threw TypeError: " + (error instanceof TypeError);
}
check(match[0], "x", "RegExp.prototype.exec still matches");
true;
