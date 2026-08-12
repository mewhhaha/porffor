function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

var ascii = RegExp("(?<word>a)");
check(ascii.test("a"), true, "ascii test");
check(ascii.exec("a").groups.word, "a", "ascii group");

var unicode = RegExp("(?<π>a)", "u");
check(unicode.test("a"), true, "unicode test");
check(unicode.exec("a").groups.π, "a", "unicode group");

var globalAlias = globalThis;
var originalRegExp = RegExp;
globalAlias.RegExp = function () { return /b/; };
var replacedCall = RegExp("a");
check(replacedCall.source, "b", "replaced call source");
check(replacedCall.exec("a"), null, "replaced call program");

globalAlias.RegExp = function () { return /c/; };
var replacedConstruct = new RegExp("a");
check(replacedConstruct.source, "c", "replaced construct source");
check(replacedConstruct.exec("a"), null, "replaced construct program");
globalAlias.RegExp = originalRegExp;

true;
