# Current implementation and remaining work — 2026-10-20

The older suite inventory below records the measured **pre-rename** compiler.
New renamed-release checks are recorded separately in the next section.
The project is now
[scriptw](scriptw-identity-migration.md); commands below use its current spelling,
while the evidence retains the original executable/package names and hashes.
The rename does not transfer these outcomes into a new full-suite verification.

Compiler source: `cbf7cc71f`; the verified checkpoint/scheduling batch is on
main at `5fa2e8432`. This inventory describes observed results, implemented
interfaces and open acceptance criteria. **Wasm is the sole product output
target. C/native output and a REPL are outside scope, not unfinished goals
or release blockers.** It supplies no current full-suite
conformance percentage. [Exact receipts, examples and provenance](current-inventory-20261020.json)
are preserved losslessly in encoded assets.

## New scriptw release checks

At source `c5053c9d4`, the default-feature opt3 CLI and Wasmtime runner build
successfully without an oracle dependency. All 634 tooling tests pass with
zero skips, plus 19 focused codec/cache native tests. Three Intl outputs
and three synchronous React 19.3.1 SSR outputs match Node byte for byte.
The public React browser entry was not retested; its prior failure remains.
[The local V8 comparison](v8-baseline.md) records cold start, hot start and
execution with all samples, machine specs and exact build/source identities
in [new evidence](v8-baseline-20261020.json).

This proves an ordinary release build and focused operation. Original230/prior219
retention, full release suites, current-source AF/default, calibrated T25 and
complete pinned conformance remain outstanding. Old suite outcomes below are
not transferred to the renamed source.

## What is done

- The supported product compiles JavaScript through parsing, early errors,
  spec IR and lowering IR into Wasm. Wasm-AOT is the default execution path.
  The interpreter is an explicit engineering oracle.
- The Rust workspace separates the frontend, IR, Wasm emitter, engine, runtime,
  runtime images, Intl and conformance harness. The library and CLI are usable
  development interfaces. Existing C/native scaffolds are outside product scope.
- Atomic checkpoint replacement preserves the previous resumable JSON if a
  write fails. The new partial-write regression passes. JSON is the resume authority.
- CLI verification partitions now use measured exact-test costs while keeping
  all 855 CLI IDs and the existing worker counts/deadlines. Its wall-time benefit
  has not been measured by a completed current run.
- Pre-rename source `cbf7cc71f` passes **14 source audits, two shell negative controls and
  624 tooling tests**. All **12 ordinary library scopes pass 3,865 tests**,
  including Test262477, CLI82, AOT442, Front177, IR1607 and Engine973.
  Four of those scopes contain zero tests; successful Rust compilation of
  the obsolete C/native scaffold packages adds no product output target.
- The current Engine Intl integration scope also passes all **299 tests**.
  The separate isolated Intl library suite remains unrun in this saved run.
- T00, T27, T28 and T29 remain recorded complete: contribution contracts,
  interpreter quarantine/product default, legacy product retirement and the
  canonical Scriptw identity. This inventory closes no additional task.

## What the current CLI can compile and run

The existing debug CLI is bound to the compiler source above by its loaded
identity, executable SHA-256 and actual successful Cargo bin producer.
Its build enables `spec-exec-oracle`; each example below explicitly reports
`backend_used: WasmAot`. These are representative programs, not complete
feature-family or production-release acceptance.

| Fresh example | What passed through the product path |
| --- | --- |
| Core JavaScript | Lexical variables, a captured mutable closure, loops, a class with a private field, thrown values and catch/finally |
| Objects and collections | Array map/filter, Symbol keys, property descriptors, Map/Set, Proxy get and Reflect.get |
| Binary data and text | Uint8Array, ArrayBuffer/DataView, BigInt exponentiation, UTF-16 slicing, Unicode RegExp and JSON round-trip |
| Async and iteration | Promise resolution, await, a generator and for-of, including drained async output |
| ES modules | A filesystem dependency, named import/export and top-level await using `.mjs` |
| Intl and Temporal | Minimal-profile en-US NumberFormat, English PluralRules/grapheme segmentation, PlainDate leap-day addition and Instant epoch nanoseconds |
| Emitted artifact | `build wasm` wrote a real Wasm program and its matching runtime sidecar; both Wasm headers and file hashes were retained |

