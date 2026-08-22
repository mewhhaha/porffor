function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

// A plain property Reference evaluates its base and raw computed-key
// expression before the RHS, but defers ToPropertyKey until PutValue.
var lifecycleTrace = [];
var lifecycleStored = 0;
var lifecycleTarget = {};
var lifecycleProxy;
Object.defineProperty(lifecycleTarget, "p", {
  configurable: true,
  set(value) {
    lifecycleTrace.push("setter:" + (this === lifecycleProxy) + ":" + value);
    lifecycleStored = value;
  }
});
lifecycleProxy = new Proxy(lifecycleTarget, {
  set(target, key, value, receiver) {
    lifecycleTrace.push(
      "proxy-set:" + key + ":" + (receiver === lifecycleProxy) + ":" + value,
    );
    return Reflect.set(target, key, value, receiver);
  }
});
var lifecycleKeyCalls = 0;
var lifecycleKey = {
  toString() {
    lifecycleKeyCalls += 1;
    lifecycleTrace.push("to-key");
    return "p";
  }
};
function lifecycleBase() {
  lifecycleTrace.push("base");
  return lifecycleProxy;
}
function lifecycleRawKey() {
  lifecycleTrace.push("raw-key");
  return lifecycleKey;
}
function lifecycleRhs() {
  lifecycleTrace.push("rhs");
  return 7;
}
var lifecycleResult = lifecycleBase()[lifecycleRawKey()] = lifecycleRhs();
check(lifecycleResult, 7, "successful assignment result");
check(lifecycleStored, 7, "successful assignment write");
check(lifecycleKeyCalls, 1, "sole ToPropertyKey");
check(
  lifecycleTrace.join(","),
  "base,raw-key,rhs,to-key,proxy-set:p:true:7,setter:true:7",
  "complete plain property Reference lifecycle",
);

// The key expression is part of reference evaluation. Abrupt base/key
// evaluation prevents every later phase.
var abruptMarker = {};
var abruptTrace = [];
function abruptBase() {
  abruptTrace.push("base");
  throw abruptMarker;
}
function unreachableRawKey() {
  abruptTrace.push("raw-key");
  return "p";
}
function unreachableRhs() {
  abruptTrace.push("rhs");
  return 1;
}
var abruptBaseCaught = false;
try {
  abruptBase()[unreachableRawKey()] = unreachableRhs();
} catch (error) {
  abruptBaseCaught = error === abruptMarker;
}
check(abruptBaseCaught, true, "abrupt base identity");
check(abruptTrace.join(","), "base", "abrupt base order");

abruptTrace = [];
function ordinaryBase() {
  abruptTrace.push("base");
  return {};
}
function abruptRawKey() {
  abruptTrace.push("raw-key");
  throw abruptMarker;
}
var abruptKeyCaught = false;
try {
  ordinaryBase()[abruptRawKey()] = unreachableRhs();
} catch (error) {
  abruptKeyCaught = error === abruptMarker;
}
check(abruptKeyCaught, true, "abrupt raw key identity");
check(abruptTrace.join(","), "base,raw-key", "abrupt raw key order");

// The RHS wins over nullish validation and key coercion. If the RHS returns,
// PutValue rejects null/undefined before observing ToPropertyKey.
function nullishReference(base, label) {
  var rhsMarker = {};
  var trace = [];
  var key = {
    toString() {
      trace.push("to-key");
      return "p";
    }
  };
  function rawKey() {
    trace.push("raw-key");
    return key;
  }
  function throwingRhs() {
    trace.push("rhs-throw");
    throw rhsMarker;
  }
  var rhsCaught = false;
  try {
    base[rawKey()] = throwingRhs();
  } catch (error) {
    rhsCaught = error === rhsMarker;
  }
  check(rhsCaught, true, label + " RHS throw identity");
  check(trace.join(","), "raw-key,rhs-throw", label + " RHS before PutValue");

  trace = [];
  function normalRhs() {
    trace.push("rhs");
    return 1;
  }
  var nullishCaught = false;
  try {
    base[rawKey()] = normalRhs();
  } catch (error) {
    nullishCaught = error instanceof TypeError;
  }
  check(nullishCaught, true, label + " TypeError");
  check(trace.join(","), "raw-key,rhs", label + " before ToPropertyKey");
}
nullishReference(null, "null base");
nullishReference(undefined, "undefined base");

