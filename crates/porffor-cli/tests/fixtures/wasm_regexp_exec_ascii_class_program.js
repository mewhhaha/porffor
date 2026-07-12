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

checkExec(/t[a-b|q-s]/.exec(true), "tr", 0, "true", "class range coercion");
checkExec(/[a-f]d/.exec(undefined), "ed", 7, "undefined", "undefined class range");
checkExec(/[a-z]n/.exec(undefined), "un", 0, "undefined", "undefined class");
check(/a[|]b/.test("a|b"), true, "literal class pipe");

var nevermore = /[Nn]evermore/g;
checkExec(nevermore.exec("Nevermore nevermore"), "Nevermore", 0, "Nevermore nevermore", "global first");
check(nevermore.lastIndex, 9, "global first lastIndex");
checkExec(nevermore.exec("Nevermore nevermore"), "nevermore", 10, "Nevermore nevermore", "global second");
check(nevermore.lastIndex, 19, "global second lastIndex");
check(nevermore.exec("Nevermore nevermore"), null, "global terminal");
check(nevermore.lastIndex, 0, "global terminal reset");

var sticky = /[a-z]n/y;
sticky.lastIndex = 1;
checkExec(sticky.exec("?an"), "an", 1, "?an", "sticky success");
check(sticky.lastIndex, 3, "sticky success lastIndex");
sticky.lastIndex = 0;
check(sticky.test("?an"), false, "sticky failure");
check(sticky.lastIndex, 0, "sticky failure reset");

var nonGlobal = /[a-f]d/;
nonGlobal.lastIndex = 7;
checkExec(nonGlobal.exec("ad"), "ad", 0, "ad", "non-global result");
check(nonGlobal.lastIndex, 7, "non-global lastIndex unchanged");

var execCoercionError = {};
var execCoercion = /[a-f]d/g;
execCoercion.lastIndex = {
  valueOf: function () {
    throw execCoercionError;
  },
};
var caughtExecCoercion = false;
try {
  execCoercion.exec("ad");
} catch (error) {
  caughtExecCoercion = error === execCoercionError;
}
check(caughtExecCoercion, true, "exec lastIndex coercion throw");

var testCoercionError = {};
var testCoercion = /[a-f]d/g;
testCoercion.lastIndex = {
  valueOf: function () {
    throw testCoercionError;
  },
};
var caughtTestCoercion = false;
try {
  testCoercion.test("ad");
} catch (error) {
  caughtTestCoercion = error === testCoercionError;
}
check(caughtTestCoercion, true, "test lastIndex coercion throw");

var lockedSuccess = /[a-f]d/g;
Object.defineProperty(lockedSuccess, "lastIndex", { writable: false });
var caughtLockedSuccess = false;
try {
  lockedSuccess.exec("ad");
} catch (error) {
  caughtLockedSuccess = error instanceof TypeError;
}
check(caughtLockedSuccess, true, "non-writable lastIndex update");

var lockedFailure = /[a-f]d/g;
Object.defineProperty(lockedFailure, "lastIndex", { writable: false });
var caughtLockedFailure = false;
try {
  lockedFailure.test("zz");
} catch (error) {
  caughtLockedFailure = error instanceof TypeError;
}
check(caughtLockedFailure, true, "non-writable lastIndex reset");

var euro = /[a-f]d/g;
checkExec(euro.exec("€ad"), "ad", 1, "€ad", "euro index");
check(euro.lastIndex, 3, "euro lastIndex");
var astral = /[a-z]n/g;
checkExec(astral.exec("𝌆an"), "an", 2, "𝌆an", "astral index");
check(astral.lastIndex, 4, "astral lastIndex");

check(/plain/.test("a plain literal"), true, "plain literal program");
RegExp = function () {
  throw "literal construction used global RegExp";
};
check(/intrinsic/.test("intrinsic"), true, "literal remains intrinsic");

var literal = /prototype/;
var literalPrototype = Object.getPrototypeOf(literal);
check(literalPrototype !== Object.prototype, true, "literal prototype is RegExp.prototype");
check(Object.getPrototypeOf(literalPrototype), Object.prototype, "literal prototype chain");

true;
