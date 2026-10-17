Current Engine verification (2026-10-17): all965 Engine tests pass with the
empty-strict-body compiler, retaining every prior literal test ID. Zero tests fail,
are ignored or filtered. Product dependencies retain their selected optimized
profiles; the complete unfiltered test run took 986.1s.
[Exact inventory and raw evidence](docs/rust-rewrite/empty-strict-body-engine-inventory-20261017.json) bind this result to the same compiler as
the407 AOT tests,8 template integration tests and retained1,599-test IR evidence.
Remaining conformance cohorts, ordinary Cargo and task closure remain open.

# Continue Lila — 2026-10-17

The task is open. Delete this file and `tasks/` only after every acceptance gate
passes; native subsets and inventories do not establish full conformance.

Current compiler checkpoint (2026-10-17): the official Annex B implicit-
arguments regression passes through Wasm-AOT. All26 earlier stress passes are
retained across the original27 executions at the unchanged60000ms deadline:
26Success/1Crash. The strict class case passes in the isolated two-case check but still
times out in the mixed campaign; its reliable repair remains open.
Typed empty strict source bodies omit unobservable binding setup while retaining
call-entry validation, activation tracking and cleanup. A representative generated
module shrinks from92,562 to79,132 bytes (14.5%).
All407 AOT tests and8 native-template integration tests pass without ignored or
filtered tests. The earlier1,599-test IR pass is reused after verifying every398
consumed source files and11 typed extern dependencies are unchanged; it was not
rerun. [Exact results and raw evidence](docs/rust-rewrite/empty-strict-body-and-annex-b-checkpoint-20261017.json) bind these scoped results to the
current compiler. Full Engine and other current native inventories, ordinary Cargo,
full pinned conformance and task closure remain open. CI now caches unchanged
workspace artifacts as well as dependencies; measured CI timing is still pending.

The earlier literal-operands compiler passed all965 Engine tests; that complete
Engine result retains its earlier compiler identity. The preceding complete2,496-test
native inventory and template-count measurements also remain earlier checkpoints.
The original27-case replay retains all26 earlier passes; the strict class case still
needs a reliable deadline repair. Remaining203 and historical5365 executions and
all full acceptance gates still require completion.

## Immediate work

1. Fix `strict-script:staging/sm/expressions/destructuring-array-default-class.js`
   without increasing its deadline or dropping any of the26 current passes.
2. Complete the current-source native library/integration inventories after
   this product change. The complete2,496-test predecessor checkpoint is at
   `docs/rust-rewrite/complete-current-native-inventories-20261017.json`;
   its compiler identity differs from this checkpoint.
3. Establish ordinaryCargo default/all-feature/all-target and actualCLI
   acceptance. Keep exact compiler identity and complete literal inventories.
4. Replay the remaining203 original executions and all5,365 historical IDs,
   preserving every earlier pass. Historical inputs exist in
   `target/historical-backlog-replay-input-20261012/executions` (SHA256
   `95d46445f4c3fda986c7cdf390dccdddfb8ab370b8a831e3f77ea7f387a1a43e`).
5. Complete every open task acceptance criterion, including full-manifest
   shard/thread and interruption/resume equivalence, current-source performance
   and idle campaigns, and two fresh unfiltered complete pinned Wasm-AOT families.
   Unsupported and infrastructure outcomes stay in the denominator.
6. Obtain actual supported-host run artifacts. Authored or queued CI is not
   execution evidence. Resolve T22's Apia expectation against accepted normative
   and Test262 authority; open issues/draft proposals do not authorize a shortcut.
7. Generate canonical real-suite status through the publisher, commit exact
   pins and source-bound closure artifacts, then remove this file and `tasks/`.

## Rejected performance trials

Keep the current compiler and cache defaults. A bounded native stencil memory
cache removed repeated disk-read opens but replayed25/27 original cases and
lost the previous sloppy-class pass. A128MiB function disk cache and native
parallelism also failed their retention trials. These are failed experiments,
not fixes or task closure. See [exact results and raw traces](docs/rust-rewrite/rejected-native-cache-experiments-20261017.json).
Measure native compilation internals before repeating cache-size changes.

## Verification discipline

Before heavy runs, use `python3 scripts/check-scratch-space.py --scratch /tmp
--minimum-free-mib 512 -- python3 scripts/limited_verification.py --cloud -- …`
under the inherited finite machine budget. Choose capacity for the command
peak; the helper checks available space before launch. Preserve explicit worker counts, deadlines and bounded caches.
Keep product compilation separate from test-harness optimization. Ensure scratch
space before heavy runs; archive only inactive outputs after lossless verification.
Completed production snapshots can be re-admitted after recording an interrupted
receipt save and rechecking every input; never silently call an interrupted run green.
Push verified progress to main continuously. Earlier checkpoints remain in Git
history and the linked evidence documents; their results must retain their own
compiler identities. T00/T27/T28/T29 are marked complete; T01–T25 remain in
progress and T26 remains blocked until their actual criteria are met.