All **six run examples and one emission check pass**. Their JavaScript,
commands, exact output and Cargo provenance are in the evidence bundle.
The initial examples used `console.log` and failed with an unbound identifier;
those results remain preserved. Corrected examples use the documented `print`
extension. The CLI does not supply `console` or establish Node/browser API compatibility.

A subsequent [React 19.3.1 SSR probe](react-ssr.md) passes three exact HTML
comparisons with Node using the unmodified synchronous ReactDOM renderer;
the actual Scriptw outcome identifies Wasm-AOT. It also emits a program/runtime
Wasm pair. The public `react-dom/server.browser` entry fails with an unbound
identifier; browser/streaming APIs are not supplied. This separate compatibility
probe is recorded in [its own evidence](react-ssr-20261020.json), outside the
six examples/one emission counts above and all suite totals below.

| Build or interface | Current state |
| --- | --- |
| Rust frontend/IR/emitter/engine/runtime libraries | Current ordinary library test targets compile and their populated test scopes pass |
| Debug `scriptw run file.js` | Demonstrated on the six examples above; defaults to Wasm-AOT |
| Debug `scriptw run file.mjs` or `run --module file.js` | Module loader is implemented; the fresh `.mjs` import/TLA example passes |
| `scriptw build wasm file.js` | Implemented Script compilation; emits program bytes and optional linked runtime sidecar when `SCRIPTW_WASM_DUMP` is set |
| Dedicated CLI module artifact build | Not exposed: `build wasm` currently calls `compile_script`; module execution is available through `run` |
| Ordinary optimized release CLI | Builds at `c5053c9d4`; default features, no oracle dependency; focused Intl/SSR and local benchmark pass. Full release acceptance remains open |
| Existing C/native and REPL stubs | Outside project scope; obsolete entries are not planned features or completion gates |

The runtime target is experimental Wasmtime with the required GC, reference,
exception and related features, matching Scriptw runtime imports, host adapters
and admitted Intl data. An emitted program/runtime pair is not a browser,
WASI or generic Wasm runtime compatibility claim. Native compilation in
verification means building the Rust compiler or Wasmtime compiling Wasm
to machine code; it does not add a native JavaScript output target.

Finite prepared eval/Function sources use the real compiler. The native host
can also compile additional supported runtime strings. Portable Wasm artifacts
do not contain a parser; unsupported source/context/Realm combinations and
selected host operations remain explicit capability gaps. Such cases remain
non-passing in real Test262 accounting. Full ECMAScript support is not established.
Dynamic-source cases requiring a parser or VM inside emitted Wasm are outside
product scope; the published aggregate has no exact count for that category.
See [runtime requirements and dynamic-source limits](runtime-requirements.md).

From the repository root, an ordinary CLI rebuild recipe is:

```sh
# On this cloud setup, activate the installed toolchain if needed.
. ../.scriptw-tools/activate.sh
python3 scripts/limited_verification.py --cloud -- \
  cargo build --locked --offline -p scriptw-cli --bin scriptw --jobs 1
```

The older capability inventory reused the existing linked debug CLI. The
new release build is recorded separately above; its reproduction command is
in [the benchmark runbook](v8-baseline.md). To use a source-bound CLI:

```sh
SCRIPTW_BIN=./target/debug/scriptw
cat > hello.js <<'JS'
function counter(n) { return () => ++n; }
const next = counter(40);
print(next() + next());
JS
"$SCRIPTW_BIN" --jobs 1 run hello.js
SCRIPTW_WASM_DUMP=hello.wasm "$SCRIPTW_BIN" --jobs 1 build wasm hello.js
# Keep hello.wasm.runtime.wasm too when one is emitted.
```

