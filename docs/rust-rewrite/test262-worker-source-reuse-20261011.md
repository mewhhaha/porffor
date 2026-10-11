# Test262 worker source reuse — 2026-10-11

The supervised case worker passes its discovered exact one-case manifest to
the existing full-run implementation. It previously discarded that manifest
and rediscovered the same execution, reading and parsing its source again.
Public full runs still discover once. Worker selector, cardinality, execution
role, dispatch and exact terminal snapshot validation remain in force.

The manifest carries the selected source in an immutable `Arc<str>`. A new
control discovers a raw parse-negative case, changes its disk source to valid
syntax, and executes the discovered manifest. It requires a pass from the
original source and an exact one-ID terminal snapshot. The structural snapshot
control now names the private full-run owner. This removes one redundant
discovery per child; no general speedup or source-hash protection is claimed.

Both worker controls and all four exact case-set structural controls pass,
with zero failures/ignores. The new test asserts persisted execution IDs
directly, keeping all thirteen production authority mentions and six delivery
sites unchanged. The [receipt](test262-worker-source-reuse-20261011.json)
preserves the corrected compile and structural-test diagnostics.
Complete workspace, supervised product and pinned acceptance remain pending.
The focused compile overlaps the independent pinned Intl
extractor under the inherited aggregate cloud budget: the extractor uses one
CPU, native compilation is capped at three workers, and Cargo/libtest remain
serial. Timing acceptance runs separately after extraction completes.
