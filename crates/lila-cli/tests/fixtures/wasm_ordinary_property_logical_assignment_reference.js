function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

function expectTypeError(thunk, label) {
  var caught = false;
  try {
    thunk();
  } catch (error) {
    caught = error instanceof TypeError;
  }
  check(caught, true, label);
}

// One ordinary property logical-assignment Reference owns base evaluation,
// the raw key, one ToPropertyKey/GetValue, its selected RHS, and PutValue.
var lifecycleTrace = [];
var lifecycleStored = 0;
var lifecycleTarget = {};
Object.defineProperty(lifecycleTarget, "p", {
  configurable: true,
  get: function() {
    lifecycleTrace.push("get:" + (this === lifecycleTarget));
    return lifecycleStored;
  },
  set: function(value) {
    lifecycleTrace.push("set:" + (this === lifecycleTarget) + ":" + value);
    lifecycleStored = value;
  }
});
var lifecycleKeyCalls = 0;
var lifecycleKey = {
  toString: function() {
    lifecycleKeyCalls += 1;
    lifecycleTrace.push("to-key");
    return lifecycleKeyCalls === 1 ? "p" : "q";
  }
};
function lifecycleBase() {
  lifecycleTrace.push("base");
  return lifecycleTarget;
}
function lifecycleRawKey() {
  lifecycleTrace.push("raw-key");
  return lifecycleKey;
}
function lifecycleRhs() {
  lifecycleTrace.push("rhs");
  return 7;
}
var lifecycleResult = lifecycleBase()[lifecycleRawKey()] ||= lifecycleRhs();
check(lifecycleResult, 7, "taken logical result");
check(lifecycleStored, 7, "taken logical write");
check(lifecycleKeyCalls, 1, "sole ToPropertyKey");
check(lifecycleTarget.q, undefined, "same canonical key for Get and Set");
check(
  lifecycleTrace.join(","),
  "base,raw-key,to-key,get:true,rhs,set:true:7",
  "complete logical property Reference lifecycle",
);

// Every logical operator keeps its RHS and PutValue in the selected branch.
var branchRhsCalls = 0;
function branchRhs(value) {
  branchRhsCalls += 1;
  return value;
}
var andTarget = { p: 2 };
var orTarget = { p: 0 };
var nullishTarget = { p: null };
check(andTarget.p &&= branchRhs(3), 3, "and taken result");
check(orTarget.p ||= branchRhs(4), 4, "or taken result");
check(nullishTarget.p ??= branchRhs(5), 5, "nullish taken result");
check(branchRhsCalls, 3, "three taken RHS evaluations");

andTarget.p = 0;
orTarget.p = 6;
nullishTarget.p = 7;
check(andTarget.p &&= branchRhs(8), 0, "and short-circuit result");
check(orTarget.p ||= branchRhs(9), 6, "or short-circuit result");
check(nullishTarget.p ??= branchRhs(10), 7, "nullish short-circuit result");
check(branchRhsCalls, 3, "short-circuit skips RHS");

// A computed raw-key expression runs before nullish validation. The TypeError
// then precedes observable ToPropertyKey coercion and the branch RHS.
var nullishTrace = [];
var nullishKey = {
  toString: function() {
    nullishTrace.push("to-key");
    return "p";
  }
};
function nullishRawKey() {
  nullishTrace.push("raw-key");
  return nullishKey;
}
function nullishRhs() {
  nullishTrace.push("rhs");
  return 1;
}
expectTypeError(function() {
  null[nullishRawKey()] ??= nullishRhs();
}, "nullish base TypeError");
check(nullishTrace.join(","), "raw-key", "nullish before ToPropertyKey and RHS");

var rawKeyMarker = {};
var rawKeyCaught = false;
try {
  null[(function() { throw rawKeyMarker; })()] ||= nullishRhs();
} catch (error) {
  rawKeyCaught = error === rawKeyMarker;
}
check(rawKeyCaught, true, "abrupt raw key precedes nullish validation");

