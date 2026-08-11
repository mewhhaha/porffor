// `new X().take(1).toArray()` where `class X extends Iterator` has no explicit
// constructor.
//
// Measured to throw `value is not callable` before the callee-acquisition
// repair: `take`'s fast path produced no call and left the destination locals
// holding stale scratch, so the `.toArray()` applied to that stale value found
// nothing callable. The chain is its own fixture because it is the shortest
// program that turns "the destination was never written" into a hard,
// unmistakable failure instead of a wrong value.
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
    if (index < 5) {
      index = index + 1;
      return { done: false, value: index };
    }
    return { done: true, value: undefined };
  }
}

index = 0;
var values = new Source().take(1).toArray();
check(Array.isArray(values), "array");
check(values.length === 1, "length-" + values.length);
check(values[0] === 1, "value-" + values[0]);

index = 0;
var chained = new Source().drop(1).take(2).toArray();
check(chained.length === 2, "chained-length-" + chained.length);
check(chained.join(",") === "2,3", "chained-values-" + chained.join(","));

index = 0;
var mapped = new Source()
  .take(2)
  .map(function (value) {
    return value * 3;
  })
  .toArray();
check(mapped.join(",") === "3,6", "mapped-" + mapped.join(","));

var outcome = "ok";
if (failures !== "") {
  outcome = failures;
}
outcome;
