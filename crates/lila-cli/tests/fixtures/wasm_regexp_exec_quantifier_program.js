function check(actual, expected, label) {
  if (actual !== expected) throw label;
}
function exec(re, input, value, index, label) {
  var result = re.exec(input);
  check(result !== null, true, label + " result");
  check(result[0], value, label + " value");
  check(result.index, index, label + " index");
}

exec(/a*ab/, "aaab", "aaab", 0, "star continuation backtracking");
exec(/a+/, "baaa", "aaa", 1, "greedy plus");
exec(/a+?a/, "aaaa", "aa", 0, "lazy plus");
exec(/a{2,4}a/, "aaaaa", "aaaaa", 0, "greedy bounded suffix");
exec(/a{2,4}?a/, "aaaaa", "aaa", 0, "lazy bounded suffix");
exec(/a{2,}/, "baaaa", "aaaa", 1, "open repeat");
exec(/[ab]?c/, "bc", "bc", 0, "optional class");
var legacyBraced = /a{b}/;
exec(legacyBraced, "xa{b}y", "a{b}", 1, "legacy nondecimal braced literal");
var legacyOpen = /a{/;
exec(legacyOpen, "za{x", "a{", 1, "legacy unclosed braced literal");
var legacyComma = /a{,2}/;
exec(legacyComma, "qa{,2}", "a{,2}", 1, "legacy comma braced literal");
var legacyClose = /a}/;
exec(legacyClose, "qa}r", "a}", 1, "legacy stray closing brace literal");

var global = /a{1,2}/g;
exec(global, "aaaa", "aa", 0, "global first");
check(global.lastIndex, 2, "global first lastIndex");
exec(global, "aaaa", "aa", 2, "global second");
check(global.lastIndex, 4, "global second lastIndex");
check(global.test("aaaa"), false, "global reset");
check(global.lastIndex, 0, "global reset lastIndex");
var sticky = /a+/y;
sticky.lastIndex = 1;
exec(sticky, "?aaa", "aaa", 1, "sticky");
sticky.lastIndex = 0;
check(sticky.test("?aaa"), false, "sticky reset");
check(sticky.lastIndex, 0, "sticky reset lastIndex");
exec(/a+/, "𝌆aaa", "aaa", 2, "astral UTF16 index");
var digitsGlobal = /\d+/g;
exec(digitsGlobal, "12x345", "12", 0, "digit global first");
check(digitsGlobal.lastIndex, 2, "digit global first lastIndex");
exec(digitsGlobal, "12x345", "345", 3, "digit global second");
check(digitsGlobal.lastIndex, 6, "digit global second lastIndex");
check(digitsGlobal.exec("12x345"), null, "digit global reset");
check(digitsGlobal.lastIndex, 0, "digit global reset lastIndex");
var digitsNonGlobal = /\d+/;
digitsNonGlobal.lastIndex = 2;
exec(digitsNonGlobal, "a12", "12", 1, "digit non-global first");
check(digitsNonGlobal.lastIndex, 2, "digit non-global first lastIndex");
exec(digitsNonGlobal, "a12", "12", 1, "digit non-global repeated");
check(digitsNonGlobal.lastIndex, 2, "digit non-global repeated lastIndex");
exec(/\d+/, "𝌆12", "12", 2, "digit astral UTF16 index");
exec(/\d/, "١2", "2", 1, "digit ASCII-only");

var lowStickyEmpty = /a*/y;
lowStickyEmpty.lastIndex = 1;
exec(lowStickyEmpty, "𝌆", "", 1, "low surrogate sticky empty");
check(lowStickyEmpty.lastIndex, 1, "low surrogate sticky empty lastIndex");
var lowGlobalEmpty = /a*/g;
lowGlobalEmpty.lastIndex = 1;
exec(lowGlobalEmpty, "𝌆", "", 1, "low surrogate global empty");
check(lowGlobalEmpty.lastIndex, 1, "low surrogate global empty lastIndex");
var lowStickyConsuming = /a/y;
lowStickyConsuming.lastIndex = 1;
check(lowStickyConsuming.test("𝌆a"), false, "low surrogate consuming does not borrow next scalar");
check(lowStickyConsuming.lastIndex, 0, "low surrogate consuming reset");

RegExp = function () { throw "literal construction"; };
var intrinsicInput = "aaa";
check(/a+/.exec(intrinsicInput)[0], "aaa", "intrinsic literal");
true;
