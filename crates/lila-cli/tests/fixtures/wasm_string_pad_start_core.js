function check(value, label) {
  if (!value) {
    throw "String padStart fixture failed: " + label;
  }
}

check("abc".padStart(6, "0") === "000abc", "single-byte filler");
check("abc".padStart(8, "01") === "01010abc", "truncated repeated filler");
check("abc".padStart(2, "0") === "abc", "target shorter than string");
check("abc".padStart(3, "0") === "abc", "target equal to string");
check("abc".padStart(6, "") === "abc", "empty filler");
check("abc".padStart(5) === "  abc", "default filler");
check("abc".padStart(6, "\uD83D\uDCA9") === "\uD83D\uDCA9\uD83Dabc", "surrogate filler prefix");
check(String.prototype.padStart.call(42, 4, "0") === "0042", "number receiver");
check(String.prototype.padStart.name === "padStart", "name");
check(String.prototype.padStart.length === 1, "length");

"f".toUpperCase().padStart(6, "0") === "00000F";
