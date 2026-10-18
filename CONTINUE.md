RegExp fixture correction — 2026-10-18: the frozen628 native run reports
one failure in the new cross-realm control: its source referenced `$262` without
loading the Test262 harness. Stage tracing identifies that unbound identifier;
all preceding descriptor, freshness and literal/eval/Function assertions pass.
The fixture now uses the existing `__lilaCreateRealm` host hook, preserving the
foreign prototype check and reporting strictness on assertion failures. Both
corrected JS modes pass against the frozen628 public Engine. This diagnostic
is separate from the joined bulk-read candidate, whose ordinary native and CLI
acceptance remain pending; the original failed native verdict is retained.

Bulk native payload-read candidate — 2026-10-18 (ordinary verification pending):
native array construction now copies the validated actual-P byte span once,
then allocates a fresh i8 array directly or decodes exact little-endian UTF-16
into the existing fresh i16 constructor. A narrow Wasmtime range-copy API keeps
store, root, element-type, bounds and no-GC borrow checks; the runtime ABI and
Wasm emitter remain unchanged. New controls exercise every UTF-16
unit, odd offsets, fresh identity, invalid ranges/types and expired roots.
An A/B/B/A microbenchmark of the unchanged dependency's whole-array API takes
0.92–1.10s per million UTF-16 units using per-byte reads and0.08–0.13s using bulk
reads, with every output unit verified. That measurement predates the ranged
API and is not a product or conformance speedup claim. Current native4192 and
CLI verification stays frozen separately at628af0f61; this follow-up is unaccepted.

Validated empty-statement extension — 2026-10-18 (native acceptance pending):
the completed source112 native checkpoint passes all4186 selected tests, but its
focused CLI replay has a retained-pass deadline failure in strict regress-610026.
Both original fixture modes also time out in a fresh isolated supervised run.
A separate traced single-worker diagnostic completes: its three actual evals
spend27.03s parsing/lowering and16.36s initializing/executing empty prepared Ps.
The candidate now admits parsed bare empty statements and empty blocks to the
existing stateless Wasm proof, after complete lowering and inert-instantiation
checks. Declarations, nonempty blocks, values and early errors remain excluded.
The joined batch now expects4192 selected native IDs and603 tooling IDs.
This joins the RegExp batch before its expensive native build; new-batch native,
CLI retention, full workspace/pinned suites and task cleanup remain unaccepted.

Next ordinary verification batch — 2026-10-18: the RegExp compiler repair
now includes the independently checked Intl scratch opt-in. The next bounded
build may use `LILA_TEST_KEEP_INTL_INCREMENTAL=1`; normal Intl scratch can be
pruned under the existing disk reserve, and every libtest still omits scratch.
The unchanged compiler batch expects4191 native IDs. Full tooling now discovers
603 tests; current-batch native/CLI acceptance remains pending. The source-only
600-test and eight-check results below remain bound to their earlier exact HEAD.

Static RegExp kernel candidate — 2026-10-18 (native verification pending):
real caller parsing, early errors and lowering produce a sealed single-literal
proof. Shared R Wasm constructs fresh strings, validated program and RegExp state,
including exact lastIndex; actual Realm mismatch preserves ordinary preparation.
IR library/test-code, AOT library and Engine library metadata checks pass, as do
full module boundaries and four source mutation controls. The empty kernel uses
the existing assembly owner with identical validated bytes in20 actual proofs.
R layout11 rejects older helper rosters. Existing tests keep their original
module/name through wrappers; native execution retention remains unmeasured.
Native execution, CLI retention, full workspace and pinned-suite gates remain open.

Preflight repair — 2026-10-18: the first joined build stopped at an invalid
wasm-encoder HeapType constant before any native test ran. Full tooling ran598
cases:590passed,7failed and1error, all8 from stale generated Intl provenance.
The corrected refresh order now passes all13 actual recipes and preserves every
tracked file on an idempotent repeat. The empty-eval proof also distinguishes
the compiler's dead intrinsic/context metadata from observable source bindings.
The API spelling, proof checks and metadata are repaired; fresh ordinary native,
CLI230, full workspace/pinned conformance and task acceptance remain pending.

