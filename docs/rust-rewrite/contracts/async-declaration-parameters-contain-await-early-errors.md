# Async-declaration parameter `Contains AwaitExpression` early errors

## Decision

An `AsyncFunctionDeclaration` or `AsyncGeneratorDeclaration` whose
`FormalParameters Contains AwaitExpression` is one closed pre-evaluation
condition:

`EarlyErrorCode::AsyncDeclarationParametersContainAwait`

Its sole wire spelling is
`E_ASYNC_DECLARATION_PARAMETERS_CONTAIN_AWAIT`. Ordinary async and
async-generator declarations share the code because pinned Boa routes both
through one fixed producer after their declaration parsers opt into the same
check.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has one producer in
`parser/statement/declaration/hoistable/mod.rs`. Its fixed raw message is:

```text
invalid await usage in generator function parameters
```

`AsyncFunctionDeclaration` and `AsyncGeneratorDeclaration` both set
`parameters_await_is_early_error()` and call that common declaration parser.
The pinned wording says `generator function` even for the ordinary async
declaration caller; the classifier follows the measured producer rather than
inventing a normalized message. `LexError::Syntax` appends the parameter-list
position, so the row uses the complete fixed text plus `at line` without
fixing a coordinate. The message contains no user source.

Async-generator expressions and async-generator methods remain outside this
code because their parser paths use distinct fixed messages. Ordinary async-
function expressions and ordinary async methods likewise retain separate
typed codes and repaired producer messages; none is folded into this
declaration-owned condition.

## Goal and containment boundary

The declaration producer is reachable under Script and Module goals for both
declaration forms. Retained Module parsing projects the same typed `Early` /
`SyntaxError` diagnostic, including default exports, with a nonempty source
span.

`Contains` stops at a nested async-function boundary. An async declaration may
use `await` in its body or inside a nested async function used as a parameter
initializer; only an `AwaitExpression` contained by its own FormalParameters
is rejected. A bare `await` binding or incomplete unary-looking form is not a
witness because pinned Boa rejects it earlier through a different parser
branch.

Pinned Boa eval parsing can also reach the declaration helper. Lila has no
product dynamic-eval parser path, so that structural fact does not justify a
T07 eval integration surface.

## Verification boundary

Front-end tests cover ordinary async and async-generator declarations under
both goals and preserve body and nested-async-function boundaries. Retained
Module tests cover named and default-export declarations.

The pinned suite has no file whose source reaches this exact fixed producer.
The nearby async-function formal-parameter files use bare or incomplete
`await` forms and fail earlier, so they are not claimed as evidence for this
code. This family therefore relies on direct front-end and retained-module
witnesses until the pinned suite contains a producer-reaching case.

This bounded family does not claim expression or method forms, direct eval,
all async grammar, T07, or aggregate parser closure.

## Evidence

At `2026-08-23`, all verification ran under the repository's eight-CPU cap and
serial test/harness settings. The complete `lila-front` gate passes `75/75`,
the retained-module early suite passes `35/35`, the focused `lila-ir` early-
error gate passes `3/3`, and `cargo xc` passes. As recorded above, the pinned
suite has no source that reaches this exact producer, so no Test262 result is
claimed for this condition.
