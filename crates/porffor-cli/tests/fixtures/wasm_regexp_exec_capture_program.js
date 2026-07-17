function check(actual, expected, label) { if (actual !== expected) throw label; }
var global = /(\d+)/g;
var first = global.exec("12x345y6");
check(first.length, 2, "global length");
check(first[0], "12", "global full");
check(first[1], "12", "global capture");
check(first.index, 0, "index");
check(first.input, "12x345y6", "input");
check(first.groups, undefined, "groups");
check(global.lastIndex, 2, "lastIndex");
var second = global.exec("12x345y6");
check(second[1], "345", "second capture");
check(global.lastIndex, 6, "second lastIndex");
var third = global.exec("12x345y6");
check(third[1], "6", "third capture");
check(global.test("12x345y6"), false, "global reset");
check(global.lastIndex, 0, "reset index");
var nested = /x(a)(\d)y/.exec("xa2y");
check(nested.length, 3, "nested length");
check(nested[1], "a", "nested one");
check(nested[2], "2", "nested two");
check(/(\d*)\d/.exec("1")[1], "", "fallback capture");
var astral = /(a)/.exec("𝌆a");
check(astral.index, 2, "astral prefix index");
check(astral[1], "a", "astral prefix");
check(/(a)/.test("a"), true, "test capture state");
var dirty = /a*/.exec("aaaaaaaaaaaaaaaaaaaaaaaa");
check(dirty[1], undefined, "no capture result");
check(new RegExp("b").test("b"), true, "post scratch allocation");
check(({}).toString(), "[object Object]", "ordinary allocation");
var noMatchCarrier = /(z)/;
for (var carrierAttempt = 0; carrierAttempt < 200; carrierAttempt = carrierAttempt + 1) {
  check(noMatchCarrier.exec("aaaaaaaa") === null, true, "captured no-match");
}
var recoveredCarrier = noMatchCarrier.exec("z");
check(recoveredCarrier.length, 2, "carrier recovery length");
check(recoveredCarrier[1], "z", "carrier recovery capture");
var nonWritableLastIndex = /(a)/g;
Object.defineProperty(nonWritableLastIndex, "lastIndex", { writable: false });
var lastIndexThrows = 0;
for (var writeAttempt = 0; writeAttempt < 500; writeAttempt = writeAttempt + 1) {
  try {
    nonWritableLastIndex.exec("a");
  } catch (error) {
    lastIndexThrows = lastIndexThrows + 1;
  }
}
check(lastIndexThrows, 500, "captured lastIndex write throws");
check(new Array(128).length, 128, "allocation after captured write throws");
RegExp = function () { throw "overwritten"; };
check(/(a)/.exec("a")[1], "a", "intrinsic literal");
var alt = /((1)|(12))((3)|(23))/.exec(new String("123"));
check(JSON.stringify(alt), JSON.stringify(["123", "1", "1", undefined, "23", undefined, "23"]), "nested alternation captures");
var quantified = /(aa|aabaac|ba|b|c)*/.exec({ toString: function () { return {}; }, valueOf: function () { return "aabaac"; } });
check(quantified[0], "aaba", "quantified full");
check(quantified[1], "ba", "quantified capture");
var nestedQuantified = /(z)((a+)?(b+)?(c))*/.exec("zaacbbbcac");
check(JSON.stringify(nestedQuantified), JSON.stringify(["zaacbbbcac", "z", "ac", "a", undefined, "c"]), "nested quantified captures");
var staleClear = /(a(b)?)+/.exec("aba");
check(staleClear[0], "aba", "stale clear full");
check(staleClear[1], "a", "stale clear group one");
check(staleClear[2], undefined, "stale clear group two");
true;
