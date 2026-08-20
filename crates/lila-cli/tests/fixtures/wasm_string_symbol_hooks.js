var $262 = {
  IsHTMLDDA: __lilaCreateHTMLDDA()
};

var total = 0;

function invoke(name, target) {
  if (name === "match") return "".match(target);
  if (name === "matchAll") return "".matchAll(target);
  if (name === "replace") return "".replace(target);
  if (name === "replaceAll") return "".replaceAll(target);
  if (name === "search") return "".search(target);
  if (name === "split") return "".split(target);
  throw name + " invoke";
}

function check(name, symbol, expectedArgs) {
  var target = $262.IsHTMLDDA;
  var gets = 0;
  Object.defineProperty(target, symbol, {
    get: function() {
      gets += 1;
      return function() {
        if (this !== target) throw name + " this";
        if (arguments.length !== expectedArgs) throw name + " argc";
        if (arguments[0] !== "") throw name + " arg0";
        return null;
      };
    },
    configurable: true
  });
  if (invoke(name, target) !== null) throw name + " result";
  if (gets !== 1) throw name + " getter";
  total += gets;
  delete target[symbol];
}

check("match", Symbol.match, 1);
check("matchAll", Symbol.matchAll, 1);
check("replace", Symbol.replace, 2);
check("replaceAll", Symbol.replace, 2);
check("search", Symbol.search, 1);
check("split", Symbol.split, 2);

if (total !== 6) throw "total";

var primitiveHookReads = 0;
Object.defineProperty(String.prototype, Symbol.match, {
  get: function() {
    primitiveHookReads += 1;
    throw "primitive match hook read";
  },
  configurable: true
});

var primitiveMatch = "a,b,c".match(",");
delete String.prototype[Symbol.match];
if (primitiveHookReads !== 0) throw "primitive match getter";
if (primitiveMatch === null) throw "primitive match null";
if (primitiveMatch.length !== 1) throw "primitive match length";
if (primitiveMatch[0] !== ",") throw "primitive match value";
if (primitiveMatch.index !== 1) throw "primitive match index";
if (primitiveMatch.input !== "a,b,c") throw "primitive match input";

function checkMatchArray(result, value, index, input, label) {
  if (result === null) throw label + " null";
  if (result.length !== 1) throw label + " length";
  if (result[0] !== value) throw label + " value";
  if (result.index !== index) throw label + " index";
  if (result.input !== input) throw label + " input";
}

var boxedBoolean = new Object(true);
boxedBoolean.match = String.prototype.match;
checkMatchArray(boxedBoolean.match(true), "true", 0, "true", "boxed object match");

var falseBoolean = new Boolean;
falseBoolean.match = String.prototype.match;
checkMatchArray(falseBoolean.match(false), "false", 0, "false", "boxed boolean match");

var boxedString = new String("true");
checkMatchArray(boxedString.match(true), "true", 0, "true", "boxed string match");

var nullMatchPattern = {};
nullMatchPattern[Symbol.match] = null;
nullMatchPattern.toString = function() { return "\\d"; };
nullMatchPattern.valueOf = function() { throw "null match valueOf"; };
if ("abc".match(nullMatchPattern) !== null) throw "null match no hit";
checkMatchArray("ab3c".match(nullMatchPattern), "3", 2, "ab3c", "null match digit");

checkMatchArray("ABBABABAB77BBAA".match(new RegExp("77")), "77", 9, "ABBABABAB77BBAA", "cold default regexp match");

var originalRegExpMatch = RegExp.prototype[Symbol.match];
var internalMatchReturn = {};
var internalMatchThis;
var internalMatchArgs;
RegExp.prototype[Symbol.match] = function() {
  internalMatchThis = this;
  internalMatchArgs = arguments;
  return internalMatchReturn;
};
var internalMatchResult = "target".match("string source");
RegExp.prototype[Symbol.match] = originalRegExpMatch;
if (!(internalMatchThis instanceof RegExp)) throw "internal regexp instanceof";
if (internalMatchThis.source !== "string source") throw "internal regexp source";
if (internalMatchThis.flags !== "") throw "internal regexp flags";
if (internalMatchThis.lastIndex !== 0) throw "internal regexp lastIndex";
if (internalMatchArgs.length !== 1) throw "internal regexp args length";
if (internalMatchArgs[0] !== "target") throw "internal regexp arg";
if (internalMatchResult !== internalMatchReturn) throw "internal regexp return";

