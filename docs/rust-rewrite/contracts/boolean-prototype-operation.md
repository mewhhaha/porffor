# Boolean prototype operation domain

Status: implemented, independently reviewed and focused-verified, 2026-08-27.

## Scope

This contract owns the result difference between
`Boolean.prototype.toString` and `Boolean.prototype.valueOf` in the Wasm-AOT
builtin emitter. It does not own Boolean construction, truthiness conversion,
boxing layout or intrinsic publication.

## Semantic law

Both prototype methods accept a Boolean primitive or a Boolean wrapper object.
Every other receiver creates the existing TypeError from the active builtin
function's Realm. Receiver validation and extraction of the Boolean payload
must complete before result materialization.

`valueOf` returns that payload with the Boolean tag. `toString` maps zero to
the existing `"false"` payload and nonzero to `"true"`, then returns the String
tag.

## Rust invariant

The private, non-derived `BooleanPrototypeOperation::{ToString, ValueOf}`
domain is the only input to the shared prototype emitter. The two outer
`BooleanBuiltin` variants forward their named operation explicitly. After the
unchanged shared receiver validation, one exhaustive match owns both complete
result instruction sequences. There is no equality, Boolean flag, wildcard,
default or unreachable fallback through which a future operation could inherit
one method's result policy.

`BooleanBuiltin` also derives no capabilities. Its constructor and two
prototype variants remain explicitly dispatched, and the three standard
builtin producers remain unchanged.

## Verification and non-claims

The bounded structure target passes `4/4`, and the exact boxed-builtin CLI
owner passes `1/1`. The four current-pin `S15.6.4.2_A1_T1.js`,
`S15.6.4.2_A2_T1.js`, `S15.6.4.3_A1_T1.js` and `S15.6.4.3_A2_T1.js` leaves each
pass both sloppy and strict Wasm-AOT variants for `8/8` aggregate. Every
reported parser, early-error, lowering, runtime, Wasm-backend, host-harness,
unsupported, not-implemented, crash and bug bucket is zero. Independent dry
review is clean.

The change moves source-equivalent emitter instructions behind a typed helper;
it adds no Wasm control flow and preserves local allocation/release order,
receiver diagnostics and static string payloads. It does not claim the full
Boolean tree, broader T24 closure or a published conformance-count change.

## Batch AS dispatcher boundary

The raw outer choice is now a private `BooleanBuiltin`, and the raw emitter is
private to `boolean.rs`. Standard dispatch can reach it only through three
fixed Boolean entries for construction, prototype `toString` and prototype
`valueOf`; it can neither construct nor pass the raw family authority. The
frozen 58-line domain/emitter selection has SHA-256
`48961edd05a7a1789538b92ad90ed76232fad5156cec5144214122dd4c52eaab`.
Restoring only the former enum and emitter visibility reproduces that source
exactly. At the 2026-08-28 Batch AS checkpoint, `cargo xc` is green, the
strengthened structure target passes `4/4`, the exact boxed-builtin CLI owner
passes `1/1`, and the four selected leaves pass all `8/8` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. This source-equivalent
boundary claims no new Boolean behavior, broader conformance or published
conformance-count change.
