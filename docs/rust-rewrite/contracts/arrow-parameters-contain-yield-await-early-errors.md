# Arrow-parameter `Contains YieldExpression` / `Contains AwaitExpression` early errors

## Decision

Arrow parameter containment is two closed pre-evaluation conditions:

- `EarlyErrorCode::ArrowParametersContainYield`
- `EarlyErrorCode::ArrowParametersContainAwait`

Their sole wire spellings are `E_ARROW_PARAMETERS_CONTAIN_YIELD` and
`E_ARROW_PARAMETERS_CONTAIN_AWAIT`. Each code covers ordinary and async arrow
forms because the typed domain is keyed by the closed static-semantics
condition, not by a parser branch. Pinned Boa emits two Yield wordings and one
Await wording; separate classifier rows map that producer variation back to
the two conditions instead of inventing syntax-form-specific codes.

## Measured parser boundary

Pinned `boa_parser-0.21.1` uses three fixed, case-sensitive raw messages across
the arrow paths:

```text
yield expression is not allowed in formal parameter list of arrow function
Yield expression not allowed in this context
Await expression not allowed in this context
```

- `expression/primary/mod.rs` converts a parenthesized cover expression into
  ordinary-arrow parameters and rejects contained Yield before returning the
  list;
- `expression/assignment/mod.rs` owns the parenthesized ordinary-arrow
  post-parameter checks; its Await producer is live, while its Yield producer
  is preceded by the primary-expression conversion check;
- `arrow_function.rs` retains the same checks, although its product caller
  currently dispatches only an unparenthesized single `BindingIdentifier`, so
  an expression cannot reach those two sites; and
- `async_arrow_function.rs` owns the parenthesized async-arrow checks.

Boa's rendered errors append the parameter position, so the classifier rows
use the complete raw messages plus `at line` without fixing a coordinate. The
messages contain no user source. The two Yield wordings require separate rows
for the same typed condition; their case and complete phrases remain disjoint
from generator-expression and method messages.

The async-arrow parser already stores the enclosing `AllowYield` grammar
parameter and uses it for its unparenthesized binding path, but its
parenthesized FormalParameters path hardcodes Yield off. That makes the
existing YieldExpression containment check unreachable and can misread a
Yield-enabled outer context. The vendored repair passes `self.allow_yield` to
FormalParameters. It changes no accepted parameter form: a contained
YieldExpression is immediately rejected by the existing early-error check.

## Goal and containment boundary

The conditions are reachable under Script and Module goals when an arrow is
nested in the corresponding generator or async context. Direct async-arrow
parameters are Await-enabled by their own grammar. Retained Module parsing
projects the same typed `Early` / `SyntaxError` diagnostic with a nonempty span
for exported enclosing functions and arrow initializers.

The checks inspect the arrow's own parameter list. Yield/Await use inside a
nested generator or async function in a default initializer remains outside
that `Contains` traversal, and AwaitExpression in an async-arrow body remains
valid.

## Verification boundary

Front-end tests cover the live ordinary and async-arrow producer paths under
both goals, plus nested-callable and body boundaries. Retained-module tests
cover exported enclosing generator/async functions and a direct async-arrow
initializer.

The exact pinned Test262 code cohort for these typed conditions is:

- `language/expressions/arrow-function/param-dflt-yield-expr.js`
- `language/expressions/async-arrow-function/await-as-param-nested-arrow-body-position.js`

Both files exercise parenthesized ordinary arrows. The Yield file reaches the
lowercase conversion producer in `expression/primary/mod.rs`; the async-arrow-
named Await file places an ordinary arrow inside an async-arrow body and reaches
the uppercase producer in `expression/assignment/mod.rs`. The pinned suite has
no source that reaches the repaired parenthesized async-arrow Yield producer,
so direct front-end and retained-module witnesses own that repair. Neither
cohort file has a mode flag, so the two-file code cohort expands to four sloppy/
strict Wasm-AOT executions. Nearby strict-identifier and bare/incomplete-
keyword files fail in earlier grammar branches and are not claimed as evidence
for these codes.

This bounded family does not claim every arrow early error, direct eval, all
callable grammar, T07, or aggregate parser closure.

## Evidence

At `2026-08-23`, the shared checkpoint ran under the repository's eight-CPU
cap with serial test and harness settings:

- `cargo test -p lila-front --lib -- --test-threads=1`: `81/81`;
- `cargo test -p lila-ir --lib 'modules::early::tests::' -- --test-threads=1`:
  `37/37`;
- `cargo test -p lila-ir early_error -- --test-threads=1`: `3/3`; and
- `cargo xc`: green.

The exact two-file pinned cohort passed `4/4` sloppy/strict Wasm-AOT
executions with every failure and non-success bucket at zero. That cohort owns
the ordinary-arrow paths only; the direct front-end and retained-module gates
own the repaired async-arrow Yield producer.
