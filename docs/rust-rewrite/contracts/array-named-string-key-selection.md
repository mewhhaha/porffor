# Array named-string key selection

Status: capability hardening implemented, reviewed and focused-verified for
Lila's Array named-property storage emitters on 2026-08-28.

## Boundary

Arrays store indexed elements separately from named property descriptors. Two
emitters traverse the named storage when `Object.getOwnPropertyNames` and
`Object.keys` first count and then write their result keys:

| Selection | Producers | Named string keys |
| --- | --- | --- |
| `All` | the count and write phases of `Object.getOwnPropertyNames` | enumerable and non-enumerable |
| `EnumerableOnly` | the count and write phases of `Object.keys` | enumerable only |

The private, capability-free `ArrayNamedStringKeySelection` passes unchanged
into both consumers. It derives and implements no clone, copy, debug, equality,
ordering, hashing or default capability. Each consumer owns one selection and
borrows it through two exhaustive matches: one opens the enumerability guard
and one closes it. There is no Boolean projection, default or wildcard arm.
Adding a selection therefore requires an explicit decision in both count and
write phases, which keeps the allocated result length aligned with the emitted
keys. Collapsing either decision to equality no longer compiles.

This is Rust-time emitter state only. It adds no emitted Wasm word or ABI and
does not change property traversal, descriptor reads, temporary-local
reservation or emitted instruction order.

## Durable witnesses

`array_named_string_key_selection_structure.rs` pins the exact two-variant
capability-free domain, two owned consumers, four borrowed exhaustive
projections and the exact four-producer mapping. The consumer span is pinned at
`3fc0be9af14967687a701b61331996f8c929fc87ce99b67c6b131702d6b0c325`; the
unchanged `Object.getOwnPropertyNames` and `Object.keys` producer spans are
pinned at `e7fa80b3cdfa13ce4f453febff5ee47a9efc5ccaad3499f6d2235aa677b9151e` and
`37d5be7e6069a3349ac95ebfee64d034d67d51f20a49d30a20a2698b77441327`.

`wasm_array_named_string_key_selection.js` uses one sparse Array with an
indexed element, enumerable named properties, a non-enumerable accessor and a
Symbol property. It observes the exact `Object.getOwnPropertyNames` order
`["2", "length", "visibleFirst", "hidden", "visibleLast"]`, the exact
`Object.keys` order `["2", "visibleFirst", "visibleLast"]`, and proves that
neither operation invokes the accessor.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test array_named_string_key_selection_structure
cargo test -p lila-cli --test cli object::run_wasm_backend_preserves_array_named_string_key_selection -- --exact --test-threads=1
rustfmt --edition 2021 --check crates/lila-aot-wasm/src/builtins/array.rs crates/lila-aot-wasm/src/builtins/object.rs crates/lila-aot-wasm/tests/array_named_string_key_selection_structure.rs crates/lila-cli/tests/cli/object.rs
git diff --check
```

The strengthened structure target passed `3/3`, the exact module-qualified CLI
witness passed `1/1`, and the shared `cargo xc` gate was green at the
2026-08-28 Batch U checkpoint.

The following workspace semantic golden passes `2/2` in 704.11 seconds with
666 dumps. It adds only this witness, removes none and preserves all 665
retained non-accounting summaries.

Pinned adjacent Test262 controls are
`Object/getOwnPropertyNames/15.2.3.4-4-48.js`,
`Object/keys/15.2.3.14-5-12.js` and
`Object/keys/15.2.3.14-5-13.js`. All six sloppy/strict Wasm-AOT executions
passed with every failure bucket at zero.

## Deferrals

This source-equivalent type closure does not alter Proxy `[[OwnPropertyKeys]]`,
TypedArray integer-indexed keys, Arguments exotic keys, ordinary-object key
collection, Symbol collection, descriptor semantics, Realm allocation,
Test262 shortcut retirement, broad Object/Test262 execution or conformance
status publication.

Batch AP makes the raw selection and both exhaustive consumers private to
`builtins/array.rs`. Four fixed sibling operations expose only the `All` and
`EnumerableOnly` count/write semantics to the Object builtins, which can no
longer import, construct or pass `ArrayNamedStringKeySelection`. This is a
source-equivalent boundary tightening with no new Array or Object behavior.
The frozen five-line raw domain and 200-line raw-consumer selections have
SHA-256 `e6364d5bb57dc6ab69a73f418f3a69e918ecc5aabeb1dbae166933124a33c2d3`
and `6fe2c42ae5df23a983b0e418974e7fe6c51f20ede96b3165e786235334119acf`.
Batch AP verification is green on 2026-08-28: the strengthened structure target
passes `4/4`, the exact module-qualified CLI witness passes `1/1`, and
`cargo xc` is green.
