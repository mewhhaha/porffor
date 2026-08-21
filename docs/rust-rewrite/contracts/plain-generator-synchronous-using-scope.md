# Plain-generator synchronous `using` scopes

## Evidence and scope

At source commit `904da7b355811ad399ff284bf0ddeac47d2cc9c2`
and pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`,
the exact file

```text
language/statements/using/initializer-disposed-at-end-of-generatorbody.js
```

reports `0/2` under Wasm AOT. Both the sloppy and strict executions are
`NotImplemented:Runtime` with the diagnostic `using declaration in a generator
or async function`. This contract owns that one-file, two-execution delta.

The supported source domain is a synchronous `using` declaration in an
ordinary statement list owned by a plain synchronous generator. This includes
nested ordinary Blocks and multiple reached declarations, which retain the
suffix nesting defined by
[`synchronous-using-scope-ir.md`](synchronous-using-scope-ir.md). It excludes
classic-`for` and `for-of` resource heads, async functions, async generators,
`await using`, modules, dynamic source, and every generator form rejected by
the existing linear generator plan.

## Normative lifetime

Calling the generator allocates its activation but does not evaluate a resource
initializer. On the first resume that reaches the declaration, the generator:

1. evaluates each initializer in declaration order;
2. validates and acquires `@@dispose` exactly once;
3. registers the resource only after acquisition succeeds; and
4. initializes the immutable lexical binding only after registration.

A `yield` suspends this execution without disposing the registered resources.
The generator activation retains the DisposeCapability while suspended. When
the scope later receives a normal, return, or throw completion, it disposes the
registered entries in reverse order, folds disposal failures through
`SuppressedError`, and only then completes or publishes the final iterator
result. An external generator `return` or `throw` therefore uses the same scope
exit; suspension itself is not a scope exit.

Nested reached `using` declarations own distinct capabilities. Their existing
suffix nesting makes the inner capability dispose before the outer capability.
An initializer or method-acquisition failure disposes only entries already
registered in reached enclosing scopes.

## Closed IR ownership

Every ordinary statement-list resource node carries one required execution
owner:

```rust
StatementIr::SyncDisposableScope {
    execution: SyncDisposableScopeExecutionIr,
    resources: SyncDisposableResourcesIr,
    body: BlockIr,
}

pub enum SyncDisposableScopeExecutionIr {
    Immediate,
    PlainGenerator(PlainGeneratorSyncDisposableCapabilityIr),
}
```

`Immediate` retains the existing non-resumable local lifetime.
`PlainGenerator` carries a private-field, non-`Copy` capability whose sole
crate constructor accepts a hidden binding allocated through the suspension
owned-binding allocator. Its public accessor exposes the storage name to the
backend but backend crates cannot mint a generator capability from an arbitrary
string. The binding is therefore present in the owning `FunctionIr`'s
activation-backed `owned_env_bindings` set by construction.

Analysis projects every function protocol through the exhaustive private
`SyncDisposableScopeOwnerPlan::{Immediate, PlainGenerator, AsyncFunction,
AsyncGenerator}` domain. Script and `Ordinary` select `Immediate`, `Generator`
allocates and consumes the plain-generator carrier, and the two async owners
remain explicit diagnostics. There is no optional plan and no boolean
resumability flag. Adding another execution kind or another scope execution
owner is a compile-time obligation at analysis, lowering, and every backend
consumer.

The capability binding stores backend-owned DisposeCapability state rather
than a user-visible lexical value. The binding is unique per emitted scope and
survives every yield because it is activation-backed. Resource bindings remain
owned by `SyncDisposableResourceIr`; the hidden capability must not duplicate
their `InitializeBinding` operation.

## Producer and consumer obligations

The IR producer must:

- locate the execution owner before lowering a resource initializer;
- allocate exactly one hidden activation binding per reached generator scope;
- preserve the existing non-empty resource and suffix-nesting invariants;
- keep plain-generator `using` out of the generic `TryFinally` representation;
- keep classic-`for`, `for-of`, async, module, and dynamic-source exclusions
  explicit; and
- visit resource initializers followed by the body in every analysis and IR
  traversal.

The Wasm consumer must exhaustively distinguish `Immediate` from
`PlainGenerator`. The latter initializes its capability only when execution
reaches the declaration, saves it in the named activation binding, keeps it
live across `GeneratorYield`, and disposes only along scope-completing paths.
No consumer may reuse ordinary temporary locals for the generator capability.

## Focused invariants and verification

Durable `lila-ir` coverage pins that:

- a plain-generator scope has the `PlainGenerator` owner;
- its private capability name appears exactly once in the function's owned
  activation bindings;
- the capability is distinct for nested scopes;
- the body still contains the `GeneratorYield` between acquisition and the
  final suffix; and
- an ordinary function still receives `Immediate`.

Central integration owns compilation and runtime execution:

```sh
cargo fmt --all -- --check
cargo check -p lila-ir
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir plain_generator_synchronous_using --quiet
./target/debug/lila test262 run \
  language/statements/using/initializer-disposed-at-end-of-generatorbody.js \
  --suite-root test262/vendor/test262 \
  --execution-backend wasm-aot --timeout-ms 60000 --threads 1
```

Only the final command can move the exact measured status from `0/2` to `2/2`.
This focused result is not a claim about the complete `using` tree or full
pinned Test262.
