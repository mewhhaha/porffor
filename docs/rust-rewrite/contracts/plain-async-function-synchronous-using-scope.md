# Plain-async-function synchronous `using` scopes

## Evidence and scope

At source commit `1f27bc71f678d5b27e08d2719c660b9777021af4`
and pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`,
the exact file

```text
language/statements/using/initializer-disposed-at-end-of-asyncfunctionbody.js
```

reports `0/2` under Wasm AOT. Both Script executions are
`Runtime/NotImplemented` with the diagnostic `using declaration in an async
function or async generator`. This contract owns that one-file, two-execution
delta.

The supported source domain is synchronous `using` in an ordinary statement
list owned by a plain async function. It includes nested ordinary Blocks,
multiple reached declarations, and suspension through the async function's
existing linear `await` plan. It excludes async generators, plain and async
generator resource heads, classic-`for` and `for-of` resource heads,
`await using`, suspension inside a resource initializer such as
`using value = await promise`, modules, dynamic source, and every async form
rejected by the existing linear async plan.

## Normative lifetime

Calling the async function reaches a `using` declaration in ordinary source
order. At that point it:

1. allocates and publishes one activation-backed DisposeCapability;
2. evaluates resource initializers in declaration order;
3. validates and acquires each `@@dispose` method exactly once;
4. registers a resource only after acquisition succeeds; and
5. initializes its immutable lexical binding only after registration.

An `await` returns control to the async driver without disposing the retained
resources. Every later invocation of the same activation reloads the same
capability and skips acquisition. When normal, return, or throw completion
leaves the scope, including a rejection resumed as a throw, disposal runs in
reverse registration order before the async driver resolves or rejects the
function's Promise. Disposal failures use the existing `SuppressedError` fold.

An initializer or method-acquisition failure follows the same exit after every
previously registered entry has been published. Nested scopes own distinct
capabilities; suffix nesting disposes the inner capability before the outer.

## Closed IR ownership

Every ordinary statement-list resource node carries one required owner:

```rust
StatementIr::SyncDisposableScope {
    execution: SyncDisposableScopeExecutionIr,
    resources: SyncDisposableResourcesIr,
    body: BlockIr,
}

pub enum SyncDisposableScopeExecutionIr {
    Immediate,
    PlainGenerator(PlainGeneratorSyncDisposableCapabilityIr),
    AsyncFunction(AsyncFunctionSyncDisposableCapabilityIr),
}
```

`AsyncFunctionSyncDisposableCapabilityIr` is a private-field, non-`Copy`,
`#[must_use]` proof. Its sole crate constructor accepts the hidden name returned
by lowering's suspension-owned binding allocator. Backend crates may read that
name but cannot manufacture the proof from an arbitrary `String`. The binding
therefore appears exactly once in the owning `FunctionIr`'s activation-backed
`owned_env_bindings` by construction.

Analysis projects every function execution kind through the exhaustive
`SyncDisposableScopeOwnerPlan::{Immediate, PlainGenerator, AsyncFunction,
AsyncGenerator}` domain. Script and ordinary functions select `Immediate`, a
plain generator selects its existing carrier, a plain async function mints the
new carrier, and an async generator remains an explicit diagnostic. There is
no optional capability and no shared boolean resumability flag. A new function
kind or scope execution owner is a compile-time obligation at producer and
consumer matches.

The async capability stores backend-owned DisposeCapability state, not a
user-visible value. Resource bindings remain owned solely by
`SyncDisposableResourceIr`; the capability must not duplicate their
`InitializeBinding` operations.

## Producer and consumer obligations

The IR producer must:

- resolve the exhaustive execution owner before lowering any initializer;
- allocate exactly one hidden async activation binding per reached scope;
- retain the non-empty resource and suffix-nesting invariants;
- keep resource initialization ordered after successful registration;
- keep async-function synchronous `using` out of generic `TryFinally`; and
- leave async generators and every excluded source form as named diagnostics.

The Wasm consumer must select `AsyncFunction` exhaustively, derive its live
suspension span through `AsyncAwait`, initialize only at the declaration's
entry state, store the capability in the named activation binding, and retain
it through every await. A resumed rejection must enter the same scope-exit
path as a source throw. No consumer may substitute temporary locals for the
activation-backed capability or settle the Promise before disposal finishes.

## Focused invariants and verification

Durable `lila-ir` coverage pins that:

- an async-function scope has the `AsyncFunction` owner;
- an `AsyncAwait` remains inside the live scope body;
- nested scopes have distinct capability names;
- each capability name appears exactly once in `owned_env_bindings`; and
- ordinary functions and plain generators retain their existing owner variants.

Central integration owns compilation and runtime execution:

```sh
cargo fmt --all -- --check
cargo check -p lila-ir
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir plain_async_function_synchronous_using --quiet
./target/debug/lila test262 run \
  language/statements/using/initializer-disposed-at-end-of-asyncfunctionbody.js \
  --suite-root test262/vendor/test262 \
  --execution-backend wasm-aot --timeout-ms 60000 --threads 1
```

Only the final command can move the exact measured status from `0/2` to `2/2`.
This focused result is not a claim about async generators, `await using`,
resource-initializer suspension, the complete `using` tree, or the full pinned
Test262 aggregate.
