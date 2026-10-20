# Remaining acceptance at eb1ca63c9 (read-only audit, 2026-10-15)

This read-only audit executed no acceptance workloads. The root subsequently
completed the standalone shortcut audit recorded below. Wait until active
long42904 and its owned children drain before remaining workloads. Keep its
evidence immutable. Its `actual-history-fullpin-2` matrix is the CURRENT EB
ordinary-product/current-pin fresh family with its publisher: once closed and
audited, it supplies T01's complete baseline AND publication. Schedule no extra
`report-all` baseline. `$PRODUCT` means the
identity-checked ordinary default release CLI already used by CLI55;
`$ORACLE_DEBUG` / `$ORACLE_RELEASE` mean actual same-source feature-enabled CLIs.
`$OUT` must be a fresh private output parent. All workload directories below
must be new. Retain original deadlines, fixtures, execution modes and pins.
Prefix native payloads with `python3 scripts/limited_verification.py --cloud --`
unless the shown command already includes it or its script owns that launcher.
The differential tier script owns its launcher; the robustness driver needs both
the shown outer launcher and its own `--cloud` admission flag.

## Already covered: do not rerun

- Published default 9,176 / all-features 9,269 native passes, ignored controls
  and docs; Python 547; formatting/module/host-ABI/product-oracle static checks.
  Read against docs-only main `2c9949137e188018160faf16c9a19ce921bc4d8c`, not the
  frozen EB handoff's historical counts. Their task/full-pin flags remain false.
- `target/task-native-coverage-eb1ca63c9-20261015/receipt.json` accounts for all
  task-required T02 IR/AOT/Engine/CLI, T03 spec-exec/Test262/Engine/test262_, and
  T05 heap_/wasm_ selectors. No additional Cargo selector is needed.
- Both canonical complete fake selections already execute in the passing native
  `test262_publish_status_supports_wasm_backend`: 187/187 and 191/191. Its real
  matrix is deliberately tiny; those fake counts establish no full-pin result.
- Artifact validation and exact product execution transcripts already pass in
  `product_artifact::representative_language_families_produce_valid_aot_artifacts`
  and `representative_artifacts_execute_with_the_expected_transcripts`.
- CLI55 covers exactly its selected 55 real execution IDs, all Success. It is
  not the historical replay, current full matrix, sustained campaign or host gate.
- Root's actual checkout shortcut audit now passes: 26 exact entries, source
  unchanged, 0.963985528 seconds on CPU4 through the existing source-command owner
  and cloud limiter. Receipt: `target/shortcut-check-eb1ca63c9-20261015-run/receipt.json`.
  It compiles only the standalone std-only scanner into `mktemp` storage
  (honors TMPDIR), not a Cargo/product/Wasmtime graph. Do not rerun it.
- The old `fuzz_regressions` selectors match zero receipt IDs in both IR/AOT;
  `snapshot_determinism` is also documented obsolete. Neither is a useful run.

## Small remaining runnable batches

1. **T25 current-image corpus determinism and sustained campaigns.**
   Dependency: same-source oracle-enabled debug and release CLIs, both explicit
   oracle gates, available retained cache/runtime dependencies. Do not rebuild the
   current ordinary executable in place while any owner still uses it.
   For each profile, run three fresh full rounds using:
   `python3 scripts/limited_verification.py --cloud -- "$ORACLE_RELEASE" --jobs 1 differential replay-corpus --output-dir "$OUT/corpus-release-1" --oracle spec-exec`
   (substitute debug/release and round 1/2/3). Compare aggregate, every copied
   input and report bytes per image; source/native corpus tests do not retain this
   repeated-byte proof. Different image identities are not byte-equal baselines.
   Then use the existing full drivers, once per required profile:
   `bash scripts/run-differential-campaign-tier.sh --cloud "$ORACLE_RELEASE" "$OUT/nightly-release" nightly`
   `python3 scripts/limited_verification.py --cloud -- python3 scripts/run-robustness-campaign-tier.py "$ORACLE_RELEASE" "$OUT/robustness-release" nightly --cloud`
   Output: 11 grammars x 64 seeds (704), and 19 targets x 64 seeds (1,216), with
   original request bytes and 5,000/300,000 ms budgets. These are bounded protocol/
   robustness results, not general equivalence or full conformance. PR-fast,
   individual generate-arithmetic/replay examples and already covered native
   campaign controls add no necessary gate once these complete tiers run.

