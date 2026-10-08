# Math.sumPrecise native runtime contract

The actual native path uses whole values and completions, the shared completed
synchronous iterator acquisition, a cached next method and a private 34-limb GC
accumulator. Acquisition precedes reduction. Elements must already be Numbers;
there is no element coercion or mutable public Array scratch storage.

The closed state domain preserves negative zero for an empty/all-negative-zero
sequence, finite exact sum, either Infinity and NaN. Iteration continues after
non-finite values so later effects and errors remain observable. Element-count
and element-type rejection call IteratorClose once while retaining the
original whole Throw. IteratorStepValue failures propagate directly. Successful reduction and explicit rejection share one native cleanup edge.
Acquisition/step failures terminate the invocation with their whole Throw; its
function scope releases the retained roots. A completed reduction is consumed
by its sole finisher, while the native owner controls accumulator cleanup.

The operational reference is [ECMAScript Math.sumPrecise](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.sumprecise).
Predecessor raw-ABI spelling guards are retired. Existing CLI controls and new
Engine GC controls remain meaningful, but this source is uncompiled and unrun.
Task acceptance and pinned suite status remain unchanged until the later capped
whole-batch verification completes.
