function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function checkMatchArray(result, value, index, input, label) {
  check(Array.isArray(result), true, label + " array");
  check(result[0], value, label + " value");
  check(result.index, index, label + " index");
  check(result.input, input, label + " input");
}

var symbolMatchInput = "ab";
var symbolMatch = symbolMatchInput.match(/a(b)?/);
checkMatchArray(symbolMatch, "ab", 0, symbolMatchInput, "symbol match program");
check(symbolMatch[1], "b", "symbol match capture");

var programInput = "zab";
var programExec = /a(b)?/g;
var programResult = programExec.exec(programInput);
checkMatchArray(programResult, "ab", 1, programInput, "exec program");
check(programResult[1], "b", "exec program capture");
check(programExec.lastIndex, 3, "exec program lastIndex");
check(programExec.exec(programInput), null, "exec program failure");
check(programExec.lastIndex, 0, "exec program failure lastIndex");

var programTest = /a(b)?/g;
check(programTest.test(programInput), true, "test program success");
check(programTest.lastIndex, 3, "test program lastIndex");
check(programTest.test("zzz"), false, "test program failure");
check(programTest.lastIndex, 0, "test program failure lastIndex");

var simpleSource = String.fromCharCode(113);
var simpleInput = String.fromCharCode(120, 113);
var simpleExec = new RegExp(simpleSource, "g");
var simpleResult = simpleExec.exec(simpleInput);
check(Array.isArray(simpleResult), true, "exec simple array");
check(simpleResult[0].charCodeAt(0), 113, "exec simple value");
check(simpleResult.index, 1, "exec simple index");
check(simpleResult.input, simpleInput, "exec simple input");
check(simpleExec.lastIndex, 2, "exec simple lastIndex");

var simpleTest = new RegExp(simpleSource, "g");
check(simpleTest.test(String.fromCharCode(120, 120)), false, "test simple failure");
check(simpleTest.lastIndex, 0, "test simple failure lastIndex");

var emptyParts = ["(?:", ")"];
var emptySource = emptyParts[0] + emptyParts[1];
var legacyExec = new RegExp(emptySource, "g");
var legacyResult = legacyExec.exec("x");
checkMatchArray(legacyResult, "", 0, "x", "exec legacy fallback");
check(legacyExec.lastIndex, 0, "exec legacy fallback lastIndex");

var legacyTest = new RegExp(emptySource, "g");
check(legacyTest.test("x"), true, "test legacy fallback");
check(legacyTest.lastIndex, 0, "test legacy fallback lastIndex");

// A compiled source must not divert @@match into a scalar-only catalogue.
// Without u/v, two NonDigit atoms consume two UTF-16 units: the astral pair.
var nonDigitLegacy = /\D{2}/g;
var nonDigitLegacyMatches = "\uD83D\uDE00a".match(nonDigitLegacy);
check(nonDigitLegacyMatches.length, 1, "compiled non-digit legacy count");
check(nonDigitLegacyMatches[0], "\uD83D\uDE00", "compiled non-digit legacy UTF-16");
check(nonDigitLegacy.lastIndex, 0, "compiled non-digit legacy final lastIndex");

var nonDigitUnicode = /\D{2}/gu;
var nonDigitUnicodeMatches = "\uD83D\uDE00a".match(nonDigitUnicode);
check(nonDigitUnicodeMatches.length, 1, "compiled non-digit unicode count");
check(nonDigitUnicodeMatches[0], "\uD83D\uDE00a", "compiled non-digit unicode code points");
check(nonDigitUnicode.lastIndex, 0, "compiled non-digit unicode final lastIndex");

