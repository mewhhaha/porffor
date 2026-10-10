# Cloud continuation, 2026-10-09–10

The managed cloud continuation resumes `CONTINUE.md` on `main` from
`c902150964c45439bade0740602f488aed2a538e`. The owner authorizes the finite
machine cgroup budget in cloud and Luna agents for read-only chores and
exploration. Root authors code. No publisher conformance totals or task states
are edited by hand.

## Environment and verification policy

The inherited memory cap is 34,359,738,368 bytes (32 GiB). Default Cargo/native
verification uses `python3 -B scripts/limited_verification.py --cloud -- ...`,
with one CPU and serial libtest/default workers. Earlier checkpoints, including
green CLI Full3, retain one module up to 64 MiB. Following the owner's
iteration-time request, the cloud launcher now allows two modules with image
bytes bounded to `min(256 MiB, max(1 byte, inherited cap / 8))`; stricter explicit
limits survive. Integration controls retain their explicit worker counts, including
CLI Full3's isolated Test262 subset with two workers on the one-CPU cap. The
opt-in `--cloud-cpus auto` mode instead permits quota-bound native affinity
(four CPUs here); Cargo and libtest remain serial. Fresh explicit fake
acceptance uses four isolated cases with one compiler worker each. Local verification
retains its separate 4 GiB/no-swap/grouped-OOM systemd policy. The launcher
refuses absent, unreadable, invalid and unbounded cloud limits. Its 28 Python
controls pass, including the unchanged local controls and optional CPU policy. Kernel memory events remained zero through the prior complete engine
checkpoint. No system service or Node runtime is needed.

Rust/Cargo 1.99.0 stable, rustfmt, Clippy and bundled LLD 23.1.1 are installed
under `/workspace/.lila-tools`; source its `activate.sh` in each Cargo shell.
Locked dependency fetch, offline metadata, default CLI build and inspect/run/
build-Wasm smoke checks passed. Three missing Unicode16 Segmenter corpora were
restored byte-for-byte from the committed ICU archive; pinned hashes and vector
counts (1,093/1,826/512) match and all eight identity/corpus Python controls pass.

## Complete diagnostic engine checkpoint

`cloud-engine-full4-20261009` completed every default-feature engine target
and doctests, locked/offline and serial, with unchanged source. It records
3,157 passes, 15 failures and zero ignored tests. This red checkpoint is
independent of the subsequently authored repairs. Its exact source identity,
commands, scope verdicts, failed function names, log hashes and SDK diagnostics
are preserved in [the compact receipt](cloud-continuation-20261009.engine-full4.json).

| Scope | Passed | Failed |
|---|---:|---:|
| lib | 854 | 0 |
| aot_async | 245 | 0 |
| aot_generators | 164 | 1 |
| aot_intl | 287 | 3 |
| aot_gc_entries | 146 | 0 |
| aot_builtins | 485 | 0 |
| aot_language | 253 | 2 |
| aot_realm_modules | 358 | 6 |
| aot_regexp | 140 | 1 |
| aot_temporal | 198 | 1 |
| runtime_cache | 4 | 0 |
| structure | 23 | 1 |
| doc | 0 | 0 |

The unchanged 200,000-iteration global loop, infinite-loop epoch interruption,
exhaustive 65,536-value Float16 round trip and Uint8Array codec cohorts pass in
this checkpoint at their original limits. Most native execution limits are
30,000 ms, native source matcher controls use 60,000 ms and the Intl projection
controls use 120,000 ms. None was increased.

The first partial process ended at 580 passes/one failure with exit120 and no
final target verdict; kernel OOM events were zero and its termination cause is
unproved. Repair2 completed lib at 853/one unchanged-deadline loop timeout,
then was deliberately retired after 66 async passes to repair descriptors.
Repair3 observed lib854/zero, but its restored driver overlapped an IR rerun;
that whole-run serial receipt is invalid. The originals and annotated receipts
remain under `target/continuation-cloud-20261009/`. They are not complete green
checkpoints. The IR rerun subsequently completed serially at 1,496/zero.