// Abrupt RHS and Set completions cannot publish a logical-assignment result.
var abruptMarker = {};
var abruptTarget = {};
var abruptSetCalls = 0;
Object.defineProperty(abruptTarget, "p", {
  configurable: true,
  get: function() { return 0; },
  set: function() {
    abruptSetCalls += 1;
    throw abruptMarker;
  }
});
var rhsThrowResult = "not published";
var rhsThrowCaught = false;
try {
  rhsThrowResult = abruptTarget.p ||= (function() { throw abruptMarker; })();
} catch (error) {
  rhsThrowCaught = error === abruptMarker;
}
check(rhsThrowCaught, true, "RHS throw identity");
check(rhsThrowResult, "not published", "RHS throw nonpublication");
check(abruptSetCalls, 0, "RHS throw prevents Set");

var setThrowResult = "not published";
var setThrowCaught = false;
try {
  setThrowResult = abruptTarget.p ||= 11;
} catch (error) {
  setThrowCaught = error === abruptMarker;
}
check(setThrowCaught, true, "Set throw identity");
check(setThrowResult, "not published", "Set throw nonpublication");
check(abruptSetCalls, 1, "taken Set exactly once");

// False Set is ignored in sloppy code but is a TypeError in strict code once
// the selected branch reaches PutValue.
var noSetterValue = 0;
var noSetter = {};
Object.defineProperty(noSetter, "p", {
  configurable: true,
  get: function() { return noSetterValue; },
  set: undefined
});
check(noSetter.p ||= 12, 12, "sloppy false Set publishes RHS");
check(noSetter.p, 0, "sloppy false Set preserves property");

function strictNoSetterAnd() {
  "use strict";
  noSetterValue = 2;
  return noSetter.p &&= 13;
}
function strictNoSetterOr() {
  "use strict";
  noSetterValue = 0;
  return noSetter.p ||= 14;
}
function strictNoSetterNullish() {
  "use strict";
  noSetterValue = undefined;
  return noSetter.p ??= 15;
}
var strictFalseSetResult = "not published";
var strictFalseSetCaught = false;
try {
  strictFalseSetResult = strictNoSetterAnd();
} catch (error) {
  strictFalseSetCaught = error instanceof TypeError;
}
check(strictFalseSetCaught, true, "strict no-set and TypeError");
check(strictFalseSetResult, "not published", "strict false Set nonpublication");
expectTypeError(strictNoSetterOr, "strict no-set or TypeError");
expectTypeError(strictNoSetterNullish, "strict no-set nullish TypeError");

var nonWritable = {};
Object.defineProperty(nonWritable, "and", { value: 2, writable: false });
Object.defineProperty(nonWritable, "or", { value: 0, writable: false });
Object.defineProperty(nonWritable, "nullish", { value: undefined, writable: false });
function strictNonWritableAnd() {
  "use strict";
  return nonWritable.and &&= 16;
}
function strictNonWritableOr() {
  "use strict";
  return nonWritable.or ||= 17;
}
function strictNonWritableNullish() {
  "use strict";
  return nonWritable.nullish ??= 18;
}
expectTypeError(strictNonWritableAnd, "strict non-writable and TypeError");
expectTypeError(strictNonWritableOr, "strict non-writable or TypeError");
expectTypeError(strictNonWritableNullish, "strict non-writable nullish TypeError");
check(nonWritable.and, 2, "non-writable and unchanged");
check(nonWritable.or, 0, "non-writable or unchanged");
check(nonWritable.nullish, undefined, "non-writable nullish unchanged");

var nonExtensible = {};
Object.preventExtensions(nonExtensible);
function strictNonExtensibleOr() {
  "use strict";
  return nonExtensible.p ||= 19;
}
function strictNonExtensibleNullish() {
  "use strict";
  return nonExtensible.p ??= 20;
}
expectTypeError(strictNonExtensibleOr, "strict non-extensible or TypeError");
expectTypeError(strictNonExtensibleNullish, "strict non-extensible nullish TypeError");
check(nonExtensible.p, undefined, "non-extensible target unchanged");

