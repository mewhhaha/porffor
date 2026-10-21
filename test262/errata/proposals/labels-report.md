# ECMA-262: labelled block-function collection differs from deployed behavior

Affected Test262 test:
`staging/sm/lexical-environment/block-scoped-functions-annex-b-label.js`
(sloppy mode). Its original fixture was imported in
[Test262 6ee9579](https://github.com/tc39/test262/commit/6ee9579acbd6ca0418a01b08c962b23ecff697f5).

The complete original fixture passes V8 13.6 and SpiderMonkey 136a1. Standalone
probes also show agreement between those engines and the installed QuickJS
1.19.4 binding on the three disputed block/case behaviors:

| Probe | Scriptw | V8 / SpiderMonkey / QuickJS |
| --- | --- | --- |
| Chained block label: before / after / outside | ReferenceError / ReferenceError / ReferenceError | 42 / 42 / 42 |
| Single block label: before / after / outside | 42 / 42 / ReferenceError | 42 / 42 / 42 |
| Switch: before / after selected declaration / outside | 42 / 42 / ReferenceError | 42 / 42 / 42 |

The pinned specification instead says:

- `LexicallyDeclaredNames` and `LexicallyScopedDeclarations` return empty for
  `LabelledItem : Statement`; an inner label therefore stops collection.
- An immediate `LabelledItem : FunctionDeclaration` does contribute a lexical
  binding. Scriptw initializes that binding at block/CaseBlock entry.
- Ordinary `VarScopedDeclarations` returns empty for a labelled function.
- Annex B's function, global and eval insertion loops select declarations
  directly contained in block/case/default statement lists, excluding labels.

`TopLevelVarScopedDeclarations` unwraps labels, but cannot lift a function out
of an enclosing block. The
[2025 labelled-function change](https://github.com/tc39/ecma262/commit/766b80c263923ee9cf67587616c90fe6513871c2)
was editorial and did not change these collectors. Current main at
`4feb0272b799ea795c5eadee9b482aea0c10ce45` retains the disputed wording.

[Proposed normative patch](ecma262-labelled-functions.patch): recurse through
label chains in both lexical collectors and extend all three Annex B candidate
loops to functions reached solely through labels. Preserve the existing early
error, strictness, formal-parameter and environment checks, and the existing
copy-at-evaluation rule. This is a proposed change to the contract, not an
editorial restatement of the current algorithm.

If accepted, implement the same shared selectors in Scriptw's declaration,
capture/storage and eval-environment planning, then run the original fixture
plus strict syntax, shadowing, parameter, catch/with, switch and eval controls.
The existing literal-spec label-test amendment must be retired for that route.
