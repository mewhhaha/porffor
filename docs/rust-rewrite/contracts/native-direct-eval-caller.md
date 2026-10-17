# Native direct-eval caller ownership

Native direct eval has an independent prepared-Script role. Its compiler takes
the actual ECMAScript UTF-16 source and a validated `RuntimeDirectEvalCaller`;
it never enters the global-Script compiler route or a fresh Script main. Parsing,
early errors, IR and Wasm compilation remain in the native Rust host. No parser,
interpreter or VM is added to emitted Wasm.

`DirectEvalContextIr::runtime_descriptor` projects immutable caller grammar:
strictness, `EvalInvocationContext`, visible private spelling-to-identity
selectors, an external derived-constructor role and an explicit caller import
origin. Its canonical `LILADE01` codec transports compiler facts, rather than
JavaScript values or runtime references. Decoding does not grant activation or
private-name authority. The native boundary must validate the descriptor from
the actual immutable calling program and decode the actual runtime records.

`RuntimeDirectEvalCaller::validate` checks each inherited private selector
against the slot count of its actual runtime `PrivateEnvironment` table. The
host supplies every row in the immutable parent chain, including classes whose
spellings are shadowed. New private class scopes start above the maximum scope
in that complete chain; duplicate scopes, missing slots and integer exhaustion
are rejected. Runtime `PrivateName` identities are retained through the passed
private environment, never recreated from their descriptions. Each lowering
analysis reserves its required scopes before allocation. Normal source sizes
prove a conservative bound in constant time; nearly exhausted spaces use an
exact count of the actual private and auto-accessor class environments.

The native request must root and retain the actual lexical, variable and private
environments, `this`, `new.target`, home object, class context and existing four
derived-activation binding cells across compilation and entry. Those records
include normal `arguments` resolution and uninitialized derived `this` state.
They belong to the current invocation. No ambient current program or synthetic
activation may replace them. The independent P2 owns its compiled source and
functions, uses the exact admitted runtime R in the same native Store, and
receives those caller records through the prepared-Script ABI. The original
execution control and deadline cover compilation and nested execution.

`DirectEvalDerivedOwner::LocalFunction` names a constructor planned inside the
same P. `ExternalCaller` cannot become a P2-local FunctionId. Its `super()` must
use the retained active-constructor cell and a typed instance-initialization
entry owned by the constructor's defining P1. An explicit absent initializer
means that constructor has no instance elements. Completion transport retains
ordinary throwing behavior and initializes fields only after successful
`BindThisValue`, including the original second-`super()` check.

External caller import origin remains distinct from a local graph record.
`ImportCallSourceOriginIr::ExternalCaller` survives on the actual `DynamicImport`
operation, including computed operands and nested or resumable function bodies.
It cannot become `Local(None)` or a P2-local numeric record. The IR retains the
real operation without rejecting the containing source eagerly. Until the true
external import bridge exists, AOT must emit a typed runtime capability
rejection only when that operation executes, after its original operand
evaluation. An unexecuted branch or uncalled function retains ordinary source
behavior. A future bridge must retain the originating program and record
instead of reinterpreting its numeric ID inside P2.

The IR component supplies a real prepared direct-eval root witness and focused
controls. Native import linkage, AOT emission, same-Store GC ownership and full
suite acceptance must be verified with the coherent ABI8 companion components.
Repeated-source code/data reuse is a separate measured compiler optimization;
this contract introduces no fixture recognition, source-pattern shortcut or
prepared-catalogue capacity increase.