// Primitive bases need distinct values for `O` and Receiver: one boxed target
// is retained through Get and a taken Set, while accessors see the primitive.
var primitiveValue = 0;
var primitiveGetReceivers = 0;
var primitiveSetReceivers = 0;
var primitiveSetValues = [];
var primitiveRhsCalls = 0;
Object.defineProperty(Number.prototype, "logicalReceiverProbe", {
  configurable: true,
  get: function() {
    "use strict";
    if (this === 1) primitiveGetReceivers += 1;
    return primitiveValue;
  },
  set: function(value) {
    "use strict";
    if (this === 1) primitiveSetReceivers += 1;
    primitiveSetValues.push(value);
  }
});
function primitiveRhs(value) {
  primitiveRhsCalls += 1;
  return value;
}

primitiveValue = 1;
check((1).logicalReceiverProbe &&= primitiveRhs(21), 21, "primitive and taken");
primitiveValue = 0;
check((1).logicalReceiverProbe ||= primitiveRhs(22), 22, "primitive or taken");
primitiveValue = undefined;
check((1).logicalReceiverProbe ??= primitiveRhs(23), 23, "primitive nullish taken");

primitiveValue = 0;
check((1).logicalReceiverProbe &&= primitiveRhs(24), 0, "primitive and short-circuit");
primitiveValue = 1;
check((1).logicalReceiverProbe ||= primitiveRhs(25), 1, "primitive or short-circuit");
check((1).logicalReceiverProbe ??= primitiveRhs(26), 1, "primitive nullish short-circuit");

check(primitiveGetReceivers, 6, "primitive Receiver on every Get");
check(primitiveSetReceivers, 3, "primitive Receiver on taken Sets");
check(primitiveSetValues.join(","), "21,22,23", "primitive taken Set values");
check(primitiveRhsCalls, 3, "primitive short-circuit skips RHS");

// A possible logical write must invalidate facts consumed by later lowering.
// The global property may now be either its old Number or the String RHS, so
// the following + remains coercive and produces concatenation at runtime.
globalThis.logicalAssignmentFact = 0;
globalThis.logicalAssignmentFact ||= "s";
check(
  globalThis.logicalAssignmentFact + 1,
  "s1",
  "logical global write invalidates stale type fact",
);

// Lowering a side-effecting RHS must not replace the fact from the skipped
// branch before both outcomes are merged. This RHS never runs at runtime, so
// Number addition remains the only correct specialization.
globalThis.logicalPreRhsFact = 1;
globalThis.logicalPreRhsFact ||= (globalThis.logicalPreRhsFact = "s");
check(
  globalThis.logicalPreRhsFact + 1,
  2,
  "logical global merge retains pre-RHS fact",
);

globalThis.logicalFailedOuterSetFact = 0;
check(
  globalThis.logicalFailedOuterSetFact ||= (
    Object.defineProperty(globalThis, "logicalFailedOuterSetFact", {
      value: "s",
      writable: false,
    }),
    2
  ),
  2,
  "logical failed outer Set publishes RHS",
);
check(
  globalThis.logicalFailedOuterSetFact + 1,
  "s1",
  "logical merge retains taken RHS mutation before failed outer Set",
);

var logicalScriptVarFact = 0;
globalThis.logicalScriptVarFact ||= "s";
check(
  logicalScriptVarFact + 1,
  "s1",
  "logical global write updates script var mirror",
);

// The conditional RHS transaction joins every flow fact, not only metadata
// for the logical assignment's own target.
globalThis.logicalUnrelatedFact = 1;
var logicalUnrelatedGuard = { p: 1 };
logicalUnrelatedGuard.p ||= (globalThis.logicalUnrelatedFact = "s");
check(
  globalThis.logicalUnrelatedFact + 1,
  2,
  "skipped logical RHS preserves unrelated global fact",
);

