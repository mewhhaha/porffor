function check(value, label) {
  if (!value) {
    throw "special exponentiation fixture failed: " + label;
  }
}

function isNaNValue(value) {
  return value !== value;
}

function isNegativeZero(value) {
  return value === 0 && 1 / value === -Infinity;
}

check(NaN ** 0 === 1, "nan exponent zero");
check(isNaNValue(NaN ** -1e21), "nan base huge negative exponent");
check(isNaNValue(Math.pow(NaN, -1e21)), "math pow nan huge negative exponent");

check(isNaNValue(1 ** Infinity), "one to infinity");
check(isNaNValue(Math.pow(-1, -Infinity)), "negative one to negative infinity");

check(Infinity ** 0.000000000000001 === Infinity, "positive infinity tiny positive");
check(Infinity ** -0.000000000000001 === 0, "positive infinity tiny negative");
check((-Infinity) ** 111111 === -Infinity, "negative infinity positive odd");
check((-Infinity) ** Math.PI === Infinity, "negative infinity positive noninteger");
check(isNegativeZero((-Infinity) ** -111111), "negative infinity negative odd");
check((-Infinity) ** -Math.E === 0, "negative infinity negative noninteger");

check(0 ** -1 === Infinity, "positive zero negative exponent");
check((-0) ** 111111 === 0, "negative zero positive odd equality");
check(isNegativeZero((-0) ** 111111), "negative zero positive odd sign");
check((-0) ** Math.PI === 0, "negative zero positive noninteger");
check((-0) ** -2 === Infinity, "negative zero negative even");
check(Math.pow(-0, -3) === -Infinity, "math pow negative zero negative odd");

true;
