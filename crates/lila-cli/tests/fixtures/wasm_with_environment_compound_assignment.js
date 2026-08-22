// Eager compound assignments retain the Object Environment Record selected by
// ResolveBinding. GetValue, RHS evaluation and PutValue all spend that same
// Reference; deleting the selected property must not restart resolution.

function allArithmeticOperatorsPassed() {
  let scope = {
    get addValue() { delete this.addValue; return 12; },
    get subValue() { delete this.subValue; return 12; },
    get mulValue() { delete this.mulValue; return 12; },
    get divValue() { delete this.divValue; return 12; },
    get modValue() { delete this.modValue; return 12; },
    get expValue() { delete this.expValue; return 3; }
  };
  let addResult;
  let subResult;
  let mulResult;
  let divResult;
  let modResult;
  let expResult;
  with (scope) {
    addResult = addValue += 2;
    subResult = subValue -= 2;
    mulResult = mulValue *= 2;
    divResult = divValue /= 3;
    modResult = modValue %= 5;
    expResult = expValue **= 3;
  }
  return addResult === 14
    && subResult === 10
    && mulResult === 24
    && divResult === 4
    && modResult === 2
    && expResult === 27
    && scope.addValue === 14
    && scope.subValue === 10
    && scope.mulValue === 24
    && scope.divValue === 4
    && scope.modValue === 2
    && scope.expValue === 27;
}

function allBitwiseOperatorsPassed() {
  let scope = {
    get andValue() { delete this.andValue; return 14; },
    get orValue() { delete this.orValue; return 8; },
    get xorValue() { delete this.xorValue; return 12; },
    get shlValue() { delete this.shlValue; return 3; },
    get shrValue() { delete this.shrValue; return -16; },
    get ushrValue() { delete this.ushrValue; return -16; }
  };
  let andResult;
  let orResult;
  let xorResult;
  let shlResult;
  let shrResult;
  let ushrResult;
  with (scope) {
    andResult = andValue &= 11;
    orResult = orValue |= 3;
    xorResult = xorValue ^= 10;
    shlResult = shlValue <<= 2;
    shrResult = shrValue >>= 2;
    ushrResult = ushrValue >>>= 2;
  }
  return andResult === 10
    && orResult === 11
    && xorResult === 6
    && shlResult === 12
    && shrResult === -4
    && ushrResult === 1073741820
    && scope.andValue === 10
    && scope.orValue === 11
    && scope.xorValue === 6
    && scope.shlValue === 12
    && scope.shrValue === -4
    && scope.ushrValue === 1073741820;
}

let allOperatorsPassed = allArithmeticOperatorsPassed()
  && allBitwiseOperatorsPassed();

// HasBinding, GetBindingValue, RHS evaluation and SetMutableBinding are
// independently observable. The getter deletes the selected property, and the
// RHS mutates the fallback, but PutValue still rechecks and writes that object.
function selectedReferenceSurvivesRhs() {
  let tracedValue = 40;
  let trace = "";
  let target = {};
  let proxy = new Proxy(target, {
    has(object, key) {
      if (key === "tracedValue") trace += "h";
      return key in object;
    },
    get(object, key) {
      if (key === Symbol.unscopables) trace += "u";
      if (key === "tracedValue") trace += "g";
      return object[key];
    },
    deleteProperty(object, key) {
      if (key === "tracedValue") trace += "d";
      return delete object[key];
    },
    set(object, key, value) {
      if (key === "tracedValue") trace += "s";
      object[key] = value;
      return true;
    }
  });
  Object.defineProperty(target, "tracedValue", {
    configurable: true,
    get() {
      delete proxy.tracedValue;
      return 2;
    }
  });
  function rhs() {
    trace += "r";
    tracedValue = 90;
    return 3;
  }
  let result;
  with (proxy) {
    result = tracedValue += rhs();
  }
  return trace === "huhgdrhs"
    && result === 5
    && target.tracedValue === 5
    && tracedValue === 90;
}

// The same-base rule also protects function, global and outer Object
// Environment Record fallbacks when GetValue removes the selected property.
function selectedAgainstFunctionFallback() {
  let functionFallbackValue = 100;
  let scope = {
    get functionFallbackValue() {
      delete this.functionFallbackValue;
      return 9;
    }
  };
  let result;
  with (scope) {
    result = functionFallbackValue -= 4;
  }
  return result === 5
    && scope.functionFallbackValue === 5
    && functionFallbackValue === 100;
}

var globalFallbackValue = 101;
let globalFallbackScope = {
  get globalFallbackValue() {
    delete this.globalFallbackValue;
    return 9;
  }
};
let globalFallbackResult;
with (globalFallbackScope) {
  globalFallbackResult = globalFallbackValue *= 4;
}
let globalFallbackPassed = globalFallbackResult === 36
  && globalFallbackScope.globalFallbackValue === 36
  && globalFallbackValue === 101;

