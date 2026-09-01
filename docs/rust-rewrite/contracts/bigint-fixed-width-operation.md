# BigInt fixed-width operation domain

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

## Closed policy

`BigInt.asIntN` and `BigInt.asUintN` share argument evaluation, `ToIndex`,
`ToBigInt`, truncation storage, and result materialization, but they do not
share signed interpretation. The standard-builtin dispatcher maps the two
entry points explicitly into
`BigIntBuiltin::FixedWidth(BigIntFixedWidthOperation::{Signed, Unsigned})`.
The operation domain derives no capabilities and is borrowed by every policy
decision, so neither equality nor a Boolean can erase which abstract operation
is being emitted.

Four exhaustive matches own the complete difference:

- signed interpretation below 64 bits;
- unsigned heap materialization for the high half of the 64-bit range;
- the nonnegative-input restriction on the wide unsigned passthrough; and
- signed interpretation of the retained high bit for widths above 64 bits.

Every match has both named arms and no wildcard, default, assertion, or
unreachable fallback. Adding an operation therefore requires defining all four
semantic choices.

## Preserved ordering and representation

The shared body still evaluates and converts `bits` before converting the
BigInt argument. Width zero still returns zero only after both conversions.
The sub-64, exactly-64, and wider paths retain their emitted instruction order,
Wasm block nesting, immediate-versus-heap representation decisions, and local
release order. This boundary changes which Rust values can reach the emitter;
it does not change emitted JavaScript semantics.

## Durable evidence

`bigint_fixed_width_operation_domain_structure.rs` pins the two-row domain,
its lack of capabilities, both exact standard producers, the one typed
consumer, all four borrowed two-arm decisions, and the absence of the former
broad-builtin equality policy.

The existing `wasm_bigint_as_n_arbitrary_width.js` fixture remains the runtime
witness. It covers both operations at widths 0, 1, 63, 64, 65, 200, and 201;
positive and negative inputs; immediate and heap results; left-to-right
conversion; and abrupt completion before the second conversion. The adjacent
Test262 witnesses are the `arithmetic.js` and `order-of-steps.js` leaves under
both `built-ins/BigInt/asIntN` and `built-ins/BigInt/asUintN`.

At the 2026-08-27 checkpoint, the bounded structure target passes `5/5`, the
existing arbitrary-width CLI regression passes `1/1`, and the four exact
Test262 leaves each pass both sloppy and strict Wasm-AOT variants for `8/8`
aggregate. Every reported parser, early-error, lowering, runtime, Wasm-backend,
host-harness, unsupported, not-implemented, crash and bug bucket is zero.
`cargo xc`, `cargo fmt --all -- --check` and `git diff --check` are green for
the coordinated batch.

## Scope

This contract does not change BigInt arithmetic, parsing, prototype methods,
ECMA-402 formatting, or conformance counts. It adds no fixture or source
rewrite and makes no full BigInt or T20 closure claim.

Batch AX additionally makes the fixed-width operation and its outer builtin
domain private to `bigint.rs`; `BigInt.asIntN` and `BigInt.asUintN` now enter
through their separately named fixed entries. The complete six-entry BigInt
dispatcher boundary and its exact source witness are recorded in the prototype
result-policy contract. This source-equivalent visibility hardening claims no
new BigInt behavior. The Batch AX shared compile and five BigInt structure
targets are green, and the exact arbitrary-width Wasm-AOT control passes `1/1`.
