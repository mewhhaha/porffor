// `Iterator.prototype.drop` on a `class X extends Iterator` receiver with no
// explicit constructor.
//
// `drop`'s fast-path guard is `receiver_is_iterator || !receiver_is_array`,
// not the plain `receiver_is_iterator` the other helpers use. That guard is
// preserved verbatim by the callee-acquisition repair, and this fixture is what
// pins the behaviour it selects.
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
var helper = new Source().drop(1);
check(typeof helper === "object", "type-" + typeof helper);
var values = helper.toArray();
check(values.length === 2, "length-" + values.length);
check(values.join(",") === "2,3", "values-" + values.join(","));

var caught = "no-throw";
try {
  new ThrowingSource().drop(1).toArray();
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