globalThis.logicalAliasFact = 0;
var logicalGlobalAlias = globalThis;
logicalGlobalAlias.logicalAliasFact ||= "a";
check(
  globalThis.logicalAliasFact + 1,
  "a1",
  "logical write through globalThis alias invalidates canonical fact",
);

function logicalConditionalGlobalAlias(flag) {
  globalThis.logicalConditionalAliasFact = 0;
  var target = flag ? globalThis : {};
  target.logicalConditionalAliasFact ||= "c";
  return globalThis.logicalConditionalAliasFact + 1;
}
check(
  logicalConditionalGlobalAlias(true),
  "c1",
  "logical write through joined globalThis alias invalidates canonical fact",
);
check(
  logicalConditionalGlobalAlias(false),
  1,
  "logical write through distinct joined object preserves canonical value",
);

function logicalDynamicGlobalKey(key) {
  globalThis.logicalDynamicKeyFact = 0;
  globalThis[key] ||= "d";
  return globalThis.logicalDynamicKeyFact + 1;
}
check(
  logicalDynamicGlobalKey("logicalDynamicKeyFact"),
  "d1",
  "dynamic logical global key invalidates every canonical fact",
);

globalThis.logicalShadowedGlobalFact = 1;
{
  let globalThis = {};
  globalThis.logicalShadowedGlobalFact = "s";
}
check(
  globalThis.logicalShadowedGlobalFact + 1,
  2,
  "shadowed globalThis write preserves canonical global fact",
);

// Constructor receiver shapes are flow facts too. The skipped write must not
// turn the later Number addition into String concatenation.
function LogicalBranchConstructor() {
  this.value = 1;
  var guard = { p: 1 };
  guard.p ||= (this.value = "s");
  this.result = this.value + 1;
}
check(
  new LogicalBranchConstructor().result,
  2,
  "skipped logical RHS preserves constructor receiver shape",
);

// A read/modify/write carrier observes a statically known getter with the
// Reference base as receiver before it decides whether the RHS is selected.
globalThis.logicalGetterValue = "global";
var logicalGetterTarget = {
  logicalGetterValue: 1,
  get p() { return this.logicalGetterValue + 1; }
};
check(
  logicalGetterTarget.p ||= 99,
  2,
  "logical getter observes property base receiver",
);

function logicalGetterByDynamicKey(key) {
  return logicalGetterTarget[key] ||= 99;
}
check(
  logicalGetterByDynamicKey("p"),
  2,
  "dynamic logical getter observes property base receiver",
);

function logicalGetterThroughJoinedShape(flag) {
  return (flag ? logicalGetterTarget : {}).p ||= 99;
}
check(
  logicalGetterThroughJoinedShape(true),
  2,
  "joined-shape logical getter observes property base receiver",
);

var logicalDynamicSetterSeen = 0;
var logicalDynamicSetterTarget = {
  marker: 7,
  set p(value) { logicalDynamicSetterSeen = this.marker + value; }
};
function logicalSetterByDynamicKey(key) {
  return logicalDynamicSetterTarget[key] ||= 5;
}
check(
  logicalSetterByDynamicKey("p"),
  5,
  "dynamic logical setter publishes selected RHS",
);
check(
  logicalDynamicSetterSeen,
  12,
  "dynamic logical setter observes property base receiver",
);

logicalDynamicSetterSeen = 0;
function logicalSetterThroughJoinedShape(flag) {
  return (flag ? logicalDynamicSetterTarget : {}).p ||= 5;
}
check(
  logicalSetterThroughJoinedShape(true),
  5,
  "joined-shape logical setter publishes selected RHS",
);
check(
  logicalDynamicSetterSeen,
  12,
  "joined-shape logical setter observes property base receiver",
);

Object.defineProperty(
  String.prototype,
  "logicalPrimitiveAccessor",
  {
    configurable: true,
    get: function() { "use strict"; return this + 1; },
  },
);
check(
  ("a").logicalPrimitiveAccessor ||= 99,
  "a1",
  "logical primitive getter observes primitive receiver",
);
delete String.prototype.logicalPrimitiveAccessor;

