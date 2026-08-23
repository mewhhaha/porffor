# Class-static-block `ContainsAwait` early errors

## Decision

A class static block whose statement list contains an `AwaitExpression` is one
closed pre-evaluation condition:

`EarlyErrorCode::ClassStaticBlockContainsAwait`

Its sole wire spelling is `E_CLASS_STATIC_BLOCK_CONTAINS_AWAIT`. Script and
Module parsing report the condition during the early-error phase as a
`SyntaxError`, with the parser's source span. Rejected source never reaches IR
lowering or static-block execution.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has one producer in
`parser/statement/declaration/hoistable/class_decl/mod.rs`. It constructs the
exact message `invalid await usage`; Boa's `Error::General` display renders
that as `invalid await usage at line L, col C` before Lila classifies it.

The adjacent fragment `invalid await usage at line` is deliberately the
classifier key. Boa also has a distinct producer whose rendered message is
`invalid await usage in generator function parameters at line L, col C`.
Matching only the shorter bare prefix would incorrectly merge those two
grammar conditions. Contextual `await` identifier, binding, label and object-
shorthand restrictions likewise remain separate producers and codes.

## Traversal boundary

`ContainsAwait` inspects the retained static-block statement list. It does not
descend into nested ordinary or arrow function bodies, including async
functions whose bodies legitimately contain `await`. Those positive controls
are part of the contract; this classification must never become a source-text
scan for the token `await`.

## Verification boundary

Front-end tests cover declaration and expression forms under Script and Module
goals, positive nested async ordinary/arrow functions, and the longer generator-
parameter message anti-collision. A retained Module parse must project the real
failure as `Early`, `SyntaxError`, the same code, and a source span.

The exact pinned Test262 witness is
`language/statements/class/static-init-invalid-await.js`. Its metadata expands
to two sloppy/strict Wasm-AOT executions. That observable test may already pass
under the generic malformed-parse bucket; the repaired contract is the typed
condition and retained-module projection, not a claim of newly visible runtime
behavior or broad T07 closure.

At `2026-08-23`, the capped serial `lila-front` gate passes `49/49`, the
focused `lila-ir` early-error gate passes `3/3`, and the exact pinned witness
passes `2/2` Wasm-AOT executions. Every parser, early-error, lowering, runtime,
backend, harness, unsupported, crash and bug bucket is zero for that witness.
