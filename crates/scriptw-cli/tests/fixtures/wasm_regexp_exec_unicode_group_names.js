function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function checkPair(pair, start, end, label) {
  check(pair[0], start, label + " start");
  check(pair[1], end, label + " end");
}

var mixed = /(?<π>a)(?<ಠ_ಠ>b)(?<$𐒤>c)\k<\u03C0>\k<ಠ_ಠ>\k<$\u{104A4}>/du.exec("abcabc");
check(mixed[0], "abcabc", "mixed full");
check(mixed.groups.π, "a", "mixed pi");
check(mixed.groups.ಠ_ಠ, "b", "mixed kannada");
check(mixed.groups.$𐒤, "c", "mixed astral continuation");
checkPair(mixed.indices.groups.π, 0, 1, "mixed pi indices");
checkPair(mixed.indices.groups.ಠ_ಠ, 1, 2, "mixed kannada indices");
checkPair(mixed.indices.groups.$𐒤, 2, 3, "mixed astral indices");

var escaped = /(?<\u{03C0}>a)\k<π>/u.exec("aa");
check(escaped[0], "aa", "escaped full");
check(escaped.groups.π, "a", "escaped canonical group");

var fixedAstral = /(?<a\uD801\uDCA4>b)\k<a\u{104A4}>/du.exec("bb");
check(fixedAstral[0], "bb", "fixed astral full");
check(fixedAstral.groups.a𐒤, "b", "fixed astral group");
checkPair(fixedAstral.indices.groups.a𐒤, 0, 1, "fixed astral indices");

var joiners = /(?<_\u200C>a)(?<_\u200D>b)\k<_\u200C>\k<_\u200D>/du.exec("abab");
check(joiners[0], "abab", "joiners full");
check(joiners.groups._\u200C, "a", "zwnj group");
check(joiners.groups._\u200D, "b", "zwj group");
checkPair(joiners.indices.groups._\u200C, 0, 1, "zwnj indices");
checkPair(joiners.indices.groups._\u200D, 1, 2, "zwj indices");

var nonUnicode = /(?<\u{03C0}>a)\k<π>/.exec("aa");
check(nonUnicode[0], "aa", "non-unicode braced full");
check(nonUnicode.groups.π, "a", "non-unicode braced group");

var duplicate = /(?:(?<π>a)|(?<\u03C0>b))\k<\u{03C0}>/du.exec("bb");
check(duplicate[0], "bb", "duplicate canonical full");
check(duplicate.groups.π, "b", "duplicate canonical group");
checkPair(duplicate.indices.groups.π, 0, 1, "duplicate canonical indices");

true;
