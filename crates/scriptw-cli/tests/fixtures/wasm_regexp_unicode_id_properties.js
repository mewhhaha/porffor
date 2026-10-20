// ECMA-262 spells identifier properties with capitals (`ID_Start`) while ICU
// registers them lowercase (`Id_Start`); both must compile to the same ranges.
function check(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

check(/\p{ID_Start}/u.test("A"), true, "ID_Start-letter");
check(/\p{ID_Start}/u.test("0"), false, "ID_Start-digit");
// U+005F is ID_Continue but not ID_Start (IdentifierStart adds `$` and `_`
// explicitly on top of the Unicode property).
check(/\p{ID_Start}/u.test("_"), false, "ID_Start-underscore");
check(/\p{ID_Start}/u.test("é"), true, "ID_Start-latin");
check(/\p{ID_Continue}/u.test("0"), true, "ID_Continue-digit");
check(/\p{ID_Continue}/u.test(" "), false, "ID_Continue-space");
check(/\p{IDS_Binary_Operator}/u.test("⿰"), true, "IDS_Binary-U2FF0");
check(/\p{IDS_Binary_Operator}/u.test("A"), false, "IDS_Binary-letter");
check(/\p{IDS_Trinary_Operator}/u.test("⿲"), true, "IDS_Trinary-U2FF2");
check(/\p{IDS_Trinary_Operator}/u.test("A"), false, "IDS_Trinary-letter");

true;
