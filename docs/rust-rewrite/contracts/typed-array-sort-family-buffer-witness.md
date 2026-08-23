# TypedArray sort-family buffer witness

Status: normative for the Wasm-AOT `%TypedArray%.prototype.sort` and
`%TypedArray%.prototype.toSorted` method-entry seam. The implementation and
structural mutation guard are independently reviewed and focused-verified under
the shared eight-core cap, 2026-08-23.

## Specification boundary

Both sort-family methods first read `comparefn`. `undefined` selects the
default TypedArray ordering; every other value must be callable. This check
precedes receiver validation, so a non-callable comparator throws before an
invalid, detached or out-of-bounds receiver is observed.

After that check, each method validates its TypedArray receiver and captures
the element length produced by that validation. A detached backing buffer or
an out-of-bounds view therefore throws before the sorting algorithm
begins. A length-tracking view uses the backing-store length observed by the
validation, while a fixed view retains its stored extent across an in-bounds
grow.

`sort` stably sorts the captured range in the original receiver and returns
that receiver. `toSorted` chooses the receiver's intrinsic element kind,
allocates a distinct same-kind TypedArray with the captured length, copies the
source elements into it before invoking the comparator, stably sorts that copy
and returns the new object. The entry migration must not change comparator
coercion, default numeric and BigInt ordering, stability, indexed read/write
behavior or abrupt-completion routing.

The previous emitters reconstructed the TypedArray private view slots inside
each method, called `emit_validate_typed_array_current_byte_length`, and divided
the resulting byte length independently. That raw validator reports buffer
failures through the entry-global error path and leaves both methods outside
the live buffer-witness invariant used by migrated TypedArray consumers.

## Closed projection

Each compiler performs comparator admissibility first. It then completes its
receiver-brand check before reading any TypedArray private state, loads exactly
one private view through `emit_load_typed_array_private_state`, constructs
exactly one `TypedArrayViewLocals` value and consumes exactly one live witness
with the closed projection:

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
the legacy raw validator, reconstruct the four private view slots independently
or derive the entry length by dividing a byte-length local.

Each compiler retains exactly one separate element-kind load. Element kind is
not a live buffer observation: `sort` needs it for the shared default ordering,
while `toSorted` also needs it to select the intrinsic constructor for its
result. It must not be folded into `TypedArrayViewLocals` or treated as part of
the witness lifecycle.

## Algorithm preservation

The migration changes only the method-entry validation seam.

- `sort` continues to call `emit_typed_array_stable_sort` exactly once with the
  receiver as its target and the witness-produced length, then publishes the
  receiver payload and tag as the result.
- `toSorted` continues to select a constructor from the separately loaded
  element kind, construct a same-kind result at the witness-produced length,
  copy the captured source range into that distinct result, call
  `emit_typed_array_stable_sort` exactly once on the result only after the copy
  is complete, and publish that result.
- The shared `emit_typed_array_stable_sort` implementation is outside this
  seam and remains unchanged.

In particular, comparator callability remains before the brand check, the
brand check remains before private-state use, and `toSorted` performs all
source reads before the comparator body can mutate or resize the source.

## Durable regression

A bounded source-structure regression isolates each sort-family compiler. For
both bodies it requires exactly one private-state load, one
`TypedArrayViewLocals` construction, one live witness, one
`ValidatedMethodEntry` projection, one separate element-kind load and one call
to the shared stable-sort emitter. Exact normalized sentinels pin the order
from comparator admissibility through the completed brand guard, private-state
load, view construction and witness consumption.

The guard rejects the legacy validator, direct reconstruction of the four
private view slots, entry-global TypeError emission and a parallel
byte-length/bytes-per-element division. It also pins the algorithm boundaries:
`sort` targets and returns its receiver; `toSorted` selects and constructs the
same intrinsic element kind, copies source elements into the result before
sorting, sorts the result rather than the receiver and returns that distinct
result. These checks are intentionally structural mutation guards for the
compiler contract, not a substitute for runtime evidence.

The existing exact CLI fixtures remain the focused runtime witnesses. Together
they cover default and custom ordering, stable equal groups, comparator-result
coercion, floating-point and BigInt ordering, SharedArrayBuffer receivers,
captured lengths across resize, detached and out-of-bounds entry states,
abrupt comparators, invalid comparators and invalid receivers. They additionally
cover `sort` receiver identity and in-place mutation, and `toSorted` source
immutability, same-kind distinct allocation and copy-before-comparator timing.
Their internal-length controls now separately assert that an own ordinary
`length` property remains `50` while checking the six integer-indexed elements;
the methods must use the witness length without rewriting or mistaking the
public shadow for their internal length.

## Focused verification

The centralized capped lane completed the promised evidence:

- `cargo xc` passed for the workspace;
- `typed_array_sort_family_witness_structure` passed `1/1`;
- the exact `sort` and `toSorted` CLI fixtures each passed `1/1`; and
- pinned `sort/return-abrupt-from-this-out-of-bounds.js` and
  `toSorted/length-property-ignored.js` each passed `2/2` Wasm-AOT executions
  with every non-success bucket at zero under `--jobs 1 --threads 1`.

The first `toSorted` CLI attempt exposed a contradictory fixture assertion: it
defined an own `length` value of `50` and then used a generic helper that
required `.length === 6`. Independent review found no witness, result-identity
or temp-local ownership defect; the allocator's exact LIFO assertions already
prevent helper temporaries from aliasing live carriers. After separating the
public-length assertion from indexed-element checks in both sibling fixtures,
both exact tests passed. No aggregate or published conformance count was
refreshed.

## Nonclaims

This seam does not change the shared stable-sort algorithm, comparator-result
coercion, default TypedArray comparison, indexed access helpers, result
allocation, integer-indexed exotic semantics or SharedArrayBuffer
synchronization. It does not migrate `copyWithin`, `with`, `set`, `slice` or any
other remaining raw TypedArray validator. It does not alter Test262
materialization or published conformance counts, and it does not by itself
prove created-Realm error-prototype identity at runtime. T17 remains in
progress.
