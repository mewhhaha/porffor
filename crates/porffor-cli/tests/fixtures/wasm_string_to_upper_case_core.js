function check(value, label) {
  if (!value) {
    throw "String toUpperCase fixture failed: " + label;
  }
}

check("abcxyz09-f".toUpperCase() === "ABCXYZ09-F", "ascii lowercase");
check("Already UPPER".toUpperCase() === "ALREADY UPPER", "mixed ascii");
check("123-!?".toUpperCase() === "123-!?", "non letters");
check("ma\u00f1ana".toUpperCase() === "MA\u00f1ANA", "non-ascii preserved");
check(String.prototype.toUpperCase.call(12345) === "12345", "number receiver");
check(String.prototype.toUpperCase.name === "toUpperCase", "name");
check(String.prototype.toUpperCase.length === 0, "length");

var threwNull = false;
try {
  String.prototype.toUpperCase.call(null);
} catch (e) {
  threwNull = true;
}
check(threwNull, "null receiver throws");

var threwUndefined = false;
try {
  String.prototype.toUpperCase.call(undefined);
} catch (e) {
  threwUndefined = true;
}
check(threwUndefined, "undefined receiver throws");

"abcxyz".toUpperCase() === "ABCXYZ";
