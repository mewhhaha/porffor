function check(value, label) {
  if (!value) {
    throw "String charAt fixture failed: " + label;
  }
}

function Test262Error(message) {
}

var assert = function (mustBeTrue, message) {
  if (!mustBeTrue) {
    throw message;
  }
};
assert.sameValue = function (actual, expected, message) {
  if (actual !== expected) {
    throw message;
  }
};

var boolObject = new Boolean;
boolObject.charAt = String.prototype.charAt;
check(boolObject.charAt(false) === "f", "boolean false index");
check(boolObject.charAt(true) === "a", "boolean true index");
check(boolObject.charAt(true + 1) === "l", "boolean index two");
check(
  boolObject.charAt(false) + boolObject.charAt(true) + boolObject.charAt(true + 1) === "fal",
  "boolean concatenated",
);
if (boolObject.charAt(false) + boolObject.charAt(true) + boolObject.charAt(true + 1) !== "fal") {
  throw new Test262Error('#1: __instance = new Boolean; __instance.charAt = String.prototype.charAt;  __instance = new Boolean; __instance.charAt = String.prototype.charAt; __instance.charAt(false)+__instance.charAt(true)+__instance.charAt(true+1) === "fal". Actual: ' + boolObject.charAt(false) + boolObject.charAt(true) + boolObject.charAt(true + 1));
}

var objectNumber = new Object(42);
objectNumber.charAt = String.prototype.charAt;
check(objectNumber.charAt(false) + objectNumber.charAt(true) === "42", "object number concatenated");
if (objectNumber.charAt(false) + objectNumber.charAt(true) !== "42") {
  throw new Test262Error('#1: __instance = new Object(42); __instance.charAt = String.prototype.charAt;  __instance = new Object(42); __instance.charAt = String.prototype.charAt; __instance.charAt(false)+__instance.charAt(true) === "42". Actual: ' + objectNumber.charAt(false) + objectNumber.charAt(true));
}

var __instance = new Object(42);
__instance.charAt = String.prototype.charAt;
if (__instance.charAt(false) + __instance.charAt(true) !== "42") {
  throw new Test262Error('#1: __instance = new Object(42); __instance.charAt = String.prototype.charAt;  __instance = new Object(42); __instance.charAt = String.prototype.charAt; __instance.charAt(false)+__instance.charAt(true) === "42". Actual: ' + __instance.charAt(false) + __instance.charAt(true));
}

__instance = new Boolean;
__instance.charAt = String.prototype.charAt;
if (__instance.charAt(false) + __instance.charAt(true) + __instance.charAt(true + 1) !== "fal") {
  throw new Test262Error('#1: __instance = new Boolean; __instance.charAt = String.prototype.charAt;  __instance = new Boolean; __instance.charAt = String.prototype.charAt; __instance.charAt(false)+__instance.charAt(true)+__instance.charAt(true+1) === "fal". Actual: ' + __instance.charAt(false) + __instance.charAt(true) + __instance.charAt(true + 1));
}

var numberObject = new Number(12345);
numberObject.charAt = String.prototype.charAt;
check(numberObject.charAt(0) === "1", "number first");
check(numberObject.charAt(4) === "5", "number last");
check(numberObject.charAt(5) === "", "number out of range");
check(numberObject.charAt(-1) === "", "number negative");

check("abcd".charAt("   +00200.0000E-0002   ") === "c", "string position coercion");
check("abc".charAt(-0.99999) === "a", "negative fractional position near -1");
check("abc".charAt(-0.00001) === "a", "negative fractional position near 0");
check("abc".charAt(0.00001) === "a", "positive fractional position near 0");
check("abc".charAt(0.99999) === "a", "positive fractional position below 1");
check("abc".charAt(1.00001) === "b", "positive fractional position above 1");
check("abc".charAt(1.99999) === "b", "positive fractional position below 2");
assert.sameValue("abcd".charAt("   +00200.0000E-0002   "), "c", "assert string position coercion");
assert.sameValue("abc".charAt(-0.99999), "a", "assert negative fractional position near -1");
assert.sameValue("abc".charAt(1.99999), "b", "assert positive fractional position below 2");

var high = "\uD83D";
var low = "\uDCA9";
var pair = high + low;
check("💩".charAt(0) === high, "astral literal high code unit");
check("💩".charAt(1) === low, "astral literal low code unit");
check(pair.charAt(0) === high, "escaped pair high code unit");
check(pair.charAt(1) === low, "escaped pair low code unit");
check(("a" + pair + "b").charAt(1) === high, "mixed string high code unit");
check(("a" + pair + "b").charAt(2) === low, "mixed string low code unit");
check(high.charAt(0) === high, "lone high surrogate");
check(low.charAt(0) === low, "lone low surrogate");
check((low + high).charAt(0) === low, "reversed pair low surrogate");
check((low + high).charAt(1) === high, "reversed pair high surrogate");
check(pair.charAt(0) === pair[0], "indexed access high parity");
check(pair.charAt(1) === pair[1], "indexed access low parity");
check(pair.charAt(-1) === "", "negative miss is empty string");
check(pair.charAt(2) === "", "past-end miss is empty string");
check(pair.charAt(Infinity) === "", "positive infinity miss");
check(pair.charAt(-Infinity) === "", "negative infinity miss");
check(pair.charAt(1e100) === "", "large finite positive miss");
check(pair.charAt(-1e100) === "", "large finite negative miss");

var stringObject = new String("one-1 two-2 three-3");
var pieces = stringObject.split(new RegExp);
check(pieces.length === stringObject.length, "split empty regexp length");
for (var i = 0; i < stringObject.length; i++) {
  check(pieces[i] === stringObject.charAt(i), "split empty regexp char " + i);
}

"abc".charAt(1) === "b";