// Sticky matching starts at the reset lastIndex and cannot search past a digit.
var nonDigitSticky = /\D{2}/gy;
nonDigitSticky.lastIndex = 2;
check("0ab".match(nonDigitSticky), null, "compiled non-digit sticky failure");
check(nonDigitSticky.lastIndex, 0, "compiled non-digit sticky final lastIndex");
var nonDigitScanning = /\D{2}/g;
check("0ab".match(nonDigitScanning)[0], "ab", "compiled non-digit ordinary search");
check(nonDigitScanning.lastIndex, 0, "compiled non-digit search final lastIndex");

// @@match observes flags; an own flags getter does not invoke global or source.
var observedFlags = "";
var nonDigitFlagsGetter = /\D{2}/g;
Object.defineProperty(nonDigitFlagsGetter, "flags", {
  get: function () { observedFlags += "flags;"; return "g"; }
});
Object.defineProperty(nonDigitFlagsGetter, "global", {
  get: function () { throw "unexpected global getter"; }
});
Object.defineProperty(nonDigitFlagsGetter, "source", {
  get: function () { throw "unexpected source getter"; }
});
check("abcd".match(nonDigitFlagsGetter).length, 2, "compiled non-digit flags getter matches");
check(observedFlags, "flags;", "compiled non-digit flags getter order");
check(nonDigitFlagsGetter.lastIndex, 0, "compiled non-digit flags getter lastIndex");

// The observable flags string chooses the @@match result shape, while exec
// retains the RegExp's original g flag and therefore advances lastIndex.
var nonDigitFlagsOverride = /\D{2}/g;
Object.defineProperty(nonDigitFlagsOverride, "flags", { value: "" });
var nonDigitSingle = "abcd".match(nonDigitFlagsOverride);
checkMatchArray(nonDigitSingle, "ab", 0, "abcd", "compiled non-digit flags override");
check(nonDigitSingle.length, 1, "compiled non-digit flags override single result");
check(nonDigitFlagsOverride.lastIndex, 2, "compiled non-digit flags override lastIndex");

// A callable own exec still owns matching and is fetched on every iteration.
var nonDigitCustomExec = /\D{2}/g;
var customExecGets = 0;
var customExecCalls = 0;
Object.defineProperty(nonDigitCustomExec, "exec", {
  get: function () {
    customExecGets += 1;
    return function (input) {
      check(this, nonDigitCustomExec, "compiled non-digit custom exec receiver");
      check(input, "abcd", "compiled non-digit custom exec input");
      customExecCalls += 1;
      return customExecCalls === 1 ? { 0: "custom" } : null;
    };
  }
});
var nonDigitCustomMatches = "abcd".match(nonDigitCustomExec);
check(nonDigitCustomMatches.length, 1, "compiled non-digit custom exec result count");
check(nonDigitCustomMatches[0], "custom", "compiled non-digit custom exec result");
check(customExecGets, 2, "compiled non-digit custom exec repeated Get");
check(customExecCalls, 2, "compiled non-digit custom exec repeated Call");
check(nonDigitCustomExec.lastIndex, 0, "compiled non-digit custom exec lastIndex");

// Input coercion can replace source and flags before matching reads them.
var recompiledDuringInput = /\D{2}/g;
var recompilingInput = {
  toString: function () { recompiledDuringInput.compile("a", "gi"); return "AA"; }
};
var recompiledMatches = RegExp.prototype[Symbol.match].call(recompiledDuringInput, recompilingInput);
check(recompiledMatches.length, 2, "compiled non-digit input recompile count");
check(recompiledMatches[0], "A", "compiled non-digit input recompile first");
check(recompiledMatches[1], "A", "compiled non-digit input recompile second");
check(recompiledDuringInput.lastIndex, 0, "compiled non-digit input recompile lastIndex");

