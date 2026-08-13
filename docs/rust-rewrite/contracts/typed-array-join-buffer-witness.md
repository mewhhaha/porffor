# TypedArray `join` buffer witness

Status: normative for the Wasm-AOT `%TypedArray%.prototype.join` method-entry
seam.

## Specification boundary

`%TypedArray%.prototype.join` first performs `ValidateTypedArray` and captures
the resulting element length. Only then does it coerce the separator and walk
that initially captured index range. A detached backing buffer or an
out-of-bounds view therefore throws before separator coercion, while a resize
during separator coercion does not change the number of indices visited.
Subsequent integer-indexed reads remain live: indices made unavailable by that
resize contribute empty fields rather than shortening the captured range.

The older Wasm emitter reconstructed the TypedArray private slots inside the
`join` compiler, called `emit_validate_typed_array_current_byte_length`, and
divided the resulting byte length itself. That legacy validator creates its
detached and out-of-bounds `TypeError` through the entry-global error path.
Borrowing a created Realm's `join` therefore could produce the entry Realm's
error even though the executing builtin carries the created Realm's snapshot.
It also left `join` outside the buffer-witness invariant already used by other
validated TypedArray methods.

## Closed projection

The `join` compiler performs its receiver-brand check before reading private
slots, then loads one `TypedArrayViewLocals` record through
`emit_load_typed_array_private_state`. It passes that record to the sole live
buffer witness with:

```rust
TypedArrayWitnessUse::ValidatedMethodEntry { length_local }
```

The witness owns all method-entry buffer semantics:

1. it observes the backing data pointer and byte length once;
2. it distinguishes detached, fixed out-of-bounds and tracking
   out-of-bounds states without changing the stored fixed extent;
3. it throws both buffer failures through the executing builtin's Realm; and
4. it derives and publishes a whole-element length from the same observation.

The `join` compiler consumes that element length directly. It may not call the
legacy raw validator, read TypedArray view offsets independently, or divide a
byte length itself. Separator coercion and the existing live indexed reads stay
after the method-entry witness, preserving the specification's ordering.

## Durable regression

The bounded AOT structure regression requires exactly one private-state load,
one immutable view record, one live witness, and the
`ValidatedMethodEntry` projection inside the `join` compiler. It rejects the
legacy validator, direct private-slot constants, entry-global error emission,
and a second local element-length derivation.

The CLI fixture covers ordinary and BigInt joining, detached and fixed-view
out-of-bounds entry, length-tracking resize behavior, Uint16 whole-element
flooring after an odd-byte resize, and shrinkage during separator coercion. Its
Realm matrix invokes a created Realm's `join` on both created-Realm and
entry-Realm receivers, proving that the executing builtin rather than the
receiver chooses the thrown `TypeError.prototype`.

## Deferred verification

While the low-memory current-pin baseline owns Cargo and Test262, this seam is
verified with scoped formatting, JavaScript syntax, source-structure and diff
checks plus independent read-only review. The centralized ladder later runs
the focused AOT structure test, the CLI fixture, the complete pinned
`built-ins/TypedArray/prototype/join` leaf, and the current-SHA binary-data
matrix.

## Nonclaims

This seam does not migrate the remaining raw TypedArray validators, complete
the universal integer-indexed exotic protocol, change the shared indexed
`Get`, retire a Test262 harness rewrite, add SharedArrayBuffer synchronization,
or establish a new conformance count. It closes one product-reachable
method-entry invariant and its created-Realm error identity; T17 remains in
progress.