2. **T02/T25 actual optimized idle measurements.**
   Dependency: fully idle machine after compilation/conformance/campaign drains,
   ordinary default release image and unchanged fixture root/resource settings.
   For `MODE` = `report`, `compile`, `runtime`, separately run:
   `python3 scripts/limited_verification.py --cloud -- "$PRODUCT" --jobs 1 performance MODE --output-dir "$OUT/perf-MODE" --samples 7 --machine-label managed-cloud --idle-machine`
   Output: original 1s/5s/5s whole-CLI gates, plus all 20 compile/runtime fixtures
   and real stage/footprint/capacity observations. Native ignored timing tests
   already passed; fresh calibrated optimized reports are a different obligation.
   `performance check --mode compiler|runtime --baseline B --candidate C --policy P --output-dir O`
   is meaningful only with admitted compatible reports and a genuinely calibrated
   policy. The documented example is explicitly not such a policy. Module-movement
   cost attribution also needs a controlled predecessor, not unlike build timings.

3. **T01/T03/T05/T26 real shard/thread/interruption-resume mechanics.**
   Dependency: ordinary current product image, exact clean vendored pin, fresh
   snapshot directory. Select one small real subtree once (`harness` is the task's
   existing selector), then use the same input set throughout:
   `"$PRODUCT" --jobs 1 test262 list harness --execution-backend wasm-aot`
   `"$PRODUCT" --jobs 1 test262 run harness --execution-backend wasm-aot --threads 1 --timeout-ms 60000 --snapshot-dir "$OUT/mechanics" --snapshot-name serial`
   `"$PRODUCT" --jobs 1 test262 shard 1/2 harness --execution-backend wasm-aot --threads 2 --timeout-ms 60000 --snapshot-dir "$OUT/mechanics" --snapshot-name shard1`
   Repeat shard2, and independently start a same-input resume family; interrupt
   its owned process group only after a genuine nonempty persisted checkpoint,
   then invoke the identical command plus `--resume` to completion. The runner
   checkpoints every 10 cases. Compare exact mode-aware IDs, result fields and
   normalized failure detail, not totals alone: shard union must equal serial,
   and resumed results must equal uninterrupted results. The existing
   `scripts/check-test262-shard-report.py` can additionally check all-green shard
   summaries, but rejects red results and does not by itself prove exact union.
   Wrap all executions in the same cloud launcher. These checks do not replace
   current full discovery or a second fresh complete release family.

4. **T01 backlog only, using the active matrix's completed baseline.**
   Dependency: `actual-history-fullpin-2` has actually closed and been audited;
   use its exact `$PRODUCT`, full snapshot family name and complete evidence.
   A complete red baseline is sufficient for T01. Do not schedule another matrix
   or publisher. Existing native controls already cover intentional regression
   and pin mismatch; do not inject new ones.
   Source `lib.rs:8470` derives the output root by lexical `suite_root.parent()`
   twice; `write_backlog_artifact` then writes `backlog/<recorded Test262 pin>/wasm-aot.json`
   and `.txt`. `--snapshot-dir` does not redirect those outputs.
   A private owned view can preserve the actual suite while redirecting outputs:
   create a real `$OWNED/vendor` and `$OWNED/backlog`, symlink the ENTIRE actual
   committed suite root to `$OWNED/vendor/test262`, and copy the exact repository
   `test262/backlog/ownership-map.tsv` to `$OWNED/backlog/ownership-map.tsv`.
   Do not symlink the backlog directory. Suite pin lookup canonicalizes the
   whole-suite link to the real committed Git context, while output-path derivation
   keeps the lexical owned parents; CLI parsing preserves that lexical path.
   A loose directory linking only test/harness children would lack the same Git
   identity (and internal directory symlinks are not recursively scanned).
   Copy the completed snapshot directory into `$OWNED/snapshots` without changing
   payloads/names; this protects original evidence even if matrix-cache loading
   writes a cache on a miss. Do not symlink writable cache files into original proof.
   Then, twice with the same actual image and complete snapshot family:
   `"$PRODUCT" --jobs 1 test262 generate-backlog --suite-root "$OWNED/vendor/test262" --snapshot-dir "$OWNED/snapshots" --snapshot-name "$COMPLETED_FAMILY" --execution-backend wasm-aot`
   Save the first owned JSON/text pair before the second and compare exact bytes.
   Expected output scope is `$OWNED/backlog/<same actual pin>/`; no source-checkout
   file changes. This is source-traced feasibility, not an executed new acceptance.

