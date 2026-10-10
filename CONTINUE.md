# CONTINUE — handoff state (2026-10-10)

Work directly on `main` and push to `origin/main`; no feature branches or
force-push. Start with `AGENTS.md` and `tasks/README.md`. The detailed cloud
receipt is [docs/rust-rewrite/cloud-continuation-20261009.md](docs/rust-rewrite/cloud-continuation-20261009.md).
Use Luna agents for read-only chores/exploration; root writes code.

## Cloud checkpoint

The cloud machine has a finite 32 GiB inherited cgroup cap. Heavy work uses
`python3 -B scripts/limited_verification.py --cloud -- <command>`, one CPU/worker,
and a two-entry retained-module cache up to 256 MiB, reduced further to at most
one eighth of the inherited cap. Stricter explicit limits survive. Earlier
checkpoints used one entry/64 MiB. Local machines keep the existing
4 GiB/no-swap/grouped-OOM systemd policy. Rust/Cargo, rustfmt, Clippy and bundled
LLD live in `/workspace/.lila-tools`; source `activate.sh` in each Cargo shell.
Locked/offline fetch/build and default CLI inspect/run/build smoke checks pass.
No Node runtime or system service is a product dependency.

Cloud CPU use remains serial by default. The verified opt-in
`--cloud-cpus auto` mode exposes up to the inherited affinity and visible
cgroup CPU quota (four CPUs here), honoring stricter inherited native-worker
limits. Cargo and libtest remain serial. Direct CLI `--jobs N` selects its
compiler pool; Test262 `--threads` independently selects case concurrency.
Explicit test counts and deadlines stay unchanged. Three real Wasm fixtures
produce identical output with one and four compiler workers and reuse the
embedded native core in both modes; all 28 production launcher controls pass.

The final frozen engine Full10 checkpoint completes all thirteen default-feature
scopes at **3,184 passes, zero failures and zero ignores**, with unchanged source
and original deadlines. Complete CLI Full3 is green at **943 passes, zero failures
and four existing ignores**. The fresh identity-checked product fake run passes **191/191 exact IDs over
190 files**, including all 187 Wasm-safe members, with zero failures/timeouts.
Four isolated cases with one compiler worker each finish in **190.096 watched
seconds** under the inherited four-CPU quota, preserving 60,000-ms case limits.
Final format, architecture, shortcut-accounting and ledger guards also pass.
The exact [fake receipt](docs/rust-rewrite/cloud-continuation-20261009.fake-final.json)
and [guard receipt](docs/rust-rewrite/cloud-continuation-20261009.final-guards.json)
preserve identities, IDs, commands and hashes.

The frozen engine Full4 checkpoint records 3,157 passes, 15 failures and zero
ignores. After the repairs, Full5 completes all thirteen scopes at 3,176 passes,
zero failures and zero ignores, with unchanged source and original deadlines.
Both exact receipts are preserved. Full5 precedes the additional CLI-discovered
BinaryData repairs below; it does not certify those newer changes. Earlier
partial and serial-invalid receipts remain diagnostic history.

The earlier source batch repairs retained binding write policies, suspended
array-pattern scopes, generator `var` pattern hoisting, ordinary descriptor
snapshots, non-simple parameter named-self policies, early `with` global
fallback References and folded RegExp descriptor parity. The serial IR
checkpoint passes all 1,496 tests; the loop/epoch, Float16, descriptor and
ordinary assignment controls pass. Three missing Segmenter corpora are now
tracked with exact committed-archive bytes; eight corpus checks and thirteen
launcher checks pass.

All 15 complete-checkpoint failures now have authored repairs or valid fixture
corrections, with original semantic intent and execution limits preserved.
The batch adds discarded generator Updates, restricted scalar global errors,
canonical shared RegExp alias accessors, lossless UTF-16 literal keys, loaded
computed imports, declared host adapters, a correct structure boundary and CLI
runtime-sidecar cleanup. Arguments configurable snapshots, required RegExp
empty captures and Hebrew YearMonth carrier limits get distinct positive and
negative controls. The first joined all-feature/all-target type check passes in
297 Cargo / 300 watched seconds. The frozen focused run records 1,540 passes and
two failures: a newly added R-free comparison and the original restricted-global
exception constructor diagnostic. The final joined type check passes in
230 Cargo / 240 watched seconds. Both corrections pass their affected controls:
one artifact unit and all seven restricted-global native tests. All fifteen
original failures also pass the complete Full5 checkpoint.