## Verification: completed, failed and unrun

| Older suite gate | Pre-rename source `cbf7cc71f` result |
| --- | --- |
| Source audits / shell controls / tooling | 14 / 2 / 624 passed |
| All-feature workspace | **Incomplete: 49/113 closed scopes, 7,560 passed, one failed, zero ignored** |
| Ordinary library subset | 12 scopes / 3,865 passes; included in the workspace count |
| Documentation tests | None executed yet; 11 expected in the full plan |
| Remaining saved AF discovery | 64 uncompleted scopes; 2,094 expected native tests still uncompleted, plus docs |
| Default workspace | Unstarted; planned 111 scopes, 9,559 native tests and 11 docs |
| Final all-target Cargo checks | Unstarted; formatter already passed in the source precheck |
| Ordinary release / original 230 retention / real shard-resume | Original old-source gates unstarted. New `c5053c9d4` release build/focused checks pass; full retention and mechanics remain open |
| Complete debug/optimized campaigns and timing gates | Unstarted at this source; no calibrated performance acceptance |
| Current pinned Test262 closure | Both fresh complete 102,994-execution mode-aware families remain unrun |
| Current canonical publication and deterministic backlog | Outstanding; reuse one completed full family rather than running a third |
| Historical 5,365 replay and supported-host reproducibility | Separate outstanding obligations |

Counts are nested: the 559 focused passes are inside the 3,865 library passes,
which are inside the 7,560 workspace passes. The seven fresh product checks
are separate examples. The previous source `78593daf2` passed 9,654 native
and 11 documentation tests across all 113 scopes; those older outcomes are
not carried into this source's run.

The one current failure is
`scriptw-test262:differential_generated_campaign::builtin_scenarios_use_actual_selected_workers_for_all_six_stateful_families`.
The Wasm worker timed out at its original **5,000 ms** deadline in the
`builtin-stateful-v1` seed-0 array-buffer scenario; the oracle completed.
The campaign stopped after one of six cases and established no semantic
equivalence. Its underlying cause is unresolved. The original transcript,
resources and failed executable remain identified in the evidence; a passing
retry must not replace this failure.

The last coordinator receipt waited for cohort011 because the previous
machine was 9,011,200 bytes below its 2 GiB build reserve. After the cloud
restore, all four old coordinator PIDs were absent at inventory time.
Those frozen-suite coordinators have not been restarted. The observed current filesystem has about 32 GB available, so the
old disk wait does not describe current capacity. Later gates were prepared
but their old waiting processes did not survive. They must be deliberately
re-admitted; documentation is not an automatic restart.

## Task inventory and remaining acceptance

The ownership codes are permanent technical areas. The recorded plan has
**four complete, 25 in progress and one blocked**. Implementation coverage
and task-wide acceptance are different; no local smoke test closes a task.
The authoritative criteria are in [conformance acceptance](conformance-acceptance.md)
and the [ownership index](conformance-ownership.md).

