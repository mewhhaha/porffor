// `Iterator.prototype.forEach` on a `class X extends Iterator` receiver with no
// explicit constructor.
//
// `forEach` was the ONE helper whose fast path already acquired its callee with
// an ordinary `[[Get]]`, and the one measured correct for this receiver shape.
// It is covered here alongside the other ten so that a future "optimisation"
// that reintroduces the builtin-value acquisition cannot quietly break the
// reference case.
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
var seen = [];
var result = new Source().forEach(function (value) {
  seen.push(value);
});
check(typeof result === "undefined", "type-" + typeof result);
check(seen.length === 3, "length-" + seen.length);
check(seen.join(",") === "1,2,3", "values-" + seen.join(","));

index = 0;
var throwCalls = 0;
var caught = "no-throw";
try {
  new Source().forEach(function () {
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
