var text = "𠮷a𠮷b𠮷";

function compareArrays(actual, expected, label) {
  if (actual.length !== expected.length) {
    throw label + " length: " + actual.length + " !== " + expected.length;
  }
  for (var i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) {
      throw label + " item " + i + ": " + actual[i] + " !== " + expected[i];
    }
  }
  return undefined;
}

function checkComparison(actual, expected, label) {
  if (compareArrays(actual, expected, label) !== undefined) {
    throw label + " comparison result";
  }
}

function matchValuesAndIndices(regex) {
  var result = Array.from(
    RegExp.prototype[Symbol.matchAll].call(regex, text),
  );
  var matches = result.map(function(match) {
    return match[0];
  });
  var indices = result.map(function(match) {
    return match.index;
  });
  return matches.concat(indices);
}

checkComparison(
  matchValuesAndIndices(/𠮷/g),
  ["𠮷", "𠮷", "𠮷", 0, 3, 6],
  "literal global",
);
checkComparison(
  matchValuesAndIndices(/𠮷/gu),
  ["𠮷", "𠮷", "𠮷", 0, 3, 6],
  "literal u",
);
checkComparison(
  matchValuesAndIndices(/𠮷/gv),
  ["𠮷", "𠮷", "𠮷", 0, 3, 6],
  "literal v",
);
checkComparison(
  matchValuesAndIndices(/\p{Script=Han}/gu),
  ["𠮷", "𠮷", "𠮷", 0, 3, 6],
  "property u",
);
checkComparison(
  matchValuesAndIndices(/\p{Script=Han}/gv),
  ["𠮷", "𠮷", "𠮷", 0, 3, 6],
  "property v",
);
checkComparison(
  matchValuesAndIndices(/./gu),
  ["𠮷", "a", "𠮷", "b", "𠮷", 0, 2, 3, 5, 6],
  "dot u",
);
checkComparison(
  matchValuesAndIndices(/./gv),
  ["𠮷", "a", "𠮷", "b", "𠮷", 0, 2, 3, 5, 6],
  "dot v",
);

if (matchValuesAndIndices(/(?:)/gu).length !== 12) {
  throw "empty u";
}
if (matchValuesAndIndices(/(?:)/gv).length !== 12) {
  throw "empty v";
}

var complexText = "a\u{20BB7}b\u{10FFFF}c";
checkComparison(
  Array.from(complexText.matchAll(/\P{ASCII}/gu), function(match) {
    return match[0];
  }),
  ["\u{20BB7}", "\u{10FFFF}"],
  "non-ASCII u",
);
checkComparison(
  Array.from(complexText.matchAll(/\P{ASCII}/gv), function(match) {
    return match[0];
  }),
  ["\u{20BB7}", "\u{10FFFF}"],
  "non-ASCII v",
);

true;
