// Identifier numeric updates retain the Object Environment Record selected by
// ResolveBinding. A getter may delete the property, but PutValue must not fall
// through to a surrounding function, global or outer object environment.

function prefixIncrementAgainstFunctionBinding() {
  var functionValue = 40;
  let scope = {
    get functionValue() {
      delete this.functionValue;
      return "2";
    }
  };
  let result;
  with (scope) {
    result = ++functionValue;
  }
  return result === 3 && scope.functionValue === 3 && functionValue === 40;
}

var globalPostfixValue = 40;
let globalPostfixScope = {
  get globalPostfixValue() {
    delete this.globalPostfixValue;
    return "2";
  }
};
let globalPostfixResult;
with (globalPostfixScope) {
  globalPostfixResult = globalPostfixValue++;
}
let postfixIncrementAgainstGlobalPassed = globalPostfixResult === 2
  && globalPostfixScope.globalPostfixValue === 3
  && globalPostfixValue === 40;

let outerScope = { nestedPrefixValue: 40 };
let innerScope = {
  get nestedPrefixValue() {
    delete this.nestedPrefixValue;
    return "2";
  }
};
let nestedPrefixResult;
with (outerScope) {
  with (innerScope) {
    nestedPrefixResult = --nestedPrefixValue;
  }
}
let prefixDecrementAgainstOuterObjectPassed = nestedPrefixResult === 1
  && innerScope.nestedPrefixValue === 1
  && outerScope.nestedPrefixValue === 40;

function postfixDecrementAgainstFunctionBinding() {
  var functionValue = 40;
  let scope = {
    get functionValue() {
      delete this.functionValue;
      return "2";
    }
  };
  let result;
  with (scope) {
    result = functionValue--;
  }
  return result === 2 && scope.functionValue === 1 && functionValue === 40;
}

// HasBinding, GetBindingValue and SetMutableBinding remain separately
// observable. The third `has` is the post-Get recheck; sloppy PutValue still
// writes through the selected object after the getter deletes the property.
let trace = "";
let traceTarget = {
  get tracedValue() {
    return "2";
  }
};
let traceProxy = new Proxy(traceTarget, {
  has(target, key) {
    if (key === "tracedValue") trace += "h";
    return key in target;
  },
  get(target, key, receiver) {
    if (key === Symbol.unscopables) trace += "u";
    if (key === "tracedValue") trace += "g";
    return target[key];
  },
  deleteProperty(target, key) {
    if (key === "tracedValue") trace += "d";
    return delete target[key];
  },
  set(target, key, value) {
    if (key === "tracedValue") trace += "s";
    target[key] = value;
    return true;
  }
});
Object.defineProperty(traceTarget, "tracedValue", {
  configurable: true,
  get() {
    delete traceProxy.tracedValue;
    return "2";
  }
});
let traceResult;
with (traceProxy) {
  traceResult = tracedValue++;
}
let observableLifecyclePassed = trace === "huhgdhs"
  && traceResult === 2
  && traceTarget.tracedValue === 3;

// HasBinding may mutate the pre-located fallback before deciding not to select
// the object record. The fallback update must therefore remain dynamically
// typed instead of retaining the Number shape observed before `with`.
var mutatedFallbackValue = 1;
let mutationScope = { mutatedFallbackValue: 99 };
Object.defineProperty(mutationScope, Symbol.unscopables, {
  get() {
    mutatedFallbackValue = 2n;
    return { mutatedFallbackValue: true };
  }
});
let mutatedFallbackResult;
with (mutationScope) {
  mutatedFallbackResult = mutatedFallbackValue++;
}
let mutatedFallbackPassed = mutatedFallbackResult === 2n
  && mutatedFallbackValue === 3n
  && mutationScope.mutatedFallbackValue === 99;

// The selected object may take the update even though HasBinding mutated the
// untouched fallback to an arbitrary tag. Post-expression metadata must retain
// that possibility instead of assuming the fallback is still numeric.
function selectedBranchMutatesFallback() {
  let selectedFallbackValue = 1;
  let selectedTarget = { selectedFallbackValue: 2 };
  let selectedScope = new Proxy(selectedTarget, {
    has(target, key) {
      if (key === "selectedFallbackValue") {
        selectedFallbackValue = "mutated";
        return true;
      }
      return key in target;
    }
  });
  let result;
  with (selectedScope) {
    result = ++selectedFallbackValue;
  }
  return result === 3
    && selectedTarget.selectedFallbackValue === 3
    && selectedFallbackValue === "mutated";
}

// A declined object binding may leave an arbitrary fallback value when
// ToNumeric throws. Catching that throw makes the mutated tag observable later.
var throwingFallbackValue = 1;
let throwingMarker = {};
let throwingReplacement = {
  valueOf() {
    throw throwingMarker;
  }
};
let throwingScope = { throwingFallbackValue: 99 };
Object.defineProperty(throwingScope, Symbol.unscopables, {
  get() {
    throwingFallbackValue = throwingReplacement;
    return { throwingFallbackValue: true };
  }
});
let throwingFallbackCaught = false;
with (throwingScope) {
  try {
    throwingFallbackValue++;
  } catch (error) {
    throwingFallbackCaught = error === throwingMarker;
  }
}
let throwingFallbackPassed = throwingFallbackCaught
  && throwingFallbackValue === throwingReplacement
  && throwingScope.throwingFallbackValue === 99;

