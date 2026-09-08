# Repairs from the September 2026 partial Wasm-AOT baseline

The input cohort is 592 distinct non-passing executions in twelve completed
matrix nodes, measured with compiler `c5115bf03ba3fdec1a8eb9b3436a939431b2ca1f`
and Test262 tree `aa55200d1310384c5cf69ea95b2a2ecba457007b`. It is a partial
baseline, not full-suite conformance. The continuing baseline retains its
unchanged compiler, suite, cache and snapshots in the original checkout; this
repair branch uses a separate worktree and bounded verification service.

## Shared causes addressed

- Suspend-owned physical bindings include uncaptured block and loop bindings;
  captured Environment Records remain the sole owner of captured cells.
  Activation slots are addressed through the current scope depth, so creating a
  captured child environment cannot redirect other suspended locals into it.
- Synchronous generator loops retain their continuation state and can advance
  through iterations whose single conditional yield is not taken.
  Local bindings are registered before resumed reads are emitted, while runtime
  initialization remains confined to fresh iteration or branch entry.
  Eager branch-local let/const declarations retain distinct activation slots.
- Async loops retain each sequential direct await's continuation state. Later
  resumptions skip earlier effects and keep the iteration environment and
  Iterator Record until the body completes. Nested suspension shapes remain
  separately validated. Async-generator preflight uses the same state-sequence
  validator as IR lowering.
- Promise capability creation requires an explicit executor Realm context.
  Await rejection retains its intrinsic Promise context instead of treating
  an ordinary lexical environment as a builtin function object.
- Rejection while unwrapping a synchronous iterator value uses synchronous
  IteratorClose. Its original throw wins, the iterator closes once, and the
  returned object's `done` and `value` properties are not observed.
- Invalid RegExp syntax is rejected before committing source, flags or matcher
  state. A later failing lastIndex write retains a successfully installed pattern.
- Arguments @@isConcatSpreadable uses ordinary symbol-property storage and Get;
  the obsolete private Boolean slot and its competing reads/writes are removed.
  Indexed reads retain mapped own values and traverse the prototype chain when
  an own index is missing, including inherited getters and their thrown values.
- Zero-argument Function constructors create distinct functions with an empty
  body and their constructor's Realm/prototype metadata. Generator, async and
  async-generator bodies pass through the ordinary IR and Wasm dispatchers;
  invocation executes the corresponding function protocol.
- Created realms publish their own AsyncDisposableStack constructor, prototype
  and methods. Construction and move use the canonical realm prototype;
  disposal promises, callbacks and errors retain their defining realm.
- The synchronous disposal fallback creates the required async wrapper. It
  discards the synchronous method's return value and preserves thrown values as
  promise rejections.
- Root and agent failures retain their typed causes, including disconnected
  workers. A worker failure cannot satisfy a test expecting a root exception,
  and a dynamic-source failure cannot hide a simultaneous crash or JavaScript
  failure. Aggregates containing only typed source capability gaps remain
  unsupported even when worker compilation and execution fail together.
- Reported async `$DONE(error)` failures survive subsequent engine errors and
  expected runtime-negative exceptions. Both failure details are retained;
  traps and timeouts remain Crash outcomes. Typed Wasm failures use an unknown
  subsystem origin until more precise provenance exists, instead of attributing
  Boa or ICU from words in a JavaScript error message.
- Runtime-negative tests compare the final thrown value's constructor name
  exactly. Diagnostic messages, changed error names and stale caught exceptions
  cannot satisfy that expectation. Metadata observation does not invoke getters
  or Proxy traps; names requiring those operations remain unavailable.

## Dynamic-source boundary

Source compilation after AOT remains explicitly unsupported. Known textual eval
and Function subsets are separate compiler-feature work; absence of that support
must not become a fake SyntaxError, runtime sentinel, source substitution or pass.
Possible intrinsic-call identity retained after invalidation authorizes rejection,
not an exact-call optimization. Definite overwrite and shadowing must remain valid.

When source proof is lost before a call, the actual runtime intrinsic rejects
unsupported source through the typed host capability boundary. The engine retains
the operation across Wasmtime and agent-worker errors; Test262 records
Unsupported/NotImplemented. This is separate from JavaScript exceptions: catching
an error or expecting a runtime SyntaxError/TypeError cannot turn missing compiler
support into a pass. Non-string `%eval%` arguments retain their ordinary return
semantics.

## Repeat the recorded failures

`test262/replays/observed-20260908.executions` contains the exact 592 execution
identities, including strict/sloppy modes. Its original outcomes were 518
NotImplemented, 68 Bug and 6 Crash. The list is an input cohort, not an expected
failure list: every execution is run and every non-passing result remains red.

After building `target/release/lila`, run the cohort with two case workers and
one compiler worker per case:

