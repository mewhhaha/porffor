function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

var plainStringSearch;
try {
  plainStringSearch = "target".search("arg");
} catch (error) {
  throw "plain string search threw";
}
check(plainStringSearch, 1, "plain string creates regexp");
var undefinedSearch;
try {
  undefinedSearch = "target".search(undefined);
} catch (error) {
  throw "undefined search threw";
}
check(undefinedSearch, 0, "undefined creates empty regexp");
var ignoreCaseSearch;
try {
  ignoreCaseSearch = new String("test string").search(/String/i);
} catch (error) {
  throw "ignoreCase regexp search threw";
}
check(ignoreCaseSearch, 5, "regexp ignoreCase literal search");

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