Compiler batch — 2026-10-18 (ordinary verification pending): repeated UTF-16
function naming is emitted once in R. Exact standalone naming and original
thousand-method controls pass; isolated cold template compilation falls from
about85s to16s. A second repair requires full actual-source parsing/lowering and
a sealed empty-execution proof before invoking shared stateless Wasm for empty
direct eval. New controls cover all65536 UTF-16 units and executable code exposed
by line terminators. This joined batch has not passed ordinary Cargo or CLI
retention; the completed230 replay remains205Success/22Crash/3Bug. Full workspace,
pinned suites and task cleanup remain open. The generated status is unchanged.

Embedded runtime reuse repair (2026-10-18, authored): the version-four native
manifest explicitly supports both Fast and SizeOptimized execution Engines for
the same immutable optimized runtime image. Source, target, configuration, keys,
digests, Wasmtime compatibility and mode-specific retention remain required.
A diagnostic of the existing Cargo image loads into both real product Engines
and rejects incompatible epoch settings. Current-source native acceptance is
pending; the earlier UTF-16 stall and rejected batch remain recorded.

Intl refresh dependency repair — 2026-10-18: List identity now refreshes before
Duration, which binds List's generated identity and manifest. Six focused controls
pass. Restoring both stale outputs and running the actual13-recipe refresh passes
in10.06s; a repeat passes in9.59s while preserving bytes, modes, inodes and mtimes
for all131704 tracked regular files in the full checkout.
[Exact checks and lossless evidence](docs/rust-rewrite/intl-list-duration-refresh-order-20261018.json).
This is tooling verification; compiler retention and full-suite gates remain open.

Ordinary verification checkpoint — 2026-10-18: all 4,180 selected native tests
and all 581 tooling tests pass. The fresh release CLI completes the exact 230-case
replay at **205 Success, 22 deadline Crash, 3 Bug**: Unicode13 passes all13,
prior197 gives192/5, and remaining20 gives0/17/3. These results fail retention;
the compiler stays on recovery, and full workspace/pinned-suite/task cleanup
remain open. [Lossless replay evidence](docs/rust-rewrite/owned-native-throughput-cli-230-20261018.json)
records all original IDs, deadlines, caches and source identities.

A separate naming repair moves repeated UTF-16 name construction into one R
helper. The original thousand-method native control passes in48.92s. An isolated
cold-template A/B/B/A comparison measures81.3/89.3s before and16.0/15.8s after;
this measures native compilation of one template, not full CLI acceptance.
[Exact diagnostic scope](docs/rust-rewrite/function-name-kernel-diagnostic-20261018.json)
retains both passing controls and the earlier failed diagnostic attempts.
The generated pinned-suite status block is unchanged.

Native verification checkpoint — 2026-10-18: all 4,180 tests pass across
six complete selected Cargo targets: AOT 428, IR 1,599, IR integration 740, Engine 968,
native templates 17 and realm modules 428. Every one of the 4,161 earlier passing
IDs is retained; zero tests fail or are ignored. All 581 tooling tests also pass.
[Literal inventories and lossless evidence](docs/rust-rewrite/native-core-4180-20261018.json)
bind this result to recovery source `cb2904258`; compiler promotion awaits the fresh
230-case CLI replay. Full workspace, pinned suites and task closure remain open.

Batched source identity audit — 2026-10-18: the checker retains its Git
input set, GNU grep expressions, binary handling and historical-proof admission,
while batching content scans and importing provenance checks once. All 22 controls
pass, including binary files, unusual filenames, symlinks and batch boundaries.
On the same frozen full checkout, the candidate reproduced every original finding
in 35.7 seconds; the earlier audit took 1,116.4 seconds at lower priority under
different load. These are observed timings, not a controlled speedup ratio.
Two historical reports now admit only their exact frozen hashes for host-path
normalization. [Literal checks and evidence](docs/rust-rewrite/batched-identity-audit-20261018.json).

