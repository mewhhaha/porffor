# Synchronous `using` scope IR

## Scope

This contract covers non-resumable synchronous `using` declarations that are
direct children of an ordinary Script, Block, or ordinary function body. This
batch does not claim modules. It deliberately does not cover `await using`,
generators, classic `for` heads,
`for-in`/`for-of` heads, Switch CaseBlocks, or dynamic `eval`. Plain synchronous
generators are the one resumable extension defined by
[`plain-generator-synchronous-using-scope.md`](plain-generator-synchronous-using-scope.md);
plain async functions are the second extension defined by
[`plain-async-function-synchronous-using-scope.md`](plain-async-function-synchronous-using-scope.md);
async generators are the third extension defined by
[`async-generator-synchronous-using-scope.md`](async-generator-synchronous-using-scope.md).
The other forms remain explicit unsupported boundaries until they acquire their
own environment, iteration, suspension, or dynamic-source contract.

An ordinary ECMAScript `try`/`finally` remains `StatementIr::TryFinally`.
Resource disposal is not a source `finally` clause and must never be lowered
to that generic node.

## Closed IR capability

The producer emits exactly this capability:

```rust
StatementIr::SyncDisposableScope {
    execution: SyncDisposableScopeExecutionIr,
    resources: SyncDisposableResourcesIr,
    body: BlockIr,
}

SyncDisposableResourceIr {
    binding_name: String,
    initializer: TypedExpr,
}
```

`binding_name` is the already-allocated IR storage name of the immutable
lexical binding. A resource entry has no disposal-kind flag: every entry in
this node is `sync-dispose`. It has no optional acquired-method field: the
backend only appends a runtime resource record after acquisition succeeds.
`SyncDisposableResourcesIr` is a private-field `first`/`rest` carrier whose
only constructor requires the first entry. It is therefore non-empty by type,
and its public iterator exposes entries in source declaration order without
letting backend crates construct an empty capability.

`execution` is required. `Immediate` retains this contract's non-resumable
local lifetime; `PlainGenerator`, `AsyncFunction`, and `AsyncGenerator` carry
distinct activation-backed capabilities from their extension contracts. There
is no absent/default execution plan.

The node does not create a Declarative Environment Record. The surrounding
Script, Block, or function-body instantiation already created every lexical
binding and keeps the node's `body` in that same environment. The node owns
only the DisposeCapability and its completion boundary.

When `using` declarations are interleaved with ordinary statements, lowering
nests suffix scopes at each declaration point. For example:

```text
s0; using a = x; s1; using b = y; s2;

Block [
  s0,
  SyncDisposableScope([a = x], Block [
    s1,
    SyncDisposableScope([b = y], Block [s2])
  ])
]
```

This structural form preserves declaration reachability without a numeric
statement index. It also makes reverse disposal and completion folding compose:
the inner resource is disposed first, and its resulting completion is the
incoming completion of the outer scope.

## Entry algorithm and binding order

For each `SyncDisposableResourceIr` in declaration order, the backend performs
the `BindingEvaluation` and `AddDisposableResource` sequence exactly once:

1. evaluate `initializer` to `value`;
2. if `value` is `null` or `undefined`, append no resource record;
3. otherwise require an Object value;
4. perform `GetMethod(value, @@dispose)` exactly once, propagating getter and
   callability failures;
5. require a present method and only then append the fully initialized
   `{ [[ResourceValue]]: value, [[DisposeMethod]]: method }` record; and
6. initialize the immutable lexical `binding_name` with `value`.

Registration therefore occurs after validation and method acquisition, while
binding initialization occurs after successful registration. If validation or
`GetMethod` throws, the current binding remains uninitialized, the current
entry was never published, and previously registered entries are still
disposed. A nullish value initializes its binding without registering an
entry.

The producer must not emit a separate `StatementIr::Lexical` for one of these
bindings: doing so would make it possible to initialize before acquisition or
twice. The resource entry is the sole runtime initialization owner. Lowering
still transitions its compile-time binding metadata to initialized after it
has lowered the initializer, so subsequent source expressions resolve with the
ordinary post-declaration facts.

## Completion and disposal

The backend enters `body` only after all entries of the node have completed
registration. It captures every completion that leaves `body`, disposes the
registered records in strict reverse order, and then resumes with the result of
`DisposeResources(stack, completion)`. This includes normal, throw, return,
break, and continue completions.

A throwing disposer does not stop the walk. Each later disposal error is
folded as `SuppressedError(error, suppressed)` in the order required by
`DisposeResources`; after the stack is empty, the backend restores the final
completion. The resource stack and completion storage are backend-private
temporaries. They are not generic `finally` locals and are not exposed as
user-addressable IR bindings.

## Producer and consumer obligations

The lowering producer is responsible for:

- rejecting every out-of-scope form listed above;
- retaining initializer and declarator source order;
- allocating the exact lexical storage name once;
- emitting a non-empty scope node only at a reached declaration point; and
- nesting the remaining statement-list suffix inside that node.

Every exhaustive `StatementIr` consumer must name `SyncDisposableScope`.
Traversal consumers visit each resource initializer in order and then the body.
The Wasm planner roots the initializer dependencies, `Symbol.dispose`,
`TypeError`, and `SuppressedError`; control-flow emission owns the dedicated
completion capture/fold/restore path. No consumer may translate the node back
into `TryFinally`.

## Verification boundary

Durable `lila-ir` tests pin the structural contract without executing Wasm:

- one declaration produces one dedicated node and no generic `TryFinally`;
- multiple declarators retain source order;
- ordinary statements before a declaration stay outside its node;
- statements and later declarations become the nested suffix body; and
- the resource entry, rather than a `Lexical` statement, owns initialization.

Central batch verification owns compilation and runtime Test262 execution.
The focused ladder is:

```sh
cargo fmt --all -- --check
cargo check -p lila-ir
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir synchronous_using --quiet
```

The integrated current-SHA checkpoint is green: `cargo xc`, all three focused
IR tests, all four bounded structure tests and the end-to-end CLI fixture pass.
The exact 18-file non-dynamic lifecycle cohort is 36/36 under Wasm-AOT. This is
focused evidence only; it does not claim the complete 78-file
`language/statements/using` directory or the full pinned aggregate.
