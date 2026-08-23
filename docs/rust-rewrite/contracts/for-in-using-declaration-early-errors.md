# For-in using-declaration early errors

## Decision

A `for-in` head whose lexical declaration is `using` or `await using` is one
closed pre-evaluation condition:

`EarlyErrorCode::ForInUsingDeclaration`

Its sole wire spelling is `E_FOR_IN_USING_DECLARATION`. Both declaration forms
share the code because pinned Boa selects them through one exhaustive match
after recognizing the one-binding, initializer-free iterable-loop head.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has one producer in
`parser/statement/iteration/for_statement.rs`. Its fixed raw message is:

```text
using declarations are not allowed in for-in loop heads
```

`LexError::Syntax` appends the source position. The classifier therefore uses
the exact fixed text plus `at line`, without hard-coding coordinates. The
message contains no user source. A broader `using declarations are not allowed`
fragment is forbidden because Boa has distinct top-level Script, direct-eval
and statement-list diagnostics.

## Goal and grammar boundary

The producer is goal-independent. Ordinary `using` reaches it under Script and
Module goals; `await using` reaches it inside an async function under both
goals. Pinned Boa direct eval can also reach the statement-local producer, but
Lila has no product direct-eval parser path; that remains T13.

`for-of` is the positive sibling: both declaration forms remain valid there.
`let` and `const` remain valid `for-in` heads, and initialized `using` remains
valid in classic `for`. An initialized `using` iterable head is a distinct
earlier parser condition and must not be classified by this code.

Retained Module parsing must project the same typed `Early`/`SyntaxError`
diagnostic for ordinary and async-function forms, with a source span.

## Verification boundary

Front-end tests cover both declaration forms under both goals and preserve the
positive grammar siblings. Retained-module tests cover both reachable forms.
The exact pinned negative cohort is:

- `language/statements/using/syntax/using-invalid-for-in.js`;
- `language/statements/await-using/syntax/await-using-invalid-for-in.js`.

Their metadata expands to four sloppy/strict Wasm-AOT executions. This bounded
family does not claim resource disposal execution, direct eval, all iterable-
loop grammar, T07 or aggregate closure.

## Evidence

At `2026-08-23`, all verification ran under the repository's eight-CPU cap and
serial test/harness settings. The complete `lila-front` gate passes `59/59`,
the focused `lila-ir` early-error filter passes `3/3`, and `cargo xc` passes.
The two exact pinned files above each pass `2/2` sloppy/strict Wasm-AOT
executions, for `4/4` total, with every failure and non-success bucket at zero.
