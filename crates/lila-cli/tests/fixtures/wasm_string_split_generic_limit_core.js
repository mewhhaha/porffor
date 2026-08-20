function check(value, label) {
  if (!value) {
    throw "generic String split fixture failed: " + label;
  }
}

var limited = new String("hello").split("l", 2);
check(limited.constructor === Array, "boxed limit constructor");
check(limited.length === 2, "boxed limit length");
check(limited[0] === "he", "boxed limit first");
check(limited[1] === "", "boxed limit second");

var callLimited = String.prototype.split.call("a,b", ",", 1);
check(callLimited.constructor === Array, "call limit constructor");
check(callLimited.length === 1, "call limit length");
check(callLimited[0] === "a", "call limit first");
String.prototype.split.call("a,b", ",", 1);
var afterCall = "none";
if (false) {
  afterCall = "bad";
}
check(afterCall === "none", "call completion before if");

var n = new Number(101201);
Number.prototype.split = String.prototype.split;

var zeroLimit = n.split(1, 0);
check(zeroLimit.constructor === Array, "number zero limit constructor");
check(zeroLimit.length === 0, "number zero limit length");

var numberParts = n.split(1);
check(numberParts.constructor === Array, "number split constructor");
check(numberParts.length === 4, "number split length");
check(numberParts[0] === "", "number split first");
check(numberParts[1] === "0", "number split second");
check(numberParts[2] === "20", "number split third");
check(numberParts[3] === "", "number split tail");
n.split(1);
var afterNumber = "none";
if (false) {
  afterNumber = "bad";
}
check(afterNumber === "none", "number completion before if");

var customSeparator = {};
var customReturn = {};
var customThis = undefined;
var customString = undefined;
var customLimit = undefined;
customSeparator[Symbol.split] = function (value, limit) {
  customThis = this;
  customString = value;
  customLimit = limit;
  return customReturn;
};
check("".split(customSeparator, "limit") === customReturn, "custom split return");
check(customThis === customSeparator, "custom split this");
check(customString === "", "custom split string argument");
check(customLimit === "limit", "custom split limit argument");

var nonStringableReceiver = {
  toString: function () {
    throw "receiver toString should not run";
  },
};
var callSeparator = {};
var callReturn = {};
var callReceiver = undefined;
var callLimit = undefined;
callSeparator[Symbol.split] = function (value, limit) {
  callReceiver = value;
  callLimit = limit;
  return callReturn;
};
check(
  String.prototype.split.call(nonStringableReceiver, callSeparator, "limit") ===
    callReturn,
  "split.call custom split return",
);
check(callReceiver === nonStringableReceiver, "split.call custom receiver argument");
check(callLimit === "limit", "split.call custom limit argument");

var separatorToStringCalled = false;
var separatorObject = {
  toString: function () {
    separatorToStringCalled = true;
    return "z";
  },
};
var zeroLimitWithObjectSeparator = "abc".split(separatorObject, 0);
check(separatorToStringCalled, "separator tostring before zero limit");
check(zeroLimitWithObjectSeparator.length === 0, "zero limit after separator tostring");

true;
