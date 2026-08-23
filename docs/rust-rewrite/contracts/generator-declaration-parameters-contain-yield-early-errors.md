# Generator-declaration parameter `Contains YieldExpression` early errors

## Decision

A `GeneratorDeclaration` or `AsyncGeneratorDeclaration` whose
`FormalParameters Contains YieldExpression` is one closed pre-evaluation
condition:

`EarlyErrorCode::GeneratorDeclarationParametersContainYield`

Its sole wire spelling is
`E_GENERATOR_DECLARATION_PARAMETERS_CONTAIN_YIELD`. Ordinary and async
generator declarations share the code because pinned Boa routes both through
one fixed producer after their declaration parsers opt into the same check.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has one producer in
`parser/statement/declaration/hoistable/mod.rs`. Its fixed raw message is:

```text
invalid yield usage in generator function parameters
```

`GeneratorDeclaration` and `AsyncGeneratorDeclaration` both set
`parameters_yield_is_early_error()` and call that common declaration parser.
`LexError::Syntax` appends the parameter-list position, so the classifier uses
the complete fixed text plus `at line` without fixing a coordinate. The message
contains no user source.

Generator expressions and generator methods are deliberately outside this
code. Their pinned parser branches use distinct messages, so merging them would
erase producer ownership rather than encode a real invariant.

## Goal and containment boundary

The declaration producer is reachable under both Script and Module goals for
ordinary and async generator declarations. Retained Module parsing projects
the same typed `Early` / `SyntaxError` diagnostic, including default-export
async generator declarations and a nonempty source span.

`Contains` stops at a nested generator-function boundary. A generator
declaration may therefore use `yield` in its body or inside a nested generator
used as a parameter initializer; only a `YieldExpression` contained by the
declaration's own FormalParameters is rejected. A bare `yield` binding name is
not a witness because Boa rejects it earlier through a different producer.

Pinned Boa eval parsing can also reach the callable-declaration producer. Lila
does not expose a product dynamic-eval parser path, so that structural fact does
not justify a T07 eval integration surface.

## Verification boundary

Front-end tests cover ordinary and async declarations under both goals and keep
body/nested-generator boundaries valid. Retained-module tests cover named and
default-export declarations. The exact pinned Test262 cohort is:

- `language/statements/generators/param-dflt-yield.js`.

Its metadata expands to two sloppy/strict Wasm-AOT executions. This bounded
family does not claim generator expressions, generator methods, all
formal-parameter early errors, T07, or aggregate parser closure.

## Evidence

At `2026-08-23`, all verification ran under the repository's eight-CPU cap and
serial test/harness settings. The complete `lila-front` gate passes `63/63`,
the focused `lila-ir` early-error filter passes `3/3`, and `cargo xc` passes.
The exact pinned file above passes `2/2` sloppy/strict Wasm-AOT executions, with
every failure and non-success bucket at zero.
