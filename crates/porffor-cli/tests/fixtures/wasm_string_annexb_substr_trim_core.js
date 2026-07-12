var ok = true;

function check(value) {
  ok = ok && value;
}

function checkMethod(property, expectedName, expectedLength, fn) {
  var desc = Object.getOwnPropertyDescriptor(String.prototype, property);
  check(typeof fn === "function");
  check(fn.name === expectedName);
  check(fn.length === expectedLength);
  check(desc.value === fn);
  check(desc.writable === true);
  check(desc.enumerable === false);
  check(desc.configurable === true);
}

checkMethod("substr", "substr", 2, String.prototype.substr);
checkMethod("trim", "trim", 0, String.prototype.trim);
checkMethod("trimStart", "trimStart", 0, String.prototype.trimStart);
checkMethod("trimLeft", "trimStart", 0, String.prototype.trimLeft);
checkMethod("trimEnd", "trimEnd", 0, String.prototype.trimEnd);
checkMethod("trimRight", "trimEnd", 0, String.prototype.trimRight);

check(String.prototype.trimLeft === String.prototype.trimStart);
check(String.prototype.trimRight === String.prototype.trimEnd);

var $262 = { createRealm: __porfCreateRealm };
var otherRealm = $262.createRealm().global;
check(typeof otherRealm.String.prototype.trimLeft === "function");
check(typeof otherRealm.String.prototype.trimRight === "function");
check(otherRealm.String.prototype.trimLeft === otherRealm.String.prototype.trimStart);
check(otherRealm.String.prototype.trimRight === otherRealm.String.prototype.trimEnd);
check(otherRealm.String.prototype.trimLeft.name === "trimStart");
check(otherRealm.String.prototype.trimRight.name === "trimEnd");
check(otherRealm.String.prototype.trimLeft.length === 0);
check(otherRealm.String.prototype.trimRight.length === 0);

check("abcdef".substr(1, 3) === "bcd");
check("abcdef".substr(-2) === "ef");
check("abcdef".substr(-20, 2) === "ab");
check("abcdef".substr(2) === "cdef");
check("abcdef".substr(2, undefined) === "cdef");
check("abcdef".substr(2, 0) === "");
check("abcdef".substr(2, -1) === "");
check("abcdef".substr() === "abcdef");
check("a\u{1D306}b".substr(0) === "a\u{1D306}b");
check("a\u{1D306}b".substr(1, 2) === "\u{1D306}");
check("a\u{1D306}b".substr(3) === "b");
check("a\u{1D306}b".substr(-1) === "b");
check("a\u{1D306}b".substr(-3, 2) === "\u{1D306}");
var astral = "\u{1D306}";
check(astral.substr(0, 1) === "\ud834");
check(astral.substr(1, 1) === "\udf06");
check(astral.substr(0, 2) === astral);
check(("x" + astral + "y").substr(1, 1) === "\ud834");
check(("x" + astral + "y").substr(2, 1) === "\udf06");
check(("x" + astral + "y").substr(1, 2) === astral);
check(String.prototype.substr.call("\u00e9" + astral + "z", 1, 1) === "\ud834");
check(String.prototype.substr.call("\u00e9" + astral + "z", 2, 1) === "\udf06");
var loneSurrogates = "\ud834x\udf06";
check(loneSurrogates.substr(0, 1) === "\ud834");
check(loneSurrogates.substr(1, 1) === "x");
check(loneSurrogates.substr(2, 1) === "\udf06");
check(loneSurrogates.substr(0) === loneSurrogates);

check(" \t\nabc\r ".trimStart() === "abc\r ");
check(" \t\nabc\r ".trimLeft() === "abc\r ");
check(" \t\nabc\r ".trimEnd() === " \t\nabc");
check(" \t\nabc\r ".trimRight() === " \t\nabc");
check(" \t\nabc\r ".trim() === "abc");
check("\u00a0\u2000abc\u2029\ufeff".trim() === "abc");
check("_\u180e".trim() === "_\u180e");
check("\u180e".trim() === "\u180e");
check("\u180e_".trim() === "\u180e_");

try {
  String.prototype.substr.call(null, 0, 1);
  check(false);
} catch (e) {
  check(e instanceof TypeError);
}

try {
  String.prototype.trimLeft.call(undefined);
  check(false);
} catch (e) {
  check(e instanceof TypeError);
}

try {
  String.prototype.trim.call(undefined);
  check(false);
} catch (e) {
  check(e instanceof TypeError);
}

var marker = {};
try {
  String.prototype.substr.call({
    toString: function() {
      throw marker;
    },
  }, 0, 1);
  check(false);
} catch (e) {
  check(e === marker);
}

try {
  "abcdef".substr({
    valueOf: function() {
      throw marker;
    },
  }, 1);
  check(false);
} catch (e) {
  check(e === marker);
}

try {
  "abcdef".substr(1, {
    valueOf: function() {
      throw marker;
    },
  });
  check(false);
} catch (e) {
  check(e === marker);
}

ok;
