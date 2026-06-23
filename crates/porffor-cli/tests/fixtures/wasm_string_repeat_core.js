function check(value, label) {
  if (!value) {
    throw "String repeat fixture failed: " + label;
  }
}

check("abc".repeat(1) === "abc", "count one");
check("abc".repeat(3) === "abcabcabc", "count three");
check("".repeat(2147483647) === "", "empty huge count");
check("foo".repeat(0) === "", "zero count");
check("foo".repeat(NaN) === "", "nan count");
check("foo".repeat(undefined) === "", "undefined count");
check("foo".repeat(null) === "", "null count");
check("foo".repeat(false) === "", "false count");
check("foo".repeat("2") === "foofoo", "string count");
check("foo".repeat(2.9) === "foofoo", "fractional count");
check(String.prototype.repeat.call(42, 2) === "4242", "number receiver");
check(String.prototype.repeat.call(true, 2) === "truetrue", "boolean receiver");
check(String.prototype.repeat.name === "repeat", "name");
check(String.prototype.repeat.length === 1, "length");

try {
  "x".repeat(-1);
  check(false, "negative count did not throw");
} catch (e) {
  check(e instanceof RangeError, "negative count RangeError");
}

try {
  "x".repeat(Infinity);
  check(false, "infinite count did not throw");
} catch (e) {
  check(e instanceof RangeError, "infinite count RangeError");
}

try {
  String.prototype.repeat.call(Symbol("s"), 1);
  check(false, "symbol receiver did not throw");
} catch (e) {
  check(e instanceof TypeError, "symbol receiver TypeError");
}

try {
  "x".repeat(Symbol("s"));
  check(false, "symbol count did not throw");
} catch (e) {
  check(e instanceof TypeError, "symbol count TypeError");
}

true;