var logicalSloppyPrimitiveSetterThis = "unset";
Object.defineProperty(String.prototype, "logicalSloppyPrimitiveGetter", {
  configurable: true,
  get: function() { return typeof this; },
});
Object.defineProperty(String.prototype, "logicalSloppyPrimitiveSetter", {
  configurable: true,
  get: function() { return 0; },
  set: function() { logicalSloppyPrimitiveSetterThis = typeof this; },
});
check(
  ("a").logicalSloppyPrimitiveGetter ||= "fallback",
  "object",
  "sloppy primitive getter receives boxed this",
);
check(
  ("a").logicalSloppyPrimitiveSetter ||= 1,
  1,
  "sloppy primitive setter logical result",
);
check(
  logicalSloppyPrimitiveSetterThis,
  "object",
  "sloppy primitive setter receives boxed this",
);
delete String.prototype.logicalSloppyPrimitiveGetter;
delete String.prototype.logicalSloppyPrimitiveSetter;

// Object ToPropertyKey, getters, and setters are implicit user-code calls.
// Their effects must invalidate static facts at the point each hook runs.
globalThis.logicalKeyCoercionEffectFact = 1;
var logicalEffectKey = {
  toString: function() {
    globalThis.logicalKeyCoercionEffectFact = "s";
    return "length";
  }
};
check(
  ("a")[logicalEffectKey] ||= 2,
  1,
  "logical object-key coercion runs before Get",
);
check(
  globalThis.logicalKeyCoercionEffectFact + 1,
  "s1",
  "logical key coercion invalidates global facts",
);

globalThis.logicalKeyReceiverOut = 0;
var logicalKeyReceiverTarget = {
  marker: 1,
  get p() {
    globalThis.logicalKeyReceiverOut = this.marker + 1;
    return 1;
  }
};
var logicalKeyReceiverKey = {
  toString: function() {
    logicalKeyReceiverTarget.marker = "s";
    return "p";
  }
};
check(
  logicalKeyReceiverTarget[logicalKeyReceiverKey] ||= 2,
  1,
  "logical key-mutated getter result",
);
check(
  globalThis.logicalKeyReceiverOut,
  "s1",
  "logical key coercion widens getter receiver shape",
);

var originalGetterEffectNumberToString = Number.prototype.toString;
globalThis.logicalGetterEffectFact = 1;
var logicalGetterEffectTarget = {};
Object.defineProperty(logicalGetterEffectTarget, "p", {
  configurable: true,
  get: function() {
    globalThis.logicalGetterEffectFact = "s";
    Number.prototype.toString = Object.prototype.toString;
    return 1;
  }
});
check(logicalGetterEffectTarget.p ||= 2, 1, "logical getter effect result");
check(
  globalThis.logicalGetterEffectFact + 1,
  "s1",
  "logical getter invalidates global facts",
);
check(
  (1).toString(),
  "[object Number]",
  "logical getter invalidates prototype guards",
);
Number.prototype.toString = originalGetterEffectNumberToString;

globalThis.logicalSetterEffectFact = 1;
var logicalSetterEffectTarget = {};
Object.defineProperty(logicalSetterEffectTarget, "p", {
  configurable: true,
  get: function() { return 0; },
  set: function() { globalThis.logicalSetterEffectFact = "s"; }
});
check(logicalSetterEffectTarget.p ||= 2, 2, "logical setter effect result");
check(
  globalThis.logicalSetterEffectFact + 1,
  "s1",
  "logical setter invalidates global facts",
);

globalThis.logicalSetterReceiverOut = 0;
var logicalSetterReceiverPrototype = {
  set p(value) {
    globalThis.logicalSetterReceiverOut = this.marker + 1;
  }
};
var logicalSetterReceiverTarget = {
  __proto__: logicalSetterReceiverPrototype,
  marker: 1,
};
check(
  logicalSetterReceiverTarget.p ||= (
    logicalSetterReceiverTarget.marker = "s",
    2
  ),
  2,
  "logical RHS-mutated setter result",
);
check(
  globalThis.logicalSetterReceiverOut,
  "s1",
  "logical RHS widens setter receiver shape",
);

