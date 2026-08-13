// Ordinary `u`-mode class grammar is independent of whether the compiler
// selects an ASCII bitmap or a code-point range-set instruction.

function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function throwsSyntaxError(pattern, flags) {
  try {
    new RegExp(pattern, flags);
  } catch (error) {
    return error.name === "SyntaxError";
  }
  return false;
}

// Static constructor arguments exercise the compiler's direct RegExp path.
var staticErrorName = "no-throw";
try {
  new RegExp("[\\q]", "u");
} catch (error) {
  staticErrorName = error.name;
}
check(staticErrorName, "SyntaxError", "static bitmap identity escape");

staticErrorName = "no-throw";
try {
  new RegExp("[\\01\\u0041]", "u");
} catch (error) {
  staticErrorName = error.name;
}
check(staticErrorName, "SyntaxError", "static range octal escape");

var staticControl = new RegExp("[\\cA]", "u");
check(staticControl.test("\x01"), true, "static control-letter value");
check(staticControl.test("c"), false, "static control escape is not literal c");
check(staticControl.test("A"), false, "static control escape consumes its letter");

// Array reads keep these arguments dynamic and exercise the emitted finite
// runtime candidate table. Each pair differs only by `\u0041`, which forces
// the range representation.
var invalidPatterns = [
  "[\\q]",
  "[\\q\\u0041]",
  "[\\c]",
  "[\\c\\u0041]",
  "[\\c0]",
  "[\\c0\\u0041]",
  "[\\c_]",
  "[\\c_\\u0041]",
  "[\\1]",
  "[\\1\\u0041]",
  "[\\8]",
  "[\\8\\u0041]",
  "[\\01]",
  "[\\01\\u0041]",
];
var unicodeFlags = ["u"];
for (var index = 0; index < invalidPatterns.length; index += 1) {
  check(
    throwsSyntaxError(invalidPatterns[index], unicodeFlags[0]),
    true,
    "computed invalid ordinary class " + index,
  );
}

// Legal siblings prevent a blanket `u`-mode rejection and pin decoded values
// through both representations.
var validPatterns = [
  "[\\cA]",
  "[\\cA\\u0002]",
  "[\\0]",
  "[\\0\\u0002]",
  "[\\-]",
  "[\\-\\u0041]",
];
for (index = 0; index < 2; index += 1) {
  var control = new RegExp(validPatterns[index], unicodeFlags[0]);
  check(control.test("\x01"), true, "computed control-letter value " + index);
  check(control.test("c"), false, "computed control escape is not c " + index);
}
for (index = 2; index < 4; index += 1) {
  check(
    new RegExp(validPatterns[index], unicodeFlags[0]).test("\x00"),
    true,
    "computed zero escape " + index,
  );
}
for (index = 4; index < validPatterns.length; index += 1) {
  check(
    new RegExp(validPatterns[index], unicodeFlags[0]).test("-"),
    true,
    "computed identity escape " + index,
  );
}

// Annex B remains live only in the legacy grammar. An invalid legacy control
// escape consumes only its backslash, leaving `c` as another class member.
var legacyPatterns = [
  "[\\c0]",
  "[\\c_]",
  "[\\01]",
  "[\\8]",
  "[\\c]",
  "[\\c\\u0041]",
];
var legacyFlags = [""];
check(new RegExp(legacyPatterns[0], legacyFlags[0]).test("\x10"), true, "legacy c0");
check(new RegExp(legacyPatterns[1], legacyFlags[0]).test("\x1f"), true, "legacy c underscore");
check(new RegExp(legacyPatterns[2], legacyFlags[0]).test("\x01"), true, "legacy octal");
check(new RegExp(legacyPatterns[3], legacyFlags[0]).test("8"), true, "legacy identity");
for (index = 4; index < legacyPatterns.length; index += 1) {
  var bareControl = new RegExp(legacyPatterns[index], legacyFlags[0]);
  check(bareControl.test("\\"), true, "legacy bare control backslash " + index);
  check(bareControl.test("c"), true, "legacy bare control c " + index);
  check(bareControl.test("\x03"), false, "legacy bare control is not U+0003 " + index);
}

true;