## Distinct blockers and later final owners

- **T26 literal all-zero:** unresolved semantic/spec authority (including the
  separately owned Temporal Apia question) and every actual unsupported dynamic
  source ID remain nonpasses. Architectural permission is no denominator waiver.
  These are not failures repaired by rerunning native selectors or proof tooling.
- **T26 fresh gate:** once its entry requirements are genuinely ready:
  `SCRIPTW_BIN="$PRODUCT" CLOSURE_EVIDENCE_DIR="$OUT/closure" bash scripts/check-test262-closure.sh --cloud`.
  The native owner runs `closure-first` and `closure-second`, validates exact
  full manifests/results, writes a truthful red/green verdict and never publishes.
  This is distinct from active `actual-history-fullpin-2`, which supplies T01's
  complete baseline/publication. That resumable publisher family cannot substitute
  for either fresh closure-owned family. No extra T01 baseline belongs before or
  after this gate. Do not run it to manufacture a known all-zero claim.
- **Independent supported hosts:** this Linux machine cannot supply macOS/Windows
  execution or cross-host receipts. Existing CI release matrix builds Linux/macOS/
  Windows, and `.github/workflows/intl-reproducibility.yaml` captures Linux/macOS
  with `scripts/check-intl-cross-host.py capture`, then `compare`. Their actual
  same-source successful receipts must be obtained; local repeat tests cannot
  replace them. Spec-exec harness/matrix runs are optional diagnostics, not product
  acceptance, and default product builds correctly refuse the oracle.
- **Publication is already authorized**, including continuous verified pushes
  to main; no new permission request is needed. The active current-pin family
  already has its publisher. Preserve the source/index phase freeze, then adopt
  its verified canonical JSON/text plus generated README pair and push through
  the existing guarded workflow once closed/audited. Do not launch a duplicate
  publisher. Informational publication is distinct from the fresh release gate.

## Actual shortcut audit record

The [record](shortcut-check-eb1ca63c9-20261015.json) preserves the exact raw
receipt and both logs. The actual checkout audit exited zero in 0.964 seconds,
matching all 26 inventory entries. It ran through the cloud limiter on CPU4,
using the inherited 32 GiB cap; the compiler source stayed unchanged. Refresh
from the checked source with:

```sh
python3 -B scripts/limited_verification.py --cloud -- \
  bash scripts/audit-test262-shortcuts.sh --check
```

## Historical dependency conflict remains a literal failure

The ongoing original-tree replay reports Bug for
`module:language/import/import-bytes/bytes-from-txt.js`: the imported
`bytes-from-txt_FIXTURE.txt` is absent from historical tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b` itself. The full materialized
historical suite faithfully preserves that absence. Current tree
`6a4268a9354a545d41c3b62efebc478ed8c521f4` contains the 13-byte dependency,
blob `980a0d5f19a64b4b30a87d4206aade58726b60e3`; the importing test has
identical bytes in both trees, blob `d332e10b065b9b75b6a6716eb1866cdad29c4617`.
This is a changed dependency input, not evidence of a module-resolver change.
Keep the actual Bug and strict prior-positive nonpass in the historical result;
do not insert newer files, change its pin, or waive its retention predicate.
Current-pin execution is a separate result and remains pending.
