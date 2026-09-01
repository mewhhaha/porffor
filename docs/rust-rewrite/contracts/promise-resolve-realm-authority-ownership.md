# PromiseResolve Realm authority ownership

Status: normative for the private Wasm AOT `PromiseResolve` Realm-selection
boundary.

## Ownership boundary

`PromiseResolveRealmAuthority` chooses exactly one source for the local
`Promise.resolve` function's defining Realm:

- `CurrentFunction` uses the executing self-backed Promise operation; and
- `AsyncExecution` borrows the Realm retained by an async activation.

The authority is private and implements no cloning, copying, debugging,
equality or default capability. It is passed by value through exactly one of
the operation-only or intrinsic paired-context factories. Both factories move
it once into the shared materialization-context selector, whose exhaustive
match consumes the selection. Reusing one selected authority for a second
Realm projection is therefore an E0382 move error.

This lifetime is distinct from the borrowed `AsyncExecutionRealmContext` held
inside the async variant. The activation context may be borrowed by other
algorithms until its explicit release; the narrower `PromiseResolve` selection
is one-shot.

## Producers and consumer

Four semantic producer routes remain:

1. intrinsic Await selects current-function authority from its move-only
   reaction initialization policy;
2. intrinsic Await selects captured-async authority from that policy;
3. async-generator await-return names its captured execution Realm; and
4. a `finally` continuation names its executing function Realm.

The recursive Promise source contains ten exact authority identifiers: four in
the parent for the declaration and three await producers, five in the private
PromiseResolve Realm-context child for three typed factory parameters and two
exhaustive consumer arms, and one in the private finally-completion child for
its producer route. The operation and intrinsic factories each forward their
owned parameter once. The shared selector is the sole exhaustive consumer.

The Rust-lexical guard removes comments and all Rust string, byte-string,
C-string, raw-string, character and byte-character literals before checking
that census, the attribute-free declaration, capability absence, by-value
factory signatures, exact forwarding and producer order.

## Nonclaims and verification

This is source-equivalent ownership hardening. It changes no heap word, Wasm
local, emitted instruction, Promise settlement, callback Realm, error Realm or
job ordering. It adds no suspended-async or Promise conformance claim.

The dedicated structure target now follows the authority recursively across
the parent, PromiseResolve Realm-context owner and finally-completion owner.
Its previous `4/4` checkpoint, the PromiseResolve Realm-context target's prior
`4/4`, the neighboring reaction-initialization target's prior `4/4`, and the
created-Realm Promise internal-callback CLI witness's prior `1/1` remain the
finite behavior baseline. At the Batch S coordinated checkpoint, this target
and `promise_resolve_realm_context_structure` each pass `4/4`, while
`promise_internal_function_realm_context_structure` passes `6/6`, for `14/14`
focused structure checks. The exact
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witness passes `1/1`, and `cargo xc` is green. Semantic goldens were not
rerun because this is a source-equivalent owner move. Broad Promise, Test262
and golden verification remain deferred to the shared batch checkpoint.