globalThis.logicalProxyEffectFact = 1;
var logicalProxyEffectTarget = new Proxy(
  { p: 1 },
  {
    get: function(target, key) {
      globalThis.logicalProxyEffectFact = "s";
      return target[key];
    }
  },
);
check(logicalProxyEffectTarget.p ||= 2, 1, "logical Proxy get result");
check(
  globalThis.logicalProxyEffectFact + 1,
  "s1",
  "logical Proxy trap invalidates global facts",
);

globalThis.logicalJoinedProxyOut = 0;
function logicalJoinedProxyTrap(target, key, receiver) {
  globalThis.logicalJoinedProxyOut = target.marker + 1;
  return 1;
}
logicalJoinedProxyTrap({ marker: 1 }, "p", null);
function logicalJoinedProxyRead(flag) {
  var handler = flag
    ? { get: logicalJoinedProxyTrap }
    : { get: logicalJoinedProxyTrap, extra: 0 };
  var proxy = new Proxy({ marker: "s" }, handler);
  return proxy.p ||= 2;
}
check(logicalJoinedProxyRead(true), 1, "joined Proxy get result");
check(
  globalThis.logicalJoinedProxyOut,
  "s1",
  "joined Proxy provenance widens trap arguments",
);

var logicalNestedDescriptorTarget = { value: 1 };
function logicalNestedDescriptorGetter() {
  "use strict";
  return this.value + 1;
}
function installLogicalNestedDescriptor() {
  Object.defineProperty(logicalNestedDescriptorTarget, "p", {
    configurable: true,
    get: logicalNestedDescriptorGetter,
  });
}
installLogicalNestedDescriptor();
check(
  logicalNestedDescriptorTarget.p ||= 99,
  2,
  "nested descriptor getter observes known-shape receiver",
);

var logicalRhsSetterSeen = 0;
var logicalRhsSetterTarget = { marker: 7, p: 0 };
function logicalRhsInstalledSetter(value) {
  "use strict";
  logicalRhsSetterSeen = this.marker + value;
}
check(
  logicalRhsSetterTarget.p ||= (
    Object.defineProperty(logicalRhsSetterTarget, "p", {
      configurable: true,
      set: logicalRhsInstalledSetter,
    }),
    5
  ),
  5,
  "logical RHS-installed setter publishes RHS",
);
check(
  logicalRhsSetterSeen,
  12,
  "logical RHS-installed setter observes receiver",
);

var logicalRhsPrototypeSetterSeen = 0;
var logicalRhsPrototypeSetterTarget = { marker: 1, p: 0 };
check(
  logicalRhsPrototypeSetterTarget.p ||= (
    Object.setPrototypeOf(logicalRhsPrototypeSetterTarget, {
      set p(value) {
        logicalRhsPrototypeSetterSeen = this.marker + value;
      }
    }),
    delete logicalRhsPrototypeSetterTarget.p,
    2
  ),
  2,
  "logical RHS prototype setter publishes RHS",
);
check(
  logicalRhsPrototypeSetterSeen,
  3,
  "logical RHS prototype setter observes receiver",
);

var logicalGetterThrowTarget = {};
Object.defineProperty(logicalGetterThrowTarget, "p", {
  configurable: true,
  get: function() { throw "getter"; },
});
function logicalGetterThrowType() {
  "use strict";
  try {
    logicalGetterThrowTarget.p ||= 1;
  } catch (error) {
    return typeof error;
  }
}
check(
  logicalGetterThrowType(),
  "string",
  "logical getter arbitrary throw catch type",
);

var logicalSetterThrowTarget = {};
Object.defineProperty(logicalSetterThrowTarget, "p", {
  configurable: true,
  get: function() { return 0; },
  set: function() { throw "setter"; },
});
function logicalSetterThrowType() {
  "use strict";
  try {
    logicalSetterThrowTarget.p ||= 1;
  } catch (error) {
    return typeof error;
  }
}
check(
  logicalSetterThrowType(),
  "string",
  "logical setter arbitrary throw catch type",
);

