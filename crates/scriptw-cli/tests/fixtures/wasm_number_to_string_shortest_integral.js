function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function checkRuntimeToString(value, expectedString, label) {
  check(value.toString(), expectedString, label + " toString");
}

function checkRuntimeExactness(value, expectedString, expectedFixed, label) {
  checkRuntimeToString(value, expectedString, label);
  check(value.toFixed(0), expectedFixed, label + " toFixed(0)");
}

checkRuntimeExactness(
  1000000000000000128,
  "1000000000000000100",
  "1000000000000000128",
  "positive exactness"
);
checkRuntimeToString(
  -1000000000000000128,
  "-1000000000000000100",
  "negative exactness"
);
checkRuntimeToString(9007199254740991, "9007199254740991", "safe integer boundary");
checkRuntimeToString(
  18014398509481992,
  "18014398509481990",
  "integral lower selection"
);
checkRuntimeToString(
  18014398509482008,
  "18014398509482010",
  "integral upper selection"
);
checkRuntimeToString(
  9.409340012568248e18,
  "9409340012568248000",
  "near 1e19"
);
checkRuntimeToString(
  9999999999999997952,
  "9999999999999998000",
  "integral near 1e19"
);
checkRuntimeToString(
  0.30000000000000004,
  "0.30000000000000004",
  "binary fraction"
);
checkRuntimeToString(
  193744829919998.375,
  "193744829919998.38",
  "fractional shortest rounding"
);
checkRuntimeToString(1e19, "10000000000000000000", "1e19 fixed threshold");
checkRuntimeToString(1e20, "100000000000000000000", "1e20 fixed threshold");
checkRuntimeToString(1e21, "1e+21", "1e21 scientific threshold");
checkRuntimeToString(5e-324, "5e-324", "minimum subnormal");
checkRuntimeToString(
  2.2250738585072014e-308,
  "2.2250738585072014e-308",
  "minimum normal"
);
checkRuntimeToString(
  1.0000000000000002,
  "1.0000000000000002",
  "successor of one"
);
checkRuntimeToString(
  0.9999999999999999,
  "0.9999999999999999",
  "predecessor of one"
);
checkRuntimeToString(
  999999999999999900000,
  "999999999999999900000",
  "predecessor below scientific threshold"
);
checkRuntimeToString(
  1000000000000000100000,
  "1.0000000000000001e+21",
  "successor above scientific threshold"
);
checkRuntimeToString(-0, "0", "negative zero");

check(
  (193744829919998.375).toString(),
  "193744829919998.38",
  "static fractional shortest rounding"
);
check((1e19).toString(), "10000000000000000000", "static 1e19 fixed");
check((1e21).toString(), "1e+21", "static 1e21 scientific");
check((5e-324).toString(), "5e-324", "static minimum subnormal");
check((-0).toString(), "0", "static negative zero");

const numericKeys = {
  [193744829919998.375]: "fractional",
  [1e21]: "scientific",
};
check(
  numericKeys["193744829919998.38"],
  "fractional",
  "static fractional property key"
);
check(numericKeys["1e+21"], "scientific", "static scientific property key");

true;
