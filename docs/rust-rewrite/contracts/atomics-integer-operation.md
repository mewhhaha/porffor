# Atomics integer-operation authority

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

## Closed operation

`AtomicsIntegerOperation` is the private nine-row authority for `load`, `add`,
`and`, `compareExchange`, `exchange`, `or`, `store`, `sub` and `xor`. It derives
no cloning, copying, formatting, equality or default capability. Each builtin
wrapper constructs exactly one row and moves it into the shared compiler.

The shared compiler borrows the authority for five exhaustive policies:

1. value-argument arity;
2. the integer-TypedArray TypeError message;
3. the index RangeError message;
4. the load, store, compare-exchange or narrowed RMW emission; and
5. result publication.

No policy accepts a Boolean, integer, string, `StandardBuiltinId`, wildcard or
equality fallback.

## Result policy

`store` emits its write and publishes the converted input value in the core
operation match. Its dedicated result-policy arm therefore emits nothing. The
other eight operations publish the old element value through the unchanged
Number/BigInt projection after their core operation. Adding an operation now
requires an explicit result choice before the backend builds; it cannot inherit
the old-value path from `operation != Store`.

The six RMW-capable rows still narrow to the separate
`AtomicsRmwOperation::{Add, And, Exchange, Or, Sub, Xor}` domain. Load, store
and compare-exchange cannot enter an RMW opcode selector.

## Ordering and source equivalence

Argument loading, receiver validation, TypedArray witnessing, index and value
coercion, address calculation, atomic instruction selection, result
publication and reverse local release retain their existing order. The new
Rust matches are compile-time emitter decisions and add no emitted Wasm branch
or instruction. The old-value publication body is unchanged inside its exact
eight-row arm.

## Durable evidence

`atomics_integer_operation_structure.rs` pins the private non-capability
declaration, lexical recursive 48-identifier census, nine exact producer
routes, exact arity and diagnostic tables, and one contiguous address/core
operation/result/reverse-release region. Its dependency-free Rust scanner
excludes comments and every string/character literal form from ownership
counts. The neighboring created-Realm publication guard retains its independent
Atomics catalog boundary.

The retained add/load, store/load, RMW and compare-exchange CLI fixtures cover
all nine operation rows. Focused Number and BigInt add/store leaves plus
compare-exchange and xor leaves cover both result representations and the
narrowed operation paths.

At the 2026-08-27 focused checkpoint, the bounded structure target passes
`3/3`, and the four exact CLI witnesses pass `4/4`. The exact Number and BigInt
add/store leaves plus Number compare-exchange and xor leaves each pass both
sloppy and strict Wasm-AOT variants, for `12/12` aggregate under `--jobs 1
--threads 1`. Every reported parser, early-error, lowering, runtime,
Wasm-backend, host-harness, unsupported, not-implemented, crash and bug bucket
is zero.

Independent dry re-review is clean after the recursive ownership and
contiguous emission guards were hardened. The following shared workspace
checkpoint passes `cargo fmt --all -- --check`, `cargo xc`, the recursive
module-boundary check, the task-plan check and `git diff --check`.

## Scope

This boundary changes no Atomics behavior, host ABI, TypedArray witness,
atomic-memory instruction or weak/multi-agent claim. It does not establish
full Atomics conformance, complete T17 or a README status change.
