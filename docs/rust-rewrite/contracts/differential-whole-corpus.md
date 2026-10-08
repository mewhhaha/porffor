# Whole-corpus differential replay

`lila differential replay-corpus --output-dir PATH --oracle spec-exec`
replays every entry in the compiled twelve-entry corpus through the existing
selected worker protocol. The library owner is
`crates/lila-test262/src/differential/corpus.rs`; the private CLI route consumes
that owner. Default product builds refuse the unlinked oracle before creating
output or resolving a worker. Enable `spec-exec-oracle` explicitly for this
developer-only command. Product JavaScript execution remains Wasm-AOT.

The default inventory includes every committed JSON entry under
`crates/lila-test262/tests/differential`, across schemas 1 through 5, once in
relative-path order. Literal `include_str!` entries require those exact fixtures
at compilation. A native filesystem inventory control checks that newly added
JSON fixtures cannot silently remain outside the compiled corpus. Adjacent JS
files are resources for existing generated/probe controls, not corpus entries.

`--corpus-root PATH` selects an entire native directory inventory. Discovery
rejects empty inventories, symlink roots or descendants, special files,
non-UTF-8 entry names, more than 128 JSON entries, directory depth above eight,
and metadata totals above 64 MiB. Discovery errors refuse the whole inventory;
they never authorize a filtered successful subset. Each subsequent file read
is bounded by the existing 16 MiB worker-request limit. Native JSON/graph
validation does not parse JavaScript. Compiler admission and execution remain
inside the existing pre-spawn worker deadlines. `--worker-bin PATH` selects the
actual worker executable for embedding callers; the real CLI defaults to its
own executable. The command has no filter, shard, skip or maximum-case option.

The output directory must be new. Before replay, `aggregate.json` binds the
controller compiler provenance and lists every entry as pending with verdict
`incomplete`. Replay is serial, Wasm-AOT then the explicitly requested spec-exec
oracle. Both attempts are retained even if either admission/lifecycle fails to
produce a backend observation. A rejected attempt cannot discard an already
observed counterpart. Actual comparison uses the same closed protocol and
comparison owner as single-case replay.

Each bounded readable input is copied byte-for-byte to `case-NNN.input.json`,
including malformed JSON and invalid UTF-8. The corresponding
`case-NNN.report.json` retains the actual comparison report or the complete
native/attempt rejection. Oversized or unreadable inputs retain their rejection
and original entry name without an input copy. Duplicate case IDs are red at
the duplicate row. Every row remains in the inventory and aggregate.

The aggregate advances only after a complete case report is written and synced.
Each new aggregate is synced as `aggregate.tmp` and renamed over
`aggregate.json`. An interrupted replay or output failure preserves the last
committed aggregate and earlier reports; pending entries remain incomplete.
`completed` counts committed case reports, not ECMAScript successes. The last
aggregate is `all_matched` only if every actual comparison report is green.
Malformed input, unsupported observation, shared failure, worker failure and
admission rejection produce `contains_failures` after complete replay. The CLI
prints the final aggregate and exits unsuccessfully for that verdict. Matching
the bounded protocol always retains `semantic_equivalence: not_established`.

The dedicated `.github/workflows/differential-corpus.yaml` is a consumed,
feature-enabled CI entry. It first checks default-build refusal, builds the
actual CLI and named worker, replays the entire compiled inventory, runs the
library/CLI controls and uploads all retained evidence even after a red replay.
Every compilation, test and replay payload passes through
`scripts/limited_verification.py --memory-mib 4096`. Its self-hosted Linux
`lila-conformance` runner must provide a delegated cgroup-v2 systemd user manager
with memory-controller authority. Before payload admission the launcher verifies
kernel memory at most 4 GiB, swap zero, whole-group OOM termination, one CPU and
serial worker defaults. A missing scope/capability fails; there is no uncapped
fallback. Later controls cannot prevent the earlier replay artifacts from being
retained. Cancellation or job timeout never yields a successful aggregate.

Source controls cover the actual twelve-entry inventory and worker provenance,
native admission limits, malformed/shared-failure/unsupported cases, duplicate
IDs, both worker admission errors, worker failures across the entire inventory,
output reuse and default-build refusal. These controls and the workflow are
freshly authored source after staging loss, not executed acceptance evidence.

The separate [performance-reporting contract](performance-reporting.md) records
the fixed twenty-fixture workload and three opt-in timing gates on an identified
idle machine. This shared corpus CI job does not run timing acceptance.
Broader generators, grammar-preserving reducers, long fuzz campaigns, arbitrary
post-execution observations and subsystem-wide performance metrics remain debt.
Whole-corpus replay is not T25 completion or full Test262 conformance.
