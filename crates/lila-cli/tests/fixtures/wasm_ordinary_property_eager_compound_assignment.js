function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

var values = {
  mul: 4,
  div: 12,
  mod: 14,
  add: 10,
  sub: 10,
  shl: 3,
  shr: -16,
  ushr: -16,
  and: 14,
  xor: 12,
  or: 8,
  exp: 3
};
var operatorKeyCoercions = "";
function operatorKey(name) {
  return {
    toString() {
      operatorKeyCoercions += name + ",";
      return name;
    }
  };
}

var mulResult = values[operatorKey("mul")] *= 3;
var divResult = values[operatorKey("div")] /= 3;
var modResult = values[operatorKey("mod")] %= 5;
var addResult = values[operatorKey("add")] += 3;
var subResult = values[operatorKey("sub")] -= 3;
var shlResult = values[operatorKey("shl")] <<= 2;
var shrResult = values[operatorKey("shr")] >>= 2;
var ushrResult = values[operatorKey("ushr")] >>>= 2;
var andResult = values[operatorKey("and")] &= 11;
var xorResult = values[operatorKey("xor")] ^= 10;
var orResult = values[operatorKey("or")] |= 3;
var expResult = values[operatorKey("exp")] **= 3;

check(mulResult, 12, "*=");
check(divResult, 4, "/=");
check(modResult, 4, "%=");
check(addResult, 13, "+=");
check(subResult, 7, "-=");
check(shlResult, 12, "<<=");
check(shrResult, -4, ">>=");
check(ushrResult, 1073741820, ">>>=");
check(andResult, 10, "&=");
check(xorResult, 6, "^=");
check(orResult, 11, "|=");
check(expResult, 27, "**= local boundary");
check(
  operatorKeyCoercions,
  "mul,div,mod,add,sub,shl,shr,ushr,and,xor,or,exp,",
  "one key coercion per eager operator",
);

// Reference evaluation is base expression, then raw key expression. An
// abrupt completion in either one prevents every later stage.
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
function abruptReferenceRhs() {
  abruptTrace.push("rhs");
  return 1;
}
var abruptBaseCaught = false;
try {
  abruptBase()[unreachableRawKey()] += abruptReferenceRhs();
} catch (error) {
  abruptBaseCaught = error === abruptMarker;
}
check(abruptBaseCaught, true, "abrupt base identity");
check(abruptTrace.join(","), "base", "abrupt base order");

abruptTrace = [];
function ordinaryBase() {
  abruptTrace.push("base");
  return { p: 1 };
}
function abruptRawKey() {
  abruptTrace.push("raw-key");
  throw abruptMarker;
}
var abruptKeyCaught = false;
try {
  ordinaryBase()[abruptRawKey()] += abruptReferenceRhs();
} catch (error) {
  abruptKeyCaught = error === abruptMarker;
}
check(abruptKeyCaught, true, "abrupt raw key identity");
check(abruptTrace.join(","), "base,raw-key", "abrupt raw key order");

// The raw key expression is evaluated for a nullish base, but GetValue rejects
// that base before observing ToPropertyKey or the RHS.
function nullishReference(base, label) {
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
  function rhs() {
    trace.push("rhs");
    return 1;
  }
  var caught = false;
  try {
    base[rawKey()] += rhs();
  } catch (error) {
    caught = error instanceof TypeError;
  }
  check(caught, true, label + " TypeError");
  check(trace.join(","), "raw-key", label + " before ToPropertyKey");
}
nullishReference(null, "null base");
nullishReference(undefined, "undefined base");

// A normal base reaches ToPropertyKey before [[Get]], and a coercion error
// prevents getter and RHS evaluation.
var coercionTrace = [];
var coercionMarker = {};
var coercionTarget = {
  get p() {
    coercionTrace.push("get");
    return 1;
  }
};
var coercionKey = {
  toString() {
    coercionTrace.push("to-key");
    throw coercionMarker;
  }
};
function coercionRawKey() {
  coercionTrace.push("raw-key");
  return coercionKey;
}
function coercionRhs() {
  coercionTrace.push("rhs");
  return 1;
}
var coercionCaught = false;
try {
  coercionTarget[coercionRawKey()] += coercionRhs();
} catch (error) {
  coercionCaught = error === coercionMarker;
}
check(coercionCaught, true, "ToPropertyKey abrupt identity");
check(coercionTrace.join(","), "raw-key,to-key", "ToPropertyKey abrupt order");

