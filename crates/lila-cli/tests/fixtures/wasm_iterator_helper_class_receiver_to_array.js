// `Iterator.prototype.toArray` on a `class X extends Iterator` receiver with no
// explicit constructor.
//
// `toArray` is the one helper with NO fast path in `emit_method_call`: it falls
// through to the generic tail, which is the measured-correct oracle for all
// eleven. It is covered here precisely because it is the oracle — if a future
// change gives `toArray` a fast path, this fixture is what notices that the
// fast path is not observationally equal to the tail.
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
var values = new Source().toArray();
check(Array.isArray(values), "array");
check(values.length === 3, "length-" + values.length);
check(values.join(",") === "1,2,3", "values-" + values.join(","));

var caught = "no-throw";
try {
  new ThrowingSource().toArray();
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
