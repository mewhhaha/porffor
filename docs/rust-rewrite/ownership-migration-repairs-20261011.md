# Ownership migration and original regression repair — 2026-10-11

The original failed scopes from `2bc1e14b7` retain their complete native test
inventories and budgets. The original failure reports are preserved with
byte identities in
`target/ownership-migration-20261011/original-failures-2bc1e14b7/manifest.json`.

| Scope | Original pass/fail | Latest completed pass/fail |
| --- | ---: | ---: |
| AOT library | 332 / 1 | 333 / 0 |
| Generated campaigns | 12 / 3 | 14 / 1 |
| Module key structure | 2 / 1 | 3 / 0 |
| Module unit structure | 0 / 3 | 3 / 0 |
| **Same 354 controls** | **346 / 8** | **353 / 1** |

Seven original failures are eliminated: the With body-growth control, the async
module and object-graph campaigns, and four Bytes/module structural assertions. The With ceiling
remains 9,000 bytes, and campaign execution retains its original five-second
transport budget. No original campaign or worker control is filtered away.

The initial ownership gates also run the complete 1,502-control IR library,
449-control Test262 library, and 16-control worker target. Their combined
inventory is 2,322 controls: 2,319 pass, three fail, and none are ignored. The
third failure is a new callback control whose oracle assertion incorrectly
inspected a diagnostic note rather than the observed completion. Its assertion
now checks the actual Boolean completion on both backends; the correction now passes on both backends.

The mutation campaign still times out. The initial object campaign executes and
exposes a graph mismatch. Comparing the same corpus against the retained older
`77d7` diagnostic proves the AOT graph is byte-for-byte identical; this mismatch
predates the current batch. It includes different permitted initial intrinsic
property chronology and the optional Boa oracle's incorrect `__proto__` setter
length 0. Neither graph ordering nor descriptor comparison is relaxed.

Original gate receipts are
`target/ownership-original-regressions-20261011/receipt.json` and
`target/ownership-original-regressions-retry-20261011/receipt.json`.
Current failure artifacts are preserved in
`target/ownership-migration-20261011/campaign-failures-current/manifest.json`.
The older/current graph identity comparison is in
`target/ownership-migration-20261011/object-probe-prior-shape-comparison.json`.

The shared-read checkpoint executes 2,907 native controls with 2,895 passes,
12 failures and no ignores. Its source fence is unchanged. The same original
354 controls still have six eliminated failures, two remaining failures and
zero new failures in those scopes. All four new Engine controls pass, including
the corrected callback harness. The full 269-control Engine target finds one
older With fixture-state failure. The two full structural targets find nine
failed assertions: five newly introduced task-to-contract marker mismatches,
and four preexisting assertions against retired inlined/map spelling. These
are tracked separately from the original failure reduction.

The With fixture declared `var caught` inside a loop without resetting it.
The preceding iteration's expected Proxy exception remained visible when the
next callback completed normally. Diagnostic replay on the same compiled
worker returns false for the original fixture and true after resetting this
variable, with the same eight print events. The fixture now resets `caught`.
The structural assertions now pin the current typed operations and shared
second-HasProperty/error/Get route. Historical finite-string verification is
preserved in a separate permanent receipt rather than checked against a
behavioral contract. Fresh native verification of these corrections is pending.

The original-budget campaigns remain red at this checkpoint. Fresh exact
mutation phase diagnostics take 6.537 seconds: lowering 0.535 seconds, emission
1.691 seconds, native module loading 1.834 seconds and execution 1.944 seconds.
The emitted program is 599,745 bytes. This diagnostic bypasses only the parent
transport watchdog for attribution and is not acceptance. The object replay
takes 4.783 seconds and retains its graph mismatch. Identity-paired diagnosis
finds three residual own-key order differences, with no descriptor/value/key-set
or prototype differences: Object constructor, Error.prototype and Error.

The optional Boa oracle adds an explicit
constructor-metadata creation point only for Object and Error, reorders Object
static method creation, and creates Error.prototype.constructor before its
methods. Graph order and descriptor comparison remain exact. The complete
original-budget object campaign now passes.

The fragment-list checkpoint executes all 3,574 selected native controls:
3,572 pass, two fail, and none are ignored, with unchanged source. All ten
additional fixture/source-contract failures from the preceding 2,907-control
gate now pass. The original 354-control comparison proves seven eliminated
failures, one remaining mutation timeout, and zero new failures in those scopes.
The additional 667 IR integration controls expose one newly introduced
task-to-contract wording assertion. Its current-contract correction is authored;
a neighboring unrun PromiseResolve guard also drops redundant retired task
wording while retaining its complete current ownership assertions.

The JSON fragment-list trial passes semantic controls but makes exact-corpus
execution slower: 2.924 seconds versus the previous 1.944 seconds. Its full
diagnostic takes 7.294 seconds and is not acceptance. That implementation is
replaced by one private geometrically grown UTF-16 buffer. Each already serialized
fragment is copied once into unused capacity; consuming publication copies only
the exact used prefix into an immutable String. No getter, replacer or JS property
is reevaluated. Empty containers, repeated growth, large individual fragments and
lone surrogates have explicit controls. This replacement awaits fresh verification.

IR integration controls now share one native executable instead of 144. All
144 source modules and 673 original controls are retained, with explicit
`ir_cases::<module>::<test>` identities. Maintained CI and contract commands
select the qualified module namespace; historical exact commands remain intact.
Actual native inventory proves a bijection for all 673 controls and every one
of the 16 CI scopes. Its 672 passes and one failed wording assertion are retained.
This reduces repeated linking and file pressure; target
count changes are not counted as fixed bugs.

Cargo artifact readers now request `json-render-diagnostics`, preserving exact
compiler-artifact selection and rendered warnings while avoiding repeated source
spans inside warning JSON. The preceding all-target/all-feature type gate proves
this format retains the complete Cargo artifact events and reports success.

Checkpoint receipts and diagnostics are:

- `target/ownership-shared-read-regressions-20261011/receipt.json`;
- `target/ownership-migration-20261011/original-regression-comparison-shared-read.json`;
- `target/ownership-migration-20261011/campaign-failures-shared-read/manifest.json`;
- `target/ownership-migration-20261011/exact-timeout-phase-diagnostics-v4/receipt.json`;
- `target/ownership-migration-20261011/with-control-diagnostic/receipt.json`; and
- `target/ir-integration-grouping-20261011/qualified-inventory-map.json`.

The latest completed evidence is
`target/ownership-fragment-regressions-20261011/receipt.json`,
`target/ownership-migration-20261011/original-regression-comparison-fragments.json`,
`target/ir-integration-grouping-20261011/native-inventory-verification.json`,
`target/ownership-migration-20261011/campaign-failures-fragments/manifest.json`
and `target/ownership-migration-20261011/exact-timeout-phase-diagnostics-v5/receipt.json`.

Permanent [ownership](conformance-ownership.md) and
[acceptance](conformance-acceptance.md) contracts now replace implementation
and test dependencies on task prose. The task ledger remains until real
acceptance. Full workspace native coverage, fresh pinned Test262 census and
closure, remaining semantic repairs, and final publication are still open.
