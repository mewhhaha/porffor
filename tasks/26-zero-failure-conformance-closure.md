# T26 — Zero-failure conformance closure and release gate

**Status:** Blocked — current pinned Wasm-AOT aggregate is not green or fully republished

**Parallel group:** Final integration/closure  
**Depends on:** T00-T25, T27  
**Blocks:** A truthful 100% Test262 claim and conformance release

## Current repository state

The fake suites were green at the preceding path-only checkpoint. Their
execution-identity denominator is derived as 191 executions from 190 physical
files: one unflagged parse-negative runs once as sloppy Script and once as
strict Script. Runtime proof of that refreshed denominator is deferred to the
centralized Cargo/Test262 verification lease. The committed pre-version-7 fake
snapshots are path-only historical evidence, not current version-7 proof. The
README explicitly states that the current pinned real Wasm-AOT aggregate is not
green and has not been fully republished.
The shortcut audit is green as an exact-drift contract over 389 classified
observations: 35 legitimate harness adaptations, 113 diagnostic instrumentation
sites and 241 semantic shortcuts. Audit green therefore does not satisfy the
final integrity criterion. The generated current-pin backlog is absent, and
several architecture/feature lanes retain explicit unsupported cases. Formal
closure entry criteria are therefore not met.

Alternate-name aggregate resolution now carries its evidence depth through the
private, non-derived `AggregateEvidenceRequirement::{Envelope, Complete}`
domain. The candidate loop borrows that policy in an exhaustive match: verified
aggregate loading requires complete node evidence, while read-only progress and
failure-detail lookup require the validated envelope and retain their existing
downstream checks. A recursive structure target pins the seven source mentions,
the exact decision and all three producer contexts; it passes `4/4`, and the
exact alternate-name product witness passes `1/1`. This is a source-equivalent
harness invariant and does not claim a complete aggregate or advance the T26
release gate. Independent review confirmed the complete census, producer
contexts, exhaustive policy bodies and preserved candidate/error order.
Coordinated `cargo xc`, full formatter, diff, module-boundary and task-plan
checks are green.

Direct case-evidence admission now carries its partial-versus-terminal policy
through the private, non-derived
`CaseSetRequirement::{UniqueSubset, Exact}` authority. Its sole validator
consumes the authority once in an exhaustive match that binds both exact-set
strictness and diagnostic wording; the former equality-plus-match double
observation is gone. The Rust-lexical `case_set_requirement_structure` target
pins the 18-to-17 source and 14-to-13 production ownership reductions, five
`Exact` and two `UniqueSubset` constructors, all six deliveries and their
order. It passes `4/4`, while the exact direct-identity and case-evidence
witnesses pass `1/1` each. This source-equivalent harness invariant does not
claim complete current evidence or advance the T26 release gate.

Negative-test discovery now converts frontmatter through the closed
`NegativePhase::{Parse, Early, Resolution, Runtime}` authority and stores that
type directly in `NegativeExpectation`. A present unknown spelling is rejected
with the test path and spelling instead of falling through to runtime routing;
an omitted phase retains the Test262 runtime default. Compile-only selection,
failure ownership, diagnostic matching, and backlog projection all consume the
typed phase, so a new phase cannot bypass an exhaustive Rust decision. Focused
unit coverage admits all four canonical spellings and rejects `run-time`; the
standalone `negative_phase_authority_structure` target pins the boundary and
all typed consumers. This source-equivalent harness invariant does not claim
complete current evidence or advance the T26 release gate.

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
- Each execution identity `(physical path, closed execution mode)` appears
  exactly once in the manifest and aggregate. An unflagged physical file has
  distinct sloppy-Script and strict-Script identities; path-only accounting is
  invalid evidence.
- `onlyStrict`, `noStrict`, `raw`, `module`, and `raw`+`module` expand through
  the closed mode law in
  `docs/rust-rewrite/contracts/test262-execution-identity.md`; conflicting
  strictness flags are invalid suite data, not a reason to choose a mode.
- `Unsupported`, parser/lowering/backend failure, runtime failure, host failure, crash, bug and timeout are all non-passing outcomes.
- Unknown failure-kind, outcome, origin, or classification-count wire spellings
  invalidate the evidence. The explicit `unknown` origin remains a recognized
  non-passing taxonomy value that must burn down to zero.
- Fake-suite counts remain separate smoke-test metrics and never contribute to the real-suite numerator or denominator.
- Pin, snapshot schema, matrix strategy, or execution-identity changes
  invalidate stale aggregate evidence and require a fresh complete matrix.

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
- commit the generated README status block with both exact canonical publisher
  outputs, `test262/snapshots/published-status-wasm-aot.json` and
  `test262/snapshots/published-status-wasm-aot.txt`; node, aggregate, focused,
  fake-suite and `spec-exec` artifacts do not authorize that block, and neither
  does only one half of the canonical pair;
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
