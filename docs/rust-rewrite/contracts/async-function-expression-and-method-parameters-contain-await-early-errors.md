# Async function-expression and method parameter `Contains AwaitExpression` early errors

## Decision

Two async callable forms have distinct closed pre-evaluation conditions:

- `EarlyErrorCode::AsyncFunctionExpressionParametersContainAwait`
- `EarlyErrorCode::AsyncMethodParametersContainAwait`

Their sole wire spellings are
`E_ASYNC_FUNCTION_EXPRESSION_PARAMETERS_CONTAIN_AWAIT` and
`E_ASYNC_METHOD_PARAMETERS_CONTAIN_AWAIT`. The codes remain distinct because
the grammar assigns the conditions to different productions and the repaired
Boa paths have separately owned fixed messages.

## Measured parser gaps and repairs

Pinned `boa_parser-0.21.1` parses Await-enabled FormalParameters in
`parser/expression/primary/async_function_expression/mod.rs`, closes the list,
and proceeds directly to the body without applying the production's
`Contains AwaitExpression` early error. The repair adds the missing check after
the closing parenthesis and before body parsing, anchored at the existing
parameter-list position. Its fixed raw message is:

```text
await expression not allowed in async function expression parameters
```

The sole caller is AsyncFunctionExpression dispatch in
`expression/primary/mod.rs`; named and anonymous expressions share it.

`AsyncMethod` in `expression/primary/object_initializer/mod.rs` has the same
gap after parsing Await-enabled UniqueFormalParameters. Its repaired check runs
before body parsing and emits:

```text
await expression not allowed in async method definition parameters
```

Object-literal dispatch and class-element dispatch share that method parser,
covering object methods plus public/private, computed/literal, static/instance
class methods in declaration and expression forms.

Both messages are fixed and contain no user source. `LexError::Syntax` appends
the parameter position, so the classifier uses each complete raw message plus
`at line` without fixing a coordinate. The production names make the rows
pairwise disjoint from declaration, async-generator-expression and async-
generator-method parameter messages.

## Goal and containment boundary

Both repaired producers are reachable under Script and Module goals. Retained
Module parsing projects each rejection as the same typed `Early` /
`SyntaxError` diagnostic with a nonempty source span from exported expression
and object initializers. A default-export async function is a declaration and
retains the separate declaration-owned code.

The checks inspect only the callable's own parameters. AwaitExpression in the
body or inside a nested async function used as a default initializer remains a
valid containment boundary.

## Verification boundary

Front-end tests cover named/anonymous async function expressions and object,
instance and static async methods under both goals, with body and nested-async-
function positive boundaries. Retained-module tests cover one exported
initializer per code.

The pinned suite has no source that reaches either repaired post-parameter
producer. Its nearby async-function, object-method and class-method negative
files use bare or incomplete `await` forms and fail earlier. Those files are
not claimed as evidence for the repaired conditions; direct front-end and
retained-module witnesses carry that contract until the pin gains a complete
AwaitExpression case.

This bounded repair does not claim async declarations, async-generator forms,
direct eval, all async grammar, T07, or aggregate parser closure.

## Evidence

At `2026-08-23`, the shared checkpoint ran under the repository's eight-CPU
cap with serial test and harness settings:

- `cargo test -p lila-front --lib -- --test-threads=1`: `81/81`;
- `cargo test -p lila-ir --lib 'modules::early::tests::' -- --test-threads=1`:
  `37/37`;
- `cargo test -p lila-ir early_error -- --test-threads=1`: `3/3`; and
- `cargo xc`: green.

The shared exact arrow cohort passed `4/4` sloppy/strict Wasm-AOT executions
with every failure and non-success bucket at zero, but it does not reach either
producer repaired by this contract. The direct front-end and retained-module
gates own those two repairs until the pinned suite gains complete
AwaitExpression cases.
