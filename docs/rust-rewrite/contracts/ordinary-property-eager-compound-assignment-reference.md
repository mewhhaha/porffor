# Ordinary property eager compound-assignment Reference

Status: normative implementation contract for eager arithmetic and bitwise
compound assignment through an ordinary property Reference.

## Exact conformance boundary

The selected Test262 cohort is the complete legacy A7 ordering matrix under
`language/expressions/compound-assignment`:

- `S11.13.2_A7.1_T{1,2,3,4}.js` through
  `S11.13.2_A7.11_T{1,2,3,4}.js`.

Those 44 physical files execute once in each strictness mode, for 88 matrix
executions. At clean head `ae1bd994b`, the exact fresh baseline is 22/88:

- every T1, T2 and T4 file is `Bug:Runtime`, 66 failing executions; and
- every T3 file is green, 22 control executions.

The files are raw vendored sources. No source rewrite, matrix mask, or
interpreter exemption owns the result.

The eleven source operators cover multiplication, division, remainder,
addition, subtraction, left shift, signed and unsigned right shift, bitwise
AND, XOR and OR. Exponentiation has no file in this legacy matrix, but it is a
member of the existing closed `EagerCompoundAssignmentOp` domain and is pinned
by the focused producer witness rather than counted as Test262 progress.

## Normative Reference lifecycle

For `base[key] op= rhs`, the compiler must preserve one ordinary property
Reference through this exact order:

1. evaluate `base` and propagate an abrupt completion;
2. evaluate the raw computed-key expression and propagate an abrupt
   completion;
3. begin GetValue on the Reference: reject a nullish base before observing
   `ToPropertyKey`;
4. apply `ToPropertyKey` exactly once, then perform `[[Get]]` with `base` as
   both target and receiver;
5. evaluate `rhs` only after GetValue completes normally;
6. apply the selected eager arithmetic or bitwise operation to the old value
   and the RHS;
7. perform PutValue through the same base, canonical property key and receiver;
8. route a `[[Set]]` result of false according to the Reference's captured
   `[[Strict]]`; and
9. publish the applied value only after PutValue completes normally.

Steps 2 and 3 are deliberately separate. The computed-key *expression* is
evaluated even when the base is nullish, but coercing that value into a
property key belongs to GetValue and must not precede the nullish-base error.
Once coercion succeeds, the resulting canonical key is the identity used by
both `[[Get]]` and `[[Set]]`; the raw key value is never coerced again.

## Closed producer shape

`OrdinaryPropertyReferencePlan` is private, non-`Clone`, non-`Copy`, and
`#[must_use]`. It owns exactly one lowered base-and-receiver expression, one
raw `PropertyKeyIr`, and the Reference's `Strictness`. Its constructor is the
only ordinary-property mutation producer. The plan is now shared with the
distinct numeric-update lifecycle specified by
`ordinary-property-numeric-update-reference.md`; the eager operation below
remains the only compound-assignment consumer.

The eager lifecycle has one consuming operation:

```text
eager_compound_assignment(old_value_binding, EagerCompoundAssignmentOp, rhs)
    -> TypedExpr
```

The operation itself mints the old-value read from `old_value_binding` and
calls the closed operation's `apply` method. A caller cannot substitute a
different left operand, ignore the old value, or add logical assignment to the
eager domain.

The resulting expression is
`ExprIr::OrdinaryPropertyEagerCompoundAssignment` carrying the public,
private-field `OrdinaryPropertyEagerCompoundAssignmentIr`. Its frozen backend
accessors are:

- `base_and_receiver() -> &TypedExpr`;
- `referenced_name() -> &PropertyKeyIr`;
- `strictness() -> Strictness`;
- `old_value_binding() -> &str`; and
- `result() -> &TypedExpr`.

The carrier is `Clone` only because `ExprIr` remains cloneable. It exposes no
public constructor. The non-cloneable producer plan remains the ownership
boundary which makes a second source Reference unavailable.

All eager arithmetic and bitwise property-access AST arms intercept before the
generic `ReferenceRecord::read` / `ReferenceRecord::write` decomposition. The
old public-field `ExprIr::PropertyCompoundAssign` specialization is removed;
otherwise a new operator or access shape could silently choose the decomposed
path while the fused path compiled unused.

## Closed backend staging

The backend consumes the fused carrier through a private typestate sequence.
Exact private names may follow the backend module's local conventions, but the
roles are required:

1. evaluated base plus raw computed-key value;
2. one read Reference owning the canonical key and old value; and
3. one ready-to-write Reference owning that same canonical key and the applied
   result.

The first transition owns the nullish check and sole `ToPropertyKey`. It
consumes the raw-key role, so no later state can compile a second coercion. The
read-to-write transition owns RHS/result emission. The final transition
consumes the canonical Reference and emits `[[Set]]` plus strict-false routing.
No public helper accepts a bare key local for the write half.

This is the load-bearing compile-enforced seam: adding a new eager operation
must satisfy the exhaustive `EagerCompoundAssignmentOp` application and every
new `ExprIr` consumer, while attempting to write from a raw key or to coerce a
read key again has no typed API.

## Focused producer invariant

The durable `lila-ir` witness lowers computed ordinary-property `+=`, `*=`,
`^=` and `**=` expressions and proves that each becomes exactly one fused IR
carrier. It pins the source-order base and raw-key operands, the captured
strictness, the carrier-minted old-value binding inside the result, and the
absence of independent `PropertyRead` / `PropertyWrite` nodes from the fused
subtree.

The backend fixture additionally observes:

- key-expression abrupt completion before nullish-base rejection;
- nullish-base rejection before `ToPropertyKey` side effects;
- one successful key coercion across both get and set;
- RHS after GetValue;
- the same receiver and canonical key for get and set; and
- strict versus sloppy handling of a false `[[Set]]` result.

## Explicit nonclaims

This batch does not change logical assignment, prefix/postfix numeric update,
plain assignment, destructuring assignment, optional chains, Super or private
References, identifier/global/Object Environment References, `with`, or any
resumable/suspended property Reference. Those lifecycles remain on their
existing dedicated IR paths.

Static property names may use the same carrier, but no conformance claim is
derived from them. Array/TypedArray fast paths remain backend implementation
choices only when they preserve the same Reference ordering and strictness.
The batch does not change dynamic source generation or the interpreter policy.

## Verification

Producer/static stage:

```sh
cargo fmt --all -- --check
cargo test -p lila-ir ordinary_property_eager_compound_assignment
git diff --check
./scripts/check-module-boundaries.sh
```

Integrated exact stage:

```sh
./target/debug/lila test262 run \
  language/expressions/compound-assignment/S11.13.2_A7 \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --snapshot-name ordinary-property-eager-compound \
  --timeout-ms 180000 --threads 1
```

The published result must state all three numbers: 44 physical files, 88
executions, and the 22 T3 control executions. A green operator subset is not a
claim that the complete A7 matrix passed.
