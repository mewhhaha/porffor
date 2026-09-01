# Array `toLocaleString` receiver kind

Status: implemented and structure-verified on 2026-08-27.

## Closed entry policy

The private `ToLocaleStringReceiverKind::{ArrayLike, TypedArray}` domain is the
complete compile-time policy shared by the two `toLocaleString` entry points.
Exactly two producers construct it: `Array.prototype.toLocaleString` chooses
`ArrayLike`, while `%TypedArray%.prototype.toLocaleString` chooses `TypedArray`.

The domain derives no clone, copy, debug, equality or default capability. Its
shared emitter borrows the value for three exhaustive decisions:

- the method name used by compiler diagnostics;
- the element-method-not-callable TypeError message; and
- whether entry performs strict TypedArray validation or the generic Array-like
  receiver path.

The last decision projects to a local branch condition only after both variants
are matched. No raw Boolean crosses the two producer boundaries, and adding a
variant requires all three policies to be stated before the crate builds. The
validator borrows the same non-copyable value before constructing its existing
validated invocation token.

## Source equivalence and evidence

This migration changes only Rust compile-time policy observation. It preserves
both receiver branches, messages, calls, locals and instruction order, so
emitted Wasm is expected to remain byte-identical.

`to_locale_string_receiver_kind_structure.rs` recursively pins the eleven
source mentions, exact private declaration, capability absence, two producers,
three exhaustive projections and their order. The existing invocation and
TypedArray buffer-witness structure targets remain the semantic owners of the
branch bodies and the validated element-call protocol.

```console
cargo test -p lila-aot-wasm --test to_locale_string_receiver_kind_structure
cargo test -p lila-aot-wasm --test to_locale_string_invocation_structure
cargo test -p lila-aot-wasm --test typed_array_to_locale_string_witness_structure
cargo test -p lila-cli --test cli array::run_wasm_backend_succeeds_for_supported_array_to_locale_string_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_succeeds_for_array_to_locale_string_invocation_fixture -- --exact --test-threads=1
```

The dedicated structure target passes `4/4`; both neighboring targets pass
`4/4` each, and the two exact CLI witnesses pass `2/2`. Independent review
confirmed the exact producer and projection tables, capability census,
neighboring guard ownership and preserved instruction order. The coordinated
`cargo xc`, full formatter, diff, module-boundary and task-plan checks are
green. Test262, semantic-golden and broad-suite verification remain deferred.

This invariant adds no Array, TypedArray, locale-formatting, resizable-buffer,
cross-Realm or broader conformance claim. The detailed behavior contracts remain
`array-to-locale-string-invocation.md` and
`typed-array-to-locale-string-buffer-witness.md`.
