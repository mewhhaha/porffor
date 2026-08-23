# TypedArray find-family buffer witness

Status: normative for the Wasm-AOT `%TypedArray%.prototype.find`,
`findIndex`, `findLast` and `findLastIndex` method-entry seam. The contract,
implementation and structural mutation guard are independently reviewed and
focused-verified under the shared eight-core cap, 2026-08-23.

## Specification boundary

The living ECMA-262 clauses for
[`find`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.find),
[`findIndex`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.findindex),
[`findLast`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.findlast)
and
[`findLastIndex`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.findlastindex),
and the corresponding 2026 clauses for
[`find`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.find),
[`findIndex`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.findindex),
[`findLast`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.findlast)
and
[`findLastIndex`](https://tc39.es/ecma262/2026/multipage/indexed-collections.html#sec-%typedarray%.prototype.findlastindex)
all begin with the same three-stage boundary:

1. create `taRecord` with `ValidateTypedArray(this, seq-cst)`;
2. derive `len` with `TypedArrayLength(taRecord)`; and
3. invoke `FindViaPredicate` with the selected ascending or descending direction.

The successful result is either the found value or its index. `find` and
`findIndex` walk in ascending order; `findLast` and `findLastIndex` walk in
descending order. A miss produces `undefined` for the value projections and
`-1` for the index projections.

Receiver validation and its element-length snapshot happen before
`FindViaPredicate` checks whether the predicate is callable. A detached backing
buffer or an out-of-bounds view therefore throws before a non-callable
predicate is observed. A length-tracking view uses the backing-store length
captured by that validation, while an in-bounds fixed view uses its stored
extent. Later predicate calls cannot grow or shrink the number of visited
indices.

The snapshot does not cache element values. Each visited index is read when
that iteration is reached, before calling the predicate with
`(value, index, receiver)`. Mutation is visible to a later read, while a detach
or resize that makes a later integer index invalid produces the current
integer-indexed result for that read. Predicate calls preserve the exact
`thisArg`, receiver identity and abrupt-completion routing.

The previous TypedArray entry reconstructed the four private view slots, called
`emit_validate_typed_array_current_byte_length`, and divided the resulting byte
length by bytes per element. That raw path leaves this four-method family
outside the live buffer-witness invariant used by migrated TypedArray consumers.

## Closed projection

`FindViaPredicateKind` remains the sole four-way compiler domain. Its exhaustive
direction and projection mappings continue to own all four method surfaces; the
buffer migration adds no boolean or parallel method dispatcher.

The shared TypedArray compiler completes its receiver-brand check before reading
private view state. It then loads exactly one immutable private view through
`emit_load_typed_array_private_state`, constructs exactly one
`TypedArrayViewLocals` value and consumes exactly one live witness with:

```rust
TypedArrayWitnessUse::ValidatedMethodEntry {
    length_local: len_local,
}
```

That single witness owns the complete `taRecord` and `TypedArrayLength`
observation for every `FindViaPredicateKind`:

1. it reads the backing data pointer and backing byte length once;
2. it distinguishes detached, fixed out-of-bounds and tracking
   out-of-bounds states without mutating the stored fixed extent;
3. it throws buffer failures through the executing builtin's Realm;
4. it floors a tracking view's available bytes to whole elements; and
5. it publishes `len_local` from the same cached observation.

The method compiler consumes that length directly. It may not call the legacy
raw validator, reconstruct any of the four private view slots independently,
observe backing-store state through a parallel helper or derive `len_local` by
dividing a byte-length local.

## Algorithm preservation

The migration changes only the TypedArray method-entry observation. The shared
predicate and iteration algorithm remains unchanged:

- predicate validation still follows the receiver witness and uses the private,
  ownership-consuming `ValidatedFindPredicateLocals` boundary;
- callable Proxies still reach the Proxy-aware call path;
- `thisArg`, element, numeric index and original receiver remain the three exact
  callback arguments and receiver;
- `emit_initialize_find_index` and `emit_advance_find_index` still consume the
  exhaustive direction projection;
- `emit_typed_array_or_object_index_read_from_locals` still performs each live
  later indexed read;
- predicate throws and indexed-read throws still propagate before truthiness or
  successful-result projection; and
- `emit_project_find_match` still exhaustively selects value versus index.

In particular, no second buffer witness replaces the per-index read path, no
callback can change the captured loop bound, and the four result policies do not
move into the buffer-validation seam.

## Durable regression

The existing bounded `find_via_predicate_structure` regression isolates the
TypedArray entry from the generic Array entry. Eight exact normalized dispatcher
sentinels bind every Array and TypedArray `StandardBuiltinId` to its matching
`FindViaPredicateKind`; per-kind occurrence totals alone are not an adequate
guard because they permit two method surfaces to exchange kinds. For the
TypedArray body the regression pins exactly one private-state load, one
`TypedArrayViewLocals` construction, one live witness and one
`ValidatedMethodEntry` projection. Exact normalized sentinels require the
completed brand guard to precede private-state loading, view construction and
witness consumption, and require that witness to precede predicate validation.
The fixed TypedArray receiver-brand error literal also has one separately
counted source owner.

The guard rejects the raw validator, entry-global TypeError construction, direct
loads of the four private view slots, direct backing-store observations, a
parallel byte-length division and any direct overwrite of the witness-produced
`len_local`. It retains the closed four-kind dispatcher checks and pins one
predicate validator, one live indexed-read boundary, one Proxy-aware predicate
consumer, one direction initializer, one direction advance and one
value-or-index projection. Normalized wiring sentinels additionally fix the
predicate consumer's `thisArg` and `(element, index, receiver)` argument vector,
both entry points' matching callback locals, and the TypedArray sequence from
live read through indexed-read propagation, callback propagation, truthiness,
successful projection and direction-aware advance. Swapping locals or merely
preserving helper counts therefore does not satisfy the guard. These are
source-structure mutation guards, not runtime evidence.

## Focused evidence

The existing exact CLI fixture is
`crates/lila-cli/tests/fixtures/wasm_typedarray_find.js`, registered by
`run_wasm_backend_succeeds_for_typedarray_find_fixture`. It already exercises
all four methods, forward and reverse order, value and index results, numeric
and BigInt element kinds, callable Proxies, exact callback arguments and
receiver identity, public private-slot spoofs, mutation visible to later reads,
snapshot lengths across grow and shrink, later `undefined` values after detach
or out-of-bounds shrink, invalid receivers, invalid predicates and abrupt
predicates.

The small exact current-pin Test262 cohort is:

- `built-ins/TypedArray/prototype/find/return-abrupt-from-this-out-of-bounds.js`;
  and
- `built-ins/TypedArray/prototype/findLastIndex/detached-buffer.js`.

The first fixes the out-of-bounds method-entry failure for an ascending value
projection. The second fixes detached-buffer failure through the same compiler
for a descending index projection. Together with the exact CLI fixture and the
closed four-kind structural dispatcher, they are the focused runtime checkpoint
for this seam.

The centralized capped lane passed `cargo xc`, the focused
`find_via_predicate_structure` regression (`4/4`) and the exact CLI fixture
(`1/1`). The two Test262 files were each run as an exact path with
`--jobs 1 --threads 1`; both pass `2/2` sloppy/strict Wasm-AOT executions, for
`4/4` total with Unsupported, Crash and Bug all at zero.

## Nonclaims

This seam does not change `FindViaPredicate`, the shared indexed-read helper,
integer-indexed exotic semantics, Proxy call semantics, callback truthiness,
SharedArrayBuffer synchronization or generic Array `find*` receiver
preparation. It does not migrate `every`, `some`, `toLocaleString`,
`copyWithin`, `with`, `set`, `slice`, `map`, `filter`, constructor validation or
any other remaining raw TypedArray validator. It removes no Test262
materializer, changes no published conformance count and does not by itself
prove created-Realm error-prototype identity at runtime. T17 remains in
progress.
