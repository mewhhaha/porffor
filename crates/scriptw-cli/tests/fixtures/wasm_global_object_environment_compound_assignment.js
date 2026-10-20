// A global Object Environment Reference retains one binding object from its
// initial HasProperty through GetValue, RHS evaluation and PutValue. The
// accessor deletes the selected property, so strict PutValue must recheck and
// throw without recreating it.

function caughtReferenceError(callback) {
  try {
    callback();
  } catch (error) {
    return error instanceof ReferenceError;
  }
  return false;
}

let getterCount = 0;
let rhsCount = 0;
let strictResult = "not written";

function compoundRhs() {
  rhsCount++;
  return 3;
}

Object.defineProperty(this, "globalXorValue", {
  configurable: true,
  get() { getterCount++; delete this.globalXorValue; return 2; }
});
Object.defineProperty(this, "globalOrValue", {
  configurable: true,
  get() { getterCount++; delete this.globalOrValue; return 2; }
});
Object.defineProperty(this, "globalMulValue", {
  configurable: true,
  get() { getterCount++; delete this.globalMulValue; return 2; }
});
Object.defineProperty(this, "globalDivValue", {
  configurable: true,
  get() { getterCount++; delete this.globalDivValue; return 2; }
});
Object.defineProperty(this, "globalModValue", {
  configurable: true,
  get() { getterCount++; delete this.globalModValue; return 2; }
});
Object.defineProperty(this, "globalAddValue", {
  configurable: true,
  get() { getterCount++; delete this.globalAddValue; return 2; }
});
Object.defineProperty(this, "globalSubValue", {
  configurable: true,
  get() { getterCount++; delete this.globalSubValue; return 2; }
});
Object.defineProperty(this, "globalShlValue", {
  configurable: true,
  get() { getterCount++; delete this.globalShlValue; return 2; }
});
Object.defineProperty(this, "globalShrValue", {
  configurable: true,
  get() { getterCount++; delete this.globalShrValue; return 2; }
});
Object.defineProperty(this, "globalUshrValue", {
  configurable: true,
  get() { getterCount++; delete this.globalUshrValue; return 2; }
});
Object.defineProperty(this, "globalAndValue", {
  configurable: true,
  get() { getterCount++; delete this.globalAndValue; return 2; }
});

let xorCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalXorValue ^= compoundRhs();
});
let orCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalOrValue |= compoundRhs();
});
let mulCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalMulValue *= compoundRhs();
});
let divCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalDivValue /= compoundRhs();
});
let modCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalModValue %= compoundRhs();
});
let addCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalAddValue += compoundRhs();
});
let subCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalSubValue -= compoundRhs();
});
let shlCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalShlValue <<= compoundRhs();
});
let shrCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalShrValue >>= compoundRhs();
});
let ushrCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalUshrValue >>>= compoundRhs();
});
let andCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = globalAndValue &= compoundRhs();
});

let strictDeletionPassed = xorCaught
  && orCaught
  && mulCaught
  && divCaught
  && modCaught
  && addCaught
  && subCaught
  && shlCaught
  && shrCaught
  && ushrCaught
  && andCaught
  && getterCount === 11
  && rhsCount === 11
  && strictResult === "not written"
  && !("globalXorValue" in globalThis)
  && !("globalOrValue" in globalThis)
  && !("globalMulValue" in globalThis)
  && !("globalDivValue" in globalThis)
  && !("globalModValue" in globalThis)
  && !("globalAddValue" in globalThis)
  && !("globalSubValue" in globalThis)
  && !("globalShlValue" in globalThis)
  && !("globalShrValue" in globalThis)
  && !("globalUshrValue" in globalThis)
  && !("globalAndValue" in globalThis);

// An initially absent binding fails during GetValue, before the RHS is run.
let absentRhsCount = 0;
function absentRhs() {
  absentRhsCount++;
  return 1;
}
let absentCaught = caughtReferenceError(function () {
  "use strict";
  initiallyAbsentGlobal += absentRhs();
});
let initiallyAbsentPassed = absentCaught
  && absentRhsCount === 0
  && !("initiallyAbsentGlobal" in globalThis);

// A sloppy Reference retains the same global Object Record. After Get removes
// the property, SetMutableBinding rechecks presence and recreates it.
let sloppyTrace = "";
Object.defineProperty(this, "sloppyGlobalValue", {
  configurable: true,
  get() {
    sloppyTrace += "g";
    delete this.sloppyGlobalValue;
    return 4;
  }
});
function sloppyRhs() {
  sloppyTrace += "r";
  return 3;
}
let sloppyResult = sloppyGlobalValue += sloppyRhs();
let sloppyPassed = sloppyTrace === "gr"
  && sloppyResult === 7
  && sloppyGlobalValue === 7
  && Object.prototype.hasOwnProperty.call(globalThis, "sloppyGlobalValue");

// HasBinding on the global Object Record uses HasProperty, so an inherited
// binding is selected too. Deleting it during Get and completing sloppy
// PutValue creates the result on the retained global binding object.
Object.defineProperty(Object.prototype, "inheritedGlobalValue", {
  configurable: true,
  get() {
    delete Object.prototype.inheritedGlobalValue;
    return 5;
  }
});
let inheritedResult = inheritedGlobalValue -= 2;
let inheritedPassed = inheritedResult === 3
  && inheritedGlobalValue === 3
  && Object.prototype.hasOwnProperty.call(globalThis, "inheritedGlobalValue")
  && !("inheritedGlobalValue" in Object.prototype);

strictDeletionPassed
  && initiallyAbsentPassed
  && sloppyPassed
  && inheritedPassed;