## Source repairs and retained controls

Earlier fixes make binding write policies mandatory independently of eval
visibility, retain suspended array-pattern scopes, hoist suspended generator
`var` BoundNames, snapshot ordinary descriptors directly into fresh GC records,
retain physical named-self policies for parameter closures and resolve `with`
global fallback References before the single RHS. Folded RegExp finite String
positions now share canonical literal/bitmap/range forms with static compilation.
The original failed assertions, broad checkpoints and focused regression receipts
remain distinct.

The first repair batch addresses all 15 complete-checkpoint failures:

- Discarded generator Update expressions enter the existing typed suspension
  plan used by value contexts. Prefix/postfix property and Annex B call controls
  cover both admitted contexts.
- Direct Engine fixtures use their declared `print`/`__lilaCreateRealm` hosts.
  The Locale number projection uses `getNumberingSystems()`. Exact artifact,
  locale preference, output and Realm assertions remain.
- Scalar restricted-global lexical declarations select R for the existing
  intrinsic SyntaxError admission path. Ordinary scalar computations stay R-free.
  Main declaration rejection also captures its intrinsic constructor diagnostic
  before the early return that precedes the main job checkpoint.
- RegExp legacy aliases share one canonical accessor per captured slot. Original
  alias identity and complete native-source matcher assertions remain, with
  explicit canonical name/source controls.
- Literal property keys use lossless UTF-16 pool encoding instead of Interner
  display text. Lone units, literal backslashes and the private encoding marker
  remain distinct across object/class keys, inferred names, patterns and templates.
- Computed imports in a loaded closure may select exact keys discovered from
  the same referrer's source, including a static deferred edge. Additional host
  rows remain excluded; complete catalogs keep their distinct scope. Original
  ordinary/mixed generator controls retain operand, promise, namespace,
  evaluation-count and abrupt-identity assertions.
- The structure test uses the actual next function as its bounded source marker.
  Its output/result ownership assertions are unchanged.
- Three invalid fixture expectations are corrected with stronger controls:
  arguments freeze supplies nonconfigurability while a separate configurable
  snapshot accepts redefinition; required RegExp empty tails retain their last
  empty capture with exact indices and finite rollback controls; Hebrew minimum
  YearMonth admits ISO April while conversion to an earlier PlainDate rejects.
  These do not change engine semantics, large count bounds or deadlines.
- CLI `LILA_WASM_DUMP` writes exact P and optional R bytes and removes an earlier
  sidecar when a replacement is runtime-free. A lifecycle control checks both
  replacement and the already-absent sidecar case.

SDK probes linked directly to the frozen checkpoint rlib before tracked source
changed. Captured arguments output distinguishes the successful mapping
retirement from the configurable-property fixture error. Both generator imports
report `TypeError: Cannot find module ./value.js`; source tracing identifies the
omitted loaded request. Literal and computed RegExp outputs agree, and finite
counts distinguish required empty captures from rejected optional attempts.
The repaired Hebrew fixture passes with exact output and normal262 against the
unchanged checkpoint library. Diagnostic probe process status alone is not a
native test pass.

## Final batch acceptance

The [final frozen engine Full10 receipt](cloud-continuation-20261009.engine-full10.json)
records all thirteen default-feature scopes on unchanged source: 3,184 passes,
zero failures and zero ignores. It covers the final Rust repairs, including the
CLI-discovered BinaryData and compiler-cache identity changes. CLI Full3 is
also complete and green as recorded below. The [fresh product fake receipt](cloud-continuation-20261009.fake-final.json)
passes all 191 exact IDs over 190 files, including 187 Wasm-safe members, with
zero failures/timeouts. Schema8 and exact membership are checked against the
executing CLI SHA256. Four isolated cases with one compiler worker each finish
in 190.096 watched seconds under the inherited four-CPU/32-GiB budget; the
60,000-ms case deadlines and original watcher stall budget remain unchanged.
A missing-activation build launch (127) and intentionally retired serial-case
run (143) remain diagnostic evidence, separately identified in that receipt.
The [final guard receipt](cloud-continuation-20261009.final-guards.json) records
passing format, task-plan, architecture, identity, host-ABI, retired-JS, product
graph, generated accounting, scanner-unit and publication-ledger checks. The
actual committed README comparison runs separately before push.

