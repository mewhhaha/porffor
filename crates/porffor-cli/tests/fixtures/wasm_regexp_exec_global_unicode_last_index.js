function check(actual, expected, label) {
  if (actual !== expected) {
    throw label;
  }
}

function checkExec(result, value, index, input, label) {
  check(result !== null, true, label + " result");
  check(result[0], value, label + " value");
  check(result.index, index, label + " index");
  check(result.input, input, label + " input");
}

var noMatch = /a/g;
var noMatchCoercions = 0;
noMatch.lastIndex = {
  valueOf: function() {
    noMatchCoercions++;
    return 42;
  }
};
check(noMatch.exec("abc"), null, "global out-of-range result");
check(noMatch.lastIndex, 0, "global out-of-range lastIndex");
check(noMatchCoercions, 1, "global out-of-range coercion");

var negative = {
  valueOf: function() {
    negativeCoercions++;
    return -1;
  }
};
var negativeCoercions = 0;
noMatch.lastIndex = negative;
check(noMatch.exec("nbc"), null, "global negative result");
check(noMatch.lastIndex, 0, "global negative lastIndex");
check(negativeCoercions, 1, "global negative coercion");

var globalDot = /./g;
var globalDotCoercions = 0;
globalDot.lastIndex = {
  valueOf: function() {
    globalDotCoercions++;
    return 0;
  }
};
checkExec(globalDot.exec("abc"), "a", 0, "abc", "global dot exec");
check(globalDot.lastIndex, 1, "global dot lastIndex");
check(globalDotCoercions, 1, "global dot coercion");

var nonGlobalDot = /./;
var nonGlobalLastIndex = {
  valueOf: function() {
    nonGlobalCoercions++;
    return 0;
  }
};
var nonGlobalCoercions = 0;
nonGlobalDot.lastIndex = nonGlobalLastIndex;
checkExec(nonGlobalDot.exec("abc"), "a", 0, "abc", "non-global dot exec");
check(nonGlobalDot.lastIndex, nonGlobalLastIndex, "non-global lastIndex object");
check(nonGlobalCoercions, 1, "non-global coercion");

var unicodeDot = /./ug;
checkExec(unicodeDot.exec("𝌆"), "𝌆", 0, "𝌆", "unicode dot exec");
check(unicodeDot.lastIndex, 2, "unicode dot lastIndex");

check(/a/g.test("a"), true, "global test");
check(/a/gy.test("ba"), false, "sticky global test");

var readonlyGlobal = /a/g;
Object.defineProperty(readonlyGlobal, "lastIndex", { writable: false });
var readonlyGlobalThrows = false;
try {
  readonlyGlobal.exec("a");
} catch (error) {
  readonlyGlobalThrows = error instanceof TypeError;
}
check(readonlyGlobalThrows, true, "readonly global TypeError");

var coercionThrowGlobal = /a/g;
coercionThrowGlobal.lastIndex = {
  valueOf: function() {
    throw 42;
  }
};
var coercionThrowCaught = false;
try {
  coercionThrowGlobal.exec("a");
} catch (error) {
  coercionThrowCaught = error === 42;
}
check(coercionThrowCaught, true, "global lastIndex coercion throw");

true;
