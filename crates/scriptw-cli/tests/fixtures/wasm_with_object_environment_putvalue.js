// ObjectEnvironmentRecord.SetMutableBinding re-runs HasProperty after the RHS.
// These cases keep initial ResolveBinding, RHS, recheck and Set separately
// observable while exercising only plain identifier `=` through `with`.

let strictRhsCalls = 0;
var x = 91;
let strictScope = { x: 1 };
let strictState = { caught: false, after: false };
var strictClosure;
function strictDeleteRhs() {
  strictRhsCalls++;
  delete strictScope.x;
  return 2;
}
with (strictScope) {
  strictClosure = function () {
    "use strict";
    try {
      x = strictDeleteRhs();
      strictState.after = true;
    } catch (error) {
      strictState.caught = error instanceof ReferenceError;
    }
  };
}
strictClosure();
let strictDeletePassed = strictState.caught
  && !strictState.after
  && strictRhsCalls === 1
  && !("x" in strictScope)
  && x === 91;

let repeatedFirst = { x: 0 };
let repeatedSecond = { x: 0 };
function makeEscapingWriter(target) {
  let write;
  with (target) {
    write = function () {
      "use strict";
      x = 7;
    };
  }
  return write;
}
let writeFirst = makeEscapingWriter(repeatedFirst);
let writeSecond = makeEscapingWriter(repeatedSecond);
writeFirst();
writeSecond();
let repeatedCapturePassed = repeatedFirst.x === 7
  && repeatedSecond.x === 7;

let shadowHasCalls = 0;
let shadowSetCalls = 0;
let shadowTarget = { x: 1 };
let shadowProxy = new Proxy(shadowTarget, {
  has(target, key) {
    if (key === "x") shadowHasCalls++;
    return key in target;
  },
  set(target, key, value) {
    if (key === "x") shadowSetCalls++;
    target[key] = value;
    return true;
  }
});
with (shadowProxy) {
  {
    let x = 0;
    x = 2;
  }
}
let declarativeShadowPassed = shadowHasCalls === 0
  && shadowSetCalls === 0
  && shadowTarget.x === 1;

let interleavedInnerHasCalls = 0;
let interleavedOuterHasCalls = 0;
let interleavedOuterSetCalls = 0;
let interleavedInnerProxy = new Proxy({}, {
  has(target, key) {
    if (key === "x") interleavedInnerHasCalls++;
    return key in target;
  }
});
let interleavedOuterTarget = { x: 1 };
let interleavedOuterProxy = new Proxy(interleavedOuterTarget, {
  has(target, key) {
    if (key === "x") interleavedOuterHasCalls++;
    return key in target;
  },
  set(target, key, value) {
    if (key === "x") interleavedOuterSetCalls++;
    target[key] = value;
    return true;
  }
});
with (interleavedOuterProxy) {
  {
    let x = 0;
    with (interleavedInnerProxy) {
      x = 3;
    }
  }
}
let interleavingPassed = interleavedInnerHasCalls === 1
  && interleavedOuterHasCalls === 0
  && interleavedOuterSetCalls === 0
  && interleavedOuterTarget.x === 1;

let sloppyTrace = "";
let sloppyTarget = { x: 1 };
let sloppyProxy = new Proxy(sloppyTarget, {
  has(target, key) {
    if (key === "x") sloppyTrace += "h";
    return key in target;
  },
  set(target, key, value) {
    if (key === "x") sloppyTrace += "s";
    target[key] = value;
    return true;
  }
});
function sloppyDeleteRhs() {
  sloppyTrace += "r";
  delete sloppyTarget.x;
  return 3;
}
with (sloppyProxy) {
  x = sloppyDeleteRhs();
}
let sloppyRecheckPassed = sloppyTrace === "hrhs" && sloppyTarget.x === 3;

let recheckMarker = {};
let recheckHasCalls = 0;
let recheckSetCalls = 0;
let recheckState = { caught: false };
let recheckProxy = new Proxy({ x: 1 }, {
  has(target, key) {
    if (key === "x") {
      recheckHasCalls++;
      if (recheckHasCalls === 2) throw recheckMarker;
    }
    return key in target;
  },
  set() {
    recheckSetCalls++;
    return true;
  }
});
with (recheckProxy) {
  try {
    x = 4;
  } catch (error) {
    recheckState.caught = error === recheckMarker;
  }
}
let abruptRecheckPassed = recheckState.caught
  && recheckHasCalls === 2
  && recheckSetCalls === 0;

let rhsMarker = {};
let rhsHasCalls = 0;
let rhsSetCalls = 0;
let rhsState = { caught: false };
let rhsProxy = new Proxy({ x: 1 }, {
  has(target, key) {
    if (key === "x") rhsHasCalls++;
    return key in target;
  },
  set() {
    rhsSetCalls++;
    return true;
  }
});
function abruptRhs() {
  throw rhsMarker;
}
with (rhsProxy) {
  try {
    x = abruptRhs();
  } catch (error) {
    rhsState.caught = error === rhsMarker;
  }
}
let abruptRhsPassed = rhsState.caught
  && rhsHasCalls === 1
  && rhsSetCalls === 0;

var unscopableFallback = 1;
let unscopableHasCalls = 0;
let unscopableSetCalls = 0;
let unscopableTarget = { unscopableFallback: 2 };
unscopableTarget[Symbol.unscopables] = { unscopableFallback: true };
let unscopableProxy = new Proxy(unscopableTarget, {
  has(target, key) {
    if (key === "unscopableFallback") unscopableHasCalls++;
    return key in target;
  },
  set(target, key, value) {
    if (key === "unscopableFallback") unscopableSetCalls++;
    target[key] = value;
    return true;
  }
});
with (unscopableProxy) {
  unscopableFallback = 5;
}
let unscopablesPassed = unscopableFallback === 5
  && unscopableTarget.unscopableFallback === 2
  && unscopableHasCalls === 1
  && unscopableSetCalls === 0;

let innerHasCalls = 0;
let innerSetCalls = 0;
let outerHasCalls = 0;
let outerSetCalls = 0;
let innerTarget = {};
let outerTarget = { x: 1 };
let innerProxy = new Proxy(innerTarget, {
  has(target, key) {
    if (key === "x") innerHasCalls++;
    return key in target;
  },
  set(target, key, value) {
    if (key === "x") innerSetCalls++;
    target[key] = value;
    return true;
  }
});
let outerProxy = new Proxy(outerTarget, {
  has(target, key) {
    if (key === "x") outerHasCalls++;
    return key in target;
  },
  set(target, key, value) {
    if (key === "x") outerSetCalls++;
    target[key] = value;
    return true;
  }
});
function nestedRhs() {
  return 6;
}
with (outerProxy) {
  with (innerProxy) {
    x = nestedRhs();
  }
}
let nestedPassed = innerHasCalls === 1
  && innerSetCalls === 0
  && outerHasCalls === 2
  && outerSetCalls === 1
  && outerTarget.x === 6
  && !("x" in innerTarget);

strictDeletePassed
  && repeatedCapturePassed
  && declarativeShadowPassed
  && interleavingPassed
  && sloppyRecheckPassed
  && abruptRecheckPassed
  && abruptRhsPassed
  && unscopablesPassed
  && nestedPassed;
