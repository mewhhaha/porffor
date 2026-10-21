# Test262: implicit arguments expectation predates the normative hoisting fix

**Existing upstream report/fix:** [issue #5113](https://github.com/tc39/test262/issues/5113)
and [PR #5112](https://github.com/tc39/test262/pull/5112) are open as of
2026-10-21. Do not submit this local draft as a duplicate. The exact proposed
fixture passes a focused Scriptw Wasm-AOT check; current main is still unchanged.
[Search and verification details](upstream-status-20261021.md).

Affected test: `annexB/language/function-code/block-decl-func-skip-arguments.js`
(sloppy mode), upstream revision `2e0a56762801e275a9fdf96dc49d90ba0cddcf63`.

The test expects an implicit Arguments object to survive evaluation of an
eligible block function named `arguments`. Its metadata quotes the old step
that appended `"arguments"` to the parameter-name list.

[ECMA-262 ea167900](https://github.com/tc39/ecma262/commit/ea167900a06af090a1158a17b5ee04c7bcafb015),
“Normative: Perform legacy hoisting for `function arguments() {}`”, deliberately
changed this in 2017. The implicit binding belongs to parameter bindings; Annex B
eligibility checks formal parameter names. Evaluation therefore copies the
function into the existing implicit binding. An explicit formal named
`arguments` still prevents the copy.

[Minimal reproducer](../reproducers/implicit-arguments.js) prints
`["object","function"]` in Scriptw, V8 13.6 and SpiderMonkey 136a1. The installed
QuickJS 1.19.4 binding prints `["object","object"]`; that disagreement is retained.
The unchanged upstream staging `block-scoped-functions-annex-b-arguments.js`
also expects replacement. V8 and SpiderMonkey fail the original 2017 test.

[Proposed patch](test262-arguments.patch) updates the description, explanatory
algorithm and three post-block expectations. It retains pre-block and in-block
assertions and adds the explicit-formal exclusion. No execution mode is removed.
