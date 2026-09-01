# Map collection weak-key admission

Status: implemented and focused-verified, 2026-08-27.

## Scope

This contract owns the key-admission difference between `Map` and `WeakMap`
for `delete`, `get`, `getOrInsert`, `getOrInsertComputed`, `has` and `set`. It
does not own receiver validation, entry storage, key equality, tracing or weak
reachability.

## Semantic law

All six shared emitters validate the receiver before reading their key.
`Map` accepts every ECMAScript value as a key. For a key that cannot be held
weakly, `WeakMap.prototype.delete` and `WeakMap.prototype.has` return Boolean
`false`, while `WeakMap.prototype.get` returns `undefined`, before entry
lookup. `WeakMap.prototype.set`, `getOrInsert` and `getOrInsertComputed` create
their existing current-function-Realm TypeError before normalization or entry
lookup.

The two insertion forms retain their distinct observable order. Direct
`getOrInsert` validates the weak key before loading its value argument.
`getOrInsertComputed` loads and validates the callback before validating the
weak key, and an invalid weak key never invokes that callback. `set` retains
both argument loads before weak-key validation.

## Rust invariant

The private `MapCollectionKind::{Map, WeakMap}` domain is the only policy input
to the six shared emitters. It has no equality capability. Each admission site
matches it exhaustively: the `Map` arm emits no admission instructions, while
the `WeakMap` arm emits the existing rejection or early-result sequence. A new
collection kind therefore cannot silently inherit Map's unrestricted policy.

`MapCollectionKind` remains `Copy` because its existing constructor, receiver,
layout, allocation and diagnostic projections consume it by value throughout
the collection backend. Its six established projections remain exhaustive,
and brand selection continues through `CollectionDataReceiverKind`.

The bounded structure regression pins the two variants and capabilities, all
six projections, fifteen producers, the recursive variant census, all six
admission matches, the exact early-result and TypeError bodies, and every
receiver, argument, callback, normalization and lookup ordering law above.

The complete get-or-insert value-source owner now lives in the private
`builtins/collections/map_get_or_insert.rs` child. Its two-variant domain, four
existing `pub(crate)` semantic entry points and sole raw parameterized emitter
moved together, so the collections parent cannot construct the value-source
policy or call the raw emitter. The four `standard.rs` product calls remain
byte-identical.

The exact five-line domain and 312-line method selection retain SHA-256
`b5db66b00f27f10e45c4b98a31220473b159564a3d292e1c9ac765a6a7ae3873`
and
`00a687c5a16c6f0c9c2ffeeeb21f714b31cc58b6dcf9d0539f5ea4a12a54acc7`;
their combined 317-line hash remains
`8666b1d64189818ecd0d108a521afdf4f0ccd9068be169436cc1c1697273d4e7`.
The resulting 6,491-line parent and 322-line child have SHA-256
`8d6c436a07bc388cf950cfaf35659d65f6de068101f2382f6e384b738c44ce9e`
and
`6022280ee176b5a20373e540763ec158a5c2914ce49fb2c8720c5f72df25d7d7`.
The four unchanged product calls have combined SHA-256
`3b45c5b84f4630f5d49ad4f75756989f010a9ff2ad9f72e4a1607b491f57d462`.
Recursive structure and module policies pin ten domain mentions, eight
qualified variants, the raw definition plus four calls, zero parent raw names,
and all four semantic methods and product calls.

## Verification and non-claims

The Batch AE shared checkpoint is green: `cargo xc` passes, and the exact
Map get-or-insert, Map collection and Set collection structure targets pass
`3/3`, `4/4` and `4/4`, respectively (`11/11` aggregate). The exact
`iterator::run_wasm_backend_preserves_map_get_or_insert_value_sources` CLI
owner passes `1/1`. The ten-file upsert cohort passes all `20/20`
sloppy/strict Wasm-AOT executions with every failure bucket at zero: Map
`getOrInsert` append-new-values; WeakMap `getOrInsert` adds-object-element;
paired Map/WeakMap `getOrInsertComputed`
does-not-evaluate-callbackfn-if-key-present,
overwrites-mutation-from-callbackfn and not-a-function-callbackfn-throws; and
WeakMap `getOrInsert` plus `getOrInsertComputed`
throw-if-key-cannot-be-held-weakly. No semantic golden was run for Batch AE.

The separately pinned six WeakMap invalid-key leaves paired with six Map
primitive-key controls retain their earlier `24/24` sloppy/strict Wasm-AOT
result with every failure bucket at zero. A separate non-cohort
computed-callback probe remains `0/2` at the explicit T13 `new Function()`
dynamic-source boundary; that result does not exercise key admission.

This change preserves emitted instructions for both existing variants. It does
not implement weak or ephemeron storage, make unreachable WeakMap keys
collectible, change SameValueZero, or claim the full pinned Map/WeakMap trees.
