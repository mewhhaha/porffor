// `Iterator.prototype.every` on a `class X extends Iterator` receiver with no
// explicit constructor. See `wasm_iterator_helper_class_receiver_some.js` for
// why this receiver shape is the one that matters.
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
var result = new Source().every(function (value) {
  calls = calls + 1;
  return value < 3;
});
check(typeof result === "boolean", "type-" + typeof result);
check(result === false, "value");
check(calls === 3, "calls-" + calls);

index = 0;
var allCalls = 0;
var allResult = new Source().every(function (value) {
  allCalls = allCalls + 1;
  return value > 0;
});
check(allResult === true, "all-value");
check(allCalls === 3, "all-calls-" + allCalls);

index = 0;
var throwCalls = 0;
var caught = "no-throw";
try {
  new Source().every(function () {
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
