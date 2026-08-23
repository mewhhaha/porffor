# TypedArray reverse-family buffer witness

Status: normative for the Wasm-AOT `%TypedArray%.prototype.reverse` and
`%TypedArray%.prototype.toReversed` method-entry seam.

## Specification boundary

Both reverse-family methods first validate their TypedArray receiver and
capture the element length derived from that validation. A detached backing
buffer or an out-of-bounds fixed view therefore throws before either method
starts its reversal algorithm. A length-tracking view uses the backing-store
length observed by that validation, while a fixed view retains its stored
extent across an in-bounds grow.

After validation, neither method evaluates a JavaScript callback or coerces a
user argument. `reverse` swaps values inside the original receiver over the
captured range and returns that receiver. `toReversed` selects the receiver's
intrinsic element kind, allocates a distinct same-kind TypedArray with the
captured length, copies values in reverse order and returns the new object. The
entry migration must not change those algorithms, their element-kind handling
or their indexed read/write order.

The previous emitters reconstructed the TypedArray private view slots inside
each method, called `emit_validate_typed_array_current_byte_length`, and divided
the resulting byte length independently. That raw validator reports buffer
failures through the entry-global error path and leaves the two methods outside
the live buffer-witness invariant used by migrated TypedArray consumers.

## Closed projection

Each compiler performs its receiver-brand check before reading private view
state. It then loads exactly one `TypedArrayViewLocals` record through
`emit_load_typed_array_private_state` and consumes exactly one live witness with
the closed projection:

```rust
TypedArrayWitnessUse::ValidatedMethodEntry { length_local }
```

The witness owns the complete method-entry buffer observation:

1. it reads the backing data pointer and backing byte length once;
2. it distinguishes detached, fixed out-of-bounds and tracking
   out-of-bounds states without mutating the stored fixed extent;
3. it throws both buffer failures through the executing builtin's Realm;
4. it floors a tracking view's available bytes to whole elements; and
5. it publishes the validated element length from that same observation.

The method compilers consume that element length directly. They may not call
the legacy raw validator, reconstruct the four view slots independently or
derive the entry length by dividing a byte-length local. `toReversed` retains
its separate element-kind load because element kind is not a live buffer
observation and remains required for same-kind result allocation.

## Durable regression

A bounded source-structure regression isolates each reverse-family compiler.
For both bodies it requires one private-state load, one immutable view record,
one live witness and one `ValidatedMethodEntry` projection. It rejects the
legacy validator, direct reconstruction of the four private view slots,
entry-global TypeError emission and a parallel division by the bytes-per-element
local. It also pins `toReversed`'s one element-kind load so the buffer migration
cannot silently change result-type selection.

The existing CLI fixtures cover odd and even lengths, numeric, BigInt and
floating element kinds, original-versus-new result identity, internal rather
than public `length`, detached buffers, fixed-view out-of-bounds behavior,
fixed-view in-bounds growth and length-tracking growth and shrinkage. They
remain the focused runtime witnesses because the algorithms and fixture
surface do not change in this migration.

## Verification evidence

The centralized eight-core lane passed `cargo xc`, the focused structure test
(`1/1`) and the existing exact `reverse` and `toReversed` CLI fixtures (`1/1`
each). The pinned `reverse/resizable-buffer.js` and
`toReversed/reverses.js` Test262 leaves each pass `2/2` Wasm-AOT executions
with Unsupported, Crash and Bug all at zero.

## Nonclaims

This seam does not migrate `copyWithin`, `sort`, `toSorted`, `with`, `set`,
`slice` or any other remaining raw TypedArray validator. It does not change the
shared indexed `Get`, the per-index read/write helpers, integer-indexed exotic
semantics, result allocation, SharedArrayBuffer synchronization, Test262
materialization or published conformance counts. The current fixtures do not
prove created-Realm error-prototype identity at runtime; the source structure
only proves that invalid buffer states route through the shared
current-function-Realm witness. T17 remains in progress.
