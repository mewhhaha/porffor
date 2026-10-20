// Identifier logical assignment retains one Reference across Object
// Environment selection, GetValue, short-circuiting and conditional PutValue.

function caughtReferenceError(callback) {
  try {
    callback();
  } catch (error) {
    return error instanceof ReferenceError;
  }
  return false;
}

// An initially absent global throws from GetValue before the RHS for every
// logical mode. None of the operations may create the missing property.
let missingRhsCount = 0;
function missingRhs() {
  missingRhsCount++;
  return 1;
}
let missingAndCaught = caughtReferenceError(function () {
  missingLogicalAnd &&= missingRhs();
});
let missingOrCaught = caughtReferenceError(function () {
  missingLogicalOr ||= missingRhs();
});
let missingNullishCaught = caughtReferenceError(function () {
  missingLogicalNullish ??= missingRhs();
});
let missingPassed = missingAndCaught
  && missingOrCaught
  && missingNullishCaught
  && missingRhsCount === 0
  && !("missingLogicalAnd" in globalThis)
  && !("missingLogicalOr" in globalThis)
  && !("missingLogicalNullish" in globalThis);

// Dynamically selected global accessors short-circuit without evaluating RHS
// or running SetMutableBinding.
let shortGetCount = 0;
let shortSetCount = 0;
let shortRhsCount = 0;
function shortRhs() {
  shortRhsCount++;
  return "rhs";
}
function defineShortGlobal(name, value) {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    get() {
      shortGetCount++;
      return value;
    },
    set(next) {
      shortSetCount++;
      value = next;
    },
  });
}
defineShortGlobal("shortLogicalAnd", 0);
defineShortGlobal("shortLogicalOr", 1);
defineShortGlobal("shortLogicalNullish", 2);
let shortAndResult = shortLogicalAnd &&= shortRhs();
let shortOrResult = shortLogicalOr ||= shortRhs();
let shortNullishResult = shortLogicalNullish ??= shortRhs();
let shortCircuitPassed = shortAndResult === 0
  && shortOrResult === 1
  && shortNullishResult === 2
  && shortGetCount === 3
  && shortRhsCount === 0
  && shortSetCount === 0;

// The located Reference owns its proven-global lhs metadata before lowering a
// RHS that would change that metadata. This branch short-circuits, so the old
// Number value and tag must survive even though the String assignment is
// present in the untaken RHS IR.
snapshotLogicalValue = 1;
let snapshotLogicalResult = snapshotLogicalValue ||= (snapshotLogicalValue = "rhs");
let provenGlobalSnapshotPassed = snapshotLogicalResult === 1
  && snapshotLogicalValue === 1;

// Taken global branches evaluate RHS and write through the same selected
// binding object. Their expression result appears only after PutValue.
let takenGetCount = 0;
let takenSetCount = 0;
let takenRhsCount = 0;
function defineTakenGlobal(name, value) {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    get() {
      takenGetCount++;
      return value;
    },
    set(next) {
      takenSetCount++;
      value = next;
    },
  });
}
function takenRhs(value) {
  takenRhsCount++;
  return value;
}
defineTakenGlobal("takenLogicalAnd", true);
defineTakenGlobal("takenLogicalOr", false);
defineTakenGlobal("takenLogicalNullish", null);
let takenAndResult = takenLogicalAnd &&= takenRhs("and");
let takenOrResult = takenLogicalOr ||= takenRhs("or");
let takenNullishResult = takenLogicalNullish ??= takenRhs("nullish");
let takenPassed = takenAndResult === "and"
  && takenLogicalAnd === "and"
  && takenOrResult === "or"
  && takenLogicalOr === "or"
  && takenNullishResult === "nullish"
  && takenLogicalNullish === "nullish"
  && takenGetCount === 6
  && takenRhsCount === 3
  && takenSetCount === 3;

// A strict Reference whose getter deletes the selected global must throw on
// the post-RHS HasProperty recheck. The enclosing result remains untouched.
let strictGetterCount = 0;
let strictRhsCount = 0;
let strictResult = "not written";
function strictRhs(value) {
  strictRhsCount++;
  return value;
}
function defineDeletingGlobal(name, value) {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    get() {
      strictGetterCount++;
      delete globalThis[name];
      return value;
    },
  });
}
defineDeletingGlobal("strictLogicalAnd", true);
defineDeletingGlobal("strictLogicalOr", false);
defineDeletingGlobal("strictLogicalNullish", null);
let strictAndCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = strictLogicalAnd &&= strictRhs("and");
});
let strictOrCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = strictLogicalOr ||= strictRhs("or");
});
let strictNullishCaught = caughtReferenceError(function () {
  "use strict";
  strictResult = strictLogicalNullish ??= strictRhs("nullish");
});
let strictDeletionPassed = strictAndCaught
  && strictOrCaught
  && strictNullishCaught
  && strictGetterCount === 3
  && strictRhsCount === 3
  && strictResult === "not written"
  && !("strictLogicalAnd" in globalThis)
  && !("strictLogicalOr" in globalThis)
  && !("strictLogicalNullish" in globalThis);

