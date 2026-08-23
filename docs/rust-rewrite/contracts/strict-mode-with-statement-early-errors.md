# Strict-mode `with` statement early errors

## Decision

A `WithStatement` contained in strict-mode code is one closed condition:

`EarlyErrorCode::StrictModeWithStatement`

The sole wire spelling is `E_STRICT_MODE_WITH_STATEMENT`. Script and Module
parsing report the condition during the early-error phase as a `SyntaxError`,
with the parser's source span. The rejected statement never reaches IR
lowering.

## Measured parser boundary

Pinned `boa_parser-0.21.1` emits the exact, case-sensitive message

```text
with statement not allowed in strict mode
```

from one branch in
`vendor/boa_parser-0.21.1/src/parser/statement/with/mod.rs`. One exact
classifier row owns that producer. The front-end code domain and parse-failure
row count remain type-level invariants, and the exhaustive `lila-ir`
projection maps the new variant to `IrDiagnosticKind::EarlyError`.

## Strictness boundaries

A Script directive prologue and a strict ordinary function body reject a
contained `with` statement. Class method bodies and all Module code are strict
without a directive and reject it as well. A sloppy Script or sloppy ordinary
function remains accepted.

This is parser-state classification, not a source scan. It does not change the
runtime Object Environment Record or lowering behavior for valid sloppy
`with` statements.

## Durable verification

Front-end tests cover strict Script, strict ordinary-function, class-method and
Module contexts, plus the two sloppy positive boundaries. The retained Module
regression sends the real parse failure through
`module_parse_failure_diagnostic` and requires the same code, phase, error type
and source span.

The exact pinned Test262 cohort is the seven negative files under
`language/statements/with`: `12.10.1-11gs.js`, the two
`strict-fn-decl-nested-*` cases, and `strict-fn-decl.js`,
`strict-fn-expr.js`, `strict-fn-method.js` and `strict-script.js`. Each has one
declared execution mode.

At `2026-08-23`, the capped serial front gate passes `44/44`, the focused IR
early-error gate passes `3/3`, and the seven exact pinned Wasm-AOT executions
pass `7/7`, with every failure and non-success bucket at zero. This is bounded
strict-`with` evidence, not runtime `with`, strict-mode, T07 or aggregate
Test262 closure.
