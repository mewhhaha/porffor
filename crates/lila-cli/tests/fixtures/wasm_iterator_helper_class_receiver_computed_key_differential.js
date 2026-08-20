// The permanent guard: a builtin-method fast path may only be FASTER, never
// DIFFERENT.
//
// `x.some(cb)` takes `emit_method_call`'s static-key fast path.
// `x["some"](cb)` and `x[k](cb)` route to the generic tail, which is the
// measured-correct oracle for this receiver shape. The three forms must be
// observationally identical. They were not: the static-key form called the
// callback zero times and answered a stale scratch value of type "object",
// while both computed forms called it twice and answered `true`. That is the
// difference this fixture exists to make impossible to reintroduce.
//
// The receiver expression is written out inline in each arm on purpose. Hiding
// it behind a parameter would erase the receiver's static heap shape, the fast
// path would not be selected at all, and the differential would compare the
// generic tail against itself — a vacuous pass.
//
// Spec-correct output is `string(ok)`.

var failures = "";
function check(condition, label) {
  if (!condition) {
    failures = failures + label + ";";
  }
}

var index = 0;
class Source extends Iterator {
  next() {
    if (index < 3) {
      index = index + 1;
      return { done: false, value: index };
    }
    return { done: true, value: undefined };
  }
}

function Sentinel() {}

// Not statically foldable to a literal key by any constant folder that does not
// also evaluate array element reads.
var keyNames = ["some", "map"];
var someKey = keyNames[0];
var mapKey = keyNames[1];

// --- some: static key -------------------------------------------------------
index = 0;
var staticCalls = 0;
var staticValue = new Source().some(function (value) {
  staticCalls = staticCalls + 1;
  return value === 2;
});
index = 0;
var staticThrowCalls = 0;
var staticCaught = "no-throw";
try {
  new Source().some(function () {
    staticThrowCalls = staticThrowCalls + 1;
    throw new Sentinel();
  });
} catch (error) {
  if (error instanceof Sentinel) {
    staticCaught = "sentinel";
  } else {
    staticCaught = "other";
  }
}
var staticRecord =
  typeof staticValue +
  "|" +
  staticValue +
  "|" +
  staticCalls +
  "|" +
  staticCaught +
  "|" +
  staticThrowCalls;

// --- some: literal computed key --------------------------------------------
index = 0;
var literalCalls = 0;
var literalValue = new Source()["some"](function (value) {
  literalCalls = literalCalls + 1;
  return value === 2;
});
index = 0;
var literalThrowCalls = 0;
var literalCaught = "no-throw";
try {
  new Source()["some"](function () {
    literalThrowCalls = literalThrowCalls + 1;
    throw new Sentinel();
  });
} catch (error) {
  if (error instanceof Sentinel) {
    literalCaught = "sentinel";
  } else {
    literalCaught = "other";
  }
}
var literalRecord =
  typeof literalValue +
  "|" +
  literalValue +
  "|" +
  literalCalls +
  "|" +
  literalCaught +
  "|" +
  literalThrowCalls;

// --- some: runtime computed key --------------------------------------------
index = 0;
var runtimeCalls = 0;
var runtimeValue = new Source()[someKey](function (value) {
  runtimeCalls = runtimeCalls + 1;
  return value === 2;
});
index = 0;
var runtimeThrowCalls = 0;
var runtimeCaught = "no-throw";
try {
  new Source()[someKey](function () {
    runtimeThrowCalls = runtimeThrowCalls + 1;
    throw new Sentinel();
  });
} catch (error) {
  if (error instanceof Sentinel) {
    runtimeCaught = "sentinel";
  } else {
    runtimeCaught = "other";
  }
}
var runtimeRecord =
  typeof runtimeValue +
  "|" +
  runtimeValue +
  "|" +
  runtimeCalls +
  "|" +
  runtimeCaught +
  "|" +
  runtimeThrowCalls;

check(staticRecord === literalRecord, "some-static-vs-literal:" + staticRecord + "!=" + literalRecord);
check(staticRecord === runtimeRecord, "some-static-vs-runtime:" + staticRecord + "!=" + runtimeRecord);
check(staticRecord === "boolean|true|2|sentinel|1", "some-absolute:" + staticRecord);

// --- map: static vs computed key -------------------------------------------
index = 0;
var staticMapCalls = 0;
var staticMapHelper = new Source().map(function (value) {
  staticMapCalls = staticMapCalls + 1;
  return value * 10;
});
var staticMapValues = staticMapHelper.toArray();
var staticMapRecord = staticMapValues.join(",") + "|" + staticMapCalls;

index = 0;
var computedMapCalls = 0;
var computedMapHelper = new Source()[mapKey](function (value) {
  computedMapCalls = computedMapCalls + 1;
  return value * 10;
});
var computedMapValues = computedMapHelper.toArray();
var computedMapRecord = computedMapValues.join(",") + "|" + computedMapCalls;

check(
  staticMapRecord === computedMapRecord,
  "map-static-vs-computed:" + staticMapRecord + "!=" + computedMapRecord
);
check(staticMapRecord === "10,20,30|3", "map-absolute:" + staticMapRecord);

var outcome = "ok";
if (failures !== "") {
  outcome = failures;
}
outcome;
