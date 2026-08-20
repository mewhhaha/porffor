// ObjectEnvironmentRecord.GetBindingValue performs a second HasProperty after
// HasBinding/@@unscopables selected the record. These cases keep resolution,
// the recheck and Get separately observable for direct identifier reads.

let state = {};

let trace = "";
let traceTarget = { x: 11 };
let traceProxy = new Proxy(traceTarget, {
  has(target, key) {
    if (key === "x") trace += "h";
    return key in target;
  },
  get(target, key, receiver) {
    if (key === Symbol.unscopables) trace += "u";
    if (key === "x") trace += "g";
    return target[key];
  }
});
with (traceProxy) {
  state.traceValue = x;
}
let secondHasPassed = trace === "huhg" && state.traceValue === 11;

let nestedTrace = "";
let outerTarget = { x: 17 };
let outerProxy = new Proxy(outerTarget, {
  has(target, key) {
    if (key === "x") nestedTrace += "o";
    return key in target;
  },
  get(target, key, receiver) {
    if (key === Symbol.unscopables) nestedTrace += "u";
    if (key === "x") nestedTrace += "g";
    return target[key];
  }
});
let innerProxy = new Proxy({}, {
  has(target, key) {
    if (key === "x") nestedTrace += "i";
    return key in target;
  }
});
with (outerProxy) {
  with (innerProxy) {
    state.nestedValue = x;
  }
}
let outerResolutionPassed = nestedTrace === "iouog"
  && state.nestedValue === 17;

let lexicalHasCalls = 0;
let lexicalProxy = new Proxy({ x: 99 }, {
  has(target, key) {
    if (key === "x") lexicalHasCalls++;
    return key in target;
  }
});
with (lexicalProxy) {
  {
    let x = 23;
    state.lexicalValue = x;
  }
}
let declarativeCutoffPassed = lexicalHasCalls === 0
  && state.lexicalValue === 23;

let sloppyTarget = { x: 5 };
let sloppyProxy = new Proxy(sloppyTarget, {
  has(target, key) {
    return key in target;
  },
  get(target, key, receiver) {
    if (key === Symbol.unscopables) {
      delete target.x;
      return null;
    }
    return target[key];
  }
});
with (sloppyProxy) {
  state.sloppyValue = x;
}
let sloppyDeletionPassed = state.sloppyValue === undefined
  && !("x" in sloppyTarget);

let strictTarget = { x: 7 };
let strictProxy = new Proxy(strictTarget, {
  has(target, key) {
    return key in target;
  },
  get(target, key, receiver) {
    if (key === Symbol.unscopables) {
      delete target.x;
      return null;
    }
    return target[key];
  }
});
var strictRead;
with (strictProxy) {
  strictRead = function () {
    "use strict";
    return x;
  };
}
let strictCaught = false;
try {
  strictRead();
} catch (error) {
  strictCaught = error instanceof ReferenceError;
}
let strictDeletionPassed = strictCaught && !("x" in strictTarget);

let abruptMarker = {};
let abruptHasCalls = 0;
let abruptGetCalls = 0;
let abruptProxy = new Proxy({ x: 1 }, {
  has(target, key) {
    if (key === "x") {
      abruptHasCalls++;
      if (abruptHasCalls === 2) throw abruptMarker;
    }
    return key in target;
  },
  get(target, key, receiver) {
    if (key === "x") abruptGetCalls++;
    return target[key];
  }
});
let abruptCaught = false;
with (abruptProxy) {
  try {
    state.abruptValue = x;
  } catch (error) {
    abruptCaught = error === abruptMarker;
  }
}
let abruptRecheckPassed = abruptCaught
  && abruptHasCalls === 2
  && abruptGetCalls === 0;

let blockedOuter = { x: 29 };
let blockedInner = { x: 1 };
blockedInner[Symbol.unscopables] = { x: true };
with (blockedOuter) {
  with (blockedInner) {
    state.blockedValue = x;
  }
}
let blockedContinuesPassed = state.blockedValue === 29;

let typeofTarget = { onlyInWith: 41 };
with (typeofTarget) {
  state.typeofWith = typeof onlyInWith;
  state.typeofMissing = typeof absentEverywhere;
  state.typeofParenthesizedMissing = typeof (absentParenthesized);
}
let typeofPassed = state.typeofWith === "number"
  && state.typeofMissing === "undefined"
  && state.typeofParenthesizedMissing === "undefined";

secondHasPassed
  && outerResolutionPassed
  && declarativeCutoffPassed
  && sloppyDeletionPassed
  && strictDeletionPassed
  && abruptRecheckPassed
  && blockedContinuesPassed
  && typeofPassed;
