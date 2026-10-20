function check(value, label) {
  if (!value) {
    throw "String empty split UTF-16 fixture failed: " + label;
  }
}

function checkUnits(value, expected, label, limit) {
  var parts = limit === undefined ? value.split("") : value.split("", limit);
  check(parts.length === expected.length, label + " length");
  for (var i = 0; i < expected.length; i++) {
    check(parts[i] === expected[i], label + " unit " + i);
    check(parts[i] === value.charAt(i), label + " charAt parity " + i);
  }
  return parts;
}

var high = "\uD83D";
var low = "\uDCA9";
var pair = high + low;

checkUnits("", [], "empty source");

var literalParts = checkUnits("💩", [high, low], "astral literal");
check(literalParts.join("") === "💩", "astral literal join roundtrip");

var escapedParts = checkUnits("\uD83D\uDCA9", [high, low], "escaped pair");
check(escapedParts.join("") === pair, "escaped pair join roundtrip");

checkUnits("a💩b", ["a", high, low, "b"], "mixed BMP and astral");
checkUnits(high, [high], "lone high surrogate");
checkUnits(low, [low], "lone low surrogate");
checkUnits(low + high, [low, high], "reversed surrogates");

checkUnits(pair, [], "zero limit", 0);
checkUnits(pair, [high], "one-unit limit", 1);
checkUnits(pair, [high, low], "exact pair limit", 2);
checkUnits(pair, [high, low], "limit beyond pair", 3);

var boxed = new String(pair);
var boxedParts = boxed.split("");
check(boxedParts.length === 2, "boxed receiver length");
check(boxedParts[0] === high, "boxed receiver high surrogate");
check(boxedParts[1] === low, "boxed receiver low surrogate");

true;
