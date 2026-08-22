# Async-generator synchronous `using` scopes

## Evidence and scope

At source commit `a5606a73cbbb2a8ffd81c0c2e2dee945bb2b9a4b`
and pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`,
the exact file

```text
language/statements/using/initializer-disposed-at-end-of-asyncgeneratorbody.js
```

reports `0/2` under Wasm AOT. Its sloppy and strict Script executions are both
`Runtime/NotImplemented` with the diagnostic `using declaration in an async
generator`. The path has no Wasm-AOT source rewrite, mask, or backlog entry.
Older passing `latest-*` evidence for this path is a spec-exec snapshot at a
different Test262 pin and is not product evidence. This contract owns only the
one-file, two-execution Wasm-AOT delta above.

The supported source domain is synchronous `using` in an ordinary statement
list owned by an async generator whose existing linear suspension plan can
represent the body. It includes nested ordinary Blocks, multiple reached
declarations, `yield`, and `await` in the scope body. It excludes `await using`,
async disposers, suspension inside a resource initializer, classic-`for` and
`for-of` resource heads, modules, dynamic source, and every nonlinear async
generator form rejected by the existing suspension plan.

## Normative lifetime

Calling the async generator creates its activation but does not evaluate a
resource initializer. The first request that reaches a `using` declaration:

1. allocates and publishes one activation-backed DisposeCapability;
2. evaluates resource initializers in declaration order;
3. validates and acquires each `@@dispose` method exactly once;
4. registers a resource only after acquisition succeeds; and
5. initializes its immutable lexical binding only after registration.

Neither a `yield` nor an `await` disposes the retained resources. Later requests
reload the same capability and skip acquisition. When normal, return, or throw
completion leaves the scope, including an externally requested return or throw
and an awaited rejection resumed as a throw, registered entries are disposed in
reverse order before the current async-generator request is completed or its
queue is drained. Disposal failures use the existing `SuppressedError` fold.

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
    AsyncGenerator(AsyncGeneratorSyncDisposableCapabilityIr),
}
```

`AsyncGeneratorSyncDisposableCapabilityIr` is a private-field, non-`Copy`,
`#[must_use]` proof. Its sole crate constructor accepts the hidden name returned
by lowering's suspension-owned binding allocator with the fixed
`async.generator.dispose.capability.` prefix. Backend crates may read that name
but cannot manufacture the proof from an arbitrary `String`. The binding is
therefore present exactly once in the owning `FunctionIr`'s activation-backed
`owned_env_bindings` by construction.

Analysis projects every function execution kind through the exhaustive
`SyncDisposableScopeOwnerPlan::{Immediate, PlainGenerator, AsyncFunction,
AsyncGenerator}` domain. The async-generator arm must mint and consume the new
carrier; it may no longer degrade to immediate storage or a named diagnostic.
There is no optional capability and no shared boolean resumability flag. A new
function kind or scope execution owner is a compile-time obligation at every
producer and consumer match.

The capability stores backend-owned DisposeCapability state, not a user-visible
value. Resource bindings remain owned solely by `SyncDisposableResourceIr`; the
capability must not duplicate their `InitializeBinding` operations.

## Producer and consumer obligations

The IR producer must:

- resolve the exhaustive execution owner before lowering any initializer;
- allocate exactly one hidden async-generator activation binding per reached
  scope;
- retain the non-empty resource and suffix-nesting invariants;
- keep registration before lexical binding initialization;
- keep async-generator synchronous `using` out of generic `TryFinally`; and
- preserve every excluded source form as an explicit named diagnostic.

The Wasm consumer must select `AsyncGenerator` exhaustively, use the async
generator resume-state and lexical-environment offsets, derive a live span that
includes both `GeneratorYield` and `AsyncAwait`, initialize only at the scope's
entry state, and retain the capability through every suspension. Normal,
external return, external throw, source throw, and awaited rejection must enter
the same disposal frame before async-generator request settlement. No consumer
may substitute generator offsets, async-function offsets, or temporary locals
for the async-generator capability.

## Focused invariants and verification

Durable `lila-ir` coverage pins that:

- an async-generator scope has the `AsyncGenerator` owner;
- both `GeneratorYield` and `AsyncAwait` remain inside the live scope body;
- nested scopes have distinct capability names;
- each capability name appears exactly once in `owned_env_bindings`; and
- ordinary, plain-generator, and plain-async-function scopes retain their
  existing owner variants.

Central integration owns compilation and runtime execution:

```sh
cargo fmt --all -- --check
cargo check -p lila-ir
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir async_generator_synchronous_using --quiet
./target/debug/lila test262 run \
  language/statements/using/initializer-disposed-at-end-of-asyncgeneratorbody.js \
  --suite-root test262/vendor/test262 \
  --execution-backend wasm-aot --timeout-ms 60000 --threads 1
```

Only the final command can move the exact measured status from `0/2` to `2/2`.
This focused result is not a claim about `await using`, async disposers,
resource-initializer suspension, resource loop heads, modules, dynamic source,
nonlinear async-generator forms, the complete `using` tree, or the full pinned
Test262 aggregate.