// Sloppy SetMutableBinding still observes its recheck, then recreates a
// property deleted during GetValue.
Object.defineProperty(globalThis, "sloppyLogicalAnd", {
  configurable: true,
  get() {
    delete globalThis.sloppyLogicalAnd;
    return true;
  },
});
Object.defineProperty(globalThis, "sloppyLogicalOr", {
  configurable: true,
  get() {
    delete globalThis.sloppyLogicalOr;
    return false;
  },
});
Object.defineProperty(globalThis, "sloppyLogicalNullish", {
  configurable: true,
  get() {
    delete globalThis.sloppyLogicalNullish;
    return null;
  },
});
let sloppyAndResult = sloppyLogicalAnd &&= "and";
let sloppyOrResult = sloppyLogicalOr ||= "or";
let sloppyNullishResult = sloppyLogicalNullish ??= "nullish";
let sloppyDeletionPassed = sloppyAndResult === "and"
  && sloppyLogicalAnd === "and"
  && sloppyOrResult === "or"
  && sloppyLogicalOr === "or"
  && sloppyNullishResult === "nullish"
  && sloppyLogicalNullish === "nullish";

// A visible with binding wins over a declarative fallback for all three modes.
function selectedWithPassed() {
  let selectedAnd = "outer";
  let selectedOr = "outer";
  let selectedNullish = "outer";
  let scope = { selectedAnd: true, selectedOr: false, selectedNullish: null };
  let andResult;
  let orResult;
  let nullishResult;
  with (scope) {
    andResult = selectedAnd &&= "and";
    orResult = selectedOr ||= "or";
    nullishResult = selectedNullish ??= "nullish";
  }
  return andResult === "and"
    && orResult === "or"
    && nullishResult === "nullish"
    && scope.selectedAnd === "and"
    && scope.selectedOr === "or"
    && scope.selectedNullish === "nullish"
    && selectedAnd === "outer"
    && selectedOr === "outer"
    && selectedNullish === "outer";
}

// A missing with binding reaches the already-located fallback. A nested
// unscopables record can also decline its own property and select an outer
// Object Environment Record without restarting resolution after the RHS.
function fallbackWithPassed() {
  let fallbackValue = 0;
  let result;
  with ({}) {
    result = fallbackValue ||= 7;
  }
  return result === 7 && fallbackValue === 7;
}
let outerScope = { nestedLogicalValue: 0 };
let innerScope = {
  nestedLogicalValue: 99,
  [Symbol.unscopables]: { nestedLogicalValue: true },
};
let nestedResult;
with (outerScope) {
  with (innerScope) {
    nestedResult = nestedLogicalValue ||= 8;
  }
}
let nestedUnscopablesPassed = nestedResult === 8
  && outerScope.nestedLogicalValue === 8
  && innerScope.nestedLogicalValue === 99;

// The selected with object exposes the complete observable order: HasBinding,
// unscopables, GetBindingValue HasProperty/Get, getter deletion, RHS,
// SetMutableBinding HasProperty, then Set.
let lifecycleTrace = "";
let lifecycleTarget = {};
let lifecycleProxy = new Proxy(lifecycleTarget, {
  has(target, key) {
    if (key === "tracedLogicalValue") lifecycleTrace += "h";
    return key in target;
  },
  get(target, key) {
    if (key === Symbol.unscopables) lifecycleTrace += "u";
    if (key === "tracedLogicalValue") lifecycleTrace += "g";
    return target[key];
  },
  deleteProperty(target, key) {
    if (key === "tracedLogicalValue") lifecycleTrace += "d";
    return delete target[key];
  },
  set(target, key, value) {
    if (key === "tracedLogicalValue") lifecycleTrace += "s";
    target[key] = value;
    return true;
  },
});
Object.defineProperty(lifecycleTarget, "tracedLogicalValue", {
  configurable: true,
  get() {
    delete lifecycleProxy.tracedLogicalValue;
    return false;
  },
});
function tracedRhs() {
  lifecycleTrace += "r";
  return "written";
}
let lifecycleResult;
with (lifecycleProxy) {
  lifecycleResult = tracedLogicalValue ||= tracedRhs();
}
let observableLifecyclePassed = lifecycleTrace === "huhgdrhs"
  && lifecycleResult === "written"
  && lifecycleTarget.tracedLogicalValue === "written";

missingPassed
  && shortCircuitPassed
  && provenGlobalSnapshotPassed
  && takenPassed
  && strictDeletionPassed
  && sloppyDeletionPassed
  && selectedWithPassed()
  && fallbackWithPassed()
  && nestedUnscopablesPassed
  && observableLifecyclePassed;
