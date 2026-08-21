# Synchronous `using` in classic `for` heads

Status: normative for non-resumable synchronous `using` in a classic
`ForStatement` initializer.

## Scope

This contract covers an ordinary Script or ordinary function body whose
classic loop has the form `for (using x = value, ...; test; update) body`.
It covers labelled and unlabelled loops. It does not cover `await using`,
generators, async functions, async generators, modules, `for-in`/`for-of`,
Switch CaseBlocks, or dynamic `eval`.

The two pinned Test262 witnesses are:

- `language/statements/using/initializer-disposed-at-end-of-forstatement.js`;
- `language/statements/using/initializer-disposed-if-subsequent-initializer-throws-in-forstatement-head.js`.

Neither uses dynamic source evaluation. With no `flags` entry, each has the
ordinary sloppy and strict Script variants: two files and four executions.

## Closed initializer capability

The producer extends the existing closed classic-loop initializer domain:

```rust
ForInitIr::SyncDisposable(SyncDisposableResourcesIr)
```

`SyncDisposableResourcesIr` is the existing private-field, non-empty,
declaration-ordered resource carrier. A synchronous using head therefore
cannot be represented by an ordinary `LexicalBlock`, a loose Boolean on
`StatementIr::For`, or an empty resource vector. Exhaustive `ForInitIr`
consumers must decide this lifecycle when the variant is added.

The containing node remains `StatementIr::For`. In particular, lowering must
not hide it inside a synthetic outer `Block`: the backend's labelled-statement
dispatcher recognizes the direct `For` node and supplies both its break and
continue targets. Keeping the node direct preserves `continue label` while the
new initializer variant carries the separate disposal obligation.

## Environment and initialization order

All head names are created as immutable, uninitialized bindings before any
initializer is lowered. Initializers then remain in source order. For each
entry, the backend performs the existing synchronous resource sequence:

1. evaluate the initializer;
2. accept a nullish value without registration;
3. otherwise validate Object, acquire `@@dispose` exactly once and validate it;
4. publish the complete resource record; and
5. initialize the immutable binding.

If a later initializer throws, every earlier registered resource is disposed
and the current binding remains uninitialized. The resource entry is the sole
runtime binding initializer; no parallel `ForInitIr::Lexical` is emitted.

The existing `ForLexicalEnvironmentIr` remains the owner of a materialized
for-head Environment Record. Its `bindings` are retained unchanged. A `using`
head contributes no `per_iteration_slots`: unlike `let`, its immutable binding
is not copied by CreatePerIterationEnvironment. The backend must enter that
environment before resource acquisition and leave it only after disposal.

## Loop completion boundary

The DisposeCapability is active before the first initializer and encloses the
test, body and update. It is consumed once when the loop exits normally or by
`break`, `return`, or throw, including an abrupt initializer, test, body, or
update. An in-loop `continue`, labelled or unlabelled, does not dispose: it
targets the loop's continue frame inside the capability. Disposal is LIFO and
uses the existing synchronous completion fold, including `SuppressedError`
ordering and restoration of the resulting completion.

The for-head lexical environment stays current during disposal and is restored
afterward. Entering the environment inside the disposal boundary would make
the initializers run outside their binding environment; leaving it before the
fold would run disposer calls in the wrong environment.

## Evidence boundary and verification

The checked-in old `codex-published-real-language_statements_using` artifact is
an obsolete-pin `spec-exec` result, not Wasm-AOT evidence. The current-pin
matrix cache records discovery topology only. The pre-batch evidence for this
lane was therefore source proof: `lower_for_loop` and
`lower_for_lexical_init` explicitly rejected `using`, while the preceding
synchronous-using contract explicitly excluded classic heads. The
implementation below does not turn that baseline into runtime evidence by
itself. The current-SHA checkpoint ran the focused commands below: `cargo xc`,
4/4 IR tests, 5/5 structure tests and the CLI oracle are green, and the five
selected vendored files report 10/10 sloppy/strict Wasm-AOT executions.

```sh
cargo fmt --all -- --check
cargo check -p lila-ir
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir synchronous_using_classic_for --quiet
cargo test -p lila-aot-wasm --test synchronous_using_classic_for_structure --quiet
cargo test -p lila-cli --test cli resource_management::wasm_using_classic_for_lifecycle -- --exact
./target/debug/lila test262 run language/statements/using/syntax/using-for-statement.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/statements/using/syntax/using-invalid-assignment-next-expression-for.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/statements/using/syntax/using-outer-inner-using-bindings.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/statements/using/initializer-disposed-at-end-of-forstatement.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/statements/using/initializer-disposed-if-subsequent-initializer-throws-in-forstatement-head.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

The batch does not claim the complete 78-file `language/statements/using`
directory or alter its published status.
