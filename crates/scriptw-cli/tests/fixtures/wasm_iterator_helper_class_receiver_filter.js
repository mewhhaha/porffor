// `Iterator.prototype.filter` on a `class X extends Iterator` receiver with no
// explicit constructor. One of the five helpers the pre-existing `iterator::`
// fixtures do not cover.
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
var helper = new Source().filter(function (value) {
  calls = calls + 1;
  return value !== 2;
});
check(typeof helper === "object", "type-" + typeof helper);
check(calls === 0, "lazy-" + calls);
var values = helper.toArray();
check(values.length === 2, "length-" + values.length);
check(values.join(",") === "1,3", "values-" + values.join(","));
check(calls === 3, "calls-" + calls);

index = 0;
var throwCalls = 0;
var caught = "no-throw";
try {
  new Source()
    .filter(function () {
      throwCalls = throwCalls + 1;
      throw new Sentinel();
    })
    .toArray();
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
