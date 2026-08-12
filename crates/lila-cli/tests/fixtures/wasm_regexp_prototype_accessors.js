let re = new RegExp("a", "dgimsyu");
let ownNames = Object.getOwnPropertyNames(re);
let lastIndexDescriptor = Object.getOwnPropertyDescriptor(re, "lastIndex");

let sourceDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "source");
let globalDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "global");
let unicodeDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "unicode");
let unicodeSetsDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "unicodeSets");

let invalidReceiverThrows = false;
try {
  globalDescriptor.get.call({});
} catch (error) {
  invalidReceiverThrows = error instanceof TypeError;
}

let beforeProtocolOverride = /a/;
let originalMatch = RegExp.prototype[Symbol.match];
let protocolMarker = {};
RegExp.prototype[Symbol.match] = function () { return protocolMarker; };
let protocolOverrideObserved = beforeProtocolOverride[Symbol.match]("a") === protocolMarker;
RegExp.prototype[Symbol.match] = originalMatch;

let beforeGlobalOverride = /a/g;
beforeGlobalOverride.lastIndex = 0;
let originalGlobal = Object.getOwnPropertyDescriptor(RegExp.prototype, "global");
Object.defineProperty(RegExp.prototype, "global", {
  get() { return false; },
  configurable: true,
});
let globalOverrideResult = beforeGlobalOverride[Symbol.match]("aa");
Object.defineProperty(RegExp.prototype, "global", originalGlobal);

let originalSource = Object.getOwnPropertyDescriptor(RegExp.prototype, "source");
Object.defineProperty(RegExp.prototype, "source", {
  value: "overridden",
  configurable: true,
});
let sourceOverrideObserved = re.source === "overridden";
Object.defineProperty(RegExp.prototype, "source", originalSource);

let v = new RegExp("v", "gv");

ownNames.length === 1
  && Object.hasOwn(re, "lastIndex")
  && !Object.hasOwn(re, "source")
  && !Object.hasOwn(re, "flags")
  && !Object.hasOwn(re, "global")
  && !Object.hasOwn(re, "unicode")
  && !Object.hasOwn(re, Symbol.match)
  && !Object.hasOwn(re, Symbol.search)
  && lastIndexDescriptor.value === 0
  && lastIndexDescriptor.writable === true
  && lastIndexDescriptor.enumerable === false
  && lastIndexDescriptor.configurable === false
  && typeof sourceDescriptor.get === "function"
  && sourceDescriptor.set === undefined
  && sourceDescriptor.enumerable === false
  && sourceDescriptor.configurable === true
  && typeof globalDescriptor.get === "function"
  && globalDescriptor.set === undefined
  && globalDescriptor.enumerable === false
  && globalDescriptor.configurable === true
  && typeof unicodeDescriptor.get === "function"
  && typeof unicodeSetsDescriptor.get === "function"
  && re.source === "a"
  && re.flags === "dgimsuy"
  && re.hasIndices === true
  && re.global === true
  && re.ignoreCase === true
  && re.multiline === true
  && re.dotAll === true
  && re.unicode === true
  && re.unicodeSets === false
  && re.sticky === true
  && v.flags === "gv"
  && v.global === true
  && v.unicode === false
  && v.unicodeSets === true
  && RegExp.prototype.source === "(?:)"
  && RegExp.prototype.flags === ""
  && RegExp.prototype.global === undefined
  && RegExp.prototype.hasIndices === undefined
  && RegExp.prototype.ignoreCase === undefined
  && RegExp.prototype.multiline === undefined
  && RegExp.prototype.dotAll === undefined
  && RegExp.prototype.unicode === undefined
  && RegExp.prototype.unicodeSets === undefined
  && RegExp.prototype.sticky === undefined
  && invalidReceiverThrows
  && protocolOverrideObserved
  && globalOverrideResult.length === 1
  && globalOverrideResult[0] === "a"
  && beforeGlobalOverride.lastIndex === 1
  && sourceOverrideObserved;
