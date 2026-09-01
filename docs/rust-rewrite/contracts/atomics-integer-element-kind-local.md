# Atomics integer element-kind local

Status: invariant implemented and dry-reviewed for the T17 Wasm-AOT Atomics
integer-operation boundary.

## Semantic boundary

The Atomics backend stores a TypedArray element kind as a runtime word in a
Wasm local. That wire value remains necessary, but the shared normalization,
load, store, read-modify-write and compare-exchange emitters must not accept an
arbitrary `u32` local index. They accept only a borrowed
`ValidatedAtomicsIntegerElementKindLocal`.

Each of the three owners reserves a
`PendingAtomicsIntegerElementKindLocal`: `Atomics.wait`, `Atomics.waitAsync`,
and the shared nine-operation load/store/RMW compiler. The sole validation
boundary consumes that pending local, loads the private TypedArray element-kind
word, applies an exhaustive
`AtomicsIntegerElementKindRequirement::{AnyInteger, Waitable}` policy, and emits
the existing current-function-Realm `TypeError` path. Only after that branch is
closed does it mint the validated authority.

The owners borrow the move-only validated authority across all repeated
element-kind projections. Their final local-release boundary consumes it with
`into_local`; code after that point cannot reuse the authority. The wrapper is
private and implements no capability traits. Its tuple field is not exposed to
other backend modules.

This retains the existing runtime domains:

- the shared integer methods admit Int8, Uint8, Int16, Uint16, Int32, Uint32,
  BigInt64 and BigUint64 TypedArrays;
- `wait` and `waitAsync` admit only Int32 and BigInt64 TypedArrays;
- the raw element-kind wire values, atomic instruction selection, coercion
  order, diagnostics and result publication are unchanged.

`Atomics.notify` keeps its separate Int32/BigInt64 validation and does not use
the shared normalize/load/store/RMW helpers, so it is outside this authority.

## Durable guard

`atomics_integer_element_kind_local_structure` pins the private capability-free
pending and validated declarations, the sole mint, both exhaustive requirement
rows, all three owner lifecycles, the exact producer/consumer census and the
seven shared emitter signatures. It rejects a return to
`element_kind_local: u32` at any shared consumer boundary or a direct private
slot load in an owner.

The neighboring `atomics_integer_operation_structure` fingerprint now requires
borrowing this authority throughout core operation and result publication. The
shared `atomics_typed_array_witness_structure` guard distinguishes notify's
direct validation from the other three owners' typed validation handoff and
keeps the complete current-Realm TypeError route census.

This is source-equivalent compiler hardening. It claims no new Atomics
semantics, real-agent evidence, Test262 progress, published-count change or T17
closure. Standalone source-only execution passes `4/4` for the new structure
guard, `3/3` for the updated integer-operation fingerprint and `5/5` for the
updated TypedArray-witness guard. Workspace compilation and runtime verification
remain owned by the shared batch checkpoint.
