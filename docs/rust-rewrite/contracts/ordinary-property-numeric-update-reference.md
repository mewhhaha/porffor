# Ordinary property numeric-update Reference

Status: normative implementation contract for prefix and postfix `++` / `--`
through an ordinary property Reference.

## Exact conformance boundary

The selected Test262 cohort is four raw vendored files:

- `language/expressions/postfix-decrement/S11.3.2_A6_T1.js`;
- `language/expressions/postfix-increment/S11.3.1_A6_T1.js`;
- `language/expressions/prefix-decrement/S11.4.5_A6_T1.js`; and
- `language/expressions/prefix-increment/S11.4.4_A6_T1.js`.

None has an explicit flags list, so each runs once in sloppy mode and once in
strict mode: four physical files and eight executions. The fresh pre-batch
measurement was 0/8, with every execution reporting `Bug:Runtime` because a
computed key's throwing `toString` won over the required nullish-base
`TypeError`. The files have no runner rewrite, matrix mask, known-failure entry,
or interpreter exemption. Read-only inspection at head `0f004c0c6` confirms
that the same ordinary `ExprIr::PropertyUpdate` producer and eager
`compile_object_key_to_local` consumer remain present.

## Normative Reference lifecycle

For either prefix or postfix `base[key]++` / `base[key]--`, the compiler must
preserve one ordinary property Reference through this exact order:

1. evaluate `base` and propagate an abrupt completion;
2. evaluate the raw computed-key expression and propagate an abrupt
   completion;
3. begin GetValue on the Reference: reject a nullish base before observing
   `ToPropertyKey`;
4. apply `ToPropertyKey` exactly once, then perform `[[Get]]` with `base` as
   both target and receiver;
5. apply `ToNumeric` exactly once to obtain `oldValue`;
6. compute `newValue` by adding or subtracting the appropriate numeric unit;
7. perform PutValue through the same base, canonical property key and receiver;
8. route a `[[Set]]` result of false according to the Reference's captured
   `[[Strict]]`; and
9. only after PutValue completes normally, publish `newValue` for prefix or
   `oldValue` for postfix.

The computed-key expression in step 2 is evaluated even when `base` is nullish.
Its coercion in step 4 is not. Once coercion succeeds, the canonical key is
owned by the Reference and reused for both `[[Get]]` and `[[Set]]`; the raw key
is not evaluated or coerced again.

The update and result domains are closed and independent:

- `NumericUpdateOp::{Increment, Decrement}` chooses the numeric delta; and
- `UpdateReturnMode::{Prefix, Postfix}` chooses the value published after the
  successful write.

Every consumer must match both without a catch-all. This makes swapping prefix
and postfix publication, or adding a new operation/mode without handling it, a
compile-visible omission.

## Closed producer shape

The existing private, non-`Clone`, non-`Copy`, `#[must_use]`
`OrdinaryPropertyReferencePlan` is the sole producer. It already owns one
lowered base-and-receiver expression, one raw `PropertyKeyIr`, and the
Reference's `Strictness`. This batch adds a second consuming operation, distinct
from eager compound assignment:

```text
numeric_update(NumericUpdateOp, UpdateReturnMode) -> TypedExpr
```

The method consumes the plan and fixes the runtime value kind to `Dynamic`,
whose result codomain is exactly Number-or-BigInt. The ordinary property Get is
observable and supplies no sound static numeric-kind proof in this batch;
omitting a caller-provided `ValueKind` makes Object/String/other invalid update
kinds unrepresentable instead of routing them to `unreachable!`. A caller
cannot reuse the base/key tuple, lose the captured strictness, or represent a
logical assignment with this operation.

The resulting expression is `ExprIr::OrdinaryPropertyNumericUpdate`, carrying
the public, private-field `OrdinaryPropertyNumericUpdateIr`. Its frozen backend
accessors are:

- `base_and_receiver() -> &TypedExpr`;
- `referenced_name() -> &PropertyKeyIr`;
- `strictness() -> Strictness`;
- `op() -> NumericUpdateOp`;
- `return_mode() -> UpdateReturnMode`; and
- `value_kind() -> ValueKind`.

The carrier is `Clone` only because `ExprIr` remains cloneable. It has no public
constructor. `ExprIr::PropertyUpdate` is removed: leaving both shapes would
allow a new ordinary update arm to compile while bypassing the consuming plan.

`lower_property_access_update` intercepts
`PropertyAccess::Simple` before the generic decomposed property-read path. It
lowers the base and raw key in source order, determines only the conservative
numeric result kind, constructs the plan, then consumes it. Super and private
accesses retain their dedicated paths.

## Closed backend staging

The backend consumes the fused carrier through typed states with distinct
roles for:

1. the evaluated base plus raw key;
2. the canonical read Reference plus `oldValue`; and
3. the ready-to-write Reference plus `newValue`.

The raw-to-read transition owns the nullish check, sole `ToPropertyKey`,
`[[Get]]`, and `ToNumeric`. The read-to-write transition exhaustively applies
`NumericUpdateOp`. The final transition consumes the same canonical Reference,
performs `[[Set]]` and strict-false routing, and only then exhaustively matches
`UpdateReturnMode` to publish its typed old- or new-value role. No write helper
accepts a raw key, and no result helper accepts an untyped payload local.

This is the load-bearing compile-enforced seam: a plausible second
`ToPropertyKey`, a write through a reconstructed Reference, or prefix/postfix
publication from the wrong role has no accepted type/API.

## Focused producer invariant

The durable `lila-ir` witness lowers all four computed ordinary-property update
modes. It proves each becomes one fused carrier with the source-order base and
raw-key operands, captured strictness, exact update operation, exact return
mode, and conservative numeric result kind. It also pins that ordinary updates
cannot remain as the removed public-field `PropertyUpdate` node.

The backend fixture additionally observes key-expression abrupt completion
before the nullish-base check, nullish rejection before key coercion, one
canonical key across get/set, one `ToNumeric`, strict/sloppy false-`[[Set]]`
handling, and old/new publication only after a normal write.

## Explicit nonclaims

This batch does not change eager or logical compound assignment, plain
assignment, Super or private References, identifier/global/Object Environment
updates, `with`, optional chains, or any resumable/suspended property
Reference. Those lifecycles remain on their dedicated IR paths.

Static property names may use the fused carrier, but no Test262 progress is
derived from them. Array and TypedArray specialization is a backend choice only
when it preserves this exact Reference lifecycle. Dynamic source generation and
the interpreter policy are unchanged.

## Verification

Producer/static stage:

```sh
cargo fmt --all -- --check
cargo test -p lila-ir ordinary_property_numeric_update
git diff --check
./scripts/check-module-boundaries.sh
```

Integrated exact stage:

```sh
./target/debug/lila test262 run \
  language/expressions/postfix-decrement/S11.3.2_A6_T1.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --snapshot-name ordinary-property-numeric-update \
  --timeout-ms 180000 --threads 1
```

Run the same command for the full paths
`language/expressions/postfix-increment/S11.3.1_A6_T1.js`,
`language/expressions/prefix-decrement/S11.4.5_A6_T1.js`, and
`language/expressions/prefix-increment/S11.4.4_A6_T1.js`. Basename-only filters
select zero cases and are not valid evidence. Publication must report four
physical files, eight executions, and the exact strict/sloppy split rather than
treating one mode as duplicate evidence.