// Reflective structure mutation discards pre-mutation shapes. Accessors
// installed through either a new prototype or a descriptor bag must remain
// possible hooks for the later retained Reference.
globalThis.logicalPrototypeMutationFact = 1;
var logicalPrototypeMutationTarget = {};
Object.setPrototypeOf(logicalPrototypeMutationTarget, {
  get p() {
    globalThis.logicalPrototypeMutationFact = "s";
    return 0;
  }
});
check(
  logicalPrototypeMutationTarget.p ||= 2,
  2,
  "prototype-installed getter logical result",
);
check(
  globalThis.logicalPrototypeMutationFact + 1,
  "s1",
  "Object.setPrototypeOf getter invalidates global facts",
);

globalThis.logicalDefinePropertiesFact = 1;
var logicalDefinePropertiesTarget = {};
Object.defineProperties(logicalDefinePropertiesTarget, {
  p: {
    configurable: true,
    get: function() {
      globalThis.logicalDefinePropertiesFact = "s";
      return 0;
    }
  }
});
check(
  logicalDefinePropertiesTarget.p ||= 3,
  3,
  "defineProperties-installed getter logical result",
);
check(
  globalThis.logicalDefinePropertiesFact + 1,
  "s1",
  "Object.defineProperties getter invalidates global facts",
);

// A write through one alias invalidates shapes which contain that same object,
// including nested object properties and tracked array elements.
var logicalNestedObjectAliasTarget = { p: 0 };
var logicalNestedObjectAliasHolder = { alias: logicalNestedObjectAliasTarget };
logicalNestedObjectAliasTarget.p ||= "s";
check(
  logicalNestedObjectAliasHolder.alias.p + 1,
  "s1",
  "logical write invalidates nested object alias shape",
);

var logicalNestedArrayAliasTarget = { p: 0 };
var logicalNestedArrayAliasHolder = [logicalNestedArrayAliasTarget];
logicalNestedArrayAliasTarget.p ||= "s";
check(
  logicalNestedArrayAliasHolder[0].p + 1,
  "s1",
  "logical write invalidates nested array alias shape",
);

// Every ordinary mutation carrier admits arbitrary values thrown by implicit
// getters, setters, and coercion hooks, even in strict mode where failed Set
// contributes an additional TypeError path.
var plainThrowTarget = {};
Object.defineProperty(plainThrowTarget, "p", {
  set: function() { throw "plain setter"; }
});
function plainSetterThrowType() {
  "use strict";
  try { plainThrowTarget.p = 1; } catch (error) { return typeof error; }
}
check(plainSetterThrowType(), "string", "plain setter arbitrary throw catch type");

var eagerThrowTarget = {};
Object.defineProperty(eagerThrowTarget, "p", {
  get: function() { throw "eager getter"; },
  set: function() {}
});
function eagerGetterThrowType() {
  "use strict";
  try { eagerThrowTarget.p += 1; } catch (error) { return typeof error; }
}
check(eagerGetterThrowType(), "string", "eager getter arbitrary throw catch type");

var numericThrowTarget = {};
Object.defineProperty(numericThrowTarget, "p", {
  get: function() { throw "numeric getter"; },
  set: function() {}
});
function numericGetterThrowType() {
  "use strict";
  try { numericThrowTarget.p++; } catch (error) { return typeof error; }
}
check(numericGetterThrowType(), "string", "numeric getter arbitrary throw catch type");

globalThis.ordinarySetterParamOut = 0;
var ordinarySetterParamTarget = {
  set p(value) { globalThis.ordinarySetterParamOut = value + 1; }
};
var ordinarySetterParamFunction = Object.getOwnPropertyDescriptor(
  ordinarySetterParamTarget,
  "p",
).set;
ordinarySetterParamFunction(1);
ordinarySetterParamTarget.p = "s";
check(
  globalThis.ordinarySetterParamOut,
  "s1",
  "property Set widens a directly called setter parameter",
);

