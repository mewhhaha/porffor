# Script top-level `new.target` early errors

## Decision

A ScriptBody whose StatementList `Contains NewTarget` is one closed
pre-evaluation condition:

`EarlyErrorCode::ScriptTopLevelNewTarget`

Its sole wire spelling is `E_SCRIPT_TOP_LEVEL_NEW_TARGET`. “Top-level” names
the ScriptBody static-semantics boundary, not literal syntax depth:
`Contains NewTarget` traverses arrow functions because they inherit
`new.target` lexically, while ordinary, async and generator functions are
traversal boundaries.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has one producer in `parser/mod.rs`. It emits raw
message `invalid new.target usage` at the fixed `Position::new(1, 1)`, which
`Error::General` renders exactly as:

```text
invalid new.target usage at line 1, col 1
```

One full-message classifier row owns that producer. The wording and fixed
position are disjoint from the existing Module condition, whose producer emits
``module cannot contain `new.target` on the top-level`` and remains
`EarlyErrorCode::ModuleTopLevelNewTarget`.

## Goal and traversal boundary

Direct Script `new.target` and a use reached through a top-level arrow reject.
Use inside an ordinary function remains valid, including arrows nested inside
that function because their lexical `new.target` has the function environment
to inherit. Constructor, method and class-static-block uses also remain valid.

This code is honestly Script-only. Retained dependencies parse as Module and
cannot produce it. A retained Module containing top-level `new.target` must
continue to report `ModuleTopLevelNewTarget`; retained exported functions and
their nested arrows may use `new.target` successfully. Direct-eval exceptions
belong to T13 and are outside this boundary.

## Verification boundary

Front-end tests cover direct and arrow-carried Script rejection plus function,
nested-arrow, class and static-block positive controls. IR retained-module
controls prove that Module rejection keeps its existing code and valid exported
function boundaries parse without inventing a retained producer for the new
Script-only code.

The exact pinned negative cohort is `language/global-code/new.target.js` and
`language/global-code/new.target-arrow.js`. Their metadata expands to four
sloppy/strict Wasm-AOT executions. Assignment/update-target negatives fail
through different parser conditions and are excluded. This bounded family does
not claim all `new.target` grammar, direct eval, T07 or aggregate closure.

At `2026-08-23`, capped serial verification passes the complete front-end gate
at `55/55`, the focused IR early-error gate at `3/3`, and the exact two-file
cohort at `4/4` Wasm-AOT executions. Every failure and non-success bucket is
zero. The workspace `cargo xc` check is also green.
