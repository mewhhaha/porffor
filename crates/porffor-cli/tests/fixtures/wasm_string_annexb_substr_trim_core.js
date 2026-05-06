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
checkMethod("trimStart", "trimStart", 0, String.prototype.trimStart);
checkMethod("trimLeft", "trimStart", 0, String.prototype.trimLeft);
checkMethod("trimEnd", "trimEnd", 0, String.prototype.trimEnd);
checkMethod("trimRight", "trimEnd", 0, String.prototype.trimRight);

check(String.prototype.trimLeft === String.prototype.trimStart);
check(String.prototype.trimRight === String.prototype.trimEnd);

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

check(" \t\nabc\r ".trimStart() === "abc\r ");
check(" \t\nabc\r ".trimLeft() === "abc\r ");
check(" \t\nabc\r ".trimEnd() === " \t\nabc");
check(" \t\nabc\r ".trimRight() === " \t\nabc");

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
