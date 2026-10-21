# Current Test262 baseline — 2026-10-20

The complete current pin contains **102,994 mode-specific executions**.
The ordinary optimized, default-feature compiler ran all of them through
Wasm-AOT: **102,978 passing, 16 bugs,
0 crashes and 0 unimplemented**.
No filters, skips or timeout extensions were used. Four supervised reusable
case workers used fresh per-case state, one compiler job each and the normal
60-second case deadline, within the cloud machine's four-CPU/32-GiB budget.

Measured build: `c5053c9d4ed1ce961f66eff8cd0d721b39108ffd`; compiler fingerprint `53007aa326f737bd5721312044ac1e8e5981e34481941eef5707bf6c48c7380e`;
executing image SHA-256 `771a15da06cbac5c7b45b24ecd8b9232af152eac9f1e24cd1e615976642ff1c6`. Checkout `e21347cc7aa790346d794cfadc41f1b7b30a6fa3` has identical
Rust/Cargo/vendor inputs; intervening commits changed documentation/tooling.
This records the loaded build honestly and does not relabel its embedded revision.
Test262 tree pin: `6a4268a9354a545d41c3b62efebc478ed8c521f4`. See [source pins](../../crates/scriptw-test262/src/source-pins.json) for upstream revisions.

[Baseline JSON](../../test262/snapshots/published-status-wasm-aot.json) and
[text](../../test262/snapshots/published-status-wasm-aot.txt) retain the byte-identical
2026-10-20 native publisher output. Future refreshes select immutable JSON/text
members through `published-status-wasm-aot.manifest.json`; the graph reader
verifies that generation rather than reading these historical filenames.
See the [publication contract](reproducible-publication-driver.md).
[Lossless evidence archive](../../test262/snapshots/current-test262-20261020.tar.gz)
retains every native matrix/node snapshot, execution ID, failure, transcript and
publication-session identity, with a SHA-256 inventory. Its absolute original
snapshot locators are retained for provenance; the archive makes their bytes portable.
The receipt beside it records the archive hash and native completeness check.
The [failure backlog](../../test262/backlog/6a4268a9354a545d41c3b62efebc478ed8c521f4/wasm-aot.json) supplies a reason
and technical owner for every non-success. Two native generations are byte-identical.

The 2026-10-02 chart used another pin and path-based accounting. Its totals
cannot establish a regression or improvement against this mode-aware run.
All red outcomes remain recorded; full ECMAScript conformance, two green release
families, native workspace suites and performance acceptance remain open.
Out-of-scope classification is still unaudited; no eval/Function cases were
removed from the denominator. [Scope policy](runtime-requirements.md#out-of-scope-tests).

## Reproduce

Build the ordinary release compiler, then run the full provenance-bound matrix:

```sh
THREADS=4 JOBS=1 REUSE_CASE_WORKERS=1 \
SCRIPTW_BIN=./target/release/scriptw \
SNAPSHOT_DIR=test262/snapshots \
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- \
  ./scripts/publish-real-status-low-ram.sh wasm-aot fresh-current-family
python3 -B scripts/generate-test-graph.py
python3 -B scripts/generate-test-graph.py --check
```

Use the same family/settings when resuming; completed execution IDs and literal
failures stay in their original native checkpoints. A fresh compiler or suite
requires a fresh family. Canonical publication remains gated on full native proof.
