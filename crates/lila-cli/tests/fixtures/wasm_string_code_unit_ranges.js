var ok = true;

function check(value) {
  ok = ok && value;
}

var astral = "\ud834\udf06";
var high = "\ud834";
var low = "\udf06";
var mixed = "x" + astral + "y";

check(astral.slice(0, 1) === high);
check(astral.slice(1, 2) === low);
check(astral.slice(0, 2) === astral);
check(astral.slice(-2, -1) === high);
check(astral.slice(-1) === low);
check(mixed.slice(1, 2) === high);
check(mixed.slice(2, 3) === low);
check(mixed.slice(1, 3) === astral);

check(astral.substring(0, 1) === high);
check(astral.substring(1, 2) === low);
check(astral.substring(0, 2) === astral);
check(astral.substring(1, 0) === high);
check(mixed.substring(1, 2) === high);
check(mixed.substring(2, 3) === low);
check(mixed.substring(3, 1) === astral);

check(String.prototype.slice.call(astral, 0, 1) === high);
check(String.prototype.slice.call(astral, 1, 2) === low);
check(String.prototype.substring.call(astral, 0, 1) === high);
check(String.prototype.substring.call(astral, 1, 2) === low);
check(String.prototype.substring.call("a\u00e9z", 1, 2) === "\u00e9");
check(String.prototype.substring.call("a\u00e9z", 2, 3) === "z");

check("abc".slice(1e100) === "");
check("abc".slice(-1e100) === "abc");
check("abc".slice(0, 1e100) === "abc");
check("abc".slice(0, -1e100) === "");
check("abc".substring(1e100) === "");
check("abc".substring(-1e100) === "abc");
check("abc".substring(1, 1e100) === "bc");
check("abc".substring(1, -1e100) === "a");

try {
  check(astral.substring(0, 1) === high);
  check(astral.substring(1, 2) === low);
} catch (error) {
  ok = false;
}

var lone = "\ud834x\udf06";
check(lone.slice(0, 1) === high);
check(lone.slice(2, 3) === low);
check(lone.substring(0, 1) === high);
check(lone.substring(2, 3) === low);
check("\udf06\ud834".slice(0, 1) === low);
check("\udf06\ud834".substring(1, 2) === high);

check(astral.substr(0, 1) === high);
check(astral.substr(1, 1) === low);
check(astral.substr(0, 2) === astral);
check(mixed.substr(1, 1) === high);
check(mixed.substr(2, 1) === low);
check(mixed.substr(1, 2) === astral);

var events = "";
var receiver = {
  substring: String.prototype.substring,
  toString: function() {
    events += "R";
    return astral;
  }
};

function makeReceiver() {
  events += "M";
  return receiver;
}

function startExpression() {
  events += "S";
  return 0;
}

function endExpression() {
  events += "E";
  return 1;
}

check(makeReceiver().substring(startExpression(), endExpression()) === high);
check(events === "MSER");

events = "";
var marker = {};
try {
  makeReceiver().substring(function() {
    events += "X";
    throw marker;
  }(), 1);
  ok = false;
} catch (error) {
  check(error === marker);
  check(events === "MX");
}

var other = __lilaCreateRealm().global;
var otherSliceError;
var otherSubstringError;
try {
  other.String.prototype.slice.call(null, 0, 1);
} catch (error) {
  otherSliceError = error;
}
try {
  other.String.prototype.substring.call(undefined, 0, 1);
} catch (error) {
  otherSubstringError = error;
}
check(Object.getPrototypeOf(otherSliceError) === other.TypeError.prototype);
check(Object.getPrototypeOf(otherSubstringError) === other.TypeError.prototype);
check(Object.getPrototypeOf(otherSliceError) !== TypeError.prototype);
check(Object.getPrototypeOf(otherSubstringError) !== TypeError.prototype);

ok;