The [cloud CPU receipt](cloud-continuation-20261009.cloud-cpu.json) records three
real Wasm fixtures with fresh caches in each of one- and four-worker modes.
All six runs pass with identical paired output and exactly one embedded native
core load per process. These small diagnostics do not establish a general
speedup. The adopted `--cloud-cpus auto` option bounds affinity by all visible
cgroup CPU quotas, the inherited CPU set and stricter native-worker defaults.
This machine provides four CPU equivalents. Cargo and libtest stay serial;
explicit CLI `--jobs N` sets its compiler pool independently of Test262
`--threads`. Default cloud verification remains serial. All 28 production
launcher controls and the actual watched four-CPU readback pass. The option
also prevents the CPU wrapper from silently halving cloud affinity, while
preserving an explicitly supplied lower CPU share.

The earlier frozen focused9 checkpoint completes all 23 scopes on unchanged
source at 1,525 passes, one failure and zero ignores. Its 1,524 non-publication
checks pass: 1,499 IR checks, three process lifecycle controls, three native
operand-order controls and nineteen selected CLI repair/positive controls.
Publication records one pass and one failure, reaching 190/191 cases before
its unchanged 900-second limit (900.43 native / 915 watched seconds).
The [focused9 receipt](cloud-continuation-20261009.focused9.json) preserves
that red result; no successful partial run replaces the failed publication.

Source review confirms that supervised workers separately hash the executable
for evidence and cache identity. The retained startup diagnostic measures a
0.47-second identity query on the 600,467,072-byte debug CLI and a 4.89-second
single fake worker. This identifies redundant work, not a demonstrated timeout
cause. The new cache fingerprint shares `CompilerIdentity`'s verified source
and loaded-image digests, invalidates older cache domains and disables
program/runtime/graph cache reads and writes when identity is unavailable.
The first type check finds a lifetime error in the new test helper; its explicit
reference annotation fixes that error. Fresh all-feature/all-target types pass
in 161 Cargo / 165 watched seconds. All 24 affected cache/identity/graph and
publication checks pass across eight unchanged-source scopes. Publication
completes in 888.52 seconds under its original 900-second deadline. The
[focused10 receipt](cloud-continuation-20261009.focused10.json) retains the exact
green source, commands, log hashes and earlier failed type check. Complete
engine and explicit product fake acceptance were pending at that checkpoint.

The [complete CLI Full3 receipt](cloud-continuation-20261009.cli-full3.json)
records all seventeen default-feature scopes on unchanged source: 943 passes,
zero failures and four pre-existing ignores. Main CLI records 846 passes/one
ignore in 7,349.98 native seconds. The 187 exact raw Wasm-safe members all pass;
their manifest, membership and compiler image identity are retained in the same
receipt. Full fake publication records two passes in 898.43 native seconds under
its original 900-second deadline. The four ignored tests are heap-page-boundary
stress and the cold-exact, warm-exact and warmed-twenty-case performance controls;
the compact receipt preserves their actual names.

The [controlled core-retention receipt](cloud-continuation-20261009.core-retention.json)
records three existing native controls in each of 1/64-MiB, 2/64-MiB and
2/256-MiB configurations. R Wasm is 41,195,754 bytes; its serialized native bundle
is 251,691,352 bytes. The first two configurations invoke the native factory six
times, while 2/256 MiB invokes it once across the same six executions. Warm
runtime module loads measure about 24 ms instead of 130 ms. Overall wall timings
also include Cargo and persistent-P warming, so no general speedup is claimed.
Rust compiler and fixture hashes still match green CLI Full3. The independently
tested cloud launcher now permits this bounded R/P retention; all fifteen
launcher controls passed before the CPU addition; all 28 current controls pass.
Local policy remains one entry/64 MiB. Future scopes
use the existing `--poll 1` watcher option to avoid the default 15-second
completion lag. Final engine and fresh product fake acceptance now pass, with
native Rust/fixture bytes unchanged across engine Full10 and CLI Full3.