CLI Full1 finishes its driver on unchanged source, but its main target is
incomplete: the wrapper incorrectly assigned 900 seconds to the Test262 subset
whose documented outer stall budget is 3,600 seconds. That target stops with
exit124 after 289 observed passes and seven observed failures. The other sixteen
targets complete at 95 passes, two stale structure-guard failures and three
pre-existing performance ignores, including the full fake publication test.
These counts are separate from a complete CLI verdict.

The next coherent repair batch fixes immutable transfer length coercion,
immutable slice RangeError and pre-allocation source revalidation, and native
waitAsync expiry before notification. Fixture corrections compare isLockFree's
boolean conversion with size1 and use an actual unsupported dynamic-source
case. The verdict structure guard explicitly includes the existing internal
case-worker producer. New native controls preserve error precedence, exact
bounds, clock domains and mixed expired/live FIFO behavior. Joined types pass
in 260 Cargo / 270 watched seconds. All 75 focused checks pass across nine
scopes with unchanged source and zero failures/ignores. Complete final CLI,
engine and identity-checked product fake acceptance were pending at that checkpoint.

CLI Full2 completes all seventeen default-feature scopes on unchanged source:
928 passes, 15 failures and four pre-existing ignores, with no incomplete native
verdict scope. The main target records 832 passes, 14 failures and one ignore;
publication reaches at least 180/191 cases before its original 900-second
deadline. Cloud limits and explicit worker/cache counts were inherited correctly;
available logs do not establish the throughput cause. The timeout remains red.

Root's next batch repairs `in` operand retention/order, runtime private-name `#`
descriptions, strong collection receiver-error classification and exact Intl
identity output. It corrects stale private-extensibility and RegExp-admission
fixtures, bind/global inspection counters and the RegExp source owner guard,
and registers all six missing integration targets in the closed hygiene domain.
Private installation controls retain duplicates, mutation and ordinary
non-extensible/sealed/frozen receivers; new strict/sloppy `in` controls retain
abrupt, coercion, Proxy and suspended-operand order. Initial joined types pass
in 325 Cargo / 330 watched seconds. The first focused attempt passes all 1,499
IR checks, then is deliberately stopped during native compilation when review
finds a missed prefixed private-name string-pool entry. The collector is now
repaired and the existing private-callable fixture adds field-only, uninitialized
static and escaped-name controls. Fresh types, complete focused checks and
final CLI/engine/fake acceptance were pending at that checkpoint.
No deadline or ignore-ledger changes were made.

The private-name correction passes joined types in 258 Cargo / 270 watched
seconds. A separate publication lifecycle review proves two case workers wrote
passing snapshots after the publisher was killed; no cross-scope overlap or
throughput cause is established. The Linux worker spawn now arms a parent-death
signal and rejects a lost-parent race before exec. A subprocess regression kills
a disposable supervisor and verifies its separately grouped worker terminates.
This joined source passes all-feature/all-target types in 7.10 Cargo / 15
watched seconds. The frozen focused9 run completes all 23 scopes: 1,525 passes,
one failure and zero ignores. All 1,524 non-publication checks pass, including
the original fourteen main CLI failures and Linux worker cleanup. Publication
still times out at 900.43 seconds after 190/191 cases. Its deadline remains
unchanged and this checkpoint remains red.

Source tracing finds that each supervised case worker independently hashes its
loaded executable for evidence and again for cache identity. A startup probe
measures one image hash at 0.47 seconds for the 600 MB debug CLI; this does not
prove the publication timeout's cause. The cache now derives a versioned key
from the verified source and loaded-image digests already owned by
`CompilerIdentity`. Unavailable identity disables program/runtime/graph cache
access while preserving compilation. A test-helper lifetime error is corrected;
fresh all-feature/all-target types pass in 161 Cargo / 165 watched seconds.
All 24 focused cache, identity, graph and publication checks pass across eight
unchanged-source scopes. Publication completes in 888.52 seconds under its
original 900-second deadline. The exact receipts retain the earlier red runs.
Complete CLI/engine and the explicit identity-checked product fake run were
pending at that checkpoint. Startup probes are diagnostic only and establish no general speedup.

