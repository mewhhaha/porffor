# Delete-reference early errors

**Status:** T07 implementation and focused verification complete, 2026-08-23

## Decision

This fixed-message batch gives the two `delete`-reference early-error
conditions distinct closed identities:

| Condition | Wire name | Complete fixed rendered prefix |
| --- | --- | --- |
| `EarlyErrorCode::StrictModeDeleteIdentifierReference` | `E_STRICT_MODE_DELETE_IDENTIFIER_REFERENCE` | `cannot delete variables in strict mode at line` |
| `EarlyErrorCode::StrictModeDeletePrivateReference` | `E_STRICT_MODE_DELETE_PRIVATE_REFERENCE` | `cannot delete private fields at line` |

Both are pre-evaluation `SyntaxError` conditions. The implementation must report
them as `Early` through entry parsing and retained dependency parsing. The table
keys are Boa's messages; the enum identities name the specification conditions.
In particular, `StrictModeDeletePrivateReference` is not limited to private
fields merely because Boa's fixed wording says “private fields”: private
methods and private accessors produce the same forbidden private reference
shape.

## One grammar production, two conditions

Both conditions belong to the static semantics for:

```text
UnaryExpression : delete UnaryExpression
```

The specification's first condition depends on two facts: `IsStrict` of the
operand is true, and the derived operand has one of these shapes:

- `PrimaryExpression : IdentifierReference`;
- `MemberExpression : MemberExpression . PrivateIdentifier`;
- `CallExpression : CallExpression . PrivateIdentifier`;
- `OptionalChain : ?. PrivateIdentifier`; or
- `OptionalChain : OptionalChain . PrivateIdentifier`.

The two codes split that one conjunction by its disjoint identifier and private
operand families without dropping the shared strictness fact. Sloppy
`delete identifier` remains valid syntax. A sloppy undeclared private name is
still invalid, but it belongs to `AllPrivateIdentifiersValid` and
`InvalidPrivateIdentifier`, not to the strict-mode delete condition.

Parentheses do not change either result. The recursively covered form must
ultimately be judged as though the uncovered operand appeared directly under
`delete`.

The codes stay distinct even though one parser production detects both:

- strictness is a required conjunct for both codes;
- an identifier reference and a private property reference are disjoint AST
  shapes;
- the private family additionally owns the two optional-chain productions;
- a sloppy undeclared private operand must remain owned by
  `InvalidPrivateIdentifier`; and
- their pinned conformance cohorts, future diagnostic ownership and likely
  regressions are independent.

Combining them as a generic `DeleteReference` code would erase all five facts
and make a future change to one rule appear to preserve the other.

## Pinned parser ownership

Pinned `boa_parser-0.21.1` has exactly one producer for each raw message, in the
same `UnaryExpression` delete branch:

- `vendor/boa_parser-0.21.1/src/parser/expression/unary.rs:92-98` emits
  `cannot delete variables in strict mode`; and
- `vendor/boa_parser-0.21.1/src/parser/expression/unary.rs:100-105` emits
  `cannot delete private fields`.

`LexError::Syntax` appends ` at line L, col C`, so the classifier rows use the
complete stable rendered prefixes in the decision table rather than shorter
fragments such as `strict mode` or `private fields`.

The pinned private branch began incomplete: it omitted the strictness guard and
recognized only `Expression::PropertyAccess(PropertyAccess::Private(_))`. This
batch repairs it before giving its message a typed identity. The branch now:

1. parse the complete unary operand;
2. call `target.flatten()`, which recursively removes outer parenthesized AST
   nodes;
3. place both the identifier and private families beneath one enclosing
   `cursor.strict()` guard;
4. recognize direct private property access; and
5. recognize `Expression::Optional` exactly when the final optional operation
   is `OptionalOperationKind::PrivatePropertyAccess`.

Inspecting the final optional operation covers both private-ending
OptionalChain productions while excluding a chain whose final operation is a
public access or call. The operation-kind match must spell every current enum
variant so adding a new optional-operation shape forces a review at compile
time.

The source inventory regression embeds `unary.rs` and pins each raw
literal at exactly one occurrence. The inventory is a test over the vendored
source, not a compile-time proof of a repository-wide literal count.

## Typed encoding

The implementation extends the closed domain from 53 to 55 variants and
the parse-failure table from 52 to 54 rows:

- add the two variants and their sole wire spellings to `EarlyErrorCode`;
- add one complete-prefix classifier row and exact rendered witness per code;
- add a `ParseClassified::from_parse_table(...)` const ownership witness per
  code, so deleting either parse row while retaining its variant fails to
  compile;
- add both variants to `lila-ir`'s exhaustive `rejection_kind` map as
  `IrDiagnosticKind::EarlyError`; and
- retain the existing const proofs for populated and disjoint rows, witness
  selection, wire-name injectivity, parse-table reachability and parse-to-IR
  phase consistency.

