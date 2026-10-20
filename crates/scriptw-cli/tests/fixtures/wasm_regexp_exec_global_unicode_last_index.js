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
checkExec(/𝌆/u.exec("x𝌆"), "𝌆", 1, "x𝌆", "unicode direct scalar");
checkExec(/\uD834\uDF06/u.exec("x𝌆"), "𝌆", 1, "x𝌆", "unicode escaped pair");
check(/\udf06/u.exec("\ud834\udf06"), null, "unicode search skips low surrogate");
checkExec(/\udf06/u.exec("x\udf06"), "\udf06", 1, "x\udf06", "unicode lone surrogate");
checkExec(/\udf06/.exec("\ud834\udf06"), "\udf06", 1, "\ud834\udf06", "non-unicode low code unit");

var unicodeStickyScalar = /𝌆/uy;
unicodeStickyScalar.lastIndex = 1;
checkExec(unicodeStickyScalar.exec("𝌆"), "𝌆", 0, "𝌆", "unicode sticky low start scalar");
check(unicodeStickyScalar.lastIndex, 2, "unicode sticky low start scalar lastIndex");
var unicodeStickyLow = /\udf06/uy;
unicodeStickyLow.lastIndex = 1;
check(unicodeStickyLow.exec("\ud834\udf06"), null, "unicode sticky low start rejects low");
check(unicodeStickyLow.lastIndex, 0, "unicode sticky low start reset");
var unicodeGlobalScalar = /𝌆/ug;
unicodeGlobalScalar.lastIndex = 1;
checkExec(unicodeGlobalScalar.exec("𝌆"), "𝌆", 0, "𝌆", "unicode global low start scalar");
check(unicodeGlobalScalar.lastIndex, 2, "unicode global low start scalar lastIndex");
var unicodeGlobalLow = /\udf06/ug;
unicodeGlobalLow.lastIndex = 1;
check(unicodeGlobalLow.exec("\ud834\udf06"), null, "unicode global low start rejects low");
check(unicodeGlobalLow.lastIndex, 0, "unicode global low start reset");

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
