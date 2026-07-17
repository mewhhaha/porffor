function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function checkPair(pair, start, end, label) {
  check(pair[0], start, label + " start");
  check(pair[1], end, label + " end");
}

var astral = "𝐁";

var codeUnit = astral.match(/./);
check(codeUnit.length, 1, "non-unicode length");
check(codeUnit[0], "\uD835", "non-unicode code unit");
check(codeUnit.index, 0, "non-unicode index");
check(codeUnit.input, astral, "non-unicode input");

var indexedCodeUnit = astral.match(/./d);
check(indexedCodeUnit[0], "\uD835", "indices code unit");
checkPair(indexedCodeUnit.indices[0], 0, 1, "indices full match");

var scalar = astral.match(/./u);
check(scalar.length, 1, "unicode length");
check(scalar[0], astral, "unicode scalar");

var sticky = /./y;
var stickyMatch = astral.match(sticky);
check(stickyMatch[0], "\uD835", "sticky code unit");
check(sticky.lastIndex, 1, "sticky lastIndex");

true;
