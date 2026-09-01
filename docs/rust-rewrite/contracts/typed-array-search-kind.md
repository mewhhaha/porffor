# TypedArray search-kind projection

Status: capability hardening implemented, reviewed and focused-verified for the
Wasm-AOT `%TypedArray%.prototype.includes`, `indexOf` and `lastIndexOf`
compiler family on 2026-08-28.

## Closed domain

`TypedArraySearchKind` has exactly three inhabitants:

- `Includes`;
- `IndexOf`; and
- `LastIndexOf`.

The three public compiler wrappers each construct exactly their matching kind,
and the `StandardBuiltinId` dispatcher maps each public builtin to its matching
wrapper. No other producer exists.

The capability-free `TypedArraySearchKind` deliberately implements no `Clone`,
`Copy`, `Debug`, `Default`, comparison, ordering or hashing capability. The
shared compiler owns the one produced authority and borrows it through every
semantic projection. Search policy may not be copied, cloned or projected
through `==`, `!=`, a Boolean or an `is_*` method. Adding a fourth search kind
therefore requires reviewing all search semantics before the compiler builds.

## Exhaustive semantic projections

The shared compiler contains exactly twelve `match &search_kind` projections:

1. method name;
2. incompatible-receiver diagnostic;
3. absent `fromIndex` tag;
4. initial result value and tag;
5. `fromIndex` normalization;
6. loop termination;
7. invalid integer-index guard opening;
8. SameValueZero or strict equality;
9. matching-result payload;
10. matching-result branch depth;
11. invalid integer-index guard close; and
12. cursor advance.

The intended shared policies are closed pairings rather than defaults:

| Pair | Shared projections |
| --- | --- |
| `Includes | IndexOf` | undefined absent `fromIndex`, forward normalization, forward loop bound, increment |
| `IndexOf | LastIndexOf` | `-1` Number result, invalid-`undefined` suppression, strict equality, numeric-index success, matching branch frame |

`LastIndexOf` alone owns the dynamic absent-argument sentinel, reverse
normalization, negative loop exit and decrement. `Includes` alone owns a
Boolean result, SameValueZero, visibility of `undefined` after a later invalid
integer index and the shallower success branch.

This projection cleanup preserves the existing method-entry
`ValidatedMethodEntry` witness and the one fresh
`IntegerIndexedProperty` witness used for each visited index. It changes no
receiver validation, length snapshot, `fromIndex` coercion, read, equality or
result behavior.

## Producer and consumer census

The source boundary contains:

- three public dispatcher producers;
- three wrapper-to-kind producers, exactly one for each inhabitant;
- twelve exhaustive semantic consumers;
- four `Includes | IndexOf` forward-policy pairings;
- six `IndexOf | LastIndexOf` index-result pairings; and
- zero equality, inequality or Boolean kind projections.

The producer-wrapper range remains
`e958795ce75a03e5ae44c0aae873180e1c6f709545e884e744efbf1ad1531bb5`,
and the `StandardBuiltinId` mapping range remains
`2f3e6ff0aeb5df6e64559916af10d70d37eba54b88fd42e40aace08c44800823`.
The source-only borrowed-projection compiler body is
`7c066b04c175e1e7cf20a459e9273bf3192324b410cf52a18381b90bb499a5bd`.

Before this closure, three consumers were exhaustive and nine used six `==`
and three `!=` decisions. A new kind could be added to the three existing
matches yet silently inherit IndexOf-like defaults from the remaining
comparisons. Removing equality from the type makes that partial migration a
compile error instead.

## Durable regression

`crates/lila-aot-wasm/tests/typed_array_search_kind_structure.rs` bounds the
shared compiler through the start of `compile_array_prototype_at_builtin`. It
pins:

- the exact three-variant declaration and absence of clone, copy, equality,
  default, ordering, hashing and debug capabilities;
- exactly twelve borrowed exhaustive projections and no by-value, clone,
  equality or `is_*` escape hatch;
- the exact three wrapper-to-kind producers;
- the exact three read-only `StandardBuiltinId` mappings;
- the four forward-search and six index-result pairings;
- the distinct SameValueZero and strict-equality consumers;
- the success-branch depths and forward/reverse advances; and
- one entry witness and one live integer-index witness.

These are source-structure mutation guards. They supplement rather than
replace behavioral execution.

## Focused evidence

The existing `wasm_typedarray_search.js` fixture, registered by
`run_wasm_backend_succeeds_for_typedarray_search_fixture`, distinguishes all
three kinds. It covers NaN SameValueZero versus strict equality, Boolean versus
numeric results, default and explicit-undefined `lastIndexOf`, forward and
reverse `fromIndex` handling, infinities, numeric and BigInt elements,
method-entry failures, snapshot growth, shrink and detachment after
`fromIndex`, invalid-index `undefined`, odd-byte flooring and fixed-view
regrowth.

The strengthened structure target passed `3/3`, the exact CLI fixture passed
`1/1`, and the shared `cargo xc` gate was green at the 2026-08-28 Batch V
checkpoint.

The earlier projection closure's shared `cargo xc` checkpoint passed. Its
semantic golden passed `2/2` in 697.36 seconds and contained 671 dumps; all 669
dumps retained from its baseline were equal after accounting normalization.
Batch V preserves the producer and mapping ranges and changes only the Rust
borrowing form of the twelve exhaustive projections.

Adjacent direct Test262 controls are:

- `built-ins/TypedArray/prototype/includes/samevaluezero.js`;
- `built-ins/TypedArray/prototype/indexOf/resizable-buffer-special-float-values.js`;
  and
- `built-ins/TypedArray/prototype/lastIndexOf/fromIndex-infinity.js`.

All six sloppy/strict Wasm-AOT executions passed with every failure bucket at
zero.

No fixture or conformance inventory entry is added by this source-only closure.
Borrowing the exhaustive matches changes neither their arms nor the emitted
instruction sequence and reserves no new locals. The shared checkpoint retains
that source-equivalent claim against compilation and behavioral controls.

## Nonclaims

This boundary does not change generic Array search methods, TypedArray witness
semantics, integer-indexed exotic behavior, numeric conversion,
`ToIntegerOrInfinity`, equality helpers, Realm selection, Test262
materialization or published conformance counts. It does not close T17.
