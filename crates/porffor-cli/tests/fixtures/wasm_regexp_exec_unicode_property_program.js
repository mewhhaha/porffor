function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function checkExec(regex, input, value, index, label) {
  var result = regex.exec(input);
  check(result !== null, true, label + " result");
  check(result[0], value, label + " value");
  check(result.index, index, label + " index");
  return result;
}

var text = "𠮷a𠮷b𠮷";
checkExec(/𠮷/, text, "𠮷", 0, "legacy astral literal");
checkExec(/𠮷/u, text, "𠮷", 0, "unicode astral literal");
checkExec(/𠮷/v, text, "𠮷", 0, "unicode-sets astral literal");
checkExec(/\p{Script=Han}/u, text, "𠮷", 0, "unicode Han");
checkExec(/\p{Script=Han}/v, text, "𠮷", 0, "unicode-sets Han");
checkExec(/./v, text, "𠮷", 0, "unicode-sets dot");
checkExec(/\p{ASCII}/u, text, "a", 2, "unicode ASCII");
checkExec(/\p{ASCII}/v, text, "a", 2, "unicode-sets ASCII");
check(/x/u.exec(text), null, "unicode no match");
check(/x/v.exec(text), null, "unicode-sets no match");

var groupsU = checkExec(/(\p{Script=Han})(.)/u, text, "𠮷a", 0, "unicode groups");
check(groupsU[1], "𠮷", "unicode group one");
check(groupsU[2], "a", "unicode group two");
var groupsV = checkExec(/(\p{Script=Han})(.)/v, text, "𠮷a", 0, "unicode-sets groups");
check(groupsV[1], "𠮷", "unicode-sets group one");
check(groupsV[2], "a", "unicode-sets group two");

var complex = "a\u{20BB7}b\u{10FFFF}c";
checkExec(/\P{ASCII}/u, complex, "\u{20BB7}", 1, "unicode non-ASCII");
checkExec(/\P{ASCII}/v, complex, "\u{20BB7}", 1, "unicode-sets non-ASCII");

checkExec(/\p{Script=Han}/u, "\u{2E80}", "\u{2E80}", 0, "Han first range start");
check(/\p{Script=Han}/u.exec("\u{2E9A}"), null, "Han first range gap");
checkExec(/\p{Script=Han}/u, "\u{33479}", "\u{33479}", 0, "Han final range end");
check(/\p{Script=Han}/u.exec("\u{3347A}"), null, "Han final range exclusion");
true;
