function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function matches(re, input, label) {
  check(re.test(input), true, label);
}

function rejects(re, input, label) {
  check(re.test(input), false, label);
}

// A class-string disjunction is a set of complete strings, not a bag of
// scalar characters. The longest member is attempted first, while ordinary
// RegExp backtracking may still select a shorter member to satisfy a suffix.
var longest = /^[\q{a|ab|key}]/v.exec("about");
check(longest[0], "ab", "longest class string first");
var shorter = /^([\q{ab|a}])b$/v.exec("ab");
check(shorter[1], "a", "shorter class string restores capture");
rejects(/^[\q{ab}]$/v, "a", "multi-character member is indivisible");

var captured = /^([\q{ab|c}])([\q{de|f}])$/v.exec("abde");
check(captured[1], "ab", "first class-string capture");
check(captured[2], "de", "second class-string capture");

// Union admits both scalar and string members in either operand order.
matches(/^[\q{ab|c}_]+$/v, "ab_c", "string-left union");
matches(/^[_\q{ab|c}]+$/v, "_cab", "string-right union");
matches(/^[\d\q{ab}]+$/v, "1ab2", "class escape union");
matches(/^[\p{ASCII_Hex_Digit}\q{ab}]+$/v, "Aab", "property escape union");

// Intersection and subtraction operate on complete set members. A scalar
// character is a one-code-point string for these operations.
matches(/^[\q{a|ab|b}&&\q{ab|b|c}]+$/v, "abb", "string intersection");
rejects(/^[\q{a|ab|b}&&\q{ab|b|c}]+$/v, "a", "intersection removes a");
matches(/^[\q{0|2|4|9\uFE0F\u20E3}&&[0-9]]+$/v, "024", "string and class intersection");
rejects(
  /^[\q{0|2|4|9\uFE0F\u20E3}&&[0-9]]+$/v,
  "9\uFE0F\u20E3",
  "intersection removes multi-code-point member",
);

matches(/^[\q{a|ab|b}--\q{a|b}]+$/v, "abab", "string subtraction");
rejects(/^[\q{a|ab|b}--\q{a|b}]+$/v, "a", "subtraction removes a");
matches(
  /^[\q{0|2|4|9\uFE0F\u20E3}--[0-9]]+$/v,
  "9\uFE0F\u20E3",
  "string-left subtraction retains keycap",
);
matches(/^[[0-9]--\q{0|2|4|9\uFE0F\u20E3}]+$/v, "1356789", "string-right subtraction");
rejects(/^[[0-9]--\q{0|2|4|9\uFE0F\u20E3}]+$/v, "2", "string-right subtraction removes scalar");

// Empty alternatives are valid class strings. They remain zero-width atoms,
// while a consuming sibling and overall global progress stay live.
matches(/^[\q{|a}]$/v, "", "empty class string");
matches(/^[\q{|a}]$/v, "a", "consuming sibling of empty class string");
check(JSON.stringify("bb".match(/[\q{}]/gv)), JSON.stringify(["", "", ""]), "global empty progress");

// Reverse matching must consume the same complete member rather than reverse
// its UTF-16 units or reinterpret it as separate scalar alternatives.
var reverse = /(?<=([\q{ab|c}]))d/v.exec("abd");
check(reverse[0], "d", "lookbehind result");
check(reverse[1], "ab", "lookbehind class-string capture");

true;
