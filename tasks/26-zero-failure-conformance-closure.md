# T26 — Zero-failure conformance closure and release gate

**Status:** Blocked — current pinned Wasm-AOT aggregate is not green or fully republished

**Parallel group:** Final integration/closure  
**Depends on:** T00-T25, T27  
**Blocks:** A truthful 100% Test262 claim and conformance release

## Current repository state

The fake suites are green, but the README explicitly states that the current
pinned real Wasm-AOT aggregate is not green and has not been fully republished.
The shortcut audit is green as an exact-drift contract over 449 classified
observations, but 368 of those observations are still semantic shortcuts; audit
green therefore does not satisfy the final integrity criterion. The generated
current-pin backlog is absent, and several architecture/feature lanes retain
explicit unsupported cases. Formal closure entry criteria are therefore not
met.

## Objective

Drive the current pinned real Test262 suite to a complete verified `passed == total` result for the Rust Wasm-AOT path, with no crashes, bugs, timeouts, silent skips, stale nodes or test-specific semantic shortcuts. Produce reproducible evidence strong enough for the README and a release gate.

This task is not a place to implement large missing features. It coordinates the final failure burn-down, integration, integrity audit and publication after the owning semantic tasks have landed.

## Entry criteria

Begin formal closure only when:

- T01 can generate a complete current-pin backlog and compare snapshots;
- T02-T04 provide stable module/operation/completion boundaries;
- every remaining failure has an owning task or an explicit dynamic-source policy record;
- the harness-integrity inventory from T03 has no unowned shortcut;
- publication can resume deterministically after interruption.

## Authoritative denominator

- Discover the complete suite from the pinned vendored Test262 checkout and current matrix strategy.
- Include all selected top-level roots, flags, negative tests, modules and async tests supported by the repository's conformance definition.
- A case appears exactly once in the manifest and aggregate.
- `Unsupported`, parser/lowering/backend failure, runtime failure, host failure, crash, bug and timeout are all non-passing outcomes.
- Fake-suite counts remain separate smoke-test metrics and never contribute to the real-suite numerator or denominator.
- Pin changes invalidate stale aggregate evidence and require a fresh complete matrix.

## Closure workflow

### 1. Freeze and verify the baseline

Record:

- Lila commit and Rust toolchain;
- ecma262/Test262 revisions;
- matrix/snapshot schema versions;
- host platform, architecture and required locale/time-zone data versions;
- build profile and relevant feature flags;
- complete initial counts by outcome, failure kind, origin, task ID and matrix node.

### 2. Burn down by stable failure families

Use normalized detail hashes and semantic owners, not one PR per random test. For each family:

1. reproduce one minimal representative and the full affected filter;
2. confirm the owning shared operation/feature layer;
3. land a general fix with a minimized regression test;
4. rerun the affected filter and adjacent shared-operation filters;
5. update the generated backlog, recording passes, regressions and changed hashes;
6. reassign newly exposed failures immediately.

Never convert a failing semantic case into a static materialization, precomputed output or permanent expected failure.

### 3. Eliminate non-semantic failures

Before the final run, reduce these counts to zero:

- Rust panic/process abort;
- invalid Wasm, instantiation trap caused by compiler/runtime bugs or memory corruption;
- harness crash or result-accounting mismatch;
- per-case timeout at the publication timeout;
- nondeterministic pass/fail result;
- stale/incomplete matrix node;
- unknown/unclassified failure origin;
- duplicate/missing manifest case.

Performance work belongs to T25, but this task verifies that formerly slow correct cases complete under the standard release budget.

### 4. Integrity audit

Search and review the repository for:

- exact Test262 paths used to choose semantics;
- source regexes/assertion text used to infer results;
- hard-coded expected values for real cases;
- harness implementations of missing standard builtins;
- same-realm fallbacks for cross-realm requests;
- no-op detach/GC/agent hooks that allow false passes;
- snapshot editing or denominator reduction;
- branches that count unsupported/timeout as success.

Every legitimate path reference for discovery, reporting or a minimized test fixture must be documented. Remove all semantic shortcuts or reopen the owning task.