function staticNullishReference(base, label) {
  var rhsCalls = 0;
  var caught = false;
  try {
    base.p = (rhsCalls += 1);
  } catch (error) {
    caught = error instanceof TypeError;
  }
  check(caught, true, label + " static-name TypeError");
  check(rhsCalls, 1, label + " static-name RHS exactly once");
}
staticNullishReference(null, "null base");
staticNullishReference(undefined, "undefined base");

// Because coercion belongs to PutValue, an RHS mutation changes which key is
// finally written. The raw key is nevertheless coerced exactly once.
var mutationTrace = [];
var mutationTarget = { p: 1, q: 2 };
var mutationKeyCalls = 0;
var mutationKey = {
  name: "p",
  toString() {
    mutationKeyCalls += 1;
    mutationTrace.push("to-key:" + this.name);
    return this.name;
  }
};
function mutationRhs() {
  mutationTrace.push("rhs");
  mutationKey.name = "q";
  return 9;
}
var mutationResult = mutationTarget[mutationKey] = mutationRhs();
check(mutationResult, 9, "mutated raw key result");
check(mutationTarget.p, 1, "mutated raw key preserves old property");
check(mutationTarget.q, 9, "RHS mutation precedes ToPropertyKey");
check(mutationKeyCalls, 1, "mutated raw key sole ToPropertyKey");
check(mutationTrace.join(","), "rhs,to-key:q", "RHS before key coercion");

// An abrupt PutValue cannot publish the assignment result.
var setMarker = {};
var throwingSetTrace = [];
var throwingSetProxy = new Proxy({}, {
  set(target, key, value, receiver) {
    throwingSetTrace.push("set:" + key + ":" + value);
    throw setMarker;
  }
});
var throwingSetPublished = "not published";
var throwingSetCaught = false;
try {
  throwingSetPublished = throwingSetProxy.p = 11;
} catch (error) {
  throwingSetCaught = error === setMarker;
}
check(throwingSetCaught, true, "abrupt Set identity");
check(throwingSetPublished, "not published", "abrupt Set nonpublication");
check(throwingSetTrace.join(","), "set:p:11", "abrupt Set exactly once");

// A false [[Set]] is ignored in sloppy code and throws in strict code. The
// primitive cases keep their original unboxed Receiver through PutValue.
var falseSetTrace = [];
var falseSetTarget = { p: 1 };
var falseSetProxy = new Proxy(falseSetTarget, {
  set(target, key, value, receiver) {
    falseSetTrace.push("set:" + key + ":" + value + ":" + (receiver === falseSetProxy));
    return false;
  }
});
var sloppyFalseSetResult = falseSetProxy.p = 12;
check(sloppyFalseSetResult, 12, "sloppy false Set result");
check(falseSetTarget.p, 1, "sloppy false Set no write");
check(falseSetTrace.join(","), "set:p:12:true", "sloppy false Set order");

function strictFalseSet() {
  "use strict";
  return falseSetProxy.p = 13;
}
falseSetTrace = [];
var strictFalseSetResult = "not published";
var strictFalseSetCaught = false;
try {
  strictFalseSetResult = strictFalseSet();
} catch (error) {
  strictFalseSetCaught = error instanceof TypeError;
}
check(strictFalseSetCaught, true, "strict false Set TypeError");
check(strictFalseSetResult, "not published", "strict false Set nonpublication");
check(falseSetTarget.p, 1, "strict false Set no write");
check(falseSetTrace.join(","), "set:p:13:true", "strict false Set order");

var sloppyPrimitiveResult = (1).p = 14;
check(sloppyPrimitiveResult, 14, "sloppy primitive assignment result");

function strictPrimitiveSet() {
  "use strict";
  return (1).p = 15;
}
var strictPrimitiveResult = "not published";
var strictPrimitiveCaught = false;
try {
  strictPrimitiveResult = strictPrimitiveSet();
} catch (error) {
  strictPrimitiveCaught = error instanceof TypeError;
}
check(strictPrimitiveCaught, true, "strict primitive Set TypeError");
check(strictPrimitiveResult, "not published", "strict primitive Set nonpublication");

true;