let outerFallbackScope = { outerFallbackValue: 102 };
let innerSelectedScope = {
  get outerFallbackValue() {
    delete this.outerFallbackValue;
    return 17;
  }
};
let outerFallbackResult;
with (outerFallbackScope) {
  with (innerSelectedScope) {
    outerFallbackResult = outerFallbackValue ^= 6;
  }
}
let outerFallbackPassed = outerFallbackResult === 23
  && innerSelectedScope.outerFallbackValue === 23
  && outerFallbackScope.outerFallbackValue === 102;

// Strictness is carried by the captured Reference. The selected getter removes
// the property, so SetMutableBinding must throw before Set and must not fall
// through to the global binding.
var strictCompoundValue = 103;
let strictCompoundScope = {
  get strictCompoundValue() {
    delete this.strictCompoundValue;
    return 7;
  }
};
let runStrictCompound;
with (strictCompoundScope) {
  runStrictCompound = function () {
    "use strict";
    strictCompoundValue &= 3;
  };
}
let strictCompoundCaught = false;
try {
  runStrictCompound();
} catch (error) {
  strictCompoundCaught = error instanceof ReferenceError;
}
let strictCompoundPassed = strictCompoundCaught
  && !("strictCompoundValue" in strictCompoundScope)
  && strictCompoundValue === 103;

// HasBinding may change the pre-located fallback before declining the object
// binding. The fallback operation and post-expression metadata must therefore
// stay Dynamic rather than retaining the Number shape seen before `with`.
function mutatedLocalFallback() {
  let dynamicFallbackValue = 1;
  let scope = { dynamicFallbackValue: 99 };
  Object.defineProperty(scope, Symbol.unscopables, {
    get() {
      dynamicFallbackValue = "4";
      return { dynamicFallbackValue: true };
    }
  });
  let result;
  with (scope) {
    result = dynamicFallbackValue += 1;
  }
  return result === "41"
    && dynamicFallbackValue === "41"
    && scope.dynamicFallbackValue === 99;
}

// Even when the selected branch wins, its HasBinding query may mutate the
// untouched fallback to an arbitrary tag that later code must still observe.
function selectedBranchMutatesFallback() {
  let selectedFallbackValue = 1;
  let marker = {};
  let target = { selectedFallbackValue: 8 };
  let scope = new Proxy(target, {
    has(object, key) {
      if (key === "selectedFallbackValue") {
        selectedFallbackValue = marker;
        return true;
      }
      return key in object;
    }
  });
  let result;
  with (scope) {
    result = selectedFallbackValue >>= 1;
  }
  return result === 4
    && target.selectedFallbackValue === 4
    && selectedFallbackValue === marker;
}

// A declined Object Environment Record can delete a previously proven global.
// The fallback must recheck presence, throw before coercion/write and not
// recreate the deleted property.
globalThis.deletedCompoundFallback = 5;
let deletionScope = new Proxy({}, {
  has(object, key) {
    if (key === "deletedCompoundFallback") {
      delete globalThis.deletedCompoundFallback;
      return false;
    }
    return key in object;
  }
});
let deletedFallbackCaught = false;
let deletedFallbackCompleted = false;
with (deletionScope) {
  try {
    deletedCompoundFallback %= 2;
    deletedFallbackCompleted = true;
  } catch (error) {
    deletedFallbackCaught = error instanceof ReferenceError;
  }
}
let deletedFallbackType = typeof deletedCompoundFallback;
let deletedFallbackPassed = deletedFallbackCaught
  && !deletedFallbackCompleted
  && !("deletedCompoundFallback" in globalThis)
  && deletedFallbackType === "undefined";

// Conversely, HasBinding can create a previously unresolvable global before
// declining. The same run-time guard must admit the newly created fallback.
let creationScope = new Proxy({}, {
  has(object, key) {
    if (key === "createdCompoundFallback") {
      globalThis.createdCompoundFallback = 4;
      return false;
    }
    return key in object;
  }
});
let createdFallbackResult;
with (creationScope) {
  createdFallbackResult = createdCompoundFallback <<= 1;
}
let createdFallbackPassed = createdFallbackResult === 8
  && globalThis.createdCompoundFallback === 8;

allOperatorsPassed
  && selectedReferenceSurvivesRhs()
  && selectedAgainstFunctionFallback()
  && globalFallbackPassed
  && outerFallbackPassed
  && strictCompoundPassed
  && mutatedLocalFallback()
  && selectedBranchMutatesFallback()
  && deletedFallbackPassed
  && createdFallbackPassed;
