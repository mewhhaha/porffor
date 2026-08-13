# Array destructuring evaluation is a closed operation, not a boolean

## Problem

`ExprIr::ArrayDestructure` represents two different ECMAScript abstract
operations:

- 8.6.3 `IteratorBindingInitialization`; and
- 13.15.5.5 `IteratorDestructuringAssignmentEvaluation`.

Before this contract, the IR stored that distinction as `assignment: bool`.
The two values were meaningful only by convention. A producer could reverse
them, and a consumer could silently treat a future third operation as one of
the existing cases.

This is not an iterator-protocol distinction. Both operations acquire and
consume an iterator with the same close obligations, already carried per
pattern by `ArrayPatternProtocol`. The missing invariant is the evaluation
operation that owns the pattern.

## Closed IR domain

Replace the boolean with:

```rust
pub enum ArrayDestructuringEvaluationIr {
    BindingInitialization,
    AssignmentEvaluation,
}
```

and rename the `ExprIr::ArrayDestructure` field to `evaluation`.

There is deliberately no `Default`, boolean conversion, or predicate helper.
Every lowering site must name its operation. Omitting the field is `E0063`, and
passing a boolean is `E0308`.

## Semantic obligations

The distinction has six consumers, all of which must match the closed domain
exhaustively:

| Operation | Expression result | Owns declaration bindings |
| --- | --- | --- |
| `BindingInitialization` | `undefined` | yes |
| `AssignmentEvaluation` | the original RHS payload and tag | no |

The result emitter, direct lexical initializer, result-tag planner, lexical
counter, hoisted-variable collector and product-name collector must bind
`evaluation` and use a nested exhaustive `match`. They must not use `matches!`,
an `if`, or a wildcard arm. Adding another evaluation operation then fails
every semantic consumer with `E0004` until its result and declaration behavior
are stated.

Transport-only visitors may continue to use `..`: they traverse the value and
pattern but do not interpret which abstract operation owns them.

## Producer map

The five construction contexts are fixed as follows:

| Lowering context | Operation |
| --- | --- |
| function-parameter array binding | `BindingInitialization` |
| array destructuring assignment, including loop-head assignment | `AssignmentEvaluation` |
| lexical array binding | `BindingInitialization` |
| `var` array binding | `BindingInitialization` |
| lexical array binding from an already materialized value | `BindingInitialization` |

Existing structural lowering tests must name these variants so that a producer
mapped to the wrong operation remains an observable regression even though both
enum variants are well typed.

## Preservation boundary

This change is intended to be Wasm-byte-neutral. It does not alter
`ArrayPatternProtocol`, iterator acquisition, stepping, value extraction,
closing, nested-pattern behavior, or evaluation order. Assignment evaluation
must still return the exact original RHS payload and tag; binding initialization
must still produce `undefined`.

It does not claim new iterator-close coverage, generator support, object
destructuring coverage, resource-management support, or a Test262 status
change.

## Verification

The implementation stage is checked first with static searches for the deleted
field and for non-exhaustive semantic shortcuts. Compilation and focused runtime
regressions are deferred to the central batch verifier so concurrent lanes can
share one build-artifact lease.