// Computed patterns must invoke the created RegExp's intrinsic @@match.
var computedPlus = String.fromCharCode(97, 43);
var computedPlusMatch = "xaaay".match(computedPlus);
checkMatchArray(computedPlusMatch, "aaa", 1, "xaaay", "created intrinsic plus");
check(computedPlusMatch.length, 1, "created intrinsic plus length");
var computedCapture = String.fromCharCode(40, 97, 41, 43);
var computedCaptureMatch = "xaaay".match(computedCapture);
checkMatchArray(computedCaptureMatch, "aaa", 1, "xaaay", "created intrinsic capture");
check(computedCaptureMatch[1], "a", "created intrinsic capture group");
check(computedCaptureMatch.length, 2, "created intrinsic capture length");

// A noncallable exec on a real RegExp uses its current intrinsic matcher.
var nullExecGlobal = /a/g;
nullExecGlobal.exec = null;
nullExecGlobal.lastIndex = 8;
var nullExecGlobalMatch = RegExp.prototype[Symbol.match].call(nullExecGlobal, "aba");
check(nullExecGlobalMatch.length, 2, "null exec global count");
check(nullExecGlobalMatch[0], "a", "null exec global first");
check(nullExecGlobalMatch[1], "a", "null exec global second");
check(nullExecGlobal.lastIndex, 0, "null exec global final lastIndex");
var numberExec = /a/;
numberExec.exec = 7;
checkMatchArray(RegExp.prototype[Symbol.match].call(numberExec, "za"), "a", 1, "za", "number exec intrinsic");
var objectExec = /a/g;
objectExec.exec = {};
check(RegExp.prototype[Symbol.match].call(objectExec, "za")[0], "a", "object exec intrinsic");
check(objectExec.lastIndex, 0, "object exec final lastIndex");

function expectTypeError(callback, label) {
  var caught = false;
  try { callback(); } catch (error) { caught = error instanceof TypeError; }
  check(caught, true, label);
}

// Non-RegExp receivers may supply a callable exec; brand is checked only on fallback.
var ordinaryResult = { 0: "ordinary", marker: 262 };
var ordinaryExecGets = 0;
var ordinaryExecCalls = 0;
var ordinaryReceiver = { flags: "" };
Object.defineProperty(ordinaryReceiver, "exec", {
  get: function () {
    ordinaryExecGets += 1;
    return function (input) {
      ordinaryExecCalls += 1;
      check(this, ordinaryReceiver, "ordinary exec receiver");
      check(input, "input", "ordinary exec input");
      return ordinaryResult;
    };
  }
});
check(RegExp.prototype[Symbol.match].call(ordinaryReceiver, "input"), ordinaryResult, "ordinary exec object identity");
check(ordinaryExecGets, 1, "ordinary exec single Get");
check(ordinaryExecCalls, 1, "ordinary exec single Call");
check(RegExp.prototype[Symbol.match].call({ flags: "", exec: function () { return null; } }, "input"), null, "ordinary exec null");
var returnedFunction = function () {};
check(RegExp.prototype[Symbol.match].call({ flags: "", exec: function () { return returnedFunction; } }, "input"), returnedFunction, "ordinary exec function is object");
expectTypeError(function () {
  RegExp.prototype[Symbol.match].call({ flags: "", exec: null }, "input");
}, "ordinary null exec brand TypeError");
expectTypeError(function () {
  RegExp.prototype[Symbol.match].call({ flags: "", exec: {} }, "input");
}, "ordinary object exec brand TypeError");
expectTypeError(function () {
  RegExp.prototype[Symbol.match].call({ flags: "", exec: function () { return 1; } }, "input");
}, "ordinary primitive result TypeError");
expectTypeError(function () {
  RegExp.prototype[Symbol.match].call({ flags: "g", exec: function () { return "a"; } }, "input");
}, "global primitive result TypeError");