checkMatchArray("ABB\u0041BABAB\u0037\u0037BBAA".match(new RegExp("77")), "77", 9, "ABBABABAB77BBAA", "default regexp unicode match");
function LocalTest262Error(message) {
  this.message = message;
}
var localSputnikRegExp = new RegExp("77");
if ("ABB\u0041BABAB\u0037\u0037BBAA".match(localSputnikRegExp)[0] !== "77") {
  throw new LocalTest262Error("sputnik regexp match");
}
var __reg = new RegExp("77");
if ("ABB\u0041BABAB\u0037\u0037BBAA".match(__reg)[0] !== "77") {
  throw new LocalTest262Error("sputnik __reg match");
}
checkMatchArray(RegExp().exec(""), "", 0, "", "empty regexp exec");
var undefinedExec = RegExp(undefined).exec("undefined");
checkMatchArray(undefinedExec, "", 0, "undefined", "undefined regexp exec");
var undefinedMatch = new String("undefined").match(undefined);
checkMatchArray(undefinedMatch, "", 0, "undefined", "undefined match");
checkMatchArray("1234567890".match(3), "3", 2, "1234567890", "number match");

var throwingMatchPattern = {
  toString: function() { throw "intostr"; }
};
var caughtMatchThrow = "";
try {
  "ABBABAB".match(throwingMatchPattern);
  caughtMatchThrow = "missing";
} catch (e) {
  caughtMatchThrow = e;
}
if (caughtMatchThrow !== "intostr") throw "match object toString throw " + caughtMatchThrow;

var globalRegexpMatch = "343443444".match(/34/g);
if (globalRegexpMatch.length !== 3) throw "global regexp match length";
if (globalRegexpMatch[0] !== "34") throw "global regexp match 0";
if (globalRegexpMatch[1] !== "34") throw "global regexp match 1";
if (globalRegexpMatch[2] !== "34") throw "global regexp match 2";

var globalDigitMatch = "123456abcde7890".match(/\d{1}/g);
if (globalDigitMatch.length !== 10) throw "global digit match length";
if (globalDigitMatch[0] !== "1") throw "global digit match 0";
if (globalDigitMatch[1] !== "2") throw "global digit match 1";
if (globalDigitMatch[2] !== "3") throw "global digit match 2";
if (globalDigitMatch[3] !== "4") throw "global digit match 3";
if (globalDigitMatch[4] !== "5") throw "global digit match 4";
if (globalDigitMatch[5] !== "6") throw "global digit match 5";
if (globalDigitMatch[6] !== "7") throw "global digit match 6";
if (globalDigitMatch[7] !== "8") throw "global digit match 7";
if (globalDigitMatch[8] !== "9") throw "global digit match 8";
if (globalDigitMatch[9] !== "0") throw "global digit match 9";

var globalDigitPairMatch = "123456abcde7890".match(/\d{2}/g);
if (globalDigitPairMatch.length !== 5) throw "global digit pair match length";
if (globalDigitPairMatch[0] !== "12") throw "global digit pair match 0";
if (globalDigitPairMatch[1] !== "34") throw "global digit pair match 1";
if (globalDigitPairMatch[2] !== "56") throw "global digit pair match 2";
if (globalDigitPairMatch[3] !== "78") throw "global digit pair match 3";
if (globalDigitPairMatch[4] !== "90") throw "global digit pair match 4";

var globalNonDigitPairMatch = "123456abcde7890".match(/\D{2}/g);
if (globalNonDigitPairMatch.length !== 2) throw "global non-digit pair match length";
if (globalNonDigitPairMatch[0] !== "ab") throw "global non-digit pair match 0";
if (globalNonDigitPairMatch[1] !== "cd") throw "global non-digit pair match 1";

262;
