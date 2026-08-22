function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

// All four update modes preserve old-versus-new result selection for both
// Number and BigInt. Each computed key is coerced exactly once.
var values = {
  numberPrefixIncrement: 1,
  numberPostfixIncrement: 1,
  numberPrefixDecrement: 2,
  numberPostfixDecrement: 2,
  bigintPrefixIncrement: 1n,
  bigintPostfixIncrement: 1n,
  bigintPrefixDecrement: 2n,
  bigintPostfixDecrement: 2n
};
var updateKeyCoercions = "";
function updateKey(name) {
  return {
    toString() {
      updateKeyCoercions += name + ",";
      return name;
    }
  };
}

var numberPrefixIncrement = ++values[updateKey("numberPrefixIncrement")];
var numberPostfixIncrement = values[updateKey("numberPostfixIncrement")]++;
var numberPrefixDecrement = --values[updateKey("numberPrefixDecrement")];
var numberPostfixDecrement = values[updateKey("numberPostfixDecrement")]--;
var bigintPrefixIncrement = ++values[updateKey("bigintPrefixIncrement")];
var bigintPostfixIncrement = values[updateKey("bigintPostfixIncrement")]++;
var bigintPrefixDecrement = --values[updateKey("bigintPrefixDecrement")];
var bigintPostfixDecrement = values[updateKey("bigintPostfixDecrement")]--;

check(numberPrefixIncrement, 2, "Number prefix increment result");
check(numberPostfixIncrement, 1, "Number postfix increment result");
check(numberPrefixDecrement, 1, "Number prefix decrement result");
check(numberPostfixDecrement, 2, "Number postfix decrement result");
check(values.numberPrefixIncrement, 2, "Number prefix increment write");
check(values.numberPostfixIncrement, 2, "Number postfix increment write");
check(values.numberPrefixDecrement, 1, "Number prefix decrement write");
check(values.numberPostfixDecrement, 1, "Number postfix decrement write");
check(bigintPrefixIncrement, 2n, "BigInt prefix increment result");
check(bigintPostfixIncrement, 1n, "BigInt postfix increment result");
check(bigintPrefixDecrement, 1n, "BigInt prefix decrement result");
check(bigintPostfixDecrement, 2n, "BigInt postfix decrement result");
check(values.bigintPrefixIncrement, 2n, "BigInt prefix increment write");
check(values.bigintPostfixIncrement, 2n, "BigInt postfix increment write");
check(values.bigintPrefixDecrement, 1n, "BigInt prefix decrement write");
check(values.bigintPostfixDecrement, 1n, "BigInt postfix decrement write");
check(
  updateKeyCoercions,
  "numberPrefixIncrement,numberPostfixIncrement,numberPrefixDecrement,numberPostfixDecrement,bigintPrefixIncrement,bigintPostfixIncrement,bigintPrefixDecrement,bigintPostfixDecrement,",
  "one key coercion per update",
);

// Reference evaluation is base first, then the raw key expression. Either
// abrupt completion prevents all later phases.
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
var abruptBaseCaught = false;
try {
  abruptBase()[unreachableRawKey()]++;
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
  ordinaryBase()[abruptRawKey()]++;
} catch (error) {
  abruptKeyCaught = error === abruptMarker;
}
check(abruptKeyCaught, true, "abrupt raw key identity");
check(abruptTrace.join(","), "base,raw-key", "abrupt raw key order");

// The computed-key expression runs for a nullish base, but GetValue rejects
// that base before observing ToPropertyKey.
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
  var caught = false;
  try {
    base[rawKey()]++;
  } catch (error) {
    caught = error instanceof TypeError;
  }
  check(caught, true, label + " TypeError");
  check(trace.join(","), "raw-key", label + " before ToPropertyKey");
}
nullishReference(null, "null base");
nullishReference(undefined, "undefined base");

