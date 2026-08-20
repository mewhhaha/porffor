function check(value) {
  if (!value) {
    throw "RegExp.escape fixture failed";
  }
}

var propDesc = Object.getOwnPropertyDescriptor(RegExp, "escape");
var lengthDesc = Object.getOwnPropertyDescriptor(RegExp.escape, "length");
var nameDesc = Object.getOwnPropertyDescriptor(RegExp.escape, "name");

check(typeof RegExp.escape === "function");
check(propDesc.writable === true);
check(propDesc.enumerable === false);
check(propDesc.configurable === true);
check(lengthDesc.value === 1);
check(lengthDesc.writable === false);
check(lengthDesc.enumerable === false);
check(lengthDesc.configurable === true);
check(nameDesc.value === "escape");
check(nameDesc.writable === false);
check(nameDesc.enumerable === false);
check(nameDesc.configurable === true);

var threwNonString = false;
try {
  RegExp.escape(1);
} catch (error) {
  threwNonString = error instanceof TypeError;
}
check(threwNonString);

var threwConstruct = false;
try {
  new RegExp.escape("");
} catch (error) {
  threwConstruct = error instanceof TypeError;
}
check(threwConstruct);

check(RegExp.escape("") === "");
check(RegExp.escape("a+a") === "\\x61\\+a");
check(RegExp.escape("1+1") === "\\x31\\+1");
check(RegExp.escape(".^$*+?()[]{}|/\\") === "\\.\\^\\$\\*\\+\\?\\(\\)\\[\\]\\{\\}\\|\\/\\\\");
check(RegExp.escape(",-=<>#&!%:;@~'`\"") === "\\x2c\\x2d\\x3d\\x3c\\x3e\\x23\\x26\\x21\\x25\\x3a\\x3b\\x40\\x7e\\x27\\x60\\x22");
check(RegExp.escape("\t\n\v\f\r") === "\\t\\n\\v\\f\\r");
check(RegExp.escape("\uFEFF \u00A0\u202F\u2028\u2029") === "\\ufeff\\x20\\xa0\\u202f\\u2028\\u2029");

var escapedFromStringForOf = "";
for (const c of ",-") {
  escapedFromStringForOf = escapedFromStringForOf + RegExp.escape(c);
}
check(escapedFromStringForOf === "\\x2c\\x2d");

true;
