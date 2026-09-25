# TypedArray Number-key lane recovery — 2026-09-25

The interrupted lane was complete and clean at `c7bbcea53`, based on
`62c4df308`. Its five commits implement deferred Number property keys,
allocation-free CanonicalNumericIndexString, Atomics access revalidation,
DataView/TypedArray constructor ordering, and their regressions and documents.
The final rebase completed before interruption; the conflicting descriptor
census commit was correctly omitted because upstream already contained it.

Recovery added only regression coverage in `12bcff8f5`: dynamic Array reads
must preserve a prototype getter's receiver and expose String keys to a Proxy
prototype, including `-0`, fractional keys and the `4294967295` boundary. The
backend needed no further change after review.

## Evidence and provenance

These are focused family results, not a published full Test262 status refresh.
The vendored suite is upstream `7ab7faf` (2026-09-23); the snapshots identify its
Git tree as `91b2052adad1f066ae031e2ff3a1e9bd6d732886`. The tree hash is not a
different upstream commit.

The original transcript is
`~/.claude/projects/-home-mewhhaha-src-porffor/8e1032a6-2542-412b-9d45-28a78003b16f/subagents/agent-a7f8df6a034aa1a03.jsonl`.
The following log names are relative to
`/tmp/claude-1000/-home-mewhhaha-src-porffor/8e1032a6-2542-412b-9d45-28a78003b16f/scratchpad/`.
That directory contains other agents' older logs too; their failures are not
results for this head.

### Completed before interruption

The `final-*` snapshots used the complete lane rebased onto `b2537c088`, before
the final rebase onto `62c4df308`. The original baseline was the branch with
immutable ArrayBuffers, before this lane; these baseline counts are recovered
from the assignment and prior records rather than newly rerun.

| Prefix | Original baseline | Completed `final-*` run |
|---|---:|---:|
| built-ins/TypedArray | 2884/2890 | 2890/2890 |
| built-ins/TypedArrayConstructors | 1444/1446 | 1446/1446 |
| built-ins/ArrayBuffer | 442/442 | 442/442 |
| built-ins/DataView | 1122/1122 | 1122/1122 |
| built-ins/Atomics | 778/778 | 778/778 |
| staging/sm/Atomics | 0/4 | 2/4 |
| staging/sm/TypedArray | 148/185 | 148/185 |
| staging/sm/extensions | 92/118 | 92/118 |

Separately, `built-ins/Array` passed `6117/6117` with the Number-key change
alone. That result does not claim the later complete rebased head was tested.

After the final rebase, the release binary copied as `lila-final2` was built
from `c7bbcea53`. Completed verification at that head:

| Check | Result | Log |
|---|---:|---|
| lila-aot-wasm tests | 2074 passed, 0 failed (401 targets) | aotwasm-tests5.log |
| lila-ir tests | 1555 passed, 0 failed (99 targets) | ir-tests2.log |
| Four focused engine targets | 27 passed, 0 failed | engine-test3.log |
| Full CLI, first chunk | 378 passed, 1 declared ignore | cli-r1.log |
| Full CLI, second chunk | 427 passed | cli-r2.log |
| Module boundaries | passed | boundaries4.log |
| built-ins/TypedArray | 2890/2890 | final2-built-ins_TypedArray.log |

The engine targets were `aot_array_index_storage`, `aot_decimal_scratch_reuse`,
`aot_for_in_enumeration`, and `aot_typed_array_number_keys`. The last included
the four-million-element bounded-heap regression. The full CLI total was
805 passed, zero failures, one declared ignore.

All these Rust commands used `--release --locked -j3` and
`--test-threads=3`; tests ran in systemd scopes with `MemoryMax=12G` (CLI
chunks: `14G`) and `MemorySwapMax=0`. Test262 used Wasm AOT, `--jobs 1`,
`--threads 3`, `--timeout-ms 60000`, `TZ=UTC`, `LC_ALL=C.UTF-8`, and
`LILA_TEST262_FORCE_CASE_RUNNER=1`, in a `12G` scope.

### Fresh recovery verification

`git diff --check 62c4df308..HEAD` and `bash scripts/check-module-boundaries.sh`
passed. After adding the Array coverage, this command passed all four tests
in 12.10 seconds (the release rebuild took 2.54 seconds):

```sh
systemd-run --scope --user --quiet --slice=lila-build.slice \
  -p MemoryMax=12G -p MemorySwapMax=0 -- nice -n 5 \
  cargo test --release --locked -j3 -p lila-engine \
  --test aot_typed_array_number_keys -- --test-threads=3
```

Broad Rust suites were not duplicated during recovery. The prior processes
for `final2-array.log` and `family2.log` continued running after the API failure;
the family queue proceeds through TypedArrayConstructors, ArrayBuffer,
DataView, Atomics, staging extensions, staging Atomics and staging TypedArray.
Only its completed TypedArray result is counted in the table above. Partial
checkpoint progress is not a pass result; inspect these logs and their
`snap/final2-*.json` outputs for eventual completion.
At 17:02 UTC on 2026-09-25 they had reached 4470/6117 Array executions and
570/1446 TypedArrayConstructors executions, respectively.

## Performance and remaining limitations

The completed `final2` TypedArray run recorded the six formerly failing
copyWithin executions at 18.945–22.962 seconds each, within the 60-second bound.
Earlier process measurements of the standalone start-detachment reproducer
recorded baseline failure after 67.6 seconds at 1487 MiB sampled peak RSS,
versus success after 32.1 seconds at 1294 MiB with the Number-key change.
Those are separate loaded-machine runs, include compilation, and use a
standalone reproducer with buffer transfer replacing the detach hook. They
are not directly comparable runtime-only measurements. An unmodified strict
Test262 execution with the Number-key change passed in 38.82 seconds at
1091.9 MiB measured peak RSS. The bounded-heap regression provides the direct
evidence against per-access allocation in the targeted copy path.

The unchanged staging failures remain explicit: host GC is unsupported
without a real collector, created-Realm SharedArrayBuffer blocks the remaining
Atomics pair, and the TypedArray staging family retains its prior failure
set. GC-free diagnostic copies are not conformance passes for the original
GC-dependent tests. Ordinary array-like property scans remain linear;
Array.from still allocates iterator results and property-key Strings.

Atomics revalidation deliberately bounds the complete element. For a
length-tracking view shrunk to a partial trailing element this is stricter
than the specification's byte-start bound, whose following buffer-access
operation requires sufficient bytes. This documented specification edge case
is covered by the regression fixture, not silently treated as full conformance.
