function check(actual, expected, label) {
  if (actual !== expected) {
    throw "dynamic exponentiation fixture failed: " + label + ": " + actual + " !== " + expected;
  }
}

var exponent = 2;
check(2 ** 3, 8, "literal");
check(3 * 2 ** 3, 24, "precedence multiply");
check(2 ** ++exponent, 8, "prefix rhs");
check(2 ** -1 * 2, 1, "negative exponent");
check(2 ** 2 * 4, 16, "multiply after");
check(2 ** 2 / 2, 2, "divide after");
check(2 ** (3 ** 2), 512, "nested paren");
check(2 ** 3 ** 2, 512, "right associative");
check(16 / 2 ** 2, 4, "divide before");
check(~(3 ** 2), -10, "bitwise not exponentiation");
check(2 ** ~"2", 0.125, "bitwise not string exponent");
check(2 ** -"2", 0.25, "unary minus string exponent");

var base = 4;
check(--base ** 2, 9, "prefix decrement lhs");
check(++base ** 2, 16, "prefix increment lhs");
check(base++ ** 2, 16, "postfix increment lhs");
check(base-- ** 2, 25, "postfix decrement lhs");

base = 4;
check(--base ** --base ** 2, 81, "nested prefix decrement");
check(++base ** ++base ** 2, 43046721, "nested prefix increment");

base = 4;
check(base-- ** base-- ** 2, 262144, "nested postfix decrement");
check(base++ ** base++ ** 2, 512, "nested postfix increment");

base = 4;
check(
  --base ** --base ** 2,
  Math.pow(3, Math.pow(2, 2)),
  "nested prefix decrement with Math.pow",
);
check(
  ++base ** ++base ** 2,
  Math.pow(3, Math.pow(4, 2)),
  "nested prefix increment with Math.pow",
);

base = 4;
check(
  base-- ** base-- ** 2,
  Math.pow(4, Math.pow(3, 2)),
  "nested postfix decrement with Math.pow",
);
check(
  base++ ** base++ ** 2,
  Math.pow(2, Math.pow(3, 2)),
  "nested postfix increment with Math.pow",
);

base = -3;
check(base **= 3, -27, "compound assignment return");
check(base, -27, "compound assignment writeback");

var capture = [];
var leftValue = {
  valueOf: function () {
    capture.push("leftValue");
    return 3;
  },
};
var rightValue = {
  valueOf: function () {
    capture.push("rightValue");
    return 2;
  },
};
(capture.push("left"), leftValue) ** +(capture.push("right"), rightValue);
check(capture[0], "left", "order left expression");
check(capture[1], "right", "order right expression");
check(capture[2], "rightValue", "order unary right coercion");
check(capture[3], "leftValue", "order exponentiation left coercion");

var throwTrace = "";
var leftThrowValue = {
  valueOf: function () {
    throwTrace += "3";
    throw "left coercion";
  },
};
var rightThrowValue = {
  valueOf: function () {
    throwTrace += "4";
    return 2;
  },
};
try {
  (throwTrace += "1", leftThrowValue) ** (throwTrace += "2", rightThrowValue);
} catch (e) {}
check(throwTrace, "123", "left coercion throw stops right coercion");

true;