// The successful path retains one canonical key and one receiver across
// Proxy [[Get]], an accessor getter, RHS, Proxy [[Set]], and the setter.
var lifecycleTrace = [];
var stored = 1;
var lifecycleTarget = {};
Object.defineProperty(lifecycleTarget, "p", {
  configurable: true,
  get() {
    lifecycleTrace.push("getter:" + (this === lifecycleProxy));
    return stored;
  },
  set(value) {
    lifecycleTrace.push("setter:" + (this === lifecycleProxy) + ":" + value);
    stored = value;
  }
});
var lifecycleProxy = new Proxy(lifecycleTarget, {
  get(target, key, receiver) {
    lifecycleTrace.push("proxy-get:" + key + ":" + (receiver === lifecycleProxy));
    return Reflect.get(target, key, receiver);
  },
  set(target, key, value, receiver) {
    lifecycleTrace.push(
      "proxy-set:" + key + ":" + (receiver === lifecycleProxy) + ":" + value,
    );
    return Reflect.set(target, key, value, receiver);
  }
});
var lifecycleKey = {
  toString() {
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
  return 2;
}
var lifecycleResult = lifecycleBase()[lifecycleRawKey()] += lifecycleRhs();
check(lifecycleResult, 3, "successful result publication");
check(stored, 3, "successful setter value");
check(
  lifecycleTrace.join(","),
  "base,raw-key,to-key,proxy-get:p:true,getter:true,rhs,proxy-set:p:true:3,setter:true:3",
  "complete Reference lifecycle",
);

// Mutating the raw key after GetValue cannot redirect PutValue: the write owns
// the canonical key produced before the RHS.
var mutationTrace = [];
var mutationTarget = { p: 4, q: 100 };
var mutationKey = {
  name: "p",
  toString() {
    mutationTrace.push("to-key:" + this.name);
    return this.name;
  }
};
function mutationRhs() {
  mutationTrace.push("rhs");
  mutationKey.name = "q";
  return 2;
}
var mutationResult = mutationTarget[mutationKey] *= mutationRhs();
check(mutationResult, 8, "mutated raw key result");
check(mutationTarget.p, 8, "canonical key write");
check(mutationTarget.q, 100, "mutated raw key not recoerced");
check(mutationTrace.join(","), "to-key:p,rhs", "sole ToPropertyKey before RHS");

// A throwing RHS and a false [[Set]] result cannot publish the applied value.
var rejectingTarget = { p: 5 };
var rejectingTrace = [];
var rejectingProxy = new Proxy(rejectingTarget, {
  get(target, key, receiver) {
    rejectingTrace.push("get:" + key);
    return Reflect.get(target, key, receiver);
  },
  set(target, key, value, receiver) {
    rejectingTrace.push("set:" + key + ":" + value);
    return false;
  }
});
var rejectingKey = {
  toString() {
    rejectingTrace.push("to-key");
    return "p";
  }
};
function rejectingRhs() {
  rejectingTrace.push("rhs");
  return 2;
}
function strictReject() {
  "use strict";
  return rejectingProxy[rejectingKey] += rejectingRhs();
}
var strictResult = "not published";
var strictCaught = false;
try {
  strictResult = strictReject();
} catch (error) {
  strictCaught = error instanceof TypeError;
}
check(strictCaught, true, "strict Set false TypeError");
check(strictResult, "not published", "strict Set false nonpublication");
check(rejectingTarget.p, 5, "strict Set false no write");
check(rejectingTrace.join(","), "to-key,get:p,rhs,set:p:7", "strict Set false order");

rejectingTrace = [];
var sloppyResult = rejectingProxy[rejectingKey] += rejectingRhs();
check(sloppyResult, 7, "sloppy Set false result");
check(rejectingTarget.p, 5, "sloppy Set false ignored write");
check(rejectingTrace.join(","), "to-key,get:p,rhs,set:p:7", "sloppy Set false order");

var rhsMarker = {};
var rhsTrace = [];
var rhsTarget = new Proxy({ p: 9 }, {
  get(target, key, receiver) {
    rhsTrace.push("get");
    return Reflect.get(target, key, receiver);
  },
  set() {
    rhsTrace.push("set");
    return true;
  }
});
var rhsPublished = "not published";
var rhsCaught = false;
function throwingRhs() {
  rhsTrace.push("rhs");
  throw rhsMarker;
}
try {
  rhsPublished = rhsTarget["p"] += throwingRhs();
} catch (error) {
  rhsCaught = error === rhsMarker;
}
check(rhsCaught, true, "RHS abrupt identity");
check(rhsPublished, "not published", "RHS abrupt nonpublication");
check(rhsTrace.join(","), "get,rhs", "RHS abrupt skips Set");

true;