```sh
LILA_MODULE_MEMORY_CACHE_ENTRIES=1 \
  ./scripts/run-watched.sh --label observed-replay --stall 900 -- \
  python3 scripts/replay-test262-executions.py \
  test262/replays/observed-20260908.executions \
  --output-dir target/observed-replay --workers 2
```

Use a fresh output directory each time. The tool freezes copies of the compiler
and execution list there before starting any cases, so rebuilding the executable
or editing the requested list cannot mix inputs within a replay. Each native runner transcript and parsed
outcome is retained separately; `summary.json` records the executable and input
list hashes. Missing, partial or inconsistent native reports are infrastructure
errors. The command exits zero only when every execution passes, one for ordinary
non-passing results and two for an inconsistent native report. It forces isolated
case runners so their timeouts remain effective.

The watcher limits stalls and CPU use. On the Linux development host, the build
and replay services additionally use `MemoryHigh=10G`, `MemoryMax=12G`,
`MemorySwapMax=0` and `OOMPolicy=stop`. These are cgroup limits; lowering Cargo's
job count or the module cache alone is not a hard RAM limit. Builds use explicit
`cargo build --release --locked -j 2 -p lila-cli`.

To rerun only the failures from a completed replay, generate another exact list
from its summary and use a new output directory:

```sh
python3 - <<'PY'
import json
from pathlib import Path
summary = json.loads(Path('target/observed-replay/summary.json').read_text())
executions = [result['execution_id'] for result in summary['results']
              if result.get('outcome') != 'Success']
Path('target/remaining.executions').write_text('\n'.join(executions) + '\n')
PY
```

An empty list means there are no failures to replay. Focused replay is the repair
loop; adjacent passing tests and periodic full baselines are still needed to find
regressions outside the cohort.

## Remaining implementation plan

T13 owns the remaining textual-source cases. Source known at compilation time
is implementable without a runtime interpreter; the missing evaluation-unit and
environment representation is compiler work, not a permanent policy exclusion.
The existing [precompiled Script contract](contracts/precompiled-realm-scripts.md)
is a design contract and has no executable registry yet.

1. **Prepare independent source units (T03/T13).** Retain syntax-proven source
   text and parse it with the ordinary front end under the correct Script,
   parameter and function-body goals. A closed prepared result distinguishes
   executable IR from deferred ECMAScript SyntaxError. Compiler bugs remain
   diagnostics. Preserve argument evaluation and runtime callable identity.
2. **Provide execution global environments (T08/T09/T13).** Created realms need
   an actual global environment; their current global-environment slot remains
   zero. Global reads, writes and declaration ownership must select that
   environment instead of the entry realm's singleton global object. This is a
   prerequisite for functions or scripts that access foreign global bindings.
3. **Compile nonempty static Function bodies (T13).** Lower parameters and bodies
   through the ordinary Function IR path, allocate a fresh function per call,
   and preserve constructor/newTarget realms and deferred grammar errors. The
   remaining static Function-family diagnostics cover twelve ordinary-function
   HTML-comment grammar executions, two AsyncFunction executions and two
   AsyncGeneratorFunction executions. Eight more AsyncGeneratorFunction calls
   reach the typed runtime capability boundary after callable identity is lost.
4. **Execute precompiled Script units (T13).** Add repeatable Script thunks and
   call-time GlobalDeclarationInstantiation. Complete conflict and descriptor
   checks before mutation, create fresh declared functions on each evaluation,
   preserve completion values, and restore the caller realm on abrupt exits.
   Use this path for indirect eval and realm evalScript.
5. **Connect direct eval to caller environments (T08/T13).** Preserve strictness,
   lexical/variable/private environments, caller-visible declarations and Annex B
   rules. A nested ordinary function or source splicing cannot supply these
   semantics. Verify scope mutation and declaration failures before broad replay.
6. **Reclassify the remaining source expressions from evidence (T13).** Some
   current runtime-source diagnostics involve finite tables or generated strings;
   determine which can gain a general compile-time proof. Truly runtime source
   compilation remains explicitly unsupported under the artifact contract.

Each step needs focused positive and negative controls, then replay of its exact
remaining execution IDs. Full conformance claims still require a complete pinned
suite publication. These steps are outstanding work, not completed repairs in
this patch.

## Verification

The recorded cohort compiler was built at `e9d398572` with executable SHA256
`d2ad0254ab36cf4e5eca1112527d8a0266021e889c374a91da740fc36a7454ee`.
Verification ran sequentially with the memory limits described above:

- All 50 core Wasmtime integration regressions passed at `5ee31c794`.
  Between that revision and the cohort checkpoint, only runner accounting and
  tests changed; the compiler/runtime implementation exercised by those
  regressions stayed unchanged.
