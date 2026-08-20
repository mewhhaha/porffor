function check(value, label) {
  if (!value) {
    throw "String well-formed fixture failed: " + label;
  }
}

var high = "\uD83D";
var low = "\uDCA9";
var pair = high + low;
var replacement = "\uFFFD";
var isWellFormed = String.prototype.isWellFormed;
var toWellFormed = String.prototype.toWellFormed;

check("abc".isWellFormed() === true, "ascii is well formed");
check(("a" + high + "c").isWellFormed() === false, "lone high surrogate");
check(("a" + low + "c").isWellFormed() === false, "lone low surrogate");
check(("a" + low + high + "c").isWellFormed() === false, "wrong-order pair");
check(("a" + pair + "c").isWellFormed() === true, "escaped pair");
check("a💩c".isWellFormed() === true, "scalar pair");

check(("a" + high + "c").toWellFormed() === "a" + replacement + "c", "replace high");
check(("a" + low + "c").toWellFormed() === "a" + replacement + "c", "replace low");
check(("a" + low + high + "c").toWellFormed() === "a" + replacement + replacement + "c", "replace wrong order");
check(("a" + pair + "c").toWellFormed() === "a" + pair + "c", "keep escaped pair");
check("a💩c".toWellFormed() === "a💩c", "keep scalar pair");

check(isWellFormed.call("ok") === true, "borrowed isWellFormed string");
check(toWellFormed.call("ok") === "ok", "borrowed toWellFormed string");
check(String.prototype.isWellFormed.call(true) === true, "boolean receiver isWellFormed");
check(String.prototype.toWellFormed.call(1) === "1", "number receiver toWellFormed");
check(String.prototype.isWellFormed.name === "isWellFormed", "isWellFormed name");
check(String.prototype.toWellFormed.name === "toWellFormed", "toWellFormed name");
check(String.prototype.isWellFormed.length === 0, "isWellFormed length");
check(String.prototype.toWellFormed.length === 0, "toWellFormed length");

try {
  String.prototype.isWellFormed.call(null);
  check(false, "null isWellFormed did not throw");
} catch (e) {
  check(e instanceof TypeError, "null isWellFormed TypeError");
}

try {
  String.prototype.toWellFormed.call(undefined);
  check(false, "undefined toWellFormed did not throw");
} catch (e) {
  check(e instanceof TypeError, "undefined toWellFormed TypeError");
}

true;