// A coercion error occurs after raw-key evaluation and before [[Get]].
var keyCoercionTrace = [];
var keyCoercionMarker = {};
var keyCoercionTarget = {
  get p() {
    keyCoercionTrace.push("get");
    return 1;
  }
};
var abruptCoercionKey = {
  toString() {
    keyCoercionTrace.push("to-key");
    throw keyCoercionMarker;
  }
};
function rawCoercionKey() {
  keyCoercionTrace.push("raw-key");
  return abruptCoercionKey;
}
var keyCoercionCaught = false;
try {
  keyCoercionTarget[rawCoercionKey()]++;
} catch (error) {
  keyCoercionCaught = error === keyCoercionMarker;
}
check(keyCoercionCaught, true, "ToPropertyKey abrupt identity");
check(keyCoercionTrace.join(","), "raw-key,to-key", "ToPropertyKey abrupt order");

// The canonical key and receiver survive GetValue and ToNumeric. Mutating the
// raw key during ToNumeric cannot redirect PutValue.
var lifecycleTrace = [];
var lifecycleKey = {
  name: "p",
  toString() {
    lifecycleTrace.push("to-key:" + this.name);
    return this.name;
  }
};
var lifecycleStored = 1;
var lifecycleOldValue = {
  valueOf() {
    lifecycleTrace.push("to-numeric");
    lifecycleKey.name = "q";
    return lifecycleStored;
  }
};
var lifecycleTarget = { q: 100 };
Object.defineProperty(lifecycleTarget, "p", {
  configurable: true,
  get() {
    lifecycleTrace.push("getter:" + (this === lifecycleProxy));
    return lifecycleOldValue;
  },
  set(value) {
    lifecycleTrace.push("setter:" + (this === lifecycleProxy) + ":" + value);
    lifecycleStored = value;
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
function lifecycleBase() {
  lifecycleTrace.push("base");
  return lifecycleProxy;
}
function lifecycleRawKey() {
  lifecycleTrace.push("raw-key");
  return lifecycleKey;
}
var lifecycleResult = ++lifecycleBase()[lifecycleRawKey()];
check(lifecycleResult, 2, "successful prefix result publication");
check(lifecycleStored, 2, "successful setter value");
check(lifecycleTarget.q, 100, "mutated raw key not recoerced");
check(
  lifecycleTrace.join(","),
  "base,raw-key,to-key:p,proxy-get:p:true,getter:true,to-numeric,proxy-set:p:true:2,setter:true:2",
  "complete numeric update Reference lifecycle",
);

// An abrupt ToNumeric and a strict false [[Set]] cannot publish a result.
var numericMarker = {};
var numericTrace = [];
var abruptNumericTarget = new Proxy(
  {
    p: {
      valueOf() {
        numericTrace.push("to-numeric");
        throw numericMarker;
      }
    }
  },
  {
    get(target, key, receiver) {
      numericTrace.push("get:" + key);
      return Reflect.get(target, key, receiver);
    },
    set() {
      numericTrace.push("set");
      return true;
    }
  },
);
var numericPublished = "not published";
var numericCaught = false;
try {
  numericPublished = abruptNumericTarget["p"]++;
} catch (error) {
  numericCaught = error === numericMarker;
}
check(numericCaught, true, "ToNumeric abrupt identity");
check(numericPublished, "not published", "ToNumeric abrupt nonpublication");
check(numericTrace.join(","), "get:p,to-numeric", "ToNumeric abrupt skips Set");

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
function strictReject() {
  "use strict";
  return rejectingProxy[rejectingKey]++;
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
check(rejectingTrace.join(","), "to-key,get:p,set:p:6", "strict Set false order");

rejectingTrace = [];
var sloppyResult = rejectingProxy[rejectingKey]++;
check(sloppyResult, 5, "sloppy Set false postfix result");
check(rejectingTarget.p, 5, "sloppy Set false ignored write");
check(rejectingTrace.join(","), "to-key,get:p,set:p:6", "sloppy Set false order");

true;
