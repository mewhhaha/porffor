# Existing upstream work — checked 2026-10-21

Do not submit the arguments draft as a new report: its exact issue and proposed
fix already exist upstream. No report or comment was posted during this check.

| Problem | Existing upstream work | Current disposition |
| --- | --- | --- |
| Implicit `arguments` | [Test262 issue #5113](https://github.com/tc39/test262/issues/5113) and [fix PR #5112](https://github.com/tc39/test262/pull/5112) | Both open. The PR updates this exact fixture's three post-block expectations and adds an explicit-formal control. It is not merged into main. |
| Johnston/Honolulu, both modes | [ECMA-402 issue #806](https://github.com/tc39/ecma402/issues/806) | Open, labeled `s: blocked`. Its comparison table explicitly discusses Johnston/Honolulu divergence. It is a broad alignment discussion, not a targeted fix for the missing backzone-Zone branch. |
| Labelled block/case functions | [ECMA-262 issue #2663](https://github.com/tc39/ecma262/issues/2663), closed by [PR #3361](https://github.com/tc39/ecma262/pull/3361) | This related traversal clarification merged in July 2024 and is already in our specification pin. It restricts traversal to the current function body; it does not resolve the label-chain/outer-copy disagreement. No exact issue or fix PR for that disagreement was found in the searches. |

## Arguments: follow the existing fix

PR #5112's head was `1f3d994e2658c2934a64781ffbb7a9b3782d3310`.
The [exact proposed fixture](https://github.com/tc39/test262/blob/1f3d994e2658c2934a64781ffbb7a9b3782d3310/test/annexB/language/function-code/block-decl-func-skip-arguments.js)
was fetched and executed as a standalone sloppy Script with the original
Test262 `sta.js` and `assert.js` harness. It passes Scriptw's actual Wasm-AOT
execution path (exit 0). The unchanged original fixture under the same harness
fails (exit 1), providing a negative control. This is a focused fixture check,
not a suite publication or a change to our source pins.

The old specification report [ecma262 #815](https://github.com/tc39/ecma262/issues/815)
was resolved by [normative PR #891](https://github.com/tc39/ecma262/pull/891)
in 2017. The separate [ecma262 #991](https://github.com/tc39/ecma262/issues/991)
remains open and also points out the incorrect Test262 assertion. Our local
arguments draft duplicates existing work; track #5112/#5113 instead.

## Timezone: distinguish the existing rule from the remaining edge

[ECMA-402 issue #825](https://github.com/tc39/ecma402/issues/825) was closed by
[PR #877](https://github.com/tc39/ecma402/pull/877), merged 2024-08-01. That is
the general primary/non-primary identifier rule, including country preservation.
The implementing commit is `602aaf0b408d3cb427f1598fe079ae3d54ed8b8c`; it is
already part of the observed specification. **825 is an issue, not the PR.**
[Test262 PR #4437](https://github.com/tc39/test262/pull/4437), merged in March
2025, adds coverage for the general #877 requirements.

Issue #806 explicitly mentions Johnston, but remains open. The inspected work
does not establish an accepted correction for the particular backzone-Zone
versus algorithm-Link case. No Johnston-specific Test262 fix issue/PR surfaced
in the named searches. Consult that existing discussion before deciding whether
a narrowly scoped additional report is warranted.

## Labels: related work is not the missing fix

PR #3361 merged as `5885b6c175dc63fa47eb6dfdb1fd440fd7c703ca`. It added the
`code Contains x` restriction to stop scanning unrelated nested functions.
The direct-containment restriction and label lexical collection remain.
[PR #2952](https://github.com/tc39/ecma262/pull/2952) is also merged and already
included; its Annex B restructuring does not supply the proposed label behavior.
[Issue #1595](https://github.com/tc39/ecma262/issues/1595) remains an open,
broader Annex B reform tracker. These are useful context, not an exact existing
resolution of the failing original staging fixture.

## Current source and search coverage

Observed upstream heads:

- Test262: `d53ebc8b1931d9f1541e43d59650776e4b766248`
- ECMA-262: `4feb0272b799ea795c5eadee9b482aea0c10ce45`
- ECMA-402: `730ca2b88c305862db7be2f83c510dcca33ab48c`

All three original failing test files fetched at the Test262 head above are
byte-identical to our vendored copies. Merely updating to current main therefore
does not remove these four failures; the arguments fix is still on an open PR.

Searches covered public GitHub issues and pull requests, open and closed, in
`tc39/test262`, `tc39/ecma262`, `tc39/ecma402`, and relevant Temporal work.
Terms included the exact filenames, `function arguments`, `legacy hoisting`,
`Johnston`, `Honolulu`, `backzone`, `labelled`/`labeled`, `LabelledItem`,
`LexicallyScopedDeclarations`, and `directly contained`. Relevant linked threads
and merged changes were inspected. Negative findings are limited to this search
coverage; they are not proof that no other discussion exists.