// The initial Object Environment HasBinding query may instead delete a global
// property that lowering had already proven present. A fallback update must
// recheck the global at run time and throw before ToNumeric or a recreating Set.
globalThis.deletedFallbackValue = 5;
let deletionScope = new Proxy({}, {
  has(target, key) {
    if (key === "deletedFallbackValue") {
      delete globalThis.deletedFallbackValue;
      return false;
    }
    return key in target;
  }
});
let deletedFallbackCaught = false;
let deletedFallbackCompleted = false;
with (deletionScope) {
  try {
    deletedFallbackValue++;
    deletedFallbackCompleted = true;
  } catch (error) {
    deletedFallbackCaught = error instanceof ReferenceError;
  }
}
let deletedFallbackType = typeof deletedFallbackValue;
let deletedFallbackPassed = deletedFallbackCaught
  && !deletedFallbackCompleted
  && !("deletedFallbackValue" in globalThis)
  && deletedFallbackType === "undefined";

// Conversely, HasBinding may create a previously unresolvable global before
// declining the object binding. The same run-time guard must admit that new
// fallback instead of taking the statically prepared missing-name throw.
let creationScope = new Proxy({}, {
  has(target, key) {
    if (key === "createdFallbackValue") {
      globalThis.createdFallbackValue = 4;
      return false;
    }
    return key in target;
  }
});
let createdFallbackResult;
with (creationScope) {
  createdFallbackResult = createdFallbackValue++;
}
let createdFallbackPassed = createdFallbackResult === 4
  && globalThis.createdFallbackValue === 5;

// Strictness belongs to the Reference, not to the surrounding `with` syntax.
// Each strict closure must throw after its getter deletes the selected property;
// none may write the new value to the object or to the global fallback.
var strictPrefixIncrementValue = 70;
let strictPrefixIncrementScope = {
  get strictPrefixIncrementValue() {
    delete this.strictPrefixIncrementValue;
    return 2;
  }
};
let runStrictPrefixIncrement;
with (strictPrefixIncrementScope) {
  runStrictPrefixIncrement = function () {
    "use strict";
    ++strictPrefixIncrementValue;
  };
}

var strictPostfixIncrementValue = 71;
let strictPostfixIncrementScope = {
  get strictPostfixIncrementValue() {
    delete this.strictPostfixIncrementValue;
    return 2;
  }
};
let runStrictPostfixIncrement;
with (strictPostfixIncrementScope) {
  runStrictPostfixIncrement = function () {
    "use strict";
    strictPostfixIncrementValue++;
  };
}

var strictPrefixDecrementValue = 72;
let strictPrefixDecrementScope = {
  get strictPrefixDecrementValue() {
    delete this.strictPrefixDecrementValue;
    return 2;
  }
};
let runStrictPrefixDecrement;
with (strictPrefixDecrementScope) {
  runStrictPrefixDecrement = function () {
    "use strict";
    --strictPrefixDecrementValue;
  };
}

var strictPostfixDecrementValue = 73;
let strictPostfixDecrementScope = {
  get strictPostfixDecrementValue() {
    delete this.strictPostfixDecrementValue;
    return 2;
  }
};
let runStrictPostfixDecrement;
with (strictPostfixDecrementScope) {
  runStrictPostfixDecrement = function () {
    "use strict";
    strictPostfixDecrementValue--;
  };
}

let strictCaught = 0;
for (let run of [
  runStrictPrefixIncrement,
  runStrictPostfixIncrement,
  runStrictPrefixDecrement,
  runStrictPostfixDecrement
]) {
  try {
    run();
  } catch (error) {
    if (error instanceof ReferenceError) strictCaught++;
  }
}
let strictRecheckPassed = strictCaught === 4
  && !("strictPrefixIncrementValue" in strictPrefixIncrementScope)
  && !("strictPostfixIncrementValue" in strictPostfixIncrementScope)
  && !("strictPrefixDecrementValue" in strictPrefixDecrementScope)
  && !("strictPostfixDecrementValue" in strictPostfixDecrementScope)
  && strictPrefixIncrementValue === 70
  && strictPostfixIncrementValue === 71
  && strictPrefixDecrementValue === 72
  && strictPostfixDecrementValue === 73;

prefixIncrementAgainstFunctionBinding()
  && postfixIncrementAgainstGlobalPassed
  && prefixDecrementAgainstOuterObjectPassed
  && postfixDecrementAgainstFunctionBinding()
  && observableLifecyclePassed
  && mutatedFallbackPassed
  && selectedBranchMutatesFallback()
  && throwingFallbackPassed
  && deletedFallbackPassed
  && createdFallbackPassed
  && strictRecheckPassed;