Opt-in Rustc code generation — 2026-10-18: the verification compiler wrapper
accepts `LILA_TEST_CODEGEN_JOBS=auto`. It keeps Cargo serial and supplies a private
jobserver bounded by the finite cloud budget, CPU quota and inherited affinity.
All 11 controls pass. Two fresh 1,024-function Cargo fixture builds return the
expected checksum; enabling or disabling the option reuses every existing output
with identical bytes, inodes and timestamps. Lila rebuild timing is still
unmeasured. [Exact fixture evidence](docs/rust-rewrite/bounded-rustc-codegen-20261018.json).

Intl provenance refresh — 2026-10-18: `intl_source_identities.py --write`
checks each producer first and regenerates only stale output, then rechecks all
13 recipes. This preserves unchanged timestamps that Cargo uses for freshness.
Five focused controls pass; an actual refresh preserves the bytes, modes, inodes
and modification times of all 5,858 checked-out tracked files. No Cargo build-time
saving is claimed. [Exact checks and lossless evidence](docs/rust-rewrite/intl-idempotent-refresh-20261018.json).
Joined throughput repair — 2026-10-18 (ordinary Cargo pending): immutable GC
type declarations share storage; template ownership checks use the existing
header/body passes; stale ABI guards follow the current layout. Manifest version4
explicitly admits the optimized runtime image under both execution modes.
The opt2 AOT/Engine diagnostic passes all17 original native template tests
unfiltered, with debug validation and unchanged deadlines. The earlier mixed
profile passed11000eval and15/16 other controls, with one method-test timeout;
that failure remains recorded. [Exact diagnostic and rejected replay evidence](docs/rust-rewrite/native-source-throughput-diagnostic-20261018.json)
records the source/profile/cache scope. The rejected c927 actual CLI completed
Unicode13 at13Success and prior197 at193Success/4deadlineCrash, including one
prior-pass regression. Its remaining20 stage was deliberately interrupted; all20
must be rerun. Fresh ordinary native, releaseCLI230, full workspace and pinned
conformance acceptance remain open. Do not delete CONTINUE.md or tasks/ yet.

Verification checkpoint (2026-10-18): the ABI10/code-sharing candidate remains
rejected: the six native inventories contain 4,180 IDs, with 4,159 completed
passes, three failures and 18 unreported Engine IDs after a 900-second stall.
Of 4,161 earlier passing IDs, 4,141 were observed passing in this rejected run.
All 579 tooling tests pass. A separate repair passes 573 isolated tooling tests,
AOT/Engine type checks and exact emitted-byte controls; its native acceptance is
pending. A 256-source opt0 diagnostic reports 12.6% less emission/canonicalization time and
confirms the precompiled runtime loads into both real execution Engine modes.
[Literal results and lossless evidence](docs/rust-rewrite/rejected-owned-native-sharing-20261018.json)
retain the failed scopes. Compiler promotion, full suites and task cleanup stay
open; generated conformance status is unchanged.

# Continue Lila — 2026-10-18

The task is open. Delete this file and `tasks/` only after every acceptance gate
passes. Push verified progress to main continuously; keep rejected and partial
results with their actual source identities.

Main is `4efcf43f7`: verified bounded cohorts, source-identity preflight and default Intl isolation. Recovery
branch `codex/cloud-recovery-20261017` contains the compiler batch and its exact
native/CLI evidence. The main tooling is merged into recovery; the compiler
batch has not passed retention and must not be promoted as accepted.

## Current completed checks

Source `35c94675fb5a56a1b6307b47eb51ac7e89b4b9`, compiler fingerprint
`e02f9b2c88d52aea126db27fbc76d47ce82a186fd713c4442b66cdf9cfe2ca3f`:

- Six complete native targets pass 4,161 tests, zero failed/ignored: AOT418,
  IR1599, IR integration740, Engine965, native templates11, Realm428.
- All13 original Unicode CLI executions pass. Full197 retention finishes
  **191Success/6deadlineCrash**, no absent IDs, no infra substitutions.
- Original60s execution/300s compile allowance, worker1 and four64MiB disk tiers
  remain unchanged. These210 IDs do not manufacture a new230 aggregate; the
  remaining20 original IDs are still unaccepted.
- Actual warm Cargo reuses all15 artifacts in0.879573s; executable bytes, inodes
  and mtimes are unchanged. Combined selection reduces normal AOT variants4→2
  and runtime-image producer cohorts2→1; no controlled wall-time gain inferred.

[Exact native evidence](docs/rust-rewrite/coherent-throughput-core-20261017.json)
and [complete CLI evidence](docs/rust-rewrite/coherent-throughput-cli-20261017.json)
retain inventories, raw transcripts and original rejected outcomes.

Six retention failures: both modes of `staging/sm/class/methDefn.js`, both modes
of `staging/sm/expressions/destructuring-array-default-simple.js`, sloppy
`destructuring-array-default-class.js`, and strict `object-literal-__proto__.js`.
Sloppy `regress-563221.js` and sloppy `object-literal-__proto__.js` now pass;
old intermediate197 verification had stopped at110 cases, so its other87 IDs
must not be treated as measured passes at that intermediate identity.

The batch shares immutable bootstrap plans, indexes compressed locals, omits
unused native-factory Main, and compiles repeated identical live reads into a
counted Wasm loop. All original reads, binding resolution, completion updates
and abrupt dispatch remain. Five10,000-read variants shrink largest bodies from
1.9MB to under6KB; shared runtime bytes remain identical. Native controls pass;
the full retention result above remains red.

## Current coherent compiler work

ABI10 and exact native body sharing are applied in the primary recovery tree.
Each fresh P owns physically immutable packed source data. A checked stateless
Instance can serve different Ps without a mutable current-payload selector.
Shared code only aliases definitions with identical concrete type, locals,
rewritten instructions and actual-P ownership witness. It remaps references
through one closed mapping and preserves original numeric SOURCE_ID values and
separate function objects, contexts, constructor plans, names and source text.
Public artifacts retain original functions and source-specific symbols; shared
native debug symbols identify code indices.

The isolated source `fdc877450` passes573 tooling tests, AOT library/test-code
metadata checks and structural emission. A1001-method artifact remains exactly
byte-identical publicly; native bodies1020→21 and bytes3510490→1681717.
Diagnostic opt0 execution gives1pass/1timeout for the two new controls. The
constructor/private-brand plan control passes; the1000-method control exceeds
its original60s limit cold, then passes in19.05s with a warm module cache.
That warm callback spends10.394s parsing/lowering,1.320s emitting,2.492s
canonicalizing and0.307s looking up its compiled module. Preserve the cold red
result. This is not optimized Cargo, releaseCLI or cold-retention acceptance.
See `docs/rust-rewrite/owned-native-code-sharing-diagnostic-20261018.json`.

The prior d3bb ordinary build was stopped before any native execution after
actual full tooling reported568/574: six generated Intl provenance files were
stale after ABI10 emitter changes. The follow-up fixes the complete dependency
chain and all66 affected controls pass. It also restores the omitted Intl
isolation flag as an enforced runner default: normal Cargo roots exclude Intl's
compiled_data dev feature. Actual8-target fixture and37 controls pass, with
all6 normal library artifacts fresh. This tooling is pushed main `4efcf43f7`.

