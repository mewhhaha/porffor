# Sync iterator protocol-error authority

The private `SyncIteratorProtocolError` is the one-shot semantic authority for
the four failures shared by synchronous iterator consumers: a non-iterable
input, a non-object iterator-method result, a non-callable `next`, and a
non-object `next` result. It has no derived or manual `Clone`, `Copy`,
comparison, formatting, default, conversion, or representation capability.

Exactly seventeen typed projector calls deliver this authority directly to
`emit_sync_iterator_protocol_type_error`. Five live in the shared acquisition
and stepping helpers. The ordinary direct and plain-async `await using`
`for-of` owners each contribute the same five checks, and the custom Array
destructuring step contributes its non-callable-`next` and primitive-result
checks. The sole consumer takes the error by value and exhaustively combines
it with `SyncIteratorConsumer` in sixteen rows. Those rows give Array
destructuring, ArrayAccumulation spread, `for-of`, and `Math.sumPrecise`
distinct diagnostics. Every row uses the current-function-Realm TypeError
emitter only after the body-Realm projection identifies a standard builtin
with a trusted self-backed environment. Main, user, host, and runtime-helper
bodies use the main-Realm emitter. The exhaustive body-source match prevents a
lexical environment from being read as Realm metadata. A second consuming
observation of the error authority fails to compile.

The focused structure guard lexically excludes comments and literals,
canonicalizes raw identifiers, and pins the private attribute-free
declaration, all variant routes, the five shared helper checks in order, the
two custom destructuring checks in order, and the complete consumer body. Its
confirmed census is 35 identifiers: the declaration, typed projector
parameter, 17 producers, and 16 mapping rows. The direct Realm guard separately
pins the two inline-owner censuses. Adding an error kind or consumer requires
updating every diagnostic row, while throw propagation and instruction order
remain independently pinned.

The ordinary direct callable-Proxy follow-up changes neither this census nor
the diagnostic projection. A non-callable Proxy method still selects the
appropriate typed error. A callable Proxy passes the general callability gate
and enters Proxy `[[Call]]`; its apply-trap or revocation completion bypasses
this algorithm-error authority and propagates unchanged. The same witness
holds 13 initialized captured bindings so a user lexical environment cannot
accidentally satisfy the function-layout Realm load.

Focused verification:

```sh
cargo test -p lila-aot-wasm --test sync_iterator_protocol_error_ownership_structure -- --test-threads=1
cargo test -p lila-aot-wasm --test sync_iterator_consumer_capability_structure -- --test-threads=1
cargo test -p lila-aot-wasm --test math_sum_precise_runtime_structure -- --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_preserves_array_destructuring_iterator_abrupt_completions -- --exact
cargo test -p lila-cli --test cli array::run_wasm_backend_preserves_array_accumulation_iterator_errors -- --exact
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_math_sum_precise_runtime -- --exact
```

Central verification for the seventeen-producer boundary passed in the
four-consumer batch: nine affected structure targets passed `42/42`, seven
exact Wasm-AOT CLI witnesses passed `7/7`, and the pinned Array-spread and
Array-destructuring cohort passed `18/18`. The callable-Proxy and body-Realm
follow-up reran the five directly affected structure targets at `23/23`, five
entry- and created-Realm CLI controls at `5/5`, and eight unchanged direct
iterator/Proxy leaves at `16/16`, with every failure bucket zero. Neither
checkpoint makes a broader iterator-conformance or `Math.sumPrecise`
conformance claim. See
[`sync-iterator-consumer-capability.md`](./sync-iterator-consumer-capability.md).
