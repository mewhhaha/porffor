# Ordinary property plain-assignment Reference

Status: normative implementation contract for plain assignment through an
ordinary property Reference.

## Exact conformance boundary

The selected raw Test262 cohort is:

- `language/expressions/assignment/target-member-computed-reference-null.js`;
- `language/expressions/assignment/target-member-identifier-reference-null.js`;
- `language/expressions/assignment/target-member-identifier-reference-undefined.js`.

None has an explicit flags list, so the three physical files produce six
sloppy/strict executions. The selected current-head baseline is 1/6:

- both null-base files are 0/2 `Runtime:NotImplemented`, with exact diagnostic
  `unsupported in lila wasm-aot first slice: property access on non-object target`;
- the undefined-base identifier file passes in strict mode but is
  `Runtime:Bug` in sloppy mode because the assignment fails to throw the
  required `TypeError`.

The adjacent raw controls
`target-member-computed-reference-undefined.js` and
`target-member-computed-reference.js` are each 2/2. These files have no
Wasm-AOT rewrite, matrix mask, known-failure entry, or dynamic-source
exemption.

## Normative Reference lifecycle

For `base[key] = rhs`, the compiler must preserve one ordinary property
Reference through this exact order:

1. evaluate `base` and propagate an abrupt completion;
2. evaluate the raw computed-key expression and propagate an abrupt
   completion;
3. evaluate `rhs` and propagate an abrupt completion;
4. begin PutValue on the retained Reference: apply `ToObject` to the retained
   base, so a nullish base throws only after steps 1-3;
5. apply `ToPropertyKey` exactly once to the retained raw key;
6. perform `[[Set]]` with `ToObject(base)` as the target and the original
   unboxed `GetThisValue` as the receiver;
7. route a `[[Set]]` result of false according to the Reference's captured
   `[[Strict]]`; and
8. publish `rhs` only after PutValue completes normally.

The raw computed-key expression and `ToPropertyKey` are deliberately separate.
An abrupt raw-key expression wins over the RHS. Once the raw expression has
completed, the RHS wins over nullish-base rejection and key coercion. No
producer may use `PropertyKeyIr` as evidence that coercion has already occurred;
for this carrier it denotes the still-raw referenced-name expression.

## Closed producer shape

The existing private, non-`Clone`, non-`Copy`, `#[must_use]`
`OrdinaryPropertyReferencePlan` is the sole producer. It owns exactly one
lowered base-and-receiver expression, one raw referenced name, and the
Reference's `Strictness`. This batch adds a third consuming operation, distinct
from eager compound assignment and numeric update:

```text
plain_assignment(rhs) -> TypedExpr
```

The operation consumes the plan and the already-lowered RHS together. A caller
cannot reconstruct the base or key after RHS lowering, coerce the key in the
producer, or publish the RHS without also carrying the write obligation.

The resulting expression is `ExprIr::OrdinaryPropertyAssignment`, carrying the
public, private-field `OrdinaryPropertyAssignmentIr`. Its frozen backend
accessors are:

- `base_and_receiver() -> &TypedExpr`;
- `referenced_name() -> &PropertyKeyIr`;
- `rhs() -> &TypedExpr`; and
- `strictness() -> Strictness`.

The carrier is `Clone` only because `ExprIr` remains cloneable. It exposes no
public constructor. `PropertyAccess::Simple` plain assignment must intercept
before the old generic `ExprIr::PropertyWrite` path. The old node remains for
internal writes, destructuring, Super/private-specific paths, and other
already-staged compiler operations; it is not a source-level ordinary plain
member-assignment alternative.

This is the load-bearing seam: adding `ExprIr::OrdinaryPropertyAssignment`
forces every exhaustive IR consumer to decide how one staged Reference is
handled, while the consuming plan makes a second base/key evaluation or a
producer-side `ToPropertyKey` unavailable by construction.

## Closed backend staging

The backend consumes the fused carrier through typed states with distinct
roles for:

1. the evaluated original base/receiver plus raw computed-key value;
2. the evaluated RHS while the raw Reference remains retained; and
3. the canonical write Reference produced by the sole `ToObject` and
   `ToPropertyKey` transition.

The transition into the final state derives the distinct `ToObject(base)`
target while retaining the original base as receiver. Only the final state can
emit `[[Set]]`, strict-false routing, and normal RHS publication. No write
helper accepts an independently reconstructed base or a second raw-key
expression.

## Focused producer invariant

The durable `lila-ir` witness lowers computed and static-name ordinary-property
assignments with effectful base, key, and RHS expressions in strict and sloppy functions.
It proves that each source assignment becomes exactly one
`OrdinaryPropertyAssignmentIr`, preserves the base/key/RHS source roles and
captured strictness, and contains no independent source-level `PropertyWrite`
or eager/numeric mutation carrier in that subtree.

## Explicit nonclaims

This batch does not change compound or logical assignment, numeric update,
destructuring assignment, optional chains, Super or private References,
identifier/global/Object Environment References, `with`, or resumable property
References. Both static and computed names use the same carrier; no broader
property-assignment conformance claim follows from the selected three files.
Dynamic source generation and the interpreter policy are unchanged.

## Verification

Producer/static stage:

```sh
cargo fmt --all -- --check
cargo test -p lila-ir ordinary_property_plain_assignment
git diff --check
./scripts/check-module-boundaries.sh
```

Integrated exact stage:

Run each exact path separately:

```sh
./target/debug/lila test262 run \
  language/expressions/assignment/target-member-computed-reference-null.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --snapshot-name ordinary-property-plain-assignment-computed-null \
  --timeout-ms 180000 --threads 1
./target/debug/lila test262 run \
  language/expressions/assignment/target-member-identifier-reference-null.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --snapshot-name ordinary-property-plain-assignment-identifier-null \
  --timeout-ms 180000 --threads 1
./target/debug/lila test262 run \
  language/expressions/assignment/target-member-identifier-reference-undefined.js \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --snapshot-name ordinary-property-plain-assignment-identifier-undefined \
  --timeout-ms 180000 --threads 1
```

Basename-only filters select zero cases and are not valid evidence.

Publication must report all three physical files, six executions, and the
strict/sloppy split. A single green control is not evidence that the complete
Reference-staging cohort passed.
