var text = "𠮷a𠮷b𠮷c👨‍👩‍👧‍👦d";

function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function doSearch(regex) {
  return RegExp.prototype[Symbol.search].call(regex, text);
}

check(doSearch(/a/), 2, "a");
check(doSearch(/a/u), 2, "a u");
check(doSearch(/a/v), 2, "a v");
check(doSearch(/𠮷/), 0, "han");
check(doSearch(/𠮷/u), 0, "han u");
check(doSearch(/𠮷/v), 0, "han v");
check(doSearch(/\p{Script=Han}/u), 0, "property u");
check(doSearch(/\p{Script=Han}/v), 0, "property v");
check(doSearch(/c./), 8, "dot");
check(doSearch(/c./u), 8, "dot u");
check(doSearch(/c./v), 8, "dot v");
check(doSearch(/👨‍👩‍👧‍👦/u), 9, "emoji u");
check(doSearch(/👨‍👩‍👧‍👦/v), 9, "emoji v");
check(doSearch(/[👨‍👩‍👧‍👦]/v), 9, "set v");
check(doSearch(/[👨‍👩‍👧‍👦]/u), 9, "set u");
check(doSearch(/x/u), -1, "x u");
check(doSearch(/x/v), -1, "x v");

true;
