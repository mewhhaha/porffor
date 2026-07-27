function abstractlyEqual(left, right) {
  return left == right;
}

if (!abstractlyEqual(9223372036854775808n, 9223372036854775808)) {
  throw "heap bigint equals exact number";
}
if (!abstractlyEqual(9223372036854775808, 9223372036854775808n)) {
  throw "number equals heap bigint";
}
if (abstractlyEqual(9223372036854775809n, 9223372036854775808)) {
  throw "rounded number differs from bigint";
}

let threeLimb = 340282366920938463463374607431768211456;
if (!abstractlyEqual(340282366920938463463374607431768211456n, threeLimb)) {
  throw "three-limb bigint equals exact number";
}
if (abstractlyEqual(340282366920938463463374607431768211457n, threeLimb)) {
  throw "rounded number differs from three-limb bigint";
}

if (!abstractlyEqual(1n, true) || !abstractlyEqual(false, 0n)) {
  throw "boolean converts before bigint comparison";
}
if (abstractlyEqual(2n, true)) {
  throw "converted boolean differs from bigint";
}

137;
