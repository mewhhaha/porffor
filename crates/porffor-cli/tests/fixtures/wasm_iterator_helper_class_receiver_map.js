// `Iterator.prototype.map` on a `class X extends Iterator` receiver with no
// explicit constructor.
//
// `map` is one of the five helpers the four pre-existing `iterator::` fixtures
// do NOT cover, so it could have stayed broken with those green. The helper is
// lazy, so the callback throw is observed at the point the result is drained,
// not at the `map` call itself.
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
var helper = new Source().map(function (value) {
  calls = calls + 1;
  return value * 10;
});
check(typeof helper === "object", "type-" + typeof helper);
check(calls === 0, "lazy-" + calls);
var values = helper.toArray();
check(values.length === 3, "length-" + values.length);
check(values.join(",") === "10,20,30", "values-" + values.join(","));
check(calls === 3, "calls-" + calls);

index = 0;
var throwCalls = 0;
var caught = "no-throw";
try {
  new Source()
    .map(function () {
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
