function check(actual, expected, label) {
  if (actual !== expected) {
    throw label;
  }
}

var flagsReads = 0;
Object.defineProperty(RegExp.prototype, "flags", {
  configurable: true,
  get: function () {
    flagsReads = flagsReads + 1;
    if (flagsReads > 1) {
      throw "duplicate flags read";
    }
    return "g";
  }
});

var callCount = 0;
var callArg;
var expectedResult = {};
RegExp.prototype[Symbol.matchAll] = function (value) {
  callCount = callCount + 1;
  callArg = value;
  return expectedResult;
};

var result = String.prototype.matchAll.call("x", /./g);
check(result, expectedResult, "result");
check(flagsReads, 1, "flags reads");
check(callCount, 1, "call count");
check(callArg, "x", "call arg");
true;
