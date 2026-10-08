// In unicode mode `\0` matches NUL when not followed by a decimal digit, and
// `\0` followed by a digit is a SyntaxError (no legacy octal escapes).
function check(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

check(/\0/u.test("\0"), true, "nul-matches");
check(/\0/u.test("0"), false, "nul-not-zero");
check(/a\0b/u.test("a\0b"), true, "nul-middle");

var digitThrows = false;
try {
  new RegExp("\\01", "u");
} catch (error) {
  if (!(error instanceof SyntaxError)) throw error;
  digitThrows = true;
}
check(digitThrows, true, "nul-digit-rejected");

var vDigitThrows = false;
try {
  new RegExp("\\01", "v");
} catch (error) {
  if (!(error instanceof SyntaxError)) throw error;
  vDigitThrows = true;
}
check(vDigitThrows, true, "nul-digit-rejected-v");

true;