CLI Full3 completes all seventeen default-feature scopes on unchanged source:
943 passes, zero failures and four pre-existing ignores. Its main scope records
846 passes/one ignore; the 187 exact raw Wasm-safe IDs all pass. Full fake
publication completes in 898.43 seconds under its original 900-second deadline.
The compact receipt names all four ignored stress/performance controls.

The owner's iteration-time request prompts a controlled shared-core probe.
The 41,195,754-byte R Wasm uses a 251,691,352-byte native bundle. Both 1/64-MiB
and 2/64-MiB configurations invoke its native factory six times; 2/256 MiB
invokes it once across the same six executions. Warm R loads measure about
24 ms instead of 130 ms. All three existing native controls pass in each
configuration; total wall timings include persistent-P warming and are not a
general performance benchmark. The cloud launcher now retains R and P within
the measured bounded ceiling; all fifteen launcher controls pass. Rust compiler
and fixture source remains identical to green CLI Full3. Local limits remain
one entry/64 MiB. Future watched commands use `--poll 1` to avoid the default
15-second completion lag. Engine Full10 and explicit product fake acceptance are now green.

## Accepted batch and integration

All thirteen engine scopes, seventeen CLI scopes, joined types, final focused
controls, 28 launcher controls and fresh 191-ID product fake acceptance pass.
Native Rust/fixture hashes are unchanged across final engine and CLI evidence.
CLI has four pre-existing stress/performance ignores; report them explicitly.
Keep the documented 3,600-second outer stall budget for the full main CLI scope
and preserve tests' individual deadlines and explicit worker/cache counts.
Final guards pass; run the actual `check-readme-status-artifacts.sh origin/main`
against the resulting commit before a normal push. The exact cloud install
script and startup instructions are tested and saved through the environment
configuration draft workflow; Review/Publish is its activation step. No task
states or publisher conformance totals are changed by hand.

## Property storage checkpoint — 2026-10-10

The next verified batch gives ordinary named-property tables an explicit logical
insertion extent and geometric capacity. Typed Wasm GC `array.copy` preserves
the rooted live prefix; deletion holes and String/Symbol insertion order stay
intact. Arguments indexed descriptors also grow geometrically with null spare
slots kept absent and the parameter map independently bounded. Rooted completion
snapshots read the logical extent. Lookup remains linear.

All 969 unique backend controls and 44 focused native controls pass, including
GC during a getter, descriptor attributes, deletion/re-addition, sparse Arguments,
Proxy/global-reference behavior and snapshots. The first backend sweep exposed
a stale source guard for already-correct loop binding modes; the corrected guard
and all remaining targets pass. A fresh identity-checked product fake run passes
all 191 exact IDs over 190 files, including all 187 Wasm-safe IDs, in 209.105
watched seconds with the original 60,000-ms deadlines and four isolated cases.
The fixed three-round 4,096-key ordinary/Arguments probe passes before and after;
execution measures 110.275 and 105.292 seconds respectively. These single samples
do not establish a general speedup. Full engine/CLI sweeps above describe the
prior checkpoint, not a fresh complete sweep of this storage change.
See the [storage receipt](docs/rust-rewrite/property-storage-20261010.md).

## Blocked historical replay and remaining work

The requested September 30 aggregate with 5,365 exact failing execution IDs is
absent; the owner believes it was local and never pushed. Do not ask again,
substitute another snapshot or claim the historical replay completed.
Its former path was `target/publication-freeze-20260930-*/test262/snapshots/`.

Further work includes an index for large global objects and folded
HasProperty/Get. Geometric ordinary/Arguments table growth is now implemented. Previously recorded compiler
gaps include the vendored parser's parenthesized member assignment target, Bytes
module kind, logical/compound `with` RHS representation and AST-based module
syntax stripping. These are separate follow-up scopes, not results of this batch.
All thirty task states remain four complete, 25 in progress and T26 blocked.

Raw watched logs and private drivers are ignored under
`target/continuation-cloud-20261009/`; the checked-in compact receipts preserve
verdicts, commands, hashes and diagnostics for fresh checkouts. Check disk before
broad scopes and clear only completed incremental state between serial targets.
