// Abrupt completions *inside* the helper dispatch itself, which every other
// fixture in this family leaves uncovered.
//
// `emit_iterator_prototype_helper_method_call` performs three steps before the
// call: it compiles the receiver, it reads the helper property off it, and it
// evaluates the arguments. Until batch 6 it checked for a throw completion
// after none of them, while the structurally identical
// `emit_custom_array_named_method_call` checked after all three. The other
// twelve fixtures all use `new Source()` as the receiver and never read a
// throwing accessor, so a throw arising in either of the first two steps was
// invisible to the whole module.
//
// Read this one as a REGRESSION test, not as a repair witness, and the
// distinction is measured rather than assumed: it answers `string(ok)` on the
// pre-repair compiler (verified by running it against `target/debug/porf` at
// `37893ef93`). Its receivers are ordinary objects, which the generic tail
// already handled correctly, and batch 6 moves exactly those receivers onto the
// shared helper dispatch. So what it pins is that the routing change does not
// lose behaviour the tail already had — which is what the three
// throw-propagation checks added to that emitter are for. A red here after the
// routing change means the dispatch swallowed an abrupt completion.
//
// What each arm covers:
//
//   1. a throwing accessor in the receiver's own shape — the property read is
//      abrupt, so the value that would be handed on as the callee is the thrown
//      error rather than a function. Without a check between the read and the
//      call, the arguments are still evaluated for their side effects and the
//      callee test answers "not a function", so the user sees a `TypeError`
//      instead of the error they threw.
//   2. an accessor that returns normally — the same read on the same emission
//      path must still work, so arm 1 cannot be satisfied by refusing accessor
//      receivers altogether.
//   3. a receiver expression that itself throws.
//
// Spec-correct output is `string(ok)`.

var failures = "";
function check(condition, label) {
  if (!condition) {
    failures = failures + label + ";";
  }
}

function Sentinel() {}

// --- 1. the [[Get]] of the helper property is abrupt ------------------------

var argsEvaluated = 0;
var throwingGetter = {
  get some() {
    throw new Sentinel();
  },
};

var getterCaught = "no-throw";
try {
  throwingGetter.some(
    (function () {
      argsEvaluated = argsEvaluated + 1;
      return function () {
        return true;
      };
    })()
  );
} catch (error) {
  if (error instanceof Sentinel) {
    getterCaught = "sentinel";
  } else {
    getterCaught = "other";
  }
}
check(getterCaught === "sentinel", "getter-caught-" + getterCaught);
// 7.3.11 GetMethod runs before the argument list is evaluated, so an abrupt
// completion there must leave the arguments unevaluated.
check(argsEvaluated === 0, "getter-args-" + argsEvaluated);

// --- 2. a normal accessor on the same emission path -------------------------

var normalGetter = {
  get some() {
    return function (value) {
      return "called-" + value;
    };
  },
};
check(normalGetter.some(7) === "called-7", "normal-getter-" + normalGetter.some(7));

// --- 3. the receiver expression is abrupt -----------------------------------

class ThrowingReceiver {
  constructor() {
    throw new Sentinel();
  }
  some() {
    return "unreachable";
  }
}

var receiverCaught = "no-throw";
try {
  new ThrowingReceiver().some(function () {
    return true;
  });
} catch (error) {
  if (error instanceof Sentinel) {
    receiverCaught = "sentinel";
  } else {
    receiverCaught = "other";
  }
}
check(receiverCaught === "sentinel", "receiver-caught-" + receiverCaught);

var outcome = "ok";
if (failures !== "") {
  outcome = failures;
}
outcome;
