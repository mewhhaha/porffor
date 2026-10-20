var ok = true;

ok = ok && Math.min() === Infinity;
ok = ok && Math.max() === -Infinity;
ok = ok && Math.min(0) === 0;
ok = ok && Math.max(0) === 0;
ok = ok && Math.min(3, 2, 1) === 1;
ok = ok && Math.max(1, 2, 3) === 3;

function minTwelve(a, b, c, d, e, f, g, h, i, j, k, l) {
  return Math.min(a, b, c, d, e, f, g, h, i, j, k, l);
}

function maxTwelve(a, b, c, d, e, f, g, h, i, j, k, l) {
  return Math.max(a, b, c, d, e, f, g, h, i, j, k, l);
}

ok = ok && minTwelve(11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, -12) === -12;
ok = ok && maxTwelve(-11, -10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 12) === 12;
ok = ok && 1 / minTwelve(11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 0, -0) === -Infinity;
ok = ok && 1 / maxTwelve(-11, -10, -9, -8, -7, -6, -5, -4, -3, -2, -0, 0) === Infinity;

var conversionOrder = "";

function observedNumber(label, value) {
  return {
    valueOf: function () {
      conversionOrder += label;
      return value;
    }
  };
}

var nanMinimum = Math.min(
  observedNumber("a", NaN),
  observedNumber("b", 4),
  observedNumber("c", 3),
  observedNumber("d", -10),
  observedNumber("e", 2)
);
ok = ok && nanMinimum !== nanMinimum && conversionOrder === "abcde";

conversionOrder = "";
var nanMaximum = Math.max(
  observedNumber("a", NaN),
  observedNumber("b", -4),
  observedNumber("c", -3),
  observedNumber("d", 10),
  observedNumber("e", -2)
);
ok = ok && nanMaximum !== nanMaximum && conversionOrder === "abcde";

var sentinel = {};
var abruptOrder = "";
var caughtSentinel = false;

function abruptNumber(label, value, shouldThrow) {
  return {
    valueOf: function () {
      abruptOrder += label;
      if (shouldThrow) throw sentinel;
      return value;
    }
  };
}

try {
  Math.max(
    abruptNumber("a", 1, false),
    abruptNumber("b", 2, false),
    abruptNumber("c", 3, false),
    abruptNumber("d", 4, true),
    abruptNumber("e", 5, false)
  );
} catch (error) {
  caughtSentinel = error === sentinel;
}

ok = ok && caughtSentinel && abruptOrder === "abcd";

ok;
