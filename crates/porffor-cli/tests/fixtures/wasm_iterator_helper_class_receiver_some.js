// `Iterator.prototype.some` on a `class X extends Iterator` receiver that has
// NO explicit constructor.
//
// This receiver shape is the one that exposed the callee-acquisition defect:
// `emit_method_call`'s `some` fast path acquired its callee with
// `emit_function_value_payload`, emitted no call for this shape, and left the
// expression's destination locals holding stale scratch — so `typeof result`
// answered `"object"`, the callback never ran, and a callback throw was
// discarded rather than reaching the `catch` below.
//
// Both halves matter and both are asserted: the returned VALUE AND TYPE, and
// that a callback throw reaches a user `catch`. A fast path that emits nothing
// fails the first; a fast path that runs the loop but drops abrupt completions
// fails the second.
//
// Spec-correct output is `string(ok)`. Any other `string(...)` names the checks
// that failed.

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

index = 0;
var calls = 0;
var result = new Source().some(function (value) {
  calls = calls + 1;
  return value === 2;
});
check(typeof result === "boolean", "type-" + typeof result);
check(result === true, "value");
check(calls === 2, "calls-" + calls);

index = 0;
var throwCalls = 0;
var caught = "no-throw";
try {
  new Source().some(function () {
    throwCalls = throwCalls + 1;
    throw new Sentinel();
  });
} catch (error) {
  if (error instanceof Sentinel) {
    caught = "sentinel";
  } else {
    caught = "other";
  }
}
check(caught === "sentinel", "caught-" + caught);
check(throwCalls === 1, "throw-calls-" + throwCalls);

var outcome = "ok";
if (failures !== "") {
  outcome = failures;
}
outcome;
