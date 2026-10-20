function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) {
    throw label + ": expected " + expected + ", got " + actual;
  }
}

function checkFixed(value, fractionDigits, expected, label) {
  assertSame(value.toFixed(fractionDigits), expected, "toFixed " + label);
}

function checkExponential(value, fractionDigits, expected, label) {
  assertSame(
    value.toExponential(fractionDigits),
    expected,
    "toExponential " + label,
  );
}

function checkPrecision(value, precision, expected, label) {
  assertSame(value.toPrecision(precision), expected, "toPrecision " + label);
}

checkFixed(1.25, 1, "1.3", "exact halfway rounds upward");
checkFixed(9.99, 1, "10.0", "rounding carries into the integer part");
checkFixed(123.456, 2, "123.46", "decimal placement");
checkFixed(0.00008, 3, "0.000", "fractional leading zeroes");
checkFixed(-1.25, 1, "-1.3", "negative rounding");
checkFixed(-0.004, 2, "-0.00", "negative value rounded to zero");
checkFixed(-0, 2, "0.00", "negative zero suppresses its sign");
checkFixed(1e21, 2, "1e+21", "large-value shortest threshold");
checkFixed(NaN, 2, "NaN", "not-a-number spelling");
checkFixed(Infinity, 2, "Infinity", "positive infinity spelling");
checkFixed(-Infinity, 2, "-Infinity", "negative infinity spelling");
checkFixed(
  1000000000000000128,
  0,
  "1000000000000000128",
  "exact integer digits differ from shortest spelling",
);
checkFixed(1.005, 2, "1.00", "binary64 value below decimal midpoint");

checkExponential(1.25, 1, "1.3e+0", "exact halfway rounds upward");
checkExponential(9.99, 1, "1.0e+1", "rounding carries into the exponent");
checkExponential(123.456, 2, "1.23e+2", "positive exponent placement");
checkExponential(
  123.456,
  17,
  "1.23456000000000003e+2",
  "exact binary64 digits beyond shortest spelling",
);
checkExponential(-0.0125, 2, "-1.25e-2", "negative sign and exponent");
checkExponential(0, 3, "0.000e+0", "zero padding");
checkExponential(-0, 1, "0.0e+0", "negative zero suppresses its sign");
checkExponential(1e21, 0, "1e+21", "explicit zero fraction digits");
checkExponential(Number.MIN_VALUE, 2, "4.94e-324", "minimum subnormal");
checkExponential(Number.MAX_VALUE, 2, "1.80e+308", "maximum finite value");
checkExponential(NaN, 2, "NaN", "not-a-number spelling");
checkExponential(-Infinity, 2, "-Infinity", "negative infinity spelling");
assertSame(
  (function (value) {
    return value.toExponential();
  })(12.34),
  "1.234e+1",
  "toExponential omitted fraction digits use shortest mantissa",
);

checkPrecision(1.25, 2, "1.3", "exact halfway rounds upward");
checkPrecision(123.456, 4, "123.5", "fixed placement");
checkPrecision(0.00123456, 3, "0.00123", "fractional leading zeroes");
checkPrecision(1234.56, 3, "1.23e+3", "upper scientific threshold");
checkPrecision(0.0000001234, 3, "1.23e-7", "lower scientific threshold");
checkPrecision(0.000001234, 3, "0.00000123", "lower fixed threshold");
checkPrecision(
  0.0000009999,
  1,
  "0.000001",
  "carry selects fixed notation after rounding",
);
checkPrecision(-12.5, 2, "-13", "negative rounding");
checkPrecision(0, 4, "0.000", "zero padding");
checkPrecision(-0, 4, "0.000", "negative zero suppresses its sign");
checkPrecision(999.5, 3, "1.00e+3", "rounding changes notation");
checkPrecision(Number.MIN_VALUE, 2, "4.9e-324", "minimum subnormal");
checkPrecision(Number.MAX_VALUE, 3, "1.80e+308", "maximum finite value");
checkPrecision(
  1.2345e27,
  18,
  "1.23449999999999996e+27",
  "exact binary64 significant digits beyond shortest spelling",
);
checkPrecision(NaN, 3, "NaN", "not-a-number spelling");
checkPrecision(Infinity, 3, "Infinity", "positive infinity spelling");
assertSame(
  (function (value) {
    return value.toPrecision();
  })(12.34),
  "12.34",
  "toPrecision omitted precision uses shortest spelling",
);

print("number-decimal-formatting:ok");

0;
