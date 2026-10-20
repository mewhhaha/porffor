let shadowedSlots = /a/;
Object.defineProperty(shadowedSlots, "source", { value: "z" });
Object.defineProperty(shadowedSlots, "flags", { value: "" });
let shadowedSlotsSearch = shadowedSlots[Symbol.search]("a");

let throwingSlots = /a/;
let slotMarker = {};
Object.defineProperty(throwingSlots, "source", {
  get() { throw slotMarker; },
});
Object.defineProperty(throwingSlots, "flags", {
  get() { throw slotMarker; },
});
let throwingSlotsSearch;
try {
  throwingSlotsSearch = throwingSlots[Symbol.search]("a");
} catch (error) {
  throwingSlotsSearch = error;
}

let ownExec = /a/;
ownExec.exec = function () { return { index: 3 }; };
let ownExecSearch = ownExec[Symbol.search]("a");

let savedPrototypeExec = RegExp.prototype.exec;
RegExp.prototype.exec = function () { return { index: 4 }; };
let replacedPrototypeExecSearch = /a/[Symbol.search]("a");
if (savedPrototypeExec === undefined) {
  delete RegExp.prototype.exec;
} else {
  RegExp.prototype.exec = savedPrototypeExec;
}
let restoredPrototypeExecSearch = /String/i[Symbol.search]("test string");

let deletedPrototypeExecCalls = 0;
delete RegExp.prototype.exec;
Object.prototype.exec = function () {
  deletedPrototypeExecCalls++;
  return { index: 6 };
};
let deletedPrototypeExecSearch = /a/[Symbol.search]("a");
delete Object.prototype.exec;
RegExp.prototype.exec = savedPrototypeExec;

let inheritedExec = /a/;
let inheritedExecReads = 0;
let inheritedExecCalls = 0;
Object.setPrototypeOf(inheritedExec, {
  get exec() {
    inheritedExecReads++;
    return function () {
      inheritedExecCalls++;
      return { index: 5 };
    };
  },
});
let inheritedExecSearch = RegExp.prototype[Symbol.search].call(inheritedExec, "a");

shadowedSlotsSearch === 0
  && throwingSlotsSearch === 0
  && ownExecSearch === 3
  && replacedPrototypeExecSearch === 4
  && restoredPrototypeExecSearch === 5
  && deletedPrototypeExecSearch === 6
  && deletedPrototypeExecCalls === 1
  && inheritedExecSearch === 5
  && inheritedExecReads === 1
  && inheritedExecCalls === 1;