- The cohort build passed 14 Arguments iteration tests, the original Array.find
  inherited-index regression, and 58 adjacent Wasmtime integration tests for
  async loops, captured bindings, control flow, Intl options and suspended
  references.
- Seven focused Test262 runner tests passed, including actual Wasm controls for
  simultaneous root/worker failures, exact runtime-negative constructor matching,
  async completion errors, timeout precedence and pinned agent host ordering.
- The complete IR command passed 1,283 tests across the library, integration
  targets and documentation. One existing ignored documentation example remains
  in `operations::SpecOperationCatalogEntry`; no Test262 execution is skipped by
  that documentation annotation.
- All three emitted-artifact checks and
  `cargo check --release --workspace --all-targets --locked -j 2` passed.
- Formatting, module boundaries, repository paths, task-plan consistency,
  product dependency checks and the unchanged 181-entry shortcut audit passed.
  Replay tooling passed five tests. The publication driver and durable-progress
  tooling each passed 45 tests after the latest main-branch update.

Two additional native runner probes used an unchanged fixture suite before and
after the accounting repair. `$DONE(error)` followed by late eval changed from
NotImplemented to Bug; `$DONE(error)` followed by an expected TypeError changed
from an incorrect Success to Bug. Both now retain the async failure detail.
These are regression fixtures, separate from pinned conformance evidence.

The exact 592-execution replay completed on 2026-09-08. Every execution ID and
native snapshot was reconciled against the captured cohort:

| Original outcome | Now Success | Now NotImplemented | Now Bug | Now Crash |
| --- | ---: | ---: | ---: | ---: |
| NotImplemented (518) | 16 | 502 | 0 | 0 |
| Bug (68) | 43 | 25 | 0 | 0 |
| Crash (6) | 6 | 0 | 0 | 0 |
| **Total (592)** | **65** | **527** | **0** | **0** |

All six original crashes now pass. The 25 Bug-to-NotImplemented transitions
remain failures: seventeen expose missing realm-eval compilation (sixteen former
sentinel TypeErrors and one wrong exception constructor), and eight expose
nonempty AsyncGeneratorFunction bodies previously represented by thrower stubs.
These are corrected failure classifications, not implemented source compilation.

Every remaining failure has kind `Unsupported`. Their diagnostic groups are:

| Missing capability | Executions | Owner |
| --- | ---: | --- |
| Direct eval caller environment | 312 | T08/T13 |
| Indirect eval target-realm environment | 160 | T09/T13 |
| Static ordinary Function body and target realm | 12 | T09/T13 |
| Static AsyncFunction body and target realm | 2 | T09/T13 |
| Static AsyncGeneratorFunction body and target realm | 2 | T09/T13 |
| Eval source-after-AOT diagnostic | 4 | T13 |
| Function source-after-AOT diagnostic | 4 | T13 |
| Runtime-selected `$262.evalScript` without specialization | 17 | T09/T13 |
| Runtime-selected AsyncGeneratorFunction without specialization | 8 | T13 |
| Runtime-selected eval without specialization | 6 | T13 |
| **Total** | **527** | |

Runtime selection and source-after-AOT diagnostics do not establish that source
is inherently unknowable: the remaining plan must distinguish recoverable static
proofs from source that actually requires runtime compilation.

The execution-list SHA256 was
`cac4c0199c8fb1392a10a6c18a48736817034c2abdb13e27d79d770aa17c7ec9`.
The cohort verification service peaked at 10,741,112,832 bytes, below its 12 GiB
hard limit. It exited one because the replay still has non-passing executions;
there were no missing reports or infrastructure failures.

Follow-up review at `1b9ecae11` corrected an unsupported-only combination outside
the cohort: one worker's compile-time source gap plus another worker's runtime
source rejection had been classified as a concurrent bug. A real Wasm regression
failed before the six-line classification correction and passed afterward, while
retaining both diagnostics. This follow-up passed all ten dynamic-source engine
regressions in debug mode, six aggregation unit tests, four affected Test262
runner tests (including both empty and TypeError runtime-negative controls for
the combined gap), and the workspace check. Its verification service peaked at
5,401,927,680 bytes. The 592-case counts above belong to the recorded `e9d398572`
compiler; that entire replay was not repeated for this classification follow-up.

CI's cold debug build exposed a 30-second fixture deadline that included worker
compilation inside `Agent.start`. The aggregation fixture now allows 120 seconds
while retaining its exact failure assertions. Its isolated cold debug check
passed in 35.96 seconds after a 3m 07s build, with a 4,765,683,712-byte service
peak. The regression workflow allows 90 minutes: the previous 45-minute budget
was nearly exhausted before its remaining runner checks could start. Production
timeout behavior and the intentional timeout regression are unchanged.

New full-suite counts require a fresh complete publication from the Rust
publisher; this partial replay does not authorize editing the generated README
status block.
