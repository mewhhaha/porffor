function check(value, label) {
  if (!value) {
    throw "dynamic fractional exponentiation fixture failed: " + label;
  }
}

function near(actual, expected, tolerance, label) {
  var difference = actual - expected;
  if (difference < 0) difference = -difference;
  check(difference <= tolerance, label);
}

var squareBase = 9;
var squareExponent = 0.5;
check(squareBase ** squareExponent === 3, "operator square root");
check(Math.pow(squareBase, squareExponent) === 3, "Math.pow square root");

var reciprocalBase = 16;
var reciprocalExponent = -0.5;
check(reciprocalBase ** reciprocalExponent === 0.25, "operator reciprocal root");
check(
  Math.pow(reciprocalBase, reciprocalExponent) === 0.25,
  "Math.pow reciprocal root",
);

var irrationalBase = 10;
var irrationalExponent = 0.5;
near(
  irrationalBase ** irrationalExponent,
  3.1622776601683795,
  1e-15,
  "operator irrational result",
);
near(
  Math.pow(irrationalBase, irrationalExponent),
  3.1622776601683795,
  1e-15,
  "Math.pow irrational result",
);

var negativeBase = -4;
check(
  negativeBase ** squareExponent !== negativeBase ** squareExponent,
  "negative base fractional operator result",
);
check(
  Math.pow(negativeBase, squareExponent) !==
    Math.pow(negativeBase, squareExponent),
  "negative base fractional Math.pow result",
);

var largeOddExponent = 2049;
check((-1) ** largeOddExponent === -1, "large odd operator exponent");
check(Math.pow(-1, largeOddExponent) === -1, "large odd Math.pow exponent");

var smallestExponent = -1074;
check(2 ** smallestExponent === Number.MIN_VALUE, "operator subnormal result");
check(
  Math.pow(2, smallestExponent) === Number.MIN_VALUE,
  "Math.pow subnormal result",
);

check(2n ** 10n === 1024n, "BigInt path remains distinct");

true;