The [startup probes before](cloud-continuation-20261009.startup-before.json)
and [after](cloud-continuation-20261009.startup-after.json) are single diagnostic
samples. Identity query is 0.467/0.459 seconds, inspect 0.032/0.034, scalar run
2.925/2.917, scalar build 1.004/1.559 and one fake case worker 4.887/4.651.
All commands succeed. These mixed timings do not establish a general startup
speedup or prove that duplicate hashing caused either earlier timeout.

The first joined all-feature/all-target workspace type check passes (297 Cargo /
300 watched seconds). The frozen 21-scope focused run then records 1,540 passes,
two failures and zero ignores. Fourteen of the fifteen original failures pass,
as do all 1,499 IR units, 23 emission controls, two UTF-16 controls, the canonical
RegExp alias control and CLI sidecar lifecycle. Its source identities, exact
commands, verdicts and log hashes are preserved in
[the focused receipt](cloud-continuation-20261009.focused5.json).
The final joined type check passes (230 Cargo / 240 watched seconds), and the
corrected artifact unit plus all seven restricted-global native controls pass.
All fifteen original engine failures now have passing focused verification.
[The final focused receipt](cloud-continuation-20261009.focused6.json) also
preserves the rejected zero-test selector and the measured qualified retry.
Formatting, product dependency and cheap policy/identity guards pass for that
source batch. The subsequent complete Full5 engine checkpoint records
3,176 passes, zero failures and zero ignores across all thirteen scopes with
unchanged source. All fifteen original failures have actual passing native
verdicts. [The Full5 receipt](cloud-continuation-20261009.engine-full5.json)
preserves scope commands, source hashes and verdicts. This green checkpoint
precedes the CLI-discovered BinaryData repairs described below.

CLI has four pre-existing ignored performance/stress controls;
ignore counts must be reported separately. The full product fake suite must
execute all 191 exact IDs over 190 files; its 187 raw Wasm-safe IDs are a
membership subset of that run, not a substitute run or full Test262 conformance.
Record the actual executable SHA256 in the fresh snapshot receipt.

## CLI baseline and additional coherent repairs

CLI Full1 finishes all seventeen driver entries with unchanged source, but it
is incomplete as native acceptance. Its main target stops with exit124 after
289 observed passes and seven observed failures, while executing the raw
Test262 subset. Root incorrectly assigned 900 seconds of outer log silence;
`scripts/rung1c-chunks.sh` documents 3,600 seconds for this exact subset because
its child captures output. The next CLI checkpoint restores that documented
outer budget without changing individual execution deadlines.

The remaining sixteen scopes complete at 95 passes, two failures and three
pre-existing performance ignores. The full fake publication witness passes
within its unchanged 900-second limit. The two failures are stale source
censuses which omitted the existing hidden `__case-worker` verdict producer.
The incomplete main scope's observed results are not added to the completed
native verdict totals. [The CLI receipt](cloud-continuation-20261009.cli-full1.json)
preserves both categories and the failed test names.

Serial diagnostics use unchanged executable SHA256
`24ebbe5cb1c9950feb0cf624f5f2685f37f6b10c594914e6e766489040ac2e29`.
They confirm six fixture failures and the successful metadata-only immutable
control. [The diagnostic receipt](cloud-continuation-20261009.cli-diagnostics1.json)
retains exact stdout/stderr and commands; the diagnostic driver's successful
exit is not native test acceptance.

Root's additional batch repairs:

- Transfer performs ToIndex before detached/immutable rejection, preserving
  arbitrary conversion throws and RangeError priority. Invalid receiver brands
  still reject before conversion, and immutable write/resize ordering is unchanged.
