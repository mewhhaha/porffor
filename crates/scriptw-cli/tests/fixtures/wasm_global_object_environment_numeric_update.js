// A global Object Environment Reference retains one binding object from its
// initial HasProperty through GetValue, ToNumeric and PutValue. Prefix/postfix
// mode and Number/BigInt domain do not change that lifecycle.

function caughtReferenceError(callback) {
  try {
    callback();
  } catch (error) {
    return error instanceof ReferenceError;
  }
  return false;
}

Object.defineProperty(globalThis, "globalPrefixNumber", {
  configurable: true,
  writable: true,
  value: 2,
});
Object.defineProperty(globalThis, "globalPostfixNumber", {
  configurable: true,
  writable: true,
  value: 2,
});
Object.defineProperty(globalThis, "globalPrefixBigInt", {
  configurable: true,
  writable: true,
  value: 4n,
});
Object.defineProperty(globalThis, "globalPostfixBigInt", {
  configurable: true,
  writable: true,
  value: 4n,
});

let prefixNumberResult = ++globalPrefixNumber;
let postfixNumberResult = globalPostfixNumber++;
let prefixBigIntResult = --globalPrefixBigInt;
let postfixBigIntResult = globalPostfixBigInt--;
let successfulModesPassed = prefixNumberResult === 3
  && globalPrefixNumber === 3
  && postfixNumberResult === 2
  && globalPostfixNumber === 3
  && prefixBigIntResult === 3n
  && globalPrefixBigInt === 3n
  && postfixBigIntResult === 4n
  && globalPostfixBigInt === 3n;

// These four strict updates mirror the exact Test262 cohort. GetValue invokes
// the accessor and deletes the selected property; PutValue must independently
// recheck HasProperty, throw, and leave each outer assignment untouched.
let strictGetterCount = 0;
let strictResult = "not written";
for (let name of [
  "strictPrefixIncrement",
  "strictPrefixDecrement",
  "strictPostfixIncrement",
  "strictPostfixDecrement",
]) {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    get() {
      strictGetterCount++;
      delete globalThis[name];
      return 2;
    },
  });
}

let strictPrefixIncrementCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = ++strictPrefixIncrement;
});
let strictPrefixDecrementCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = --strictPrefixDecrement;
});
let strictPostfixIncrementCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = strictPostfixIncrement++;
});
let strictPostfixDecrementCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = strictPostfixDecrement--;
});
let strictDeletionPassed = strictPrefixIncrementCaught
  && strictPrefixDecrementCaught
  && strictPostfixIncrementCaught
  && strictPostfixDecrementCaught
  && strictGetterCount === 4
  && strictResult === "not written"
  && !("strictPrefixIncrement" in globalThis)
  && !("strictPrefixDecrement" in globalThis)
  && !("strictPostfixIncrement" in globalThis)
  && !("strictPostfixDecrement" in globalThis);

// Sloppy SetMutableBinding still performs its post-Get HasProperty observation,
// then Set recreates the deleted property on the retained global object.
Object.defineProperty(globalThis, "sloppyPostfixNumber", {
  configurable: true,
  get() {
    delete globalThis.sloppyPostfixNumber;
    return 4;
  },
});
Object.defineProperty(globalThis, "sloppyPrefixBigInt", {
  configurable: true,
  get() {
    delete globalThis.sloppyPrefixBigInt;
    return 4n;
  },
});
let sloppyPostfixResult = sloppyPostfixNumber++;
let sloppyPrefixResult = --sloppyPrefixBigInt;
let sloppyDeletionPassed = sloppyPostfixResult === 4
  && sloppyPostfixNumber === 5
  && Object.prototype.hasOwnProperty.call(globalThis, "sloppyPostfixNumber")
  && sloppyPrefixResult === 3n
  && sloppyPrefixBigInt === 3n
  && Object.prototype.hasOwnProperty.call(globalThis, "sloppyPrefixBigInt");

// A Proxy in the global object's prototype chain makes the three HasProperty
// observations distinct. The selected inherited getter also makes ToNumeric
// observable before the final HasProperty and Set.
let originalGlobalPrototype = Object.getPrototypeOf(globalThis);
let lifecycleTrace = "";
let observedName = "orderedGlobalUpdate";
let missingName = "initiallyMissingGlobalUpdate";
let observingPrototype = new Proxy(originalGlobalPrototype, {
  has(target, key) {
    if (key === observedName || key === missingName) lifecycleTrace += "h";
    return Reflect.has(target, key);
  },
  get(target, key, receiver) {
    if (key === observedName || key === missingName) lifecycleTrace += "g";
    return Reflect.get(target, key, receiver);
  },
  set(target, key, value, receiver) {
    if (key === observedName || key === missingName) lifecycleTrace += "s";
    return Reflect.set(target, key, value, receiver);
  },
});
Object.setPrototypeOf(globalThis, observingPrototype);

let missingCaught = caughtReferenceError(function () {
  ++initiallyMissingGlobalUpdate;
});
let missingPassed = missingCaught
  && lifecycleTrace === "h"
  && !(missingName in globalThis);

lifecycleTrace = "";
Object.defineProperty(originalGlobalPrototype, observedName, {
  configurable: true,
  get() {
    lifecycleTrace += "d";
    delete originalGlobalPrototype[observedName];
    return {
      valueOf() {
        lifecycleTrace += "n";
        return 4;
      },
    };
  },
});
let orderedResult = orderedGlobalUpdate++;
Object.setPrototypeOf(globalThis, originalGlobalPrototype);
let lifecyclePassed = lifecycleTrace === "hhgdnhs"
  && orderedResult === 4
  && orderedGlobalUpdate === 5
  && Object.prototype.hasOwnProperty.call(globalThis, observedName)
  && !(observedName in originalGlobalPrototype);

successfulModesPassed
  && strictDeletionPassed
  && sloppyDeletionPassed
  && missingPassed
  && lifecyclePassed;
