// `Iterator.prototype.find` on a `class X extends Iterator` receiver with no
// explicit constructor. See `wasm_iterator_helper_class_receiver_some.js`.
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

index = 0;
var calls = 0;
var result = new Source().find(function (value) {
  calls = calls + 1;
  return value === 2;
});
check(typeof result === "number", "type-" + typeof result);
check(result === 2, "value");
check(calls === 2, "calls-" + calls);

index = 0;
var missingCalls = 0;
var missing = new Source().find(function (value) {
  missingCalls = missingCalls + 1;
  return value === 99;
});
check(typeof missing === "undefined", "missing-type-" + typeof missing);
check(missingCalls === 3, "missing-calls-" + missingCalls);

index = 0;
var throwCalls = 0;
var caught = "no-throw";
try {
  new Source().find(function () {
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