// Get(exec) occurs once after input/flags; abrupt getters are propagated unchanged.
var execOrder = "";
var getterSentinel = {};
var getterReceiver = {};
Object.defineProperty(getterReceiver, "flags", {
  get: function () { execOrder += "flags;"; return ""; }
});
Object.defineProperty(getterReceiver, "exec", {
  get: function () { execOrder += "exec;"; throw getterSentinel; }
});
var observedGetterThrow = null;
try {
  RegExp.prototype[Symbol.match].call(getterReceiver, {
    toString: function () { execOrder += "input;"; return "input"; }
  });
} catch (error) { observedGetterThrow = error; }
check(observedGetterThrow, getterSentinel, "exec getter throw identity");
check(execOrder, "input;flags;exec;", "exec getter observable order");

var getterRecompile = /a/;
var recompileGets = 0;
Object.defineProperty(getterRecompile, "exec", {
  get: function () {
    recompileGets += 1;
    getterRecompile.compile("b", "g");
    return null;
  }
});
checkMatchArray(RegExp.prototype[Symbol.match].call(getterRecompile, "ab"), "b", 1, "ab", "exec getter recompile intrinsic");
check(recompileGets, 1, "exec getter recompile single Get");
check(getterRecompile.lastIndex, 2, "exec getter recompile new global lastIndex");

// IsCallable admits callable proxies and does not unwrap a RegExp proxy's brand.
var proxyExecCalls = 0;
var proxyExecReceiver = { flags: "" };
proxyExecReceiver.exec = new Proxy(function (input) {
  proxyExecCalls += 1;
  check(this, proxyExecReceiver, "proxy exec receiver");
  check(input, "input", "proxy exec input");
  return ordinaryResult;
}, {});
check(RegExp.prototype[Symbol.match].call(proxyExecReceiver, "input"), ordinaryResult, "proxy exec callable result");
check(proxyExecCalls, 1, "proxy exec single Call");
var classExecReceiver = /a/;
classExecReceiver.exec = class ExecConstructor {};
expectTypeError(function () {
  RegExp.prototype[Symbol.match].call(classExecReceiver, "a");
}, "class exec throws through Call instead of intrinsic fallback");
var revokedExecReceiver = /a/;
var revokedExec = Proxy.revocable(function () { return null; }, {});
revokedExecReceiver.exec = revokedExec.proxy;
revokedExec.revoke();
expectTypeError(function () {
  RegExp.prototype[Symbol.match].call(revokedExecReceiver, "a");
}, "revoked callable exec throws through Call instead of intrinsic fallback");
var proxyRegExp = new Proxy(/a/, {
  get: function (target, key) {
    if (key === "flags") return "";
    if (key === "exec") return null;
    return target[key];
  }
});
expectTypeError(function () {
  RegExp.prototype[Symbol.match].call(proxyRegExp, "a");
}, "proxy RegExp has no intrinsic matcher brand");

// String.match's synthetic RegExp Invoke observes even an overridden prototype hook.
var originalMatchDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, Symbol.match);
var createdHookOrder = "";
var createdHookCalls = 0;
Object.defineProperty(RegExp.prototype, Symbol.match, {
  configurable: true,
  get: function () {
    createdHookOrder += "hook;";
    return new Proxy(function (input) {
      createdHookCalls += 1;
      check(this instanceof RegExp, true, "created hook RegExp receiver");
      check(input, "xaaay", "created hook coerced input");
      return 71;
    }, {});
  }
});
var createdHookResult = String.prototype.match.call({
  toString: function () { createdHookOrder += "input;"; return "xaaay"; }
}, {
  toString: function () { createdHookOrder += "pattern;"; return computedPlus; }
});
Object.defineProperty(RegExp.prototype, Symbol.match, originalMatchDescriptor);
check(createdHookResult, 71, "created hook arbitrary call result");
check(createdHookCalls, 1, "created hook single Call");
check(createdHookOrder, "input;pattern;hook;", "created hook coercion and Get order");

Object.defineProperty(RegExp.prototype, Symbol.match, { configurable: true, value: null });
expectTypeError(function () { "xaaay".match(computedPlus); }, "created hook null TypeError");
Object.defineProperty(RegExp.prototype, Symbol.match, originalMatchDescriptor);

262;
