# PromiseResolve Realm context

Status: implemented with focused source, structure, runtime and workspace-check
evidence.

## Ownership

The backend uses local `Promise.resolve` function objects to implement the
specification's `PromiseResolve(C, value)` operation. Those functions do not
escape, but their Realm remains observable through errors and through the
canonical `%Promise%` constructor selected by intrinsic await normalization.
A raw function payload with a zero environment silently selected entry-Realm
error prototypes even when a created-Realm `finally` continuation was
executing.

Intrinsic await normalization has a second inseparable authority: the
canonical `%Promise%` passed as `C`. Async functions and async generators use
the Realm captured by their activation. Async builtins use the defining Realm
of the executing self-backed builtin, with zero environment reserved for the
entry publication route. The Promise job that happens to be active is not an
ownership source.

## Typed boundary

`PromiseResolveRealmAuthority` is the closed private source domain:

- `CurrentFunction` selects the executing self-backed function, with the
  explicit entry route for a zero environment; and
- `AsyncExecution` borrows an opaque `AsyncExecutionRealmContext` retained by
  an async activation.

`PromiseResolveOperationRealmContext` is private, non-`Copy` and must-use. Its
factory materializes the local `Promise.resolve` function through the existing
Promise internal-function choke point. The function therefore receives its
defining Realm, that Realm's Function, TypeError and RangeError prototypes, and
a self environment before it can be called.

`IntrinsicPromiseResolveRealmContext` pairs that operation context with the
canonical Promise constructor loaded from the same Realm intrinsic catalog.
The shared await-normalization path uses the paired constructor both for the
initial `PromiseResolve` call and for the rejected wrapper created when that
call abruptly completes. Missing nonentry Realm, intrinsic table, constructor
or function/error prototype state traps rather than substituting entry globals.

Both carriers and their complete factory/call/release lifecycle belong to the
private `builtins/promise/promise_resolve_realm_context.rs` owner. Rust requires
their names to be `pub(super)` because parent and sibling method calls expose
the inferred return and parameter types, but every field remains child-private.
The parent and `promise_finally_completion.rs` therefore cannot construct or
project either raw context. They import and re-export neither carrier. The
child also owns the narrow abrupt-normalization capability operation, so the
parent never projects the paired constructor payload.

The `finally` continuation intentionally consumes only the operation context:
its constructor `C` was selected earlier by SpeciesConstructor and is a
separate authority. Both NewPromiseCapability TypeErrors now use the executing
Promise operation function's Realm, so a species constructor that fails to
initialize resolving functions cannot fall back to the entry TypeError.

## Covered sites

The boundary removes both remaining raw `Promise.resolve` materializations:

1. shared intrinsic await normalization, including its abrupt-to-rejected
   wrapper branch;
2. async-generator await-return normalization; and
3. the shared ThenFinally/CatchFinally continuation body.

No heap layout, trace descriptor, job record or Promise reaction wire word
changes.

## Focused evidence

The bounded structure target pins context privacy and lifecycle, the exhaustive
authority projection, same-catalog constructor/function selection, strict
nonentry traps, the exact three semantic consumers, the absence of raw Promise
function materialization and entry-constructor reads in those consumers, and
current-operation Realm errors in NewPromiseCapability.

The owner move selected the exact ten-line carrier block, 103-line factory
block, 41-line call/release block and 26-line intrinsic-resolve block at
SHA-256
`d15798a3e31f7b38ad6f9779797a480304d12321e438aef6d74de711a6c801f9`,
`77f4c4bf5164acc705865aef1e567e930f326df895c94e062dd6c5bd85a3113c`,
`2a895e67fa55537efdc0d230f253ccef95172cacce32d0f6f12b6250c9a44110`
and
`a76484e712c9c620512d11804f541fbc0e39fd128520728a7887747e62c1c3`.
Normalizing only the required `pub(super)` visibility restores each original
hash after relocation. The abrupt-wrapper operation retains its exact original
eight-line `emit_new_promise_capability` block at SHA-256
`557738476f2f2f01137b243dda634ec95cd0e119f2b23059a246c78a5b7a627f`.
The 206-line child has SHA-256
`9aefd81a7d9fee98addd74c002a897fd6c9815306556e6e22d99320970814842`
and reduces the concurrent parent from 7,488 to 7,304 lines. The recursive
guard pins child-only carrier construction/projection, zero import/re-export
paths, the exact factory/call/release owners and unchanged inferred callers.
At the coordinated checkpoint, this structure target and
`promise_resolve_realm_authority_ownership_structure` each pass `4/4`, while
`promise_internal_function_realm_context_structure` passes `6/6`, for `14/14`
focused structure checks. The exact
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witness passes `1/1`, and `cargo xc` is green. Semantic goldens were not
rerun because this is a source-equivalent owner move.

The finite Promise callback fixture borrows `Promise.prototype.finally` from a
created Realm, supplies a species constructor that does not initialize the
capability executor, invokes the captured ThenFinally closure and requires the
resulting TypeError to inherit from the borrowed method's TypeError prototype.

```sh
cargo test -p lila-aot-wasm --test promise_resolve_realm_context_structure --quiet
cargo test -p lila-aot-wasm --test promise_resolve_realm_authority_ownership_structure --quiet
cargo test -p lila-aot-wasm --test promise_internal_function_realm_context_structure --quiet
cargo test -p lila-cli --test cli functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations --quiet
cargo xc
```

The following workspace semantic golden passes `2/2` in 771.49 seconds with
669 dumps. It adds only the independent Temporal arithmetic witness, removes
none, and leaves 667 of 668 retained dumps equal after accounting
normalization. The expanded Promise internal-callback Realm witness is the sole
retained structural change.

This boundary does not close generic Promise combinator iterator errors,
general AggregateError construction, Promise allocation in every async builtin
or the complete T06/T14/Test262 acceptance matrices.
