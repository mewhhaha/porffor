# Array iterator receiver policy

Status: normative for Lila's Array and TypedArray iterator-method entry points.

## Boundary

Six standard builtins share one iterator receiver algorithm while selecting one
of two receiver protocols:

| Policy | Producers | Receiver behavior |
| --- | --- | --- |
| `GenericArrayLike` | `Array.prototype.keys`, `entries` and `values` | accepts generic array-like receivers and selects the TypedArray iterator record only when the runtime receiver is a TypedArray |
| `TypedArray` | `%TypedArray%.prototype.keys`, `entries` and `values` | requires a validated TypedArray and creates its iterator record directly |

The private, capability-free `ArrayIteratorReceiverPolicy` is selected
explicitly by all six builtins. It implements no clone, copy, debug, default,
comparison, ordering or hashing capability. One private emitter owns the policy
and borrows it through two direct exhaustive matches: first for strict
TypedArray validation, then for iterator materialization. There is no copied,
equality, Boolean, default, wildcard or unreachable projection. Adding another
receiver protocol therefore requires an explicit validation and materialization
decision, while the current decision cannot be duplicated into independently
transposable authorities.

This policy exists only while Rust emits a standard builtin. It adds no Wasm or
heap word and does not alter the stable iterator-kind word ABI, iterator record
layout, temporary-local lifetime, validation order or emitted materialization
sequence. The word authority is specified separately in
`array-iterator-kind-wire-domain.md`.

## Durable witnesses

`array_iterator_receiver_policy_structure.rs` pins the exact two-variant
capability-free domain, the 12-name and 3/3 producer census and both borrowed
exhaustive projections. It also pins the validation-before-materialization
order, the existing two TypedArray/one ordinary iterator creation sites, and
normalized source hashes for the compiler and producer block.

The compiler body, normalized only by removing the two borrow tokens, retains
raw SHA-256
`4a216f733e6662fc93633fa1c26a2e71317fc3cc1c7bafeac60af38b315e601f`.
The byte-identical six-producer block retains
`125b6fb9bf2123a2612ab5e530b339e8af1456cacfd362e69b4ad66956f2b77a`.
The borrow-only source edit changes no emitted Wasm instructions or order.

`wasm_array_iterator_receiver_policy.js` executes all six method producers. It
covers borrowed Array methods on an ordinary array-like, the generic-policy
runtime TypedArray specialization, native TypedArray methods, terminal iterator
results and strict TypedArray rejection of an ordinary array-like.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test array_iterator_receiver_policy_structure
cargo test -p lila-cli --test cli array::run_wasm_backend_preserves_array_iterator_receiver_policies -- --exact --test-threads=1
rustfmt --edition 2021 --check crates/lila-aot-wasm/src/builtins/standard.rs crates/lila-aot-wasm/tests/array_iterator_receiver_policy_structure.rs crates/lila-cli/tests/cli/array.rs
git diff --check
```

At the shared Batch AE checkpoint, `cargo xc` is green, the bounded structure
target passes `3/3`, and the exact module-qualified CLI witness passes `1/1`.
The exact `built-ins/Array/prototype/keys/returns-iterator-from-object.js`,
`built-ins/Array/prototype/entries/resizable-buffer.js` and
`built-ins/TypedArray/prototype/keys/this-is-not-typedarray-instance.js` leaves
each pass both sloppy and strict Wasm-AOT execution, for `6/6` with every
failure bucket at zero.

The shared checkpoint repaired only the structure guard's producer census slice
so it includes the first `ArrayPrototypeKeys` row. Product source, its six
producers and both frozen hashes are unchanged. No semantic golden was run for
Batch AE.

## Raw Test262 retirement follow-up

On 2026-08-29, the self-contained rewrite for
`built-ins/Array/prototype/entries/resizable-buffer.js` and the four keys/entries
grow- and shrink-mid-iteration rewrites were deleted. The entries base case also
lost a second path-specific transform that had replaced its destructuring loop
and `Array.from` calls. Real-source materialization units now require all five
vendored bodies to survive byte-for-byte and pin the declared full assertion,
compare-array and resizable ArrayBuffer helper provenance. The resizable helper
retains the separately owned T13 static replacement of its dynamic subclass
construction.

The base entries case passes both sloppy and strict Wasm-AOT execution (`2/2`),
and the four mid-iteration cases pass all eight executions (`8/8`). They cover
the upstream constructor fan-out and fixed-length, fixed-offset,
length-tracking and offset-tracking view matrices. These raw results supersede
the earlier Batch AE entries result, which ran behind a self-contained rewrite;
they do not change the receiver-policy implementation or its frozen hashes.

The earlier workspace semantic golden passed `2/2` in 702.89 seconds with 667
dumps. It predates capability hardening, adds only this witness, removes none,
and preserves every retained non-accounting summary except the independently
expanded Promise callback witness; it is historical evidence, not a Batch AE
golden run.

## Deferrals

This source-equivalent type closure does not change iterator kind encoding,
`IteratorResult` materialization, resizable or detached buffer behavior, Realm
selection, Proxy semantics, generator iterators, Test262 shortcuts, broad
Array/TypedArray execution or conformance status publication.
