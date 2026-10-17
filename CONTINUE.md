# Continue Lila — 2026-10-17

The task is open. Delete this file and `tasks/` only after every acceptance gate
passes; native subsets and inventories do not establish full conformance.

Current compiler checkpoint (2026-10-17): checked per-program string counts
and ordinals now bind through fresh immutable globals in shared native templates.
All404 AOT tests and all6 native-template Engine integration tests pass. The
same1,836-request compile-only workload needs500 distinct templates instead of600.
The original27-case replay improves to26Success/1Crash and retains all25 cases
that passed in either previous checkpoint. Both prototype cases now pass; the
strict class destructuring case still times out at the original60000ms deadline.
[Exact results and raw evidence](docs/rust-rewrite/native-literal-operands-checkpoint-20261017.json) distinguish these current-source checks
from the earlier complete2,496-test native checkpoint. OrdinaryCargo, full pinned
conformance and task closure remain open.

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
