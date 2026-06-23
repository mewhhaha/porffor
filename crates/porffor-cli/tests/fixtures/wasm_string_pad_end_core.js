function check(value, label) {
  if (!value) {
    throw "String padEnd fixture failed: " + label;
  }
}

check("abc".padEnd(6, "0") === "abc000", "single-byte filler");
check("abc".padEnd(8, "01") === "abc01010", "truncated repeated filler");
check("abc".padEnd(2, "0") === "abc", "target shorter than string");
check("abc".padEnd(3, "0") === "abc", "target equal to string");
check("abc".padEnd(6, "") === "abc", "empty filler");
check("abc".padEnd(5) === "abc  ", "default filler");
check("abc".padEnd(10, false) === "abcfalsefa", "boolean filler");
check("abc".padEnd(10, null) === "abcnullnul", "null filler");
check("abc".padEnd(10, 0) === "abc0000000", "number filler");
check(String.prototype.padEnd.call(42, 4, "0") === "4200", "number receiver");
check(String.prototype.padEnd.name === "padEnd", "name");
check(String.prototype.padEnd.length === 1, "length");

try {
  String.prototype.padEnd.call(undefined, 4);
  check(false, "undefined receiver did not throw");
} catch (e) {
  check(e instanceof TypeError, "undefined receiver TypeError");
}

try {
  "abc".padEnd(5, Symbol("s"));
  check(false, "symbol filler did not throw");
} catch (e) {
  check(e instanceof TypeError, "symbol filler TypeError");
}

"f".toUpperCase().padEnd(6, "0") === "F00000";