Additionally re-run the T27 interpreter-quarantine audit: the default engine/CLI backend is Wasm-AOT, no product path falls back to `spec-exec`, product/release builds link no interpreter engine crate, and `build wasm` artifacts contain compiled user semantics rather than source fed to an evaluator.

### 5. Full verification runs

The `wasm-aot` matrix is the release gate; it is the only backend whose results are published or gated on. A complete `spec-exec` oracle matrix may additionally be run to validate the Rust host/harness and identify oracle limitations, but it is diagnostic only and can neither pass nor block the release on its own numbers.

For Wasm-AOT, perform at least:

- one complete low-RAM resumable publication run;
- one independent clean-snapshot rerun or equivalent statistically strong shard rerun;
- shard/thread-count equivalence checks;
- interruption/resume checks;
- representative runs on every supported release host/architecture;
- regression corpus and stress tests from T25.

All completed runs must reconcile manifest totals and produce the same semantic result set.

## Dynamic-source accounting

`AGENTS.md` permits dynamic source evaluation features such as `eval`, `new Function` and cross-realm Function constructors to remain explicitly unsupported when supporting them would require bundling a parser/interpreter/VM into emitted Wasm. This architectural permission must not become a false pass:

- T13 defines the selected capability/policy and implements every direct-compilation or host-compiler path the project adopts.
- Remaining dynamic-source cases stay visible as non-passing in literal Test262 accounting until resolved.
- A release may describe an architecture exception separately, but it may not claim literal 100% Test262 while any such case remains outside `passed`.
- The final project target remains `passed == total` with a deliberate architecture that preserves direct JS-to-Wasm compilation.

## Publication and release evidence

After all checks pass:

- publish status only through `lila test262 publish-status` or `scripts/publish-real-status-low-ram.sh`;
- commit generated JSON/text snapshots and the generated README status block together as required by repository policy;
- include exact counts, pins, date and refresh commands;
- archive a closure report listing matrix nodes, duration, slowest cases, artifact hashes and integrity-audit result;
- tag the baseline/release only after CI verifies the committed snapshot matches a fresh aggregate;
- do not hand-edit the generated status totals.

## Required final commands

```sh
cargo fmt --all --check
cargo test --workspace --quiet
cargo build -p lila-cli

./target/debug/lila test262 run language/wasm/pass \
  --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 \
  --execution-backend wasm
./target/debug/lila test262 run \
  --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262

rm -f test262/snapshots/final-wasm-aot-*.json test262/snapshots/final-wasm-aot-*.txt
./scripts/publish-real-status-low-ram.sh wasm-aot final-wasm-aot

# Optional oracle-validation matrix; diagnostic only, never published as product conformance:
rm -f test262/snapshots/final-spec-exec-*.json test262/snapshots/final-spec-exec-*.txt
./target/debug/lila test262 report-all --execution-backend spec-exec \
  --snapshot-name final-spec-exec --resume

./target/debug/lila test262 progress-status --execution-backend wasm-aot \
  --snapshot-name final-wasm-aot
./target/debug/lila test262 triage-status --execution-backend wasm-aot \
  --snapshot-name final-wasm-aot
```

Adjust snapshot cleanup to the implemented CLI's safe reset command if one exists; never delete unrelated snapshot families.

## Acceptance criteria

- The current pinned Wasm-AOT aggregate is complete and reports `passed == total`, `failed == 0`.
- Counts for every failure kind and non-success outcome are zero.
- Timeout, crash, unknown-origin, missing-case and duplicate-case counts are zero.
- Independent rerun/sharding/resume checks produce the same complete result set.
- The integrity audit finds no test-specific semantic branch, fake standard builtin or silent host fallback.
- The T27 interpreter-quarantine audit is green: Wasm-AOT is the default backend everywhere, no interpreter engine crate is linked into product builds, and no emitted artifact embeds an interpreter or user source consumed by an evaluator.
- All workspace tests, fake suites, differential corpus and required stress checks are green.
- Published README/status artifacts are generated, current, internally consistent and tied to exact revisions.
- Any architecture limitation still present is stated separately and prevents a literal 100% claim until it is eliminated.
