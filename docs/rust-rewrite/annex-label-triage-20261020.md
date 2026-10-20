# Remaining labelled-function case — 2026-10-20

The completed original-pin baseline is **102,978/102,994 passing**, with
16 bugs, zero crashes and zero unimplemented executions. The global-prelude
compiler repair passes all twelve affected original executions in focused
verification. Three other executions pass only with the separately reviewed
fixture corrections. **One additional execution remains open**:

`sloppy-script:staging/sm/lexical-environment/block-scoped-functions-annex-b-label.js`.

The global-prelude candidate still fails that original test with an unbound
identifier. Splitting its thirteen literal `expectSloppyPass`/`expectSyntaxError`
calls into independent diagnostic inputs gives **10 passes and 3 failures**.
These are diagnostic inputs, not thirteen new upstream Test262 executions.
Node 24.19.0 / V8 13.6.233.17-node.51 passes all thirteen calls and the unchanged
original test.

The three failing expectations are a two-label function inside a block, a
single-labelled block function referenced after leaving its block, and a
single-labelled switch function referenced after leaving the switch.
The pinned static semantics distinguish a direct `LabelledItem : FunctionDeclaration`
from `LabelledItem : Statement`: the latter contributes no lexical declarations.
Annex B's outer-copy loops separately select functions directly contained in
block/case statement lists. Existing IR tests deliberately preserve these
distinctions. Top-level label chains follow different declaration rules and pass.

This is an open disagreement between the pinned algorithms and an imported
SpiderMonkey staging test whose `esid` and description are `pending`. V8's
different behavior is recorded, rather than treated as proof that either the
test or compiler should be changed without further review. No compiler or
vendor-test edit was made for this case, and it is not skipped or reclassified
as passing. T24/T07/T03 own the Annex B/static-declaration/conformance review.

[Exact diagnostic results and source identities](annex-label-triage-20261020.json) ·
[Original test, isolated inputs, native/V8 results and pinned specification](../../test262/snapshots/annex-label-triage-20261020.tar.gz) ·
[Compiler repair](global-prelude-hosts-fix-20261020.md) ·
[Explicit fixture corrections](test262-errata-20261020.md).

The README chart retains the complete pre-fix compiler's sixteen literal
failures. Focused fixes are not transferred into that measurement. The next
steps are to resolve this labelled-function disagreement and run a fresh full
campaign on the integrated compiler. No complete green fixed-source campaign
has been demonstrated, and the broader native/release acceptance gates remain
open.