globalThis.logicalDeleteShapeFact = 1;
var logicalDeletePrototype = {
  get q() {
    globalThis.logicalDeleteShapeFact = "s";
    return 0;
  }
};
var logicalDeleteTarget = { __proto__: logicalDeletePrototype, q: 1 };
delete logicalDeleteTarget.q;
check(logicalDeleteTarget.q ||= 2, 2, "delete-exposed getter logical result");
check(
  globalThis.logicalDeleteShapeFact + 1,
  "s1",
  "delete invalidates own shape before inherited getter",
);

Object.defineProperty(globalThis, "__proto__", {
  value: 1,
  writable: true,
  configurable: true,
});
var logicalGlobalDeleteAlias = globalThis;
check(delete globalThis.__proto__, true, "direct global delete succeeds");
check(
  logicalGlobalDeleteAlias.__proto__ ||= null,
  Object.prototype,
  "direct global delete invalidates globalThis alias shape",
);

globalThis.logicalDestructureShapeFact = 1;
var logicalDestructurePrototype = {
  get q() {
    globalThis.logicalDestructureShapeFact = "s";
    return 0;
  }
};
var logicalDestructureTarget = {};
({ value: logicalDestructureTarget.__proto__ } = {
  value: logicalDestructurePrototype,
});
check(
  logicalDestructureTarget.q ||= 2,
  2,
  "destructuring prototype setter logical result",
);
check(
  globalThis.logicalDestructureShapeFact + 1,
  "s1",
  "destructuring property write invalidates later shape",
);

// A possible Number.prototype write invalidates intrinsic-call knowledge.
// The following call must observe the runtime method rather than fold to "0".
var originalNumberToString = Number.prototype.toString;
var logicalNumberPrototypeAlias = Number.prototype;
logicalNumberPrototypeAlias.extra = 1;
logicalNumberPrototypeAlias.toString &&= Object.prototype.toString;
check(
  (1).toString(),
  "[object Number]",
  "logical Number.prototype alias invalidates toString fast path",
);
Number.prototype.toString = originalNumberToString;
delete logicalNumberPrototypeAlias.extra;

function logicalConditionalNumberPrototypeAlias(flag) {
  var target = flag ? Number.prototype : {};
  target.toString &&= Object.prototype.toString;
  return (1).toString();
}
check(
  logicalConditionalNumberPrototypeAlias(true),
  "[object Number]",
  "joined Number.prototype alias invalidates toString fast path",
);
Number.prototype.toString = originalNumberToString;

function logicalDynamicNumberPrototypeKey(key) {
  Number.prototype[key] &&= Object.prototype.toString;
  return (1).toString();
}
check(
  logicalDynamicNumberPrototypeKey("toString"),
  "[object Number]",
  "dynamic Number.prototype key invalidates toString fast path",
);
Number.prototype.toString = originalNumberToString;

var originalBooleanToString = Boolean.prototype.toString;
var logicalBooleanPrototypeAlias = Boolean.prototype;
logicalBooleanPrototypeAlias.toString &&= Object.prototype.toString;
check(
  (true).toString(),
  "[object Boolean]",
  "logical Boolean.prototype alias invalidates toString fast path",
);
Boolean.prototype.toString = originalBooleanToString;

function logicalShadowedNumberConstructor() {
  let Number = { prototype: {} };
  Number.prototype.toString = Object.prototype.toString;
  return (1).toString();
}
check(
  logicalShadowedNumberConstructor(),
  "1",
  "shadowed Number write preserves intrinsic toString fast path",
);

// Mutating Array.prototype must disable direct intrinsic method selection for
// subsequent arrays even though the logical branch is selected at runtime.
var originalArrayToString = Array.prototype.toString;
Array.prototype.toString &&= function() {
  return "logical array prototype";
};
check(
  [1].toString(),
  "logical array prototype",
  "logical Array.prototype write disables builtin fast path",
);
Array.prototype.toString = originalArrayToString;

true;