- Immutable slice reobserves source validity and the exact final bound after
  both conversions, before intrinsic allocation, and throws RangeError on shrink.
  Ordinary species copy keeps its surviving prefix and prefilled suffix.
- Native waitAsync registration owns the same deadline and HostClock domain as
  its GC timeout checkpoint. Notify skips expired entries without spending count
  or losing store ownership; a notification made before expiry still wins later
  promise processing. Strict/sloppy blocking and independent-clock FIFO controls
  exercise the exposed gap.
- The isLockFree fixture compares `true` with size1, as required by ToInteger.
  Unsupported run/shard uses a genuine runtime-source diagnostic and retains
  every red-verdict/snapshot assertion. A separate supported metadata-only
  control protects the discovery/execution distinction.
- The private Run/Shard verdict domain stays closed. Its structure guard now
  exactly includes the internal case-worker Run producer and its output order.

The additional joined all-feature/all-target type check passes in 260 Cargo /
270 watched seconds. All 75 focused checks pass across nine scopes with
unchanged source, zero failures and zero ignores. These include all six exposed
BinaryData fixtures, unsupported run/shard and the separate metadata control,
three verdict structure checks, two Atomics emission checks, strict/sloppy
new native scripts and independent-clock registry controls. [The focused7
receipt](cloud-continuation-20261009.focused7.json) preserves exact commands,
scope verdicts, hashes and the verified code digest map.

## Complete CLI Full2 and repair batch eight

[CLI Full2](cloud-continuation-20261009.cli-full2.json) completes all seventeen
scopes on unchanged source at 928 passes, 15 failures and four pre-existing
ignores. Every scope has a complete native verdict. Main CLI is 832/14/1;
the other sixteen scopes are 96/1/3. No new ledger row or ignore was added.
The main outer stall is the documented 3,600 seconds, with original individual
deadlines preserved.

[Actual panic diagnostics](cloud-continuation-20261009.cli-failures2.json)
retain all fourteen main failures and the publication failure. Publication
reaches at least 180/191 cases before its own unchanged 900-second deadline.
Both original and explicit worker/cache-count settings were inherited correctly.
The prior run passed in 654.35 seconds; the available evidence does not identify
the throughput cause. The completed subset is separately 187/187 with no
timeouts. That pass does not turn the timed-out publication into acceptance.

[Inspection diagnostics](cloud-continuation-20261009.cli-diagnostics2.json)
use the unchanged CLI binary and retain original stdout/log hashes. Bind
prefixes demonstrate one recognized possible bind target and zero exact bound
result facts. The first potentially observable property Get widens later global
facts and uses generic acquired-method lowering; native allocation still creates
bound functions, and all six runtime bind checks remain. The global catalog
reports 55 builtin roots, including ShadowRealm at ordinal54.

Root's repair batch retains the `in` key value before RHS evaluation without
changing canonical HasProperty operand slots or conversion order. New
strict/sloppy native controls cover GetValue, mutation, coercion, Proxy traps,
abrupt precedence and suspended operands. Runtime private-name descriptions
retain `#`; collection errors select primitive versus missing-slot failures
through closed strong receiver/error domains. Intl inspect prints exactly the
canonical SDK identity bytes, including its own newline.

Private non-extensibility expectations are corrected against current
PrivateFieldAdd, PrivateMethodOrAccessorAdd and HostEnsureCanAddPrivateElement.
Ordinary receivers accept first private installation while duplicates still
throw; public fields retain their independent extensibility check. The
[normative reference](cloud-continuation-20261009.private-normative.json) records
the retrieved source URL and SHA256, not an execution verdict or an unverified
upstream Git commit. Pinned optional legacy tests are unchanged and still run
when selected. Other corrections retain valid computed Unicode-set admission,
update the actual GC RegExp source owner, and add the six existing Cargo targets
to the closed hygiene domain without changing its four ignore rows.

