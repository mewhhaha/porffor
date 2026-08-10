// `Iterator.prototype.reduce` on a `class X extends Iterator` receiver with no
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
var result = new Source().reduce(function (accumulator, value) {
  calls = calls + 1;
  return accumulator + value;
}, 100);
check(typeof result === "number", "type-" + typeof result);
check(result === 106, "value-" + result);
check(calls === 3, "calls-" + calls);

index = 0;
var noSeedCalls = 0;
var noSeed = new Source().reduce(function (accumulator, value) {
  noSeedCalls = noSeedCalls + 1;
  return accumulator + value;
});
check(noSeed === 6, "no-seed-value-" + noSeed);
check(noSeedCalls === 2, "no-seed-calls-" + noSeedCalls);

index = 0;
var throwCalls = 0;
var caught = "no-throw";
try {
  new Source().reduce(function () {
    throwCalls = throwCalls + 1;
    throw new Sentinel();
  }, 0);
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
