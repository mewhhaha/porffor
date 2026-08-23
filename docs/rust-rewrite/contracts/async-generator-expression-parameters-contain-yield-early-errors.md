# Async-generator-expression parameter `Contains YieldExpression` early errors

## Decision

An `AsyncGeneratorExpression` whose
`FormalParameters Contains YieldExpression` is one closed pre-evaluation
condition:

`EarlyErrorCode::AsyncGeneratorExpressionParametersContainYield`

Its sole wire spelling is
`E_ASYNC_GENERATOR_EXPRESSION_PARAMETERS_CONTAIN_YIELD`. It remains distinct
from declarations, ordinary generator expressions, async-generator parameter
`Contains AwaitExpression`, and generator methods because pinned Boa gives
those forms different producers and messages.

## Measured parser boundary

Pinned `boa_parser-0.21.1` dispatches contiguous `async function *` primary
expressions to one AsyncGeneratorExpression parser. Its sole yield producer
emits:

```text
yield expression not allowed in async generator expression parameters
```

`LexError::Syntax` appends the parameter-list position, so the classifier uses
the complete fixed text plus `at line` without fixing a coordinate. The message
contains no user source. Anonymous and named expressions share this parser.

The adjacent `Contains AwaitExpression` check has its own fixed message and is
deliberately a separate code rather than a catch-all parameter restriction.

## Goal and containment boundary

The producer is reachable under Script and Module goals. Retained Module
parsing projects the same typed `Early` / `SyntaxError` diagnostic with a
nonempty source span when the expression initializes an exported binding.

`Contains` stops at a nested generator-function boundary. An async-generator
expression may use `yield` in its body or inside a nested generator used as a
parameter initializer; only a `YieldExpression` contained by its own
FormalParameters is rejected. A bare `yield` binding name is not a witness
because Boa rejects it earlier through a different producer.

Pinned Boa eval parsing can also reach the same primary-expression parser.
Lila has no product dynamic-eval parser path, so that structural fact does not
justify a T07 eval integration surface.

## Verification boundary

Front-end tests cover anonymous and named forms under both goals and preserve
body/nested-generator boundaries. A retained-module test covers an exported
initializer. The exact pinned Test262 cohort is:

- `language/expressions/async-generator/early-errors-expression-formals-contains-yield-expr.js`.

Its metadata expands to two sloppy/strict Wasm-AOT executions. This bounded
family does not claim the bare-yield binding rejection, the adjacent
AwaitExpression rule, declarations, ordinary generator expressions, methods,
direct eval, T07, or aggregate parser closure.

## Evidence

At `2026-08-23`, all verification ran under the repository's eight-CPU cap and
serial test/harness settings. The complete `lila-front` gate passes `67/67`,
the focused `lila-ir` early-error filter passes `3/3`, and `cargo xc` passes.
The exact pinned file above passes `2/2` sloppy/strict Wasm-AOT executions, with
every failure and non-success bucket at zero.