| Task | Recorded state | Implementation / work still required for closure |
| --- | --- | --- |
| T00 contribution protocol | Complete | Keep ownership, provenance, resource and publication guards passing |
| T01 baseline/backlog | In progress | Tools exist; fresh complete current-pin baseline, deterministic backlog twice, comparison and canonical publication remain |
| T02 IR/Wasm architecture | In progress | Modular Rust pipeline exists; remaining architecture/behavior/build-cost acceptance and final broad checks |
| T03 harness integrity | In progress | Source audits and Test262477 pass; full current harness/fake-publication/host and shortcut-removal acceptance remain |
| T04 shared operations/completion | In progress | Shared conversion/completion machinery exists; complete affected semantic families and side-effect/abrupt-order criteria |
| T05 heap/GC | In progress | Wasm GC/rooting machinery exists; remaining weak/ephemeron/finalization and cycle/root stress acceptance |
| T06 Realms | In progress | Distinct-Realm mechanisms exist; complete cross-Realm semantic and lifetime evidence |
| T07 parser/early errors | In progress | Front177 passes; complete current negative/parser buckets and specified invariants |
| T08 environments/control flow | In progress | Closure/loop/exception example passes; full scope, TDZ, destructuring and abrupt-completion families |
| T09 functions/classes | In progress | Closures/private fields example passes; full function/class/constructor families and Realm/new-target behavior |
| T10 object model | In progress | Descriptor/proxy example passes; complete exotic/property-order/integrity criteria |
| T11 Proxy/Reflect | In progress | Representative dispatch works; full trap invariants, revocation and abrupt-order families |
| T12 modules | In progress | Import/TLA example passes; full linking, namespaces, cycles and host-flow criteria |
| T13 dynamic source | In progress | Finite prepared sources work; unmatched runtime source/context stays explicit debt and can block literal closure |
| T14 Promise/async | In progress | Await/job output example passes; full jobs, reaction, async and completion criteria |
| T15 iteration/resources | In progress | Generator example passes; complete async iteration, closing and disposal families |
| T16 arrays | In progress | Map/filter example passes; complete Array/species/sparse/order families |
| T17 binary data/Atomics | In progress | TypedArray/DataView example passes; full resize/detach/shared-memory/Atomics criteria |
| T18 strings/Unicode | In progress | UTF-16 example passes; complete String and Unicode families |
| T19 RegExp | In progress | Representative Unicode RegExp works; complete syntax, captures, state and protocol families |
| T20 numeric/JSON | In progress | BigInt/JSON example passes; complete coercion, numeric and JSON criteria |
| T21 symbols/collections/weak refs | In progress | Symbol/Map/Set example passes; complete collection and weak/finalization acceptance |
| T22 Date/Temporal | In progress | Selected date/instant example passes; full Temporal/time-zone/rounding semantics and evidence |
| T23 Intl | In progress | Selected formatting/segmentation works and Engine Intl299 passes; full Intl, isolated native suite and supported-host reproducibility |
| T24 globals/errors/Annex B/host | In progress | Error/control-flow example passes; complete semantic families and host integrity |
| T25 differential/performance | In progress | Tooling and scheduling changes exist; repair current worker timeout, finish debug/optimized corpus, 704/1,216 campaigns and compatible calibrated timings |
| T26 release closure | Blocked | Zero non-successes in two fresh complete current-pin families, real shard/resume, required stress/product/host checks and generated publication |
| T27 interpreter quarantine | Complete | Default Wasm-AOT policy retained; ordinary product must continue to exclude the oracle |
| T28 legacy retirement | Complete | Rust-only product/development boundary retained |
| T29 identity | Complete | Canonical Scriptw names and provenance guards retained |

## Next work, in order

1. Reconcile the stopped run's exact closed inventories and source/artifact
   hashes. Preserve its timeout and execute only the 64 uncompleted AF scopes
   when continuing discovery; do not rerun completed tests to recreate receipts.
2. Diagnose the timeout under its original request and budget, implement a
   general repair if needed, and verify the coherent repair batch. Retain the
   prior passing IDs and every literal red result.
3. Complete fresh AF/default and final Cargo checks, ordinary release, original
   230-case retention, real serial/shard/resume, and complete debug/optimized
   campaigns and idle measurements at the admitted source.
4. Run the two fresh full pinned families; generate backlog twice and compare
   bytes, then use the native canonical publisher with its actual fake-suite
   measurements. The old README generated publication stays historical until
   current complete evidence authorizes replacement.
5. Finish supported-host and calibrated performance obligations, resolve every
   remaining semantic failure, and only then remove `CONTINUE.md` and `tasks/`.
