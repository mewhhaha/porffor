# Ordinary generator destructuring assignment and classic For heads

An eager assignment pattern can consume a complete suspended RHS in an ordinary
synchronous generator. `GeneratorPatternAssignmentSource` retains the actual
pattern and RHS together, checks the ordinary-generator admission, and rejects
Yield or Await inside the pattern. The complete RHS source plan must exist before
this source carrier can be published. Shared source planning, scalar suspension
counting, staged value lowering and discarded assignment lowering consume this
same checked source.

`lowering/generator_pattern_assignment.rs` first lowers every RHS suspension and
then passes its Normal whole value to the existing `lower_pattern_assign_value`.
The existing Array/Object assignment owners evaluate target References, computed
keys, Gets, defaults and rest operations in their established order. They also
own iterator acquisition and IteratorClose. No target, key or pattern Get is
evaluated while the RHS remains suspended. An injected Return or Throw uses the
existing generator completion/finalizer route and skips the pattern operation.

The result remains the exact RHS value, including its raw kind and callable
target identity. Pattern code may mutate that value, so the resulting compiler
metadata discards its mutable heap-shape fact. The backend uses the existing
rooted Value/Completion and pattern emitter; this change adds no runtime record,
object representation or expression interpreter.

Classic ordinary-generator For initialization also consumes the existing paired
`GeneratorPatternInitializerSource` when a binding pattern's initializer yields.
Its phase graph comes from the existing `generator_loop_source` allocator; the
initializer does not add a second continuation graph. BoundNames retain the exact
source-name-to-scoped-storage mapping used by environment analysis. Binding
initialization receives that map through the existing pattern consumer, and
the head consists of its real initialization statements. Eager pattern heads
use the same map. All lexical head names remain uninitialized before the first
initializer evaluates.

`generator_loop_control/head_bindings.rs` is the single declaration-binding
visitor used by loop-constructor validation and lowering's head-cell census.
It visits direct lexical statements, same-scope lexical lists, transparent blocks
and the actual Array/Object declaration initialization leaves. It preserves the
real BindingMode and does not visit nested function or method bodies. Both the
expected mutable iteration set and the emitted per-iteration slots intersect
these names with the actual analyzed head environment. Let cells clone at the
existing first-test and update boundaries; Const cells keep their initialized
head record. Uncaptured head bindings remain in their real activation cells.

Patterns with their own suspended computed keys, targets or defaults remain
unsupported until a pattern continuation owner exists. This batch does not widen
async-generator or asynchronous loop admission.

The authored IR controls cover the paired RHS/assignment consumer, exact callable
result metadata, complete RHS branches, eager-pattern refusal boundaries, scoped
pattern cell identity and Let/Const iteration ownership. The paired strict/sloppy
Engine fixture requires real Wasm AOT execution and observes target/key/Get/default
ordering, rest exclusion, normal and abrupt IteratorClose, GC, same callable/RHS
identity, injected whole completions, initializer TDZ and distinct captured
iteration cells. Compilation and execution remain pending the complete batch's
capped verification checkpoint.
