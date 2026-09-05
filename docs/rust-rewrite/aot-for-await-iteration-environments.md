# Captured for-await iteration environments

This follow-up closes the captured-head environment gap for async-generator
`for await...of` bodies that suspend with `yield`.

A captured `let` or `const` loop head requires a fresh declarative Environment
Record for every iteration. Before this change, the Wasm AOT dispatcher refused
that shape whenever the body could suspend because the environment was created
and destroyed inside one Wasm invocation. Splitting the body at `yield` would
otherwise allocate a second record on resume and disconnect closures from the
cell created before suspension.

The for-await emitter now applies the same ownership rule as resumable classic
loops. The invocation resuming from `await next()` first observes `done`; only an
active iteration allocates a fresh record. It publishes the exact current
environment pointer into the owning activation. A body-resume invocation starts
with that pointer already restored by function entry. Both runtime paths then
attach one compiler binding scope, so outer activation-owned slots acquire the
correct parent hop and the loop head uses the same cell captured before `yield`.

Suspension returns without cleanup. Normal and abrupt iteration completion
converge on one inner cleanup block, restore the parent pointer in the activation,
and only then continue through the existing completion and IteratorClose logic.
This keeps local `continue`, `break`, `return`, throws, and normal fallthrough from
double-unwinding or leaking the child environment into the next iteration.

The compiler/runtime split is explicit:
`emit_allocate_lexical_environment_record` creates a runtime record without
mutating compiler scope state, while `begin_existing_lexical_environment_scope`
attaches the compile-time binding view after the fresh and resumed paths converge.

## Verification

The bounded publication gate runs formatting, `git diff --check`, and
`cargo check --locked -p lila-aot-wasm` before executing the focused Wasmtime
target. The implementation is published only after all of those checks pass.

`cargo test --locked -p lila-engine --test aot_async_for_of -- --test-threads=1`
contains a captured-`let` Wasmtime regression. It proves that a closure observes a
mutation made after resumption and that the next iteration receives a distinct
cell. The normal AOT regression workflow also continues to run the complete
backend shards.

## Deliberate boundary

This batch does not remove the separate refusal for an async-generator for-await
body whose own top-level block Environment Record spans suspension, or for nested
`for await` / body-`await` state ownership. It does not change iterator
acquisition, AsyncFromSyncIterator semantics, IteratorClose precedence, or the
runtime dynamic-source policy.
