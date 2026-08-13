# Contract: duplicate formal-parameter early errors

**Status:** Normative T07 extension, 2026-08-13

## Spec invariant

Duplicate `BoundNames` in a formal-parameter list are not one unconditional
grammar error. ECMAScript rejects them when the list is non-simple, when the
surrounding function code is strict, or when the grammar requires
`UniqueFormalParameters`. A sloppy Script ordinary function with a simple list
is the intentional exception: `function duplicate(a, a) {}` remains valid.

The rejection is decided before evaluation and is a `SyntaxError`. Lila therefore
needs one closed condition, `DuplicateFormalParameter`, but must not add an
AST-wide duplicate-name check: such a check would also reject the valid sloppy
simple-list case.

## Measured Boa boundary

The pinned `boa_parser-0.21.1` represents the condition with exactly two
case-sensitive message literals:

| Boa message | Measured producer shape |
|---|---|
| `Duplicate parameter name not allowed in this context` | Ten sites: non-simple ordinary parameters and strict/context checks in ordinary, async, generator, arrow, object-method, and hoistable-function parsers |
| `duplicate parameter name not allowed in unique formal parameters` | The shared `UniqueFormalParameters` parser used by grammar forms whose parameter names must always be unique |

The ten capitalized literal sites are
`parser/function/mod.rs:139`,
`parser/expression/assignment/mod.rs:238`,
`parser/expression/assignment/arrow_function.rs:102`,
`parser/expression/assignment/async_arrow_function.rs:108`,
`parser/expression/primary/function_expression/mod.rs:92`,
`parser/expression/primary/generator_expression/mod.rs:98`,
`parser/expression/primary/async_function_expression/mod.rs:97`,
`parser/expression/primary/async_generator_expression/mod.rs:130`,
`parser/expression/primary/object_initializer/mod.rs:515`, and
`parser/statement/declaration/hoistable/mod.rs:189`. The one lowercase literal
is `parser/function/mod.rs:199`; every `UniqueFormalParameters` consumer reaches
that shared parser. All paths are relative to
`vendor/boa_parser-0.21.1/src/`.

Before this extension, neither message matched the one
`PARSE_FAILURE_RULE_TABLE`. Entry parsing consequently reported
`P_PARSE_MALFORMED`, while a failed dependency-module parse was converted by
`module_parse_failure_diagnostic` to `Unsupported`. The latter lost the required
early `SyntaxError` rejection entirely.

## Encoding

- Add one `EarlyErrorCode::DuplicateFormalParameter` variant with wire name
  `E_DUPLICATE_FORMAL_PARAMETER`.
- Add exactly two table rows, each containing one complete, case-sensitive Boa
  literal. Do not use a shared fragment such as `uplicate parameter name`: Boa
  has nearby formal-parameter diagnostics for different spec conditions, and a
  broad fragment would turn future wording into an unearned spec claim.
- Map the new variant through `lila-ir`'s exhaustive `rejection_kind` match to
  `IrDiagnosticKind::EarlyError`. Phase and error type then remain derived as
  `Early` and `SyntaxError` for both entry and retained dependency failures.
- Keep `ParseClassified` as the only parse-stage witness. The table rows make
  the new code constructible there; the exhaustive IR match makes a future
  omitted consumer fail to compile.

At this extension's checkpoint, the closed domain had 19 variants and the
classifier had 17 rows. The later duplicate-catch-parameter extension grows the
current counts without changing this condition. The existing const gates still
prove table population, witness disjointness, wire-name closure, classifier
reachability, and parse-to-IR phase consistency.

## Durable regressions

The source-level contract covers all three boundaries without depending on a
Test262 path or source-text special case:

- a non-simple ordinary function and an arrow function exercise Boa's
  capitalized wording across distinct formal-parameter contexts;
- a class method exercises the lowercase `UniqueFormalParameters` wording;
- one fixture, `function duplicate(a, a) {}`, succeeds under sloppy Script goal
  and rejects under strict Module goal with
  `E_DUPLICATE_FORMAL_PARAMETER`.

The dependency regression must parse real Module source through `lila-front`
and then pass the retained `ParseError` through
`module_parse_failure_diagnostic`. Hand-constructing only the diagnostic would
not prove that Boa's current wording reaches the closed table.

## Nonclaims

This extension does not classify duplicate catch parameters, strict
`eval`/`arguments` parameter names, an illegal `"use strict"` directive in a
function with non-simple parameters, or `yield`/`await` binding restrictions.
Those are distinct spec conditions with distinct Boa diagnostics. It also does
not implement runtime parameter environments, mapped `arguments`, or parameter
initialization semantics; those remain T09 work.

No Test262 pass-count claim follows from this local closure. Focused Cargo and
current-pin Test262 verification remain deferred until the implementation batch
is reviewed and the shared verification lane is available.
