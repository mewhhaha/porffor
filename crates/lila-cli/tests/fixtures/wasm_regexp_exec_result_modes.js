function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function checkMatchArray(result, value, index, input, label) {
  check(Array.isArray(result), true, label + " array");
  check(result[0], value, label + " value");
  check(result.index, index, label + " index");
  check(result.input, input, label + " input");
}

var symbolMatchInput = "ab";
var symbolMatch = symbolMatchInput.match(/a(b)?/);
checkMatchArray(symbolMatch, "ab", 0, symbolMatchInput, "symbol match program");
check(symbolMatch[1], "b", "symbol match capture");

var programInput = "zab";
var programExec = /a(b)?/g;
var programResult = programExec.exec(programInput);
checkMatchArray(programResult, "ab", 1, programInput, "exec program");
check(programResult[1], "b", "exec program capture");
check(programExec.lastIndex, 3, "exec program lastIndex");
check(programExec.exec(programInput), null, "exec program failure");
check(programExec.lastIndex, 0, "exec program failure lastIndex");

var programTest = /a(b)?/g;
check(programTest.test(programInput), true, "test program success");
check(programTest.lastIndex, 3, "test program lastIndex");
check(programTest.test("zzz"), false, "test program failure");
check(programTest.lastIndex, 0, "test program failure lastIndex");

var simpleSource = String.fromCharCode(113);
var simpleInput = String.fromCharCode(120, 113);
var simpleExec = new RegExp(simpleSource, "g");
var simpleResult = simpleExec.exec(simpleInput);
check(Array.isArray(simpleResult), true, "exec simple array");
check(simpleResult[0].charCodeAt(0), 113, "exec simple value");
check(simpleResult.index, 1, "exec simple index");
check(simpleResult.input, simpleInput, "exec simple input");
check(simpleExec.lastIndex, 2, "exec simple lastIndex");

var simpleTest = new RegExp(simpleSource, "g");
check(simpleTest.test(String.fromCharCode(120, 120)), false, "test simple failure");
check(simpleTest.lastIndex, 0, "test simple failure lastIndex");

var emptyParts = ["(?:", ")"];
var emptySource = emptyParts[0] + emptyParts[1];
var legacyExec = new RegExp(emptySource, "g");
var legacyResult = legacyExec.exec("x");
checkMatchArray(legacyResult, "", 0, "x", "exec legacy fallback");
check(legacyExec.lastIndex, 0, "exec legacy fallback lastIndex");

var legacyTest = new RegExp(emptySource, "g");
check(legacyTest.test("x"), true, "test legacy fallback");
check(legacyTest.lastIndex, 0, "test legacy fallback lastIndex");

262;
