function check(value, label) {
  if (!value) throw "String at fixture failed: " + label;
}

var high = "\uD800";

check("12345".at(0) === "1", "zero");
check("12345".at(4) === "5", "last positive");
check("12345".at(-1) === "5", "negative one");
check("12345".at(-3) === "3", "negative three");
check("".at(0) === undefined, "empty zero");
check("12345".at(5) === undefined, "past end");
check("12345".at(-6) === undefined, "before start");
check(("12" + high + "34").at(2) === high, "lone surrogate code unit");
check("01".at(false) === "0", "false index");
check("01".at(true) === "1", "true index");
check("01".at(null) === "0", "null index");
check("01".at(undefined) === "0", "undefined index");
check("01".at("1") === "1", "string index");
check(String.prototype.at.call(42, 1) === "2", "number receiver");
check(String.prototype.at.name === "at", "name");
check(String.prototype.at.length === 1, "length");

try {
  "01".at(Symbol());
  check(false, "symbol index did not throw");
} catch (error) {
  check(error instanceof TypeError, "symbol TypeError");
}

try {
  String.prototype.at.call(null, 0);
  check(false, "null receiver did not throw");
} catch (error) {
  check(error instanceof TypeError, "null receiver TypeError");
}

true;
