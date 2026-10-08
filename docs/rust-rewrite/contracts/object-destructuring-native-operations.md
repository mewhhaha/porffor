# Shared native object destructuring operations

Source status (2026-10-06): authored and unverified. This packet runs no
compilation, runtime, structural guard, generator or Test262 command.

`control_flow/object_destructuring.rs` owns the existing complete object-pattern
consumer and the incremental `ObjectDestructuringOperationIr` consumer. Their
GetV calls share one physical body: the lookup base is the boxed source, the
receiver is the original whole value, and the exact thrown value follows the
existing Completion route. A strict inherited getter on a primitive therefore
receives that primitive. Neither consumer substitutes a boxed receiver.

The checked source factory captures the raw source and performs ToObject before
any property operation. The checked key factory performs ToPropertyKey once and
retains the accepted String or Symbol in an ordinary activation binding. The
operations-private native projection accepts the whole opaque operation and
exhaustively selects its actual GetV key or Rest exclusions. It compiles those
retained reads directly into `PropertyKeyLocals`; it accepts no independent raw
key and emits no repeated conversion. The resulting closed native operation
keeps the remaining borrowed operands paired with those keys.

Rest shares the existing fresh-object allocation and CopyDataProperties body.
That body remains in `control_flow.rs`, preserving own-key order, exclusion by
String or Symbol identity, descriptor/enumerability checks, Proxy hooks, getters,
and complete abrupt routing. The ordinary complete consumer retains its keys
through the same Rest call and clears them in reverse order. Incremental Rest
projects the captured exclusions and clears its local copies after consumption.

PutTarget evaluates its whole supplied value once, invokes the existing
`prepare_destructuring_target` and `put_destructuring_target`, and returns that
same value after success. Its checked member targets read previously retained
raw base and key cells. This preserves the original Reference acquired before
GetV or a suspended default, while deferring nullish rejection and target-key
conversion to PutValue. Private writes keep their existing brand checks; eager
nested arrays keep their existing iterator and IteratorClose owner. Assignment
Identifiers and var targets use the separately captured write-only Identifier
Reference path. Genuine lexical binding initialization keeps its original
predeclared storage and binding mode.

The direct lexical-instantiation walker consumes an incremental declaration's
actual `visit_bindings` projection. It uses the same scope lookup, original
storage allocation and uninitialized-cell setup as complete Array/Object
destructuring. Var remains on its separate Reference path. Thus the new
DeclarationEvaluation form participates in fresh/resumed TDZ setup instead of
falling through the walker's non-declaration arm.

No GC schema, object representation, interpreter, dispatcher fallback or
second destructuring algorithm is added. The complete ordinary consumer's
entry/source boxing/default sequence is retained, and its target preparation
still precedes GetV and default evaluation. Moving that physical body only
changes the endpoint used to bound the existing synchronous for-of lifecycle
source witness; its assertions and test names are unchanged.

The joined object-pattern source controls cover suspended defaults and keys,
captured assignment targets, whole thrown identities, rest exclusions, nested
patterns and GC retention. Existing var-destructuring order controls and
prepared-target/iterator ownership controls remain required regression coverage.
Those controls are not executed by this source packet. Pattern-owned suspension
is admitted only where the checked source/lowering owner supplies the complete
operation sequence; this native seam does not widen source admission.
