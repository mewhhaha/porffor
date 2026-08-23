# Script top-level using-declaration early errors

## Decision

A ScriptBody containing an immediate top-level `UsingDeclaration` or
`AwaitUsingDeclaration` is one closed pre-evaluation condition:

`EarlyErrorCode::ScriptTopLevelUsingDeclaration`

Its sole wire spelling is `E_SCRIPT_TOP_LEVEL_USING_DECLARATION`. The code owns
pinned Boa's single `contains_top_level_using_declaration` producer. That
predicate structurally matches both closed lexical-declaration variants, so a
future parser-reachability repair does not need a second diagnostic code.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has one conditional producer in `parser/mod.rs`. The
ordinary Script branch emits raw message
`` `using` declarations are not allowed at the top level of scripts `` at the
fixed `Position::new(1, 1)`, rendered exactly as:

```text
`using` declarations are not allowed at the top level of scripts at line 1, col 1
```

One full-message classifier row owns that reachable product producer. Pinned
Boa's ordinary Script grammar currently reaches it for top-level `using`; it
rejects top-level `await using` earlier as an ordinary expression-statement
parse error. The latter remains an honest parse-phase `SyntaxError`, not a
fabricated typed classification. The row is deliberately narrower than Boa's
distinct statement-list and for-in-head messages. The adjacent direct-eval
branch is unreachable from Lila's ordinary Script parser entry and belongs to
T13 dynamic-source evaluation.

## Goal and nesting boundary

The scan examines immediate ScriptBody statement-list items. Top-level `using`
and `await using` reject under the Script goal, while declarations inside a
block, function, async function, classic-for head, for-of head or class static
block remain parse-valid.

This code is honestly Script-only. Retained dependencies parse as Module, where
both top-level declaration forms are allowed; no retained negative producer may
be invented. Retained positive controls must keep ordinary top-level `using`
and top-level `await using` parse-valid under the Module goal.

## Verification boundary

Front-end tests cover the typed top-level `using` rejection and the nested
Script boundaries. IR retained-module controls prove both top-level forms
remain valid Module syntax. The exact pinned negative cohort separately checks
the typed `using` producer and Boa's earlier parse rejection for `await using`:

- `language/statements/using/syntax/using-not-allowed-at-top-level-of-script.js`;
- `language/statements/await-using/syntax/await-using-not-allowed-at-top-level-of-script.js`.

Their metadata expands to four sloppy/strict Wasm-AOT executions. Passing the
`await using` file does not claim that Boa reaches the shared post-parse
predicate for that form. The runtime direct-eval tests and other using-
declaration syntax conditions are excluded. This bounded family does not claim
disposal execution, direct eval, T07 or aggregate closure.

## Evidence

At `2026-08-23`, all verification ran under the repository's eight-CPU cap and
serial test/harness settings. The complete `lila-front` gate passes `57/57`,
the focused `lila-ir` early-error filter passes `3/3`, and `cargo xc` passes.
The two exact pinned files above each pass `2/2` sloppy/strict Wasm-AOT
executions, for `4/4` total, with every failure and non-success bucket at zero.
