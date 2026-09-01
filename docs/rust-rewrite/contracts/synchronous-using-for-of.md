# Synchronous `using` in `for-of` heads

Status: normative for non-resumable synchronous `using` in a `for-of` head.

## Scope and evidence

This contract covers an ordinary Script or ordinary function body whose loop
has the form `for (using x of iterable) body`. The resource head must contain
exactly one BindingIdentifier. Labelled and unlabelled loops are in scope.
Pattern-looking source such as `for (using[resource] of iterable) body` is an
ordinary element-access assignment head under the grammar, not a resource
binding pattern. `await using`, `for-await-of`, generators, async functions,
async generators, modules, `for-in`, and dynamic source are not.

The exact current-pin positive witnesses are:

- `language/statements/for-of/head-using-bound-names-fordecl-tdz.js`;
- `language/statements/for-of/head-using-fresh-binding-per-iteration.js`;
- `language/statements/using/syntax/using-invalid-assignment-statement-body-for-of.js`.

None uses dynamic source. With no `flags` entry, each has the ordinary sloppy
and strict Script variants: three files and six executions.

At commit `681ca415ba1e74c220fa8a5982cba1e7adedc151`, a focused `lila
inspect` of each file reaches `unsupported-features-recorded` with
`unsupported in lila wasm-aot first slice: for-of initializer`. This is current
source evidence, not an inference from an older published snapshot: the
rejection is the unmatched `IterableLoopInitializer::Using` arm in
`ScriptLowerer::lower_for_of_head`. Adjacent invalid-head files are already
parser errors and are boundary evidence, not claimed conformance delta.

## Closed head domain

The two direct for-of statement shapes do not share a loose `(mode, name)`
pair. Their heads use this closed split:

```rust
ForOfAssignmentIr { mode: BindingMode, name: String }

ForOfIteratorHeadIr::Assignment {
    binding: ForOfAssignmentIr,
    async_plan: Option<AsyncForOfIteratorPlanIr>,
    protocol: IteratorProtocolWitness,
}
ForOfIteratorHeadIr::SyncDisposable(SyncDisposableForOfHeadIr)
```

`StatementIr::ForOfIterator` accepts the exhaustive `ForOfIteratorHeadIr`, and
all direct synchronous Array and String heads now use that generic statement.
`SyncDisposableForOfHeadIr` has one private
`binding_name` field, one crate-private constructor, and a read-only accessor.
It contains no mode, vector, pattern, initializer, disposal-kind Boolean, or
async plan. The only producer therefore means exactly one immutable synchronous
resource binding.

This split is load-bearing. The IR has no direct synchronous Array or String
walk variant on which a future lowerer could place a resource head. A backend
consumer of the generic iterator must exhaustively choose ordinary assignment
or synchronous acquisition/disposal. The ordinary variant
owns both its optional async plan and its protocol witness. The synchronous
variant owns neither, so pairing it with an async plan or async protocol is not
constructible. Its sole lowering path supplies
`IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL` to `ForOfLoweringIr`.
`for-await-of` remains a different unsupported source form rather than an
optional flag on the resource head.

Because the synchronous resource head shares `compile_for_of_iterator` with
ordinary assignment heads, its source `@@iterator` and cached `next` methods
now use general `IsCallable` and Proxy-aware `Call`. Callable Proxies receive
the original iterable or iterator with no arguments. Their apply-trap and
revoked-Proxy completions propagate before resource acquisition and do not
enter IteratorClose. The source-kind-independent runtime witness uses an
ordinary assignment head; the bounded owner guard also covers the synchronous
resource-head route.

## Environment and head evaluation

`using` is Const-like for lexical analysis, TDZ, capture aliases and materialized
Environment Records:

1. the head binding exists uninitialized while `iterable` is evaluated;
2. every entered iteration creates a fresh immutable binding;
3. the iterator's `nextValue` is validated and registered as the iteration's
   synchronous resource; and
