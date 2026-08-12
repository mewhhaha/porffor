function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

var callCount = 0;
var callArg0;
var callArg1;
var speciesGetterCalls = 0;
var regexp = /\d/u;
regexp.constructor = RegExp;
var arbitraryExecCalls = 0;
var arbitraryMatcher = {
  lastIndex: 0,
  exec: function (value) {
    arbitraryExecCalls = arbitraryExecCalls + 1;
    check(value, input, "arbitrary exec input");
    return null;
  }
};
var species = function (pattern, flags) {
  callCount = callCount + 1;
  callArg0 = pattern;
  callArg1 = flags;
  return arbitraryMatcher;
};
Object.defineProperty(RegExp, Symbol.species, {
  configurable: true,
  get: function () {
    speciesGetterCalls = speciesGetterCalls + 1;
    return species;
  }
});

var input = "a*b";
var iterator = regexp[Symbol.matchAll](input);
check(speciesGetterCalls, 1, "species getter count");
check(callCount, 1, "call count");
check(callArg0 === regexp, true, "arg0");
check(callArg1, "u", "arg1");
var result = iterator.next();
check(result.done, true, "arbitrary matcher done");
check(arbitraryExecCalls, 1, "arbitrary exec calls");
true;
