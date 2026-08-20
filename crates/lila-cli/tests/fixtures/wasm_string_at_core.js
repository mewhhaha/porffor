function check(value, label) {
  if (!value) throw "String at fixture failed: " + label;
}

var loneHigh = "\uD800";
var high = "\uD83D";
var low = "\uDCA9";
var pair = high + low;

check("12345".at(0) === "1", "zero");
check("12345".at(4) === "5", "last positive");
check("12345".at(-1) === "5", "negative one");
check("12345".at(-3) === "3", "negative three");
check("".at(0) === undefined, "empty zero");
check("12345".at(5) === undefined, "past end");
check("12345".at(-6) === undefined, "before start");
check(("12" + loneHigh + "34").at(2) === loneHigh, "lone surrogate code unit");
check("💩".at(0) === high, "astral literal high code unit");
check("💩".at(1) === low, "astral literal low code unit");
check(pair.at(0) === high, "escaped pair high code unit");
check(pair.at(1) === low, "escaped pair low code unit");
check(pair.at(-2) === high, "escaped pair relative high code unit");
check(pair.at(-1) === low, "escaped pair relative low code unit");
check(("a" + pair + "b").at(1) === high, "mixed string high code unit");
check(("a" + pair + "b").at(2) === low, "mixed string low code unit");
check(low.at(0) === low, "lone low surrogate");
check((low + high).at(-2) === low, "reversed pair relative low surrogate");
check((low + high).at(-1) === high, "reversed pair relative high surrogate");
check(pair.at(0) === pair[0], "indexed access high parity");
check(pair.at(1) === pair[1], "indexed access low parity");
check(pair.at(Infinity) === undefined, "positive infinity miss");
check(pair.at(-Infinity) === undefined, "negative infinity miss");
check(pair.at(1e100) === undefined, "large finite positive miss");
check(pair.at(-1e100) === undefined, "large finite negative miss");
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