4. only successful registration initializes that iteration's binding.

`null` and `undefined` initialize the binding without registering a resource.
Every other value must be an Object. `GetMethod(value, @@dispose)` runs exactly
once; a missing or non-callable method throws. The acquired record is published
before InitializeBinding, so a getter failure leaves the binding uninitialized.

`ForInOfEnvironmentIr` remains the compiler's materialization witness. Its TDZ
and iteration records include the `using` binding exactly as they include a
`const` binding, including the storage alias used by a closure in the body.
`None` means the records were proven safe to elide, not that the language-level
fresh-binding obligation disappeared.

Evaluation of the iterable is abrupt-completion preserving at every expression
boundary. In particular, both ordinary Wasm consumers of `ExprIr::Comma` route
a throwing left operand before evaluating the right operand. This keeps a TDZ
read such as `(x, iterable)` observable rather than replacing its
`ReferenceError` with the right operand's later behavior.

The resource value is the iterator's `nextValue`; it is not represented as a
synthetic initializer expression or a one-entry `SyncDisposableScope`. The
head entry is the sole runtime InitializeBinding owner.

## Per-iteration completion order

Each entered iteration owns a fresh one-entry DisposeCapability. After body
evaluation, the backend disposes that entry before deciding LoopContinues and
before any IteratorClose:

1. capture the body's normal, continue, break, return, or throw completion;
2. dispose the registered value with the acquired receiver and method;
3. fold a disposer failure with an existing throw using the existing
   `SuppressedError(error, suppressed)` rule;
4. leave the iteration Environment Record;
5. apply LoopContinues to the folded completion; and
6. if it does not continue this loop, perform IteratorClose with that folded
   completion.

An unlabelled continue or a continue targeting this loop therefore disposes the
current resource and advances without closing the iterator. A continue targeting
an outer loop, break, return, body throw, acquisition failure, or disposer throw
is abrupt for this loop and reaches IteratorClose. Disposal happens first, so a
disposer error is the completion supplied to IteratorClose. Direct synchronous
Array and String iteration have no specialized walk statement, keeping this
ordering on the generic protocol path.

The enclosing label still targets the direct `ForOfIterator` statement. No
synthetic outer Block or body-only disposal scope may take ownership of its
break or continue target.

## Producer and verification obligations

Lowering and analysis must:

- recognize `IterableLoopInitializer::Using(Binding::Identifier(_))` only and
  leave pattern-looking element access in the ordinary assignment domain;
- reject `for-await-of` or a resumable owner before constructing the closed
  head;
- lower the iterable in the using binding's TDZ scope;
- publish the fresh iteration binding and capture alias as Const-like; and
- force the generic synchronous iterator protocol regardless of static
  iterable shape.

Durable IR tests pin the closed structure, current diagnostic boundaries, TDZ
alias, and fresh captured iteration storage. Central verification owns Cargo,
Wasm execution and pinned Test262. The intended focused ladder is:

```sh
cargo fmt --all -- --check
cargo check -p lila-ir
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir synchronous_using_for_of --quiet
cargo test -p lila-aot-wasm --test synchronous_using_for_of_structure --quiet
cargo test -p lila-cli --test cli resource_management::wasm_using_for_of_lifecycle -- --exact
./target/debug/lila test262 run language/statements/for-of/head-using-bound-names-fordecl-tdz.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/statements/for-of/head-using-fresh-binding-per-iteration.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/statements/using/syntax/using-invalid-assignment-statement-body-for-of.js --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

The integrated current-SHA checkpoint is green: `cargo xc`, 3/3 focused IR
tests, 5/5 bounded structure tests and the end-to-end CLI lifecycle oracle pass.
The three exact files above report 6/6 sloppy/strict Wasm-AOT executions with
every failure bucket at zero. This remains focused evidence, not a claim about
the complete `language/statements/using` directory or the full pinned
aggregate.
