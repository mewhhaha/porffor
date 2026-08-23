# Generator-method parameter `Contains` early errors

## Decision

Generator-method parameter containment is three closed pre-evaluation
conditions:

- `EarlyErrorCode::GeneratorMethodParametersContainYield`
- `EarlyErrorCode::AsyncGeneratorMethodParametersContainYield`
- `EarlyErrorCode::AsyncGeneratorMethodParametersContainAwait`

Their sole wire spellings are, respectively:

- `E_GENERATOR_METHOD_PARAMETERS_CONTAIN_YIELD`
- `E_ASYNC_GENERATOR_METHOD_PARAMETERS_CONTAIN_YIELD`
- `E_ASYNC_GENERATOR_METHOD_PARAMETERS_CONTAIN_AWAIT`

The three conditions share one parser subsystem, but not one diagnostic code.
Pinned Boa has a distinct fixed producer message for each condition, and the
closed Rust enum preserves that ownership. Generator and async-generator
expressions, plus declaration forms, retain their separate producer-owned
codes.

## Measured parser boundary

Pinned `boa_parser-0.21.1` owns all three checks in
`parser/expression/primary/object_initializer/mod.rs`. The fixed raw messages
are:

```text
yield expression not allowed in generator method definition parameters
yield expression not allowed in async generator method definition parameters
await expression not allowed in async generator method definition parameters
```

`LexError::Syntax` appends the parameter-list position, so each classifier row
uses its complete fixed text plus `at line` without fixing a coordinate. None
of the messages contains user source. The rows are pairwise disjoint: the
ordinary and async-generator messages differ, as do the async-generator yield
and await messages. They cannot overlap the existing expression rows because
those say `expression parameters`, not `method definition parameters`.

Object-literal and class-element dispatch both call the same method parsers.
One typed condition therefore covers public, computed and private class method
names, static and instance placement, and class declaration/expression forms
without copying parser-shape detail into the code domain.

## Goal and containment boundary

All three producers are reachable under Script and Module goals. Retained
Module parsing projects each rejection as the same typed `Early` /
`SyntaxError` diagnostic with a nonempty source span when an exported binding
contains the method.

`Contains` stops at nested function boundaries. Generator and async-generator
methods may use `yield` or `await` in their bodies as their grammar permits,
and a default initializer may contain the same expression inside a nested
generator or async function. Only the expression contained by the method's own
FormalParameters is rejected.

Pinned Boa eval parsing can also reach these method parsers. Lila has no
product dynamic-eval parser path, so that structural fact does not justify a
T07 eval integration surface.

## Verification boundary

Front-end tests cover object and class methods under both goals and preserve
body and nested-function boundaries. Retained-module tests cover exported
object initializers for all three conditions. The exact pinned Test262 cohort
that reaches the ordinary generator-method yield producer is:

- `language/expressions/object/method-definition/generator-param-init-yield.js`
- `language/expressions/class/gen-method-param-dflt-yield.js`
- `language/expressions/class/static-gen-method-param-dflt-yield.js`
- `language/statements/class/gen-method-param-dflt-yield.js`
- `language/statements/class/static-gen-method-param-dflt-yield.js`

Their metadata expands to nine Wasm-AOT executions: the object-literal file is
`noStrict`, while the four class files each run in sloppy and strict variants.
The pinned suite has no source that reaches either async-generator-method fixed
producer, so those two codes rely on direct front-end and retained-module
witnesses rather than claiming unrelated bare-keyword tests.

This bounded family does not claim declaration or expression forms, direct
eval, all generator grammar, T07, or aggregate parser closure.

## Evidence

At `2026-08-23`, all verification ran under the repository's eight-CPU cap and
serial test/harness settings. The complete `lila-front` gate passes `75/75`,
the retained-module early suite passes `35/35`, the focused `lila-ir` early-
error gate passes `3/3`, and `cargo xc` passes. The exact five-file ordinary
generator-method cohort passes `9/9` Wasm-AOT executions, with every failure
and non-success bucket at zero. As recorded above, the pinned suite has no
source that reaches either async-generator-method producer, so no Test262
result is claimed for those two conditions.
