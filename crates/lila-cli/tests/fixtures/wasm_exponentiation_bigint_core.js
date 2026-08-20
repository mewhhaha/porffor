function fail(label) {
  throw "bigint exponentiation fixture failed: " + label;
}

function check(actual, expected, label) {
  if (actual !== expected) {
    fail(label + ": " + actual + " !== " + expected);
  }
}

check(2n ** 3n, 8n, "positive bigint exponentiation");
check((-2n) ** 3n, -8n, "negative base odd exponent");
check((-2n) ** 2n, 4n, "negative base even exponent");
check(2n ** 0n, 1n, "zero exponent");
check(Object(2n) ** 3n, 8n, "boxed bigint base");
check(2n ** Object(3n), 8n, "boxed bigint exponent");

let leftObject = {
  valueOf: function () {
    return 2n;
  },
  toString: function () {
    fail("left toString should not run");
  },
};
let rightObject = {
  valueOf: function () {
    return 3n;
  },
  toString: function () {
    fail("right toString should not run");
  },
};
check(leftObject ** rightObject, 8n, "object valueOf bigint exponentiation");

let stringFallback = {
  valueOf: {},
  toString: function () {
    return 2n;
  },
};
check(stringFallback ** 3n, 8n, "object toString bigint fallback");

__lilaAssertThrows(RangeError, function () {
  1n ** -1n;
});
__lilaAssertThrows(RangeError, function () {
  0n ** -1n;
});
__lilaAssertThrows(TypeError, function () {
  1n ** 1;
});
__lilaAssertThrows(TypeError, function () {
  1 ** 1n;
});
__lilaAssertThrows(TypeError, function () {
  Object(1n) ** Object(1);
});
__lilaAssertThrows(TypeError, function () {
  Symbol("x") ** 0n;
});

let trace = "";
let orderedLeft = {
  valueOf: function () {
    trace += "3";
    return 2n;
  },
};
let orderedRight = {
  valueOf: function () {
    trace += "4";
    return 3n;
  },
};
check((trace += "1", orderedLeft) ** (trace += "2", orderedRight), 8n, "ordered object pow");
check(trace, "1234", "operand expression before bigint ToNumeric order");

let throwTrace = "";
let throwingLeft = {
  valueOf: function () {
    throwTrace += "3";
    throw "left coercion";
  },
};
let skippedRight = {
  valueOf: function () {
    throwTrace += "4";
    return 2n;
  },
};
try {
  (throwTrace += "1", throwingLeft) ** (throwTrace += "2", skippedRight);
} catch (e) {}
check(throwTrace, "123", "left bigint coercion throw stops right coercion");

true;
