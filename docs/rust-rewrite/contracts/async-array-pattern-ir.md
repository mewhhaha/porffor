# Plain async array pattern ownership

`AsyncFunctionArrayDestructuringIr` is an opaque, consumed plain async statement
owner. The source planner traverses the actual ArrayPattern and its original
target, default, nested pattern and expression operands. It reserves acquisition,
body and exit states, the actual Await suspension/resume tape, and this pattern's
ordered iterator operations. Defaults reserve both branches of their undefined
guard. Conditional, logical and optional expressions keep their own branch
ranges, including an optional tail whose original base is known to be nullish.

The constructor folds the actual lowered body once. Every Await, branch range,
nested array owner and iterator operation must consume those source reservations
exactly. It checks the raw RHS and iterator cell against the function's actual
owned environment inventory, and rejects duplicate names or slots across the
complete nested pattern tree. Each operation must use its own pattern's concrete
iterator storage. The accepted body excludes foreign control owners and
materialized body environments. Its private Await census is consumed by the
existing synchronous ForOf async body validator; general async state readers
continue to reject bare array iterator operations.

Lexical and var initializers and both used and discarded assignment expressions
consume the checked source owner. Target selection and Put use the same physical
methods as the ordinary generator path. The RHS is retained once and assignment
returns that original value. Identifier and member References are acquired in
source order before the element's default. Await prefixes and their undefined
guard preserve those original targets and lexical initialization cells.

Native execution uses the same IteratorRecord, BindingCell GC edge, invocation
environment and physical acquisition/body/close pipeline as ordinary generators.
Acquisition is outside this pattern's own close scope. Fresh and resumed body
execution enter the same close scope before an Await rejection is injected.
Protocol, step and value errors retain the shared DONE behavior. Target,
default, Put and injected abrupt completions reach synchronous IteratorClose,
which preserves the complete pending completion before the record is retired.
Nested acquisition failures reach their enclosing pattern's close owner.

This source batch assumes the experimental Wasmtime GC/reference feature surface.
It introduces no second iterator storage representation. Ordinary generator and
async generator continuation domains keep their separate admission boundaries.

Private constructor controls mutate real source-produced carriers to exercise
removed/substituted operations, missing/displaced Await states, foreign storage,
missing/ambiguous inventory rows and cross-pattern aliases. Separate IR, emitted
GC/Wasm and engine controls cover lazy defaults, retained NEXT and References,
nested close order, whole rejection preservation, original RHS identity and the
older eager awaited-initializer path. These controls are authored; compilation
and runtime verification remain deferred to the final coherent batch checkpoint.
