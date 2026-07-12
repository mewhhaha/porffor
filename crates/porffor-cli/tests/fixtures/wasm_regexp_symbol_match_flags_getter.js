let custom = /a/g;
let execCount = 0;
custom.exec = function () {
  execCount++;
  return null;
};
Object.defineProperty(custom, "global", { value: false, writable: true });
let customMatch = custom[Symbol.match]("a");

let marker = {};
let unicodeRegex = /a/g;
let flagsDescriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, "flags");
let unicodeSetsRegex = new RegExp("a", "gv");
let unicodeSetsChecks = unicodeSetsRegex.flags === "gv"
  && unicodeSetsRegex.global === true
  && unicodeSetsRegex.unicodeSets === true;
let order = "";
let orderedFlags = flagsDescriptor.get.call({
  get hasIndices() { order += "d"; return true; },
  get global() { order += "g"; return true; },
  get ignoreCase() { order += "i"; return true; },
  get multiline() { order += "m"; return true; },
  get dotAll() { order += "s"; return true; },
  get unicode() { order += "u"; return true; },
  get unicodeSets() { order += "v"; return true; },
  get sticky() { order += "y"; return true; },
});
let flagsThrowMarker = {};
let flagsThrowCaught = false;
try {
  flagsDescriptor.get.call({
    get hasIndices() { return false; },
    get global() { return false; },
    get ignoreCase() { throw flagsThrowMarker; },
    get multiline() { throw 99; },
  });
} catch (error) {
  flagsThrowCaught = error === flagsThrowMarker;
}
let nonObjectFlagsThrows = false;
try {
  flagsDescriptor.get.call(null);
} catch (error) {
  nonObjectFlagsThrows = error instanceof TypeError;
}
Object.defineProperty(unicodeRegex, "unicode", {
  get() {
    throw marker;
  },
});
let caught;
try {
  unicodeRegex[Symbol.match]("a");
} catch (error) {
  caught = error;
}

let resetBeforeUnicodeThrow = /a/g;
resetBeforeUnicodeThrow.lastIndex = 4;
let resetOrder = "";
let resetMarker = {};
Object.defineProperty(resetBeforeUnicodeThrow, "global", {
  get() { resetOrder += "g"; return true; },
});
Object.defineProperty(resetBeforeUnicodeThrow, "unicode", {
  get() { resetOrder += "u"; throw resetMarker; },
});
let resetCaught = false;
try {
  resetBeforeUnicodeThrow[Symbol.match]("a");
} catch (error) {
  resetCaught = error === resetMarker;
}

let flagsConflict = /a/g;
Object.defineProperty(flagsConflict, "flags", { value: "" });
let flagsConflictMatch = flagsConflict[Symbol.match]("aa");

let falseGlobal = /a/g;
falseGlobal.lastIndex = 3;
let falseGlobalExecCount = 0;
Object.defineProperty(falseGlobal, "global", { value: false });
Object.defineProperty(falseGlobal, "unicode", {
  get() { throw new Error("unicode must not be read"); },
});
falseGlobal.exec = function () {
  falseGlobalExecCount++;
  return null;
};
let falseGlobalMatch = falseGlobal[Symbol.match]("a");

let inheritedExec = /a/;
let inheritedExecReads = 0;
let inheritedExecCalls = 0;
Object.setPrototypeOf(inheritedExec, {
  get exec() {
    inheritedExecReads++;
    return function () {
      inheritedExecCalls++;
      return null;
    };
  },
});
let inheritedExecMatch = RegExp.prototype[Symbol.match].call(inheritedExec, "a");

let replacedPrototypeExec = RegExp.prototype.exec;
RegExp.prototype.exec = function () { return null; };
let prototypeReplacementMatch = /a/[Symbol.match]("a");
RegExp.prototype.exec = replacedPrototypeExec;

customMatch === null
  && execCount === 1
  && caught === marker
  && resetCaught
  && resetOrder === "gu"
  && resetBeforeUnicodeThrow.lastIndex === 0
  && flagsConflictMatch.length === 2
  && flagsConflictMatch[0] === "a"
  && flagsConflictMatch[1] === "a"
  && falseGlobalMatch === null
  && falseGlobalExecCount === 1
  && falseGlobal.lastIndex === 3
  && inheritedExecMatch === null
  && inheritedExecReads === 1
  && inheritedExecCalls === 1
  && prototypeReplacementMatch === null
  && !Object.prototype.hasOwnProperty.call(/a/, "flags")
  && unicodeSetsChecks
  && order === "dgimsuvy"
  && orderedFlags === "dgimsuvy"
  && flagsThrowCaught
  && nonObjectFlagsThrows
  && typeof flagsDescriptor.get === "function";
