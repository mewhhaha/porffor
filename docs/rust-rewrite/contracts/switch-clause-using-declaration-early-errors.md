# Switch-clause using-declaration early errors

## Decision

A `CaseClause` or `DefaultClause` whose direct StatementList item is a `using`
or `await using` declaration is one closed pre-evaluation condition:

`EarlyErrorCode::SwitchClauseUsingDeclaration`

Its sole wire spelling is `E_SWITCH_CLAUSE_USING_DECLARATION`. Both clause
kinds and both declaration forms share the code because pinned Boa routes them
through one fixed producer and exactly two callers that disable direct using
declarations.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has one producer in `parser/statement/mod.rs`. Its
fixed raw message is:

```text
`using` declarations are not allowed in this statement list
```

The only `with_allow_using_declarations(false)` call sites are the CaseClause
and DefaultClause StatementLists in `parser/statement/switch/mod.rs`.
`Error::General` appends the declaration position, so the classifier uses the
complete fixed text plus `at line` without hard-coding coordinates. The message
contains no user source and is disjoint from Boa's top-level, direct-eval and
for-in using restrictions.

## Goal and containment boundary

The producer is goal-independent. Ordinary `using` reaches it under Script and
Module goals; `await using` reaches it in an async-function switch under both.
Pinned Boa eval parsing can also reach this statement-local producer, but Lila
has no product eval parser path; that remains T13.

The restriction is direct clause containment. A declaration inside a nested
block, classic `for`, `for-of`, or nested function remains valid. Direct `let`
and `const` clause declarations also remain valid. These boundaries prevent the
condition from widening into “any using declaration beneath a switch.”

Retained Module parsing must project the same typed `Early`/`SyntaxError`
diagnostic for both clause kinds and declaration forms, with a source span.

## Verification boundary

Front-end tests cover both clauses and declaration forms under both goals, plus
the nesting and ordinary-lexical positives. Retained-module tests cover an
ordinary case clause and an async default clause. The exact pinned cohort is:

- `language/statements/using/syntax/using-invalid-switchstatement-caseclause.js`;
- `language/statements/using/syntax/using-invalid-switchstatement-defaultclause.js`;
- `language/statements/await-using/syntax/await-using-invalid-switchstatement-caseclause.js`;
- `language/statements/await-using/syntax/await-using-invalid-switchstatement-defaultclause.js`.

Their metadata expands to eight sloppy/strict Wasm-AOT executions. This bounded
family does not claim resource disposal execution, direct eval, all switch
grammar, T07 or aggregate closure.

## Evidence

At `2026-08-23`, all verification ran under the repository's eight-CPU cap and
serial test/harness settings. The complete `lila-front` gate passes `61/61`,
the focused `lila-ir` early-error filter passes `3/3`, and `cargo xc` passes.
The four exact pinned files above each pass `2/2` sloppy/strict Wasm-AOT
executions, for `8/8` total, with every failure and non-success bucket at zero.