The two new rows must remain disjoint from each other, from
`InvalidPrivateIdentifier`, and from every existing strict-mode row. In strict
code, an undeclared private name beneath `delete` belongs to
`StrictModeDeletePrivateReference`: the delete-specific condition is detected
while parsing the unary expression before the later whole-source private-name
validity check. In sloppy Script code it must proceed to that later check and
become `InvalidPrivateIdentifier` instead.

## Durable regressions

The front-end source matrix must assert the exact code, `Early` phase,
`SyntaxError` type and a nonempty span for each rejection.

For `StrictModeDeleteIdentifierReference`, it must cover:

- a Script Use Strict Directive followed by `delete identifier`;
- default-strict Module code;
- strict code nested in an ordinary function; and
- direct, parenthesized and recursively parenthesized operands.

Its positive boundaries must keep sloppy Script `delete identifier`, strict
public-property deletion and strict deletion of a non-reference expression
parse-valid.

For `StrictModeDeletePrivateReference`, the matrix must cover class declarations
and expressions, field initializers and method bodies, member and
call-expression bases, both optional-chain private productions, direct and
recursively parenthesized operands, declared private fields, methods and
accessors, and an undeclared private name. Shared parser ownership means these
axes need representative sources, not a handwritten copy of all 192 generated
Test262 cases.

An anti-conflation witness must parse sloppy Script
`delete object.#missing`, allow it past the delete-specific branch, and assert
that the later whole-source check reports `InvalidPrivateIdentifier`.

Its positive boundaries must keep public and optional-public property deletion,
declared ordinary and optional private reads, and optional chains with a private
intermediate operation but a final public access or call parse-valid. Those are
syntax boundaries only; this contract does not claim their runtime results.

A real retained Module parse failure must exercise each code, for example with
a top-level strict identifier deletion and an exported class containing a
private deletion. Hand-built diagnostics do not prove retained-source
projection.

## Exact pinned cohorts

The strict-identifier cohort is exactly these two physical files:

- `language/expressions/delete/identifier-strict.js`;
- `language/expressions/delete/identifier-strict-recursive.js`.

Both declare `flags: [onlyStrict]`, so they expand to two strict Script
executions total.

The private-reference cohort is every `.js` file directly under both paths:

- `language/expressions/class/elements/syntax/early-errors/delete` — 96 files;
- `language/statements/class/elements/syntax/early-errors/delete` — 96 files.

All 192 files are procedurally generated. Their sole flag is `generated`; none
has an execution-mode flag such as `onlyStrict`, `noStrict`, `raw` or `module`.
The harness must therefore expand them to 384 sloppy/strict executions. Class
bodies themselves remain strict in both materializations.

The combined exact batch is 194 physical files and 386 Wasm-AOT executions.
Those pinned generated private-delete cases cover MemberExpression and
CallExpression private endings; the current-spec OptionalChain endings do not
yet have a matching pinned delete cohort, so permanent source witnesses carry
that parser-shape contract.
The similarly relevant staging test
`staging/sm/expressions/delete-name-parenthesized-early-error-strict-mode.js`
constructs source through `Function` and indirect `eval`; it is dynamic-source
debt and is deliberately outside this AOT-focused cohort.

No focused pre-change snapshot exists. Because the sources may already have
received an uncoded parse-phase `SyntaxError`, the green result establishes
typed diagnostic closure and bounded no-regression only. It is not a pass gain
without separate baseline evidence.

## Nonclaims

This batch does not implement runtime `delete` semantics, private element
installation or access, dynamic `eval` or `Function`, general strict-mode
binding restrictions, assignment-target validity, every unary-expression
error, all class grammar, T07 closure or aggregate Test262 closure. It does not
classify malformed, source-interpolating or engine-only messages that merely
contain words shared with either fixed prefix.

## Evidence

Implemented and focused-verified on `2026-08-23` under the repository's capped,
serial verification policy:

- `cargo check -p lila-front -p lila-ir --all-targets` passes.
- `cargo test -p lila-front --lib -- --test-threads=1` passes `89/89`, including
  both optional-chain private productions, sloppy private-name precedence and
  final-public/final-call controls.
- `cargo test -p lila-ir modules::early::tests -- --test-threads=1` passes
  `38/38`; the exact retained-dependency graph witness separately passes `1/1`.
- `cargo test -p lila-ir early_error -- --test-threads=1` passes `3/3`.
- `cargo xc` and a fresh `cargo build -p lila-cli --release` are green.
- The exact isolated cohort passes all `386/386` Wasm-AOT executions: 192
  sloppy Script and 194 strict Script identities over 194 physical sources.
  The verified snapshot has 386 unique `Success` outcomes, zero failures,
  timeouts or non-success kinds, and an exact completed-ID set match. The
  temporary suite separately verified 194 links, 194 resolved files, no
  dangling links and the vendored harness, then was moved to trash.
- Two independent post-repair reviews pass with no remaining findings.

This is typed diagnostic closure and bounded no-regression evidence. There is
no focused pre-change snapshot, so it is not evidence of a pass gain.
