# Plain async With ownership

`AsyncFunctionWithIr` is an opaque complete statement owner. Its constructor
consumes a plan from the actual With AST, the completed head prefix, the boxed
head read, the original analyzed Object Environment Record, the body and the
function's actual owned binding inventory. The invocation head cell must have
one matching name and slot. The child object cell is a separate domain and must
remain the sole slot-zero binding in the original With environment.

The head completes ToObject before entering the child environment. Head and
body consume exactly their reserved ranges. The actual Await tape must match
the source tape, including nested With, guarded expressions, array patterns,
class evaluation and selector-first Switch traversal. Matching range endpoints
alone cannot admit a body that loses source Await operations. The complete owner
also retains entry and exit phases when its source has no Await.

Existing async branch, Try, Switch and labelled state readers consume this
checked whole statement. Ordinary generators, eager loop bodies and array
pattern bodies keep their foreign continuation refusals. Actual enclosing loop
lowering rejects an uncomposed Async With owner even if it contains no Await;
it does not infer ownership from a protocol name or a flat suspension count.
The original eager With paths inside existing loop regions retain their current
admission boundaries. Nested callable activations have independent source plans.

Global storage, completion, early-error, throw-inference and backend walks inspect
the actual head, child environment and body. Native execution consumes the same
original Object Environment Record and invocation BindingCells as the existing
generator With pipeline, on the experimental Wasmtime GC/reference target.

Private constructor controls damage real source-produced carriers: missing or
aliased head inventory, incomplete ToObject publication, foreign child records,
duplicate body environments, displaced Await states and equal-extent source
loss. Source, artifact and engine controls cover References selected before an
awaited RHS, resumed object identity and abrupt environment restoration. These
controls are authored; compilation and runtime verification remain deferred to
the complete source batch checkpoint.
