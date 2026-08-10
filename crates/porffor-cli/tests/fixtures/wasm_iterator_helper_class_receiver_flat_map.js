// `Iterator.prototype.flatMap` on a `class X extends Iterator` receiver with no
// explicit constructor.
//
// `flatMap` is doubly worth pinning: it is one of the five helpers the
// pre-existing `iterator::` fixtures do not cover, and `emit_method_call` used
// to carry TWO `name == "flatMap"` blocks — an array-only one and an iterator
// one whose own array tail was unreachable. They are now one block.
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
var helper = new Source().flatMap(function (value) {
  calls = calls + 1;
  return [value, value * 10];
});
check(typeof helper === "object", "type-" + typeof helper);
check(calls === 0, "lazy-" + calls);
var values = helper.toArray();
check(values.length === 6, "length-" + values.length);
check(values.join(",") === "1,10,2,20,3,30", "values-" + values.join(","));
check(calls === 3, "calls-" + calls);

// An array receiver must still reach the Array.prototype.flatMap path, which is
// what the folded block's early return preserves.
var arrayFlat = [1, 2].flatMap(function (value) {
  return [value, value];
});
check(arrayFlat.join(",") === "1,1,2,2", "array-" + arrayFlat.join(","));

index = 0;
var throwCalls = 0;
var caught = "no-throw";
try {
  new Source()
    .flatMap(function () {
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
