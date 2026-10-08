function check(actual, expected, label) {
  if (actual !== expected) {
    throw label;
  }
}

var savedFlagsDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags");
var savedMatchAllDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, Symbol.matchAll);
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
Object.defineProperty(RegExp.prototype, "flags", savedFlagsDescriptor);
Object.defineProperty(RegExp.prototype, Symbol.matchAll, savedMatchAllDescriptor);

// An absent inherited method on the original RegExp is read once. Invoke
// subsequently reads the same property with the distinct created receiver.
for (var absentIndex = 0; absentIndex < 2; absentIndex++) {
  var original = /b/g;
  var originalReads = 0, createdReads = 0, proxyCalls = 0, receiverReads = 0;
  var created, createdResult = {}, originalFlagsReads = 0, patternReads = 0;
  Object.defineProperty(original, "flags", { configurable: true, get: function() {
    originalFlagsReads++;
    return "g";
  }});
  Object.defineProperty(original, "toString", { value: function() {
    patternReads++;
    return "b";
  }});
  var originalReceiver = { toString: function() { receiverReads++; return "abc"; } };
  var createdTarget = function() { throw "created matchAll target called"; };
  var createdProxy = new Proxy(createdTarget, { apply: function(target, receiver, args) {
    proxyCalls++;
    if (target !== createdTarget || receiver !== created || receiver === original ||
        Object.getPrototypeOf(receiver) !== RegExp.prototype || args.length !== 1 ||
        args[0] !== "abc" || receiver.source !== "b" || receiver.flags !== "g") throw "created matchAll Proxy arguments";
    return createdResult;
  }});
  Object.defineProperty(RegExp.prototype, Symbol.matchAll, { configurable: true, get: function() {
    if (this === original) {
      originalReads++;
      return absentIndex === 0 ? null : undefined;
    }
    createdReads++;
    created = this;
    return createdProxy;
  }});
  try {
    check(String.prototype.matchAll.call(originalReceiver, original), createdResult, "inherited absent result");
    check(originalReads, 1, "original inherited method read once");
    check(createdReads, 1, "created required method read once");
    check(proxyCalls, 1, "created Proxy called once");
    check(receiverReads, 1, "receiver converted only in fallback");
    check(originalFlagsReads, 1, "original flags validation once");
    check(patternReads, 1, "pattern converted once after absent method");
  } finally { Object.defineProperty(RegExp.prototype, Symbol.matchAll, savedMatchAllDescriptor); }
}

var savedTypeErrorPrototype = TypeError.prototype;
for (var requiredIndex = 0; requiredIndex < 2; requiredIndex++) {
  Object.defineProperty(RegExp.prototype, Symbol.matchAll, { configurable: true,
    value: requiredIndex === 0 ? null : undefined });
  var requiredCaught = false;
  try { String.prototype.matchAll.call("abc", "b"); }
  catch (error) { requiredCaught = Object.getPrototypeOf(error) === savedTypeErrorPrototype; }
  finally { Object.defineProperty(RegExp.prototype, Symbol.matchAll, savedMatchAllDescriptor); }
  if (!requiredCaught) throw "created matchAll nullish Invoke must throw";
}

// MatchAll and ReplaceAll finish IsRegExp and flags validation before GetMethod.
var orderedEntries = [[String.prototype.matchAll, Symbol.matchAll, 1],
  [String.prototype.replaceAll, Symbol.replace, 2]];
var orderedMarker = {};
for (var orderedIndex = 0; orderedIndex < orderedEntries.length; orderedIndex++) {
  var orderedEntry = orderedEntries[orderedIndex];
  var orderedPattern = {}, orderedReceiver = { toString: function() { throw "ordered receiver conversion"; } };
  var secondArgument = {}, orderedResult = {}, order = "";
  var orderedTarget = function() { throw "ordered target called"; };
  var orderedHook = new Proxy(orderedTarget, { apply: function(target, receiver, args) {
    order += "apply;";
    if (target !== orderedTarget || receiver !== orderedPattern || args.length !== orderedEntry[2] ||
        args[0] !== orderedReceiver || (args.length === 2 && args[1] !== secondArgument)) {
      throw "ordered Proxy arguments";
    }
    return orderedResult;
  }});
  Object.defineProperty(orderedPattern, Symbol.match, { get: function() {
    order += "isRegExp;";
    return true;
  }});
  Object.defineProperty(orderedPattern, "flags", { configurable: true, get: function() {
    order += "flags;";
    return { toString: function() { order += "flags-string;"; return "g"; } };
  }});
  Object.defineProperty(orderedPattern, orderedEntry[1], { get: function() {
    order += "hook;";
    return orderedHook;
  }});
  check(orderedEntry[0].call(orderedReceiver,
    (order += "pattern;", orderedPattern), (order += "second;", secondArgument)),
    orderedResult, "ordered result");
  check(order, "pattern;second;isRegExp;flags;flags-string;hook;apply;", "flags before hook order");

  order = "";
  Object.defineProperty(orderedPattern, "flags", { configurable: true, get: function() {
    order += "flags;";
    return "i";
  }});
  var missingGlobalCaught = false;
  try { orderedEntry[0].call(orderedReceiver, orderedPattern, secondArgument); }
  catch (error) { missingGlobalCaught = Object.getPrototypeOf(error) === savedTypeErrorPrototype; }
  if (!missingGlobalCaught || order !== "isRegExp;flags;") throw "missing global before hook";

  order = "";
  Object.defineProperty(orderedPattern, "flags", { configurable: true, get: function() {
    order += "flags;";
    return { toString: function() { order += "flags-string;"; throw orderedMarker; } };
  }});
  var flagsMarkerCaught = false;
  try { orderedEntry[0].call(orderedReceiver, orderedPattern, secondArgument); }
  catch (error) { flagsMarkerCaught = error === orderedMarker; }
  if (!flagsMarkerCaught || order !== "isRegExp;flags;flags-string;") throw "flags original abrupt before hook";
}

true;
