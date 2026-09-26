# Claude worktree recovery — 2026-09-26

Audited all 37 Claude worktrees against integration `a2d720530` on
`feat/test262-upstream-20260924`. All 36 inactive checkouts have been removed; the active integration checkout
is retained.
The audit compares ancestry, patch equivalence, adapted commits and dirty source;
a distinct commit hash alone is not evidence of missing implementation.

One useful source change was recovered from `wonderful-wiles-50df3d`: hoisted
functions could inherit an incorrect `undefined` return kind from captured
script lexical bindings whose initializers the static inferencer cannot model.
Commit `8ddf54ef5` uses the existing unknown-runtime-value representation for
those initializers and destructuring bindings. An absent initializer retains
its actual `undefined` value. It also preserves open callable-target knowledge.
The regression covers object and callable initializers, destructuring, a TDZ
throw before initialization, and an uninitialized `let`. The original recovered
fixture also returns `boolean(true)` through Wasm AOT, including its object
coercion, bitwise, `in`, null-prototype and array assertions.

The remaining audited worktree changes are already integrated or superseded. Two older Intl
approaches are deliberately rejected: runtime interval-whitespace rewriting
and canonical-only scalar patterns. The integrated provider instead retains
ASCII scalar alternates and uses checked canonical noncollapsed range data.
Legacy Porffor Date wiring and string-equality changes in `test262-conformance`
already have current Lila implementations; their snapshots remain historical
measurement evidence, not current conformance results.

## Preservation and cleanup

The [inventory](worktree-recovery-20260926.json) records each original head,
dirty file list, decision and removal state. Local Git refs
`refs/archive/worktrees-20260926/<name>` preserve heads; corresponding `-dirty`
refs preserve stash commits with tracked changes and untracked files in their
third parent. The four inactive dirty worktrees were snapshotted before removal.
No branches were deleted. These recovery refs are local and were not published.

To inspect a preserved tree, use `git show <recovery-ref>` or create a new
worktree from it. For a dirty snapshot, `git stash apply <dirty-recovery-ref>`
in a checkout of its recorded original head restores tracked and untracked work.
Five stale registrations for already absent directories were separately
archived and pruned. The old Claude detach-worktree lock was removed only after
its recorded host PID was confirmed absent.

## Verification

The preceding repair batch completed 4,522 core tests with zero failures or
ignored tests at `a2d720530`. An execution-environment reset then removed the
ignored `target/` artifacts and interrupted the engine integration run. That
unfinished run is not reported as passing. The earlier Test262 replay results
remain recorded in the [failure discovery checkpoint](failure-discovery-20260926.md).

The recovered capture regression passes at `ba604d2a1`. The repeated core
checkpoint passes all 4,522 tests across 529 targets, with zero failures or
ignored tests, at `670fd9d2d`. All 395 selected engine integration tests pass
across 50 targets. The engine library passes all 778 tests at `fa55e19a6`,
with no failures or ignored tests. The main CLI target passes 806 tests across
22 disjoint module chunks, with zero failures and one existing ignored T05
heap allocation stress test. The inventory audit accounts for all 807 compiled
tests exactly once; seven additional source tests require the disabled
`spec-exec-oracle` feature. The CLI source contains 814 tests in total.

Commands use release/locked Cargo, three build jobs,
three test threads, a watched 900-second stall guard, and a 10 GiB systemd scope
with no swap. Formatting, diff and module-boundary checks pass. Receipts and logs live under
`target/worktree-audit-20260926/` and `target/watched/worktree-recovery-*`.

The exact Cargo arguments and completed CLI inventory are saved in the JSON
verification record. A focused refresh is:

```sh
systemd-run --scope --user --quiet --slice=lila-build.slice \
  -p MemoryMax=10G -p MemorySwapMax=0 -- nice -n 5 \
  ./scripts/run-watched.sh --label recovered-capture --stall 900 -- \
  cargo test --release --locked -j3 -p lila-cli --test cli -- \
  --test-threads=3 --exact \
  language::run_wasm_backend_types_a_hoisted_functions_const_capture_from_its_initializer
```

The main CLI target is verified as disjoint per-module chunks; substring filters
exclude overlapping module names so every compiled test is accounted for once.
This is not a claim to have run every engine integration target or the full
pinned Test262 suite.

The first engine library run passed 777 tests and found one incorrect new bytes
loader assertion. Two identical imports retain separate binding entries but
share one canonical request edge. Commit `fa55e19a6` checks those properties
explicitly. The focused case, seven related structure tests and the complete
778-test library rerun pass; production loader behavior was unchanged.
