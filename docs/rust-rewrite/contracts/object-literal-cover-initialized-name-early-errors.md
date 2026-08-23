# Object-literal CoverInitializedName early errors

## Decision

A `CoverInitializedName` that survives cover-grammar reinterpretation inside
an actual ObjectLiteral is one closed pre-evaluation condition:

`EarlyErrorCode::ObjectLiteralCoverInitializedName`

Its sole wire spelling is `E_OBJECT_LITERAL_COVER_INITIALIZED_NAME`. The name
describes the static-semantics condition rather than inheriting Boa's broader
rendered phrase `invalid object literal`.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has four producers after
`contains_invalid_object_literal` finds the same
`PropertyDefinition::CoverInitializedName` AST node:

- `invalid object literal in script statement list`;
- `invalid object literal in function statement list`;
- `invalid object literal in module item list`;
- `invalid object literal in class static block statement list`.

The fixed, case-sensitive prefix `invalid object literal in` matches exactly
those four producers in the pinned parser. One classifier row therefore owns
the condition across Script, function body, Module and class-static-block
contexts. It does not overlap the distinct duplicate-`__proto__` object-literal
diagnostic or source-interpolating expected/unexpected-token diagnostics.

## Cover-grammar boundary

`({ a = 1 });` is an ObjectLiteral containing a surviving
CoverInitializedName and must reject. The same cover syntax remains valid when
the grammar reinterprets it as an assignment pattern, binding pattern or arrow
parameter list. In particular, these are outside this diagnostic:

```js
({ a = 1 } = target);
let { a = 1 } = {};
const f = ({ a = 1 }) => a;
```

Ordinary shorthand and data properties such as `({ a })` and `({ a: 1 })`
also remain valid. The classifier changes only the structured diagnostic for a
parser rejection; it does not alter cover-grammar parsing or lowering.

## Verification boundary

Direct front-end witnesses cover all four pinned producer contexts under the
applicable goals. A retained Module failure must preserve the typed code,
`Early` phase, `SyntaxError` constructor and source span. Positive controls
cover assignment/binding reinterpretation, arrow parameters, shorthand and
ordinary properties under Script and Module goals.

The exact pinned negative cohort is
`language/expressions/object/cover-initialized-name.js`. Its metadata expands
to two sloppy/strict Wasm-AOT executions. This bounded family does not claim
general ObjectLiteral grammar, destructuring execution or broad T07 closure.

At `2026-08-23`, capped serial verification passes the complete front-end gate
at `53/53`, the focused IR early-error gate at `3/3`, and the exact pinned file
at `2/2` Wasm-AOT executions. Every failure and non-success bucket is zero. The
workspace `cargo xc` check is also green.