Initial batch-eight joined types pass in 325 Cargo / 330 watched seconds.
The [first focused attempt](cloud-continuation-20261009.focused8-interrupted.json)
passes all 1,499 IR tests, then is deliberately stopped during native compilation.
Read-only review finds that the new prefixed private-name description also needs
collection for classes with no initializer or private callable metadata. Root
verifies all native children closed before changing the collector; the existing
private-callable fixture now includes field-only, uninitialized static and
escaped-name controls. RegExp chain comments are corrected to describe their
successful admission assertions. Fresh types and complete focused/native
acceptance were pending at that checkpoint. The final complete CLI, engine and
fresh identity-checked product fake runs now pass, as recorded above. No skips or individual execution deadline increases were added.

The private-name collection correction passes joined types in 258 Cargo / 270
watched seconds. Read-only publication lifecycle review separately establishes
that two case workers continued after publisher death: the Cargo log ends at
11:40:10.225Z, while their retained passing snapshots are written at
11:40:12.689Z and 11:40:14.937Z. The next target starts at 11:40:22Z; no worker
exit times were retained, so cross-scope overlap is not established. Each worker
owns a separate process group, outside publisher/Cargo-group cleanup. Root's
additional Linux repair arms `PR_SET_PDEATHSIG(SIGKILL)` before worker exec and
checks the captured supervisor PID after arming to reject the fork/arming race.
The existing process-group deadline retirement remains in place. A subprocess
control kills a disposable supervisor and checks its separately grouped worker
stops. This repairs direct compiler-worker lifetime; it does not establish the
cause of the original publication slowdown. Worker/cache assertions and all
original native deadlines remain unchanged. Joined lifecycle/private-name types
pass in 7.10 Cargo / 15 watched seconds; focused checks were pending at that
checkpoint and subsequently pass as recorded above.

The first new restricted-global artifact unit passed every restricted case but
failed its added `let fresh` R-free comparison: ordinary lexical declarations
can own runtime binding cells. The comparison now uses a scalar computation;
the original restricted runtime-negative assertions remain unchanged. Preserve
`cloud-repair5-restricted-scalar-artifact-20261010.log` as the first receipt.

The original pinned restricted-global runtime-negative control still failed
because its ordinary JavaScript exception had no exported constructor name.
Declaration admission returns before the main checkpoint captures that
diagnostic. The final correction captures the actual constructor through the
existing data-only observer before the early main return, with additional
scalar and class controls in both Script modes. The affected controls pass;
the final complete engine and CLI checkpoints now pass, as recorded above.

Reproduce the required checks from a configured cloud checkout:

```sh
. /workspace/.lila-tools/activate.sh
python3 -B scripts/limited_verification.py --cloud -- cargo check --locked --offline --workspace --all-features --all-targets
python3 -B scripts/limited_verification.py --cloud -- cargo test --locked --offline -p lila-ir --lib -- --test-threads=1
python3 -B scripts/limited_verification.py --cloud -- cargo test --locked --offline -p lila-engine -- --test-threads=1
python3 -B scripts/limited_verification.py --cloud -- cargo test --locked --offline -p lila-cli -- --test-threads=1
python3 -B scripts/limited_verification.py --cloud -- cargo build --locked --offline -p lila-cli
python3 -B scripts/limited_verification.py --cloud --cloud-cpus auto -- ./target/debug/lila --jobs 1 test262 run --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm-aot --threads 4 --timeout-ms 60000 --snapshot-dir target/test262-scratch/FRESH-NAME --snapshot-name FRESH-NAME
```

Use distinct watched labels for retained failures and retries. Run heavy scopes
sequentially; check disk space and remove only completed incremental build state
between targets when needed. No native child may overlap another scope.

## Historical replay remains blocked

The 2026-09-30 aggregate listing 5,365 exact failing `mode:path` IDs is absent
from this checkout. The owner believes it was local and never pushed.
`target/publication-freeze-20260930-*` is unavailable. Do not substitute another
snapshot, invent historical IDs or claim the requested replay completed.
Publisher-generated pinned conformance totals and all thirty task states remain
unchanged. Further property-table/index growth work and separately recorded
compiler gaps remain follow-up work after verified current-suite acceptance.
