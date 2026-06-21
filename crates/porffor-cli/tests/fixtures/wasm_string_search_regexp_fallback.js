function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

check("target".search("arg"), 1, "plain string creates regexp");
check("target".search(undefined), 0, "undefined creates empty regexp");
check(new String("test string").search(/String/i), 5, "regexp ignoreCase literal search");

var nullSearch = {};
nullSearch[Symbol.search] = null;
nullSearch.toString = function() {
  return "\\d";
};
nullSearch.valueOf = function() {
  throw "null search fallback used valueOf";
};

check("abc".search(nullSearch), -1, "null @@search fallback miss");
check("ab3c".search(nullSearch), 2, "null @@search fallback hit");

var originalSearch = RegExp.prototype[Symbol.search];
var returnVal = {};
var result;
var thisVal;
var args;

RegExp.prototype[Symbol.search] = function() {
  thisVal = this;
  args = arguments;
  return returnVal;
};

try {
  result = "target".search("string source");
  if (result !== returnVal) throw "override result";
  if (!(thisVal instanceof RegExp)) throw "override this";
  if (thisVal.source !== "string source") throw "override source";
  if (thisVal.flags !== "") throw "override flags";
  if (args.length !== 1) throw "override argc";
  if (args[0] !== "target") throw "override arg0";
} finally {
  RegExp.prototype[Symbol.search] = originalSearch;
}

true;
