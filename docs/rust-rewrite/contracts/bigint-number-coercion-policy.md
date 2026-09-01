# BigInt Number coercion policy

Status: normative for the Wasm-AOT value-to-BigInt Number-admission seam. The
implementation and bounded structural guard are independently reviewed and
focused-verified under the shared eight-core cap, 2026-08-23.

## Specification boundary

[ECMA-262 `ToBigInt`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-tobigint)
first applies `ToPrimitive(argument, number)`. Its primitive conversion table
then rejects a Number with a `TypeError`; Number is deliberately not an
implicitly accepted BigInt source.

[ECMA-262 `BigInt(value)`](https://tc39.es/ecma262/2026/multipage/numbers-and-dates.html#sec-bigint-constructor-number-value)
has a narrower explicit exception. After the same `ToPrimitive(value, number)`
step, a Number primitive is sent to `NumberToBigInt`, while every other
primitive is sent to `ToBigInt`. `NumberToBigInt` accepts an integral Number,
throws a `RangeError` for a non-integral Number (including non-finite values),
and returns the corresponding BigInt otherwise.

The distinction therefore belongs at the Number branch after primitive
conversion. It is not a general permissive mode for `ToBigInt`, and it must not
change how Boolean, BigInt, String, Symbol, null or undefined are handled.

## Closed Rust policy

The crate-visible value helper and private primitive helper carry a
crate-private closed policy rather than a Boolean whose meaning is recoverable
only from call-site position:

```rust
pub(crate) enum BigIntNumberPolicy {
    RejectNumber,
    NumberToBigInt,
}
```

The policy derives no cloning, copying, equality or debug capability. Each
outer consumer constructs one named policy and moves it into the value helper;
that helper moves the same value through its sole forwarding edge, and the
private primitive helper consumes it in the exhaustive Number-branch match.
There is no reason to duplicate or compare the authority before that final
projection.

`emit_value_to_bigint_locals` accepts the policy, performs exactly one
number-hinted `ToPrimitive`, preserves its abrupt-completion route, and forwards
the policy unchanged to `emit_primitive_to_bigint_locals`. The primitive helper
projects the policy only inside its Number-tag branch with an exhaustive
two-arm `match` and no catch-all:

- `RejectNumber` emits the existing `TypeError` completion;
- `NumberToBigInt` emits the existing integral check, `RangeError` completion
  for rejected Numbers and conversion for accepted Numbers.

The match is an emitter-time Rust decision, not a runtime Wasm Boolean branch.
Adding a third policy must make this projection fail to compile until its
Number semantics are chosen. Non-Number branches neither inspect nor
reinterpret the policy.

The migration must preserve observable order. Object conversion and any user
code reached by `ToPrimitive` happen before the primitive type is inspected;
an abrupt completion from that conversion returns unchanged before either
Number policy can run. Number conversion errors continue through the same
current-function completion path. The policy migration must not duplicate,
delay or bypass `ToPrimitive`, and must not add a second completion probe at a
caller.

## Current caller ownership

The current source has exactly seven callers outside the two helpers. Six own
ordinary `ToBigInt` semantics and must select `RejectNumber`:

- the value-only and local-pair projections of `SpecOperationIr::ToBigInt` in
  `operations.rs`;
- the value-plus-low-word conversion used by BigInt typed-data storage in
  `objects.rs`;
- the `Temporal.ZonedDateTime` epoch-nanoseconds constructor conversion in
  `builtins/temporal.rs`;
- the `Temporal.Instant` constructor conversion in `builtins/temporal.rs`; and
- `Temporal.Instant.fromEpochNanoseconds` in
  `builtins/temporal_instant.rs`.

Only the `%BigInt%` function path in `builtins/bigint.rs` may select
`NumberToBigInt`. There is exactly one internal call from
`emit_value_to_bigint_locals` to the private
`emit_primitive_to_bigint_locals`; sibling modules cannot bypass the value
helper merely to gain Number admission.

This inventory is intentionally exact. A new consumer must choose one of the
two named specification policies and update both this contract and its
structural witness.

## Durable mutation guard

A focused source-structure regression must pin the closed policy rather than
only observe a few successful values. It must require:

- exactly the two variants above and an exhaustive Number-branch match without
  `_` or an `if` over a Boolean;
- absence of `allow_number` from both helper signatures and bodies;
- one unchanged policy forwarding site from the value helper to the private
  primitive helper, with no other primitive-helper caller;
- exactly seven external value-helper callers: six `RejectNumber` projections
  and one `NumberToBigInt` projection; and
- the `%BigInt%` function path as the sole `NumberToBigInt` owner.

The witness must fail if the two policies are swapped at any inventoried site,
if a caller reintroduces a literal Boolean, if a new bypass or caller appears,
if an incidental capability is added, or if the exhaustive match is weakened.
It should also pin that number-hinted `ToPrimitive` precedes the single policy
forwarding call so a mechanical signature migration cannot silently change
evaluation or completion order.

## Focused verification

Independent review accepted the implementation and tightened the mutation
guard around helper privacy, exhaustive-arm count and method-item escapes. The
centralized capped lane then completed these checks:

- `cargo xc` passed for the workspace;
- `bigint_number_policy_structure` passed `2/2`, including the exact caller
  inventory and the closed Number-branch projection;
- the exact `wasm_bigint_minimal_validation.js` CLI test passed `1/1`, covering
  integral `BigInt(Number)` admission plus fractional, NaN and infinite
  `RangeError` rejection; and
- the exact `wasm_typedarray_prototype_with.js` CLI test passed `1/1`, including
  a Number rejected with `TypeError` by the BigInt typed-data write path.

These are focused ownership and runtime witnesses, not a broad BigInt,
Temporal or Test262 refresh. No aggregate or published count was changed.

## Nonclaims

This seam does not implement a new BigInt representation, arbitrary-precision
`NumberToBigInt` for the full finite Number range, or any missing large-value
conversion behavior. It does not change string parsing, boxed primitive
conversion, error-Realm selection, Temporal range validation, typed-array or
DataView modulo semantics, BigInt operators, or `ToNumeric`. It does not prove
the semantic ownership of future callers merely because they select a variant.
It does not establish full BigInt, Temporal, binary-data or Test262 conformance,
change published conformance counts, or complete T04 or T20.
