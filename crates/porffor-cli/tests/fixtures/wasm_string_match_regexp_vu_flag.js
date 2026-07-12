var text = "𠮷a𠮷b𠮷c👨‍👩‍👧‍👦d";

function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function checkArray(actual, expected, label) {
  if (actual === null || actual === undefined) {
    throw label + ": missing result";
  }
  check(actual.length, expected.length, label + " length");
  for (var i = 0; i < expected.length; i++) {
    check(actual[i], expected[i], label + " item " + i);
  }
}

function doMatch(regex) {
  return RegExp.prototype[Symbol.match].call(regex, text);
}

checkArray(doMatch(/𠮷/g), ["𠮷", "𠮷", "𠮷"], "han global");
checkArray(doMatch(/𠮷/u), ["𠮷"], "han u");
checkArray(doMatch(/𠮷/v), ["𠮷"], "han v");

checkArray(doMatch(/\p{Script=Han}/gu), ["𠮷", "𠮷", "𠮷"], "property u");
checkArray(doMatch(/\p{Script=Han}/gv), ["𠮷", "𠮷", "𠮷"], "property v");

checkArray(
  doMatch(/./g),
  [
    "\uD842",
    "\uDFB7",
    "a",
    "\uD842",
    "\uDFB7",
    "b",
    "\uD842",
    "\uDFB7",
    "c",
    "\uD83D",
    "\uDC68",
    "\u200D",
    "\uD83D",
    "\uDC69",
    "\u200D",
    "\uD83D",
    "\uDC67",
    "\u200D",
    "\uD83D",
    "\uDC66",
    "d",
  ],
  "dot no unicode",
);

checkArray(
  doMatch(/./gu),
  ["𠮷", "a", "𠮷", "b", "𠮷", "c", "👨", "‍", "👩", "‍", "👧", "‍", "👦", "d"],
  "dot u",
);
checkArray(
  doMatch(/./gv),
  ["𠮷", "a", "𠮷", "b", "𠮷", "c", "👨", "‍", "👩", "‍", "👧", "‍", "👦", "d"],
  "dot v",
);

checkArray(
  doMatch(/.(.)./g),
  ["𠮷a", "𠮷b", "𠮷c", "👨‍", "👩‍", "👧‍", "👦d"],
  "dot sequence no unicode",
);
checkArray(
  doMatch(/.(.)./gu),
  ["𠮷a𠮷", "b𠮷c", "👨‍👩", "‍👧‍"],
  "dot sequence unicode",
);

checkArray(doMatch(/[👨‍👩‍👧‍👦]/v), ["👨"], "emoji v");
checkArray(doMatch(/[👨‍👩‍👧‍👦]/u), ["👨"], "emoji u");
check(doMatch(/x/u), null, "x u null");
check(doMatch(/x/v), null, "x v null");

true;
