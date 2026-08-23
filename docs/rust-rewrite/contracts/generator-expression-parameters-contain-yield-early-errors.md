# Generator-expression parameter `Contains YieldExpression` early errors

## Decision

A `GeneratorExpression` whose `FormalParameters Contains YieldExpression` is
one closed pre-evaluation condition:

`EarlyErrorCode::GeneratorExpressionParametersContainYield`

Its sole wire spelling is
`E_GENERATOR_EXPRESSION_PARAMETERS_CONTAIN_YIELD`. It remains distinct from
generator declarations, async-generator expressions and generator methods
because pinned Boa gives those forms different producers and messages.

## Measured parser boundary

Pinned `boa_parser-0.21.1` dispatches `function *` primary expressions to one
ordinary GeneratorExpression parser. Its sole fixed producer emits:

```text
generator expression cannot contain yield expression in parameters
```

`LexError::Syntax` appends the parameter-list position, so the classifier uses
the complete fixed text plus `at line` without fixing a coordinate. The message
contains no user source. Anonymous and named generator expressions share this
one parser path.

## Goal and containment boundary

The producer is reachable under Script and Module goals. Retained Module
parsing projects the same typed `Early` / `SyntaxError` diagnostic with a
nonempty source span when the expression initializes an exported binding.

`Contains` stops at a nested generator-function boundary. A generator
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

- `language/expressions/generators/param-dflt-yield.js`.

Its metadata expands to two sloppy/strict Wasm-AOT executions. This bounded
family does not claim generator declarations, async-generator expressions,
generator methods, direct eval, T07, or aggregate parser closure.

## Evidence

At `2026-08-23`, all verification ran under the repository's eight-CPU cap and
serial test/harness settings. The complete `lila-front` gate passes `65/65`,
the focused `lila-ir` early-error filter passes `3/3`, and `cargo xc` passes.
The exact pinned file above passes `2/2` sloppy/strict Wasm-AOT executions, with
every failure and non-success bucket at zero.
