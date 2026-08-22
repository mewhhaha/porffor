# Super Property Reference Mutation

Status: normative for the non-resumable Wasm-AOT slice described here.

This contract closes the shared Reference lifecycle for numeric update and eager
compound assignment through a `super` property. It does not broaden logical
assignment, private-field update, suspended assignment, or dynamic-source
support.

## Conformance boundary

The exact pinned cohort is four Test262 files, each with sloppy and strict
execution variants:

- `language/expressions/super/prop-expr-getsuperbase-before-topropertykey-putvalue-increment.js`
- `language/expressions/super/prop-expr-uninitialized-this-putvalue-increment.js`
- `language/expressions/super/prop-expr-uninitialized-this-putvalue-compound-assign.js`
- `language/expressions/super/prop-expr-getsuperbase-before-topropertykey-putvalue-compound-assign.js`

At the pre-batch `b0d1d1300` source boundary, the available nearby debug binary
reported `2/8`: the two increment files were `0/4` Runtime/NotImplemented with
`super property update target`; the uninitialized-`this` compound file was
`0/2` Runtime/Bug with the wrong error constructor; and the GetSuperBase
compound file was the existing `2/2` guard. The binary timestamp preceded the
commit by four minutes, so these are near-HEAD measurements rather than an
exact-commit artifact. Current source independently contains both the explicit
update refusal and the decomposed compound read/write lifecycle.

## Reference lifecycle

For `super[key] op= rhs` and `super[key]++`/`++super[key]`, the compiler must
preserve one Reference Record through GetValue and PutValue. Its observable
order is:

1. evaluate GetThisBinding and retain the actual receiver;
2. evaluate the raw property-key expression and GetValue it;
3. evaluate GetSuperBase once and reject a null base;
4. during GetValue, apply ToPropertyKey exactly once and retain the resulting
   property key in the same Reference Record;
5. perform `Get(superBase, propertyKey, receiver)`;
6. for eager compound assignment, evaluate the right-hand side and apply the
   selected arithmetic or bitwise operation; for numeric update, apply
   ToNumeric once and compute the increment or decrement;
7. perform `Set(superBase, propertyKey, newValue, receiver)` using the retained
   base, key, and receiver;
8. route a failed Set according to the Reference's captured `Strictness`;
9. expose the eager, prefix, or postfix result only after successful PutValue.

The key expression is evaluated before GetSuperBase, but ToPropertyKey is not.
Changing the HomeObject's prototype during key coercion must therefore not
change the retained super base used by either Get or Set. A detached method call
must use the detached call's actual `this` as Receiver for both operations.

## Closed lowering representation

The public IR contains one fused `SuperPropertyMutationIr` with private fields
and crate-private constructors. It owns:

- the receiver expression;
- the raw `PropertyKeyIr`;
- the Reference `Strictness`;
- exactly one exhaustive `SuperPropertyMutationOperationIr`.

The operation domain is:

- `NumericUpdate { op: NumericUpdateOp, return_mode: UpdateReturnMode,
  value_kind: ValueKind }`;
- `EagerCompound { old_value_binding: String, result: Box<TypedExpr> }`.

The eager result is constructed from the carrier-provided old-value binding and
the already-lowered right-hand side. Arithmetic and bitwise operators must map
exhaustively into that operation. `LogicalBinaryOp` is absent by construction:
logical assignment requires branch-local RHS evaluation and PutValue and is a
separate future contract.

A private, non-`Clone`, non-`Copy`, `#[must_use]`
`SuperPropertyReferencePlan` is the sole lowerer producer. It is consumed by
either `numeric_update` or `eager_compound_assignment`; no API exposes a read
and a separately rebuilt write. The general `ReferenceRecord` path remains the
authority for ordinary property, private, global, and logical Reference
lifecycles, but must not decompose an eager Super mutation into independent
`SuperPropertyRead` and `SuperPropertyWrite` nodes.

## Closed AOT lifecycle

The AOT emitter reserves one non-`Copy` local carrier:

- `EvaluatedRawSuperPropertyReferenceLocals { receiver, base,
  referenced_name }`.

It is produced only after receiver, raw key, and GetSuperBase have been emitted
in that order. Its consuming GetValue transition performs the sole
ToPropertyKey and `Get`, returning both the old value and:

- `CoercedSuperPropertyReferenceLocals { receiver, base, property_key }`.

Only the coerced carrier may be consumed by the mutation PutValue/Set path.
There is no write helper accepting raw locals and no path that reloads
GetSuperBase or recoerces the key. Both carriers have private fields, are
non-`Clone`/non-`Copy`, are `#[must_use]`, and release their locals in allocator
LIFO order after their sole consumption.

For eager compound assignment, the emitter materializes the old GetValue result
under the operation's private binding, compiles the sealed result expression,
then consumes the coerced carrier for PutValue. For numeric update, the emitter
uses the existing exhaustive numeric operation and return-mode domains while
retaining old/new values until PutValue succeeds.

Every exhaustive `ExprIr` consumer must name the fused mutation node, including
throw inference, summaries, early-error traversal, planning, string/data
collection, dynamic-result classification, and expression emission. Adding a
new mutation operation must create compile errors in both lowering and AOT
operation consumers.

## Durable oracle

The fixture must use accessor-bearing `baseA` and `baseB`. A computed key's
`toString` changes the object method's prototype from A to B and returns `p`.
Calling the method with an alien receiver must make eager compound assignment
produce result `3`, one coercion, and exactly:

```text
key,getA,rhs,setA:3:true
```

A prefix-update sibling must produce result `2`, one coercion, and exactly:

```text
key,getA,setA:2:true
```

These traces make a second ToPropertyKey, a GetSuperBase reload before Get or
Set, receiver loss, and RHS-order drift independently observable. The retained
fixture also covers all four prefix/postfix increment/decrement modes, Number
and BigInt, failed strict Set, and uninitialized-`this` ordering.

## Nonclaims

This batch does not claim:

- logical assignment through a Super Reference;
- private-field numeric update or compound assignment;
- References retained across `yield` or `await`;
- dynamic source generation;
- full object-expression or class conformance beyond the exact cohort and
  durable fixture.

Verification must keep the pre-batch evidence distinct from post-batch results.
Publication requires the workspace compile, focused IR/structure/CLI tests, all
eight exact cohort executions, and the existing green compound guard without a
rewrite, skip, or materializer.

## Verified batch result

The completed batch passed the workspace all-target check and `cargo xc`; the
focused IR invariant `1/1`; the bounded structure executable `5/5`; and the
compiled Wasm fixture `1/1` in `10.82s`. The exact four-file cohort is `8/8`
with zero unsupported, crash or bug outcomes. The adjacent
`prop-expr-uninitialized-this` and
`prop-expr-getsuperbase-before-topropertykey` filters are each `8/8`, preserving
the plain GetValue/PutValue controls alongside the newly fused mutations. These
focused results do not broaden the nonclaims above or represent a full pinned
Test262 publication.
