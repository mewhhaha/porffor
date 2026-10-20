// `Iterator.prototype.take` on a `class X extends Iterator` receiver with no
// explicit constructor.
//
// `take` has no callback, so the abrupt-completion half is driven by a `next`
// that throws: the throw must reach the user `catch` when the helper is
// drained, not be discarded.
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

class ThrowingSource extends Iterator {
  next() {
    throw new Sentinel();
  }
}

index = 0;
var helper = new Source().take(2);
check(typeof helper === "object", "type-" + typeof helper);
var values = helper.toArray();
check(values.length === 2, "length-" + values.length);
check(values.join(",") === "1,2", "values-" + values.join(","));

// `take` must not exhaust the source beyond its limit.
check(index === 2, "consumed-" + index);

var caught = "no-throw";
try {
  new ThrowingSource().take(1).toArray();
} catch (error) {
  if (error instanceof Sentinel) {
    caught = "sentinel";
  } else {
    caught = "other";
  }
}
check(caught === "sentinel", "caught-" + caught);

var outcome = "ok";
if (failures !== "") {
  outcome = failures;
}
outcome;