Run `python3 -B scripts/intl_source_identities.py` before expensive compilation;
use `--write` after changing consumed compiler inputs. It runs13 recipes in
producer order and checks them again. Primary now includes all4 bounded-cohort
controls missing from the isolated tree and2 new isolation controls: its full
tooling universe is579. Finish exact ordinary optimized six-target native
coverage first, then actual release CLI13+197+20 at unchanged original budgets.
Use a fresh bounded replay cache so diagnostic warming cannot establish a cold
repair. Retain every old pass/ID and all earlier rejected raw evidence.

The remaining measured crate split is a small downstream native-template crate
between stable AOT and Engine. AOT would keep typed raw artifacts and wire
producers; template edits could then avoid rebuilding AOT. The current ownership
validation/private GC helper seam needs a deliberate API. Do not place a new
crate below AOT and assume dependent rebuilding disappears. No split or profile
change is accepted yet. AOT-only incremental query evidence remains metadata-
only and does not establish controlled dirty-edit improvement.

## Remaining scopes and owners

- The original20 focused IDs:14 eval/RegExp Store-limit failures,2 regexp
  timeouts,1 strict mixed-campaign class timeout,2 Johnston identity cases,
  and1 staging AnnexB label-chain case. Rerun all20 after the coherent repair.
- The pinned ECMA262 lexical algorithm explicitly returns an empty list for
  `LabelledItem : Statement`; the nested-label staging fixture expects a
  binding anyway. A review initially conflated syntax acceptance with lexical
  instantiation. Preserve the exact failure and pinned normative boundary;
  do not recursively change lexical collection just to satisfy staging.
- Johnston is a default Link to Honolulu while backzone declares a Zone. The
  pinned country/backzone algorithm's default-link branch reaches an unmet
  assertion; a backzone identity profile retains Johnston. The current pinned
  equivalence fixture assumes the default alias. No CLDR or arbitrary cross-
  country remap is accepted. Preserve exact source/spec/test evidence.
- T22 Apia issue3310 is open; Temporal PR3318 and Test262 PR5044 remain open
  drafts. They do not authorize changing accepted semantics.

## Full verification and closure

1. Complete the runtime/performance repair and verify all original210 CLI IDs
   plus remaining20 without increasing limits or losing retained passes.
2. Run full current-source native coverage with the bounded coordinator. Current
   default metadata has98 native targets (13libs,84integrations,1bin) plus13docs;
   all-features adds its required-feature binary. The earlier99 default count
   was stale. See `scripts/run_workspace_cohorts.py` and batch-workflow commands.
   Admit actual Boa/spec-exec separately; finish default/all-feature/all-target
   and CI (`cargo fmt --all -- --check`, `cargo xc --keep-going --locked --offline`).
3. Replay all5,365 historical IDs from
   `target/historical-backlog-replay-input-20261012/executions`, SHA256
   `95d46445f4c3fda986c7cdf390dccdddfb8ab370b8a831e3f77ea7f387a1a43e`.
4. Finish task acceptance: full-manifest shard/thread and interruption/resume
   equivalence, current performance/idle/differential campaigns, and two fresh
   unfiltered complete pinned Wasm-AOT families. No silent skips or lost IDs.
5. Admit actual supported-host artifacts and resolve remaining authority/data
   issues. Hosted Intl run39045297297 passed Ubuntu/macOS/cross-host at its own
   identity; local artifact retrieval is not yet admitted. Other hosts remain.
6. Generate canonical real-suite status with the publisher, commit exact pins
   and closure evidence, then remove this file and `tasks/`.

Use `. /workspace/.lila-tools/activate.sh` for every Rust command, finite inherited
cloud cgroup budget, serial Cargo/libtest defaults and bounded retained caches.
Keep actual source/HEAD/index frozen during source-bound runs. Preserve warm
libraries; never link all84 integration ELFs upfront with current free disk.
The new coordinator preserves exact coverage, keeps library roots fresh, and
retires only sealed completed integration/bin outputs. Failed/deferred cohorts
remain unaccepted. T00/T27/T28/T29 are done; T01–T25 are in progress; T26 blocked.
