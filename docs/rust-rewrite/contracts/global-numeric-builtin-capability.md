# Global numeric builtin capability

Status: implemented and focused-verified, 2026-08-27.

## Scope

This contract owns the compile-time policy boundary between the coercing
global `isFinite` and `isNaN` builtins in the Wasm-AOT emitter. It does not own
the non-coercing `Number.isFinite` or `Number.isNaN` methods, numeric conversion
itself, global publication or Realm construction.

## Rust invariant

The private, non-derived `GlobalNumericBuiltin::{IsFinite, IsNaN}` domain is
consumed once by the family emitter. Two exact fixed producer entries select
their correspondingly named row, while standard dispatch cannot import,
construct or pass the raw domain. One exhaustive outer match admits both
rows to the unchanged shared argument coercion and result publication path;
one exhaustive inner match emits the `isFinite` infinity exclusions or the
empty `isNaN` continuation. No clone, copy, debug, equality, default, wildcard
or Boolean projection lets a future row inherit either result policy.

The common path still allocates and releases the same two locals in the same
order. It converts argument zero before testing the numeric payload, publishes
the same Boolean representation, and adds no Wasm instruction or runtime
branch.

## Verification and non-claims

The bounded recursive structure target passes `4/4`. The exact current-pin
`built-ins/isFinite/return-false-on-nan-or-infinities.js` and
`built-ins/isNaN/return-true-nan.js` leaves each pass both sloppy and strict
Wasm-AOT variants, for `4/4` aggregate with every failure bucket at zero.
Independent review is clean. The shared checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the
module-boundary check and the task-plan check.

This is a source-equivalent capability closure. It does not claim the full
`isFinite` or `isNaN` trees, broader T24 completion, a Wasm golden result or a
published conformance-count change.

Batch AR makes the raw domain and family emitter private to
`global_numeric.rs`; standard dispatch sees only fixed `isFinite` and `isNaN`
entries. The frozen 47-line domain/emitter body has SHA-256
`3057db4769633e0293b564bd3e61383677777bb91780936f92b2dd21fb80cda2`;
normalizing only the two narrowed visibilities reproduces that hash exactly. At
the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, and the exact pinned `isFinite` and `isNaN`
leaves pass all `4/4` sloppy/strict Wasm-AOT executions with every failure
bucket at zero. This source-equivalent boundary claims no new global numeric
behavior, broader conformance or published conformance-count change.
