var ok = true;

ok = ok && 1 / Math.hypot() === Infinity;
ok = ok && 1 / Math.hypot(-0) === Infinity;
ok = ok && 1 / Math.hypot(0, -0, -0) === Infinity;
ok = ok && Math.hypot(3, 4) === 5;
ok = ok && Math.hypot(3, 4, 12) === 13;
ok = ok && Math.hypot(0, 0, 0, 0, 0, 0, 0, 0, 15) === 15;

ok = ok && Math.hypot(3, 4, Infinity) === Infinity;
var nanResult = Math.hypot(3, 4, NaN);
ok = ok && nanResult !== nanResult;
ok = ok && Math.hypot(NaN, 3, 4, Infinity) === Infinity;

var order = "";
function observed(label, value) {
  return {
    valueOf: function () {
      order += label;
      return value;
    },
  };
}

ok = ok && Math.hypot(
  observed("a", Infinity),
  observed("b", NaN),
  observed("c", 0),
  observed("d", 0),
  observed("e", 0),
  observed("f", 0),
  observed("g", 0),
  observed("h", 0),
  observed("i", 0),
) === Infinity;
ok = ok && order === "abcdefghi";

var marker = {};
var afterAbrupt = false;
var abruptIdentity = false;
try {
  Math.hypot(
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    {
      valueOf: function () {
        throw marker;
      },
    },
    {
      valueOf: function () {
        afterAbrupt = true;
        return 0;
      },
    },
  );
} catch (error) {
  abruptIdentity = error === marker;
}
ok = ok && abruptIdentity && !afterAbrupt;

ok = ok && Math.hypot(1e308) === 1e308;
ok = ok && Math.hypot(1e-300) === 1e-300;
var huge = Math.hypot(1e308, 1e308, 1e308);
ok = ok && huge !== Infinity && huge > 1.7e308;
var tiny = Math.hypot(1e-300, 1e-300, 1e-300);
ok = ok && tiny > 1.7e-300 && tiny < 1.8e-300;

ok;
