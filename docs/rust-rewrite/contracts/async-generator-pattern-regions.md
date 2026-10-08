# Mixed async-generator pattern regions

Suspended Super targets consume the same captured Reference owner as mixed
assignments. The receiver, original Super base and raw referenced name occupy
distinct original activation cells before GetV, IteratorValue or a lazy default.
The final Put spends that Reference once, so a prototype change during suspension
cannot reselect the Super base. Ordinary Generator and plain Async patterns use
the same physical target producer with their original source protocols.

Checked FunctionBody patterns use one actual mixed source allocator. The Array
source records acquisition entry, complete body range, exit, every Await/Yield
point, and each StepValue/Elision/RestArray operation at its actual state. Object
keys, targets, defaults and nested arrays append to that same allocator. Nested
callable activations remain separate.

The whole RHS completes first. Lexical BoundNames retain their original analyzed
TDZ and scoped storage; classic For heads use the same source-name/storage map.
Identifier References and raw Member operands are captured before the original
step/Get/default order. Undefined alone selects a default, whose complete mixed
branches stay under the checked guard. Used assignment expressions return the
original whole RHS identity with stale shape facts cleared.

Generator, plain Async and mixed patterns consume one physical Array/Object
element lowering body and the original GetV, normalized-key, rest and Put owners.
Only the concrete mixed Array constructor validates its exact operation/storage
and global cell-alias census before minting the body. Operations remain dedicated
statements and cannot be buried in an expression.

The native owner uses the existing synchronous IteratorRecord in BindingCell
field 11. Acquisition precedes its own close scope. Fresh and resumed bodies
establish close before rejected Await or injected Yield completion. Protocol
step failures set Done; target/default/Put failures and injected whole abrupt
completions use the original IteratorClose precedence. Nested owners retire and
close independently. No second object/iterator model is introduced.

Foreign ForOf/ForAwait/resource bodies, Super targets and unsupported opposite
composite expression owners retain their explicit refusals. Return still has its
original implicit Await; Yield adoption adds no invented source Await.

The authored IR and Engine controls are unrun. Compilation, focused regressions
and broad verification remain deferred until the complete dry coding batch.
