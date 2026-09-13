# Expression semantics baseline follow-up — 2026-09-13

This batch starts from `ac017904aa72c07caf44b70db7ba3a3eb58a911a`, the merge
of PR #50, on a new branch from freshly fetched `origin/main`.

The running historical compiler baseline was frozen at 70,258 of 102,043
executions and 509 of 744 nodes: 58,984 Success, 8,262 Bug, 746 Crash and
2,266 NotImplemented. Compared with the preceding complete frozen observation
at 66,569 executions, 437 additional failures were available for investigation.
All 66,569 preceding execution identities/outcomes and 487 leaf hashes were
preserved. A later counters-only check had reached 69,403 executions; 283
additional failures were counted after that check, but counters cannot identify
which exact executions make up that delta. The 437-entry inventory has no
overlap with PR #50's 102-entry replay.

The repair scope is the 210 newly inventoried executions outside dynamic
import, plus adjacent controls. Every selected execution is reproduced using
a frozen compiler built from merged main before comparing candidate results.
Historical failures already fixed by the merged work remain passing controls.
The complete 437-entry main replay records 175 Success, 118 Bug and
144 NotImplemented, with no Crash or timeout. Within the selected 210 entries,
175 already pass and 35 still fail (17 Bug and 18 NotImplemented).
The 163 adjacent executions add 149 passing controls and 14 failures. The
combined paired baseline contains 373 executions: 324 Success, 27 Bug and
22 NotImplemented, with no Crash or timeout.

| Confirmed main failure family | Executions |
| --- | ---: |
| Finite eval, cross-realm class brands and indirect non-string eval | 16 |
| Nullish, primitive and `with` deletion | 13 |
| BigInt/string equality and object coercion | 8 |
| Number remainder and compound remainder | 8 |
| SharedArrayBuffer subclass feature admission | 4 |

## Changes

- Number `%` and `%=` share an exact binary-significand remainder emitter.
  It handles extreme finite operands, subnormals, infinities, NaN and signed
  zero, while retained operand locals protect the left value across nested
  right-hand expressions.
- Property deletion evaluates the base and raw key once, performs the required
  object and property-key conversions, then uses the shared deletion operation.
  Nullish bases throw; boxed primitive properties retain their descriptors.
  Computed `length` keys compare string contents for arrays and boxed strings.
  Identifier deletion inside `with` uses the existing environment selection,
  including `Symbol.unscopables`, without reading the property's value.
- Loose BigInt/string equality parses the string exactly and treats invalid
  BigInt text as unequal. Object coercion still occurs in its specified order,
  and the BigInt constructor retains its own SyntaxError behavior.
- Indirect and cross-realm eval can use finite source text held in captured
  bindings, reassigned identifiers, and positional parameters and defaults of
  known functions. Existing array callback source hints remain available.
  The compiler prepares those sources ahead of time; runtime callable
  identity, source equality, realm ownership and fresh private environments
  still determine execution. Unmatched source remains an explicit AOT
  capability failure. Indirect eval with widened argument types checks the
  actual value at runtime, returning non-string values unchanged without
  attempting source conversion. Source discovery remains bounded; this batch
  does not add arbitrary helper forwarding through `.call` or `.apply`.
- Private fields, methods and accessors keep their entries under their unique
  Private Name, so class construction and use after foreign eval returns can
  find the original definitions. Fresh class evaluations still have distinct
  brands. The obsolete realm-owned list is removed, and every new private-name
  slot initializes its own entry head.
- The Test262 runner admits the expression and declaration fixtures for
  `SharedArrayBuffer` subclassing. Direct merged-main Wasm execution already
  supports their construction/prototype behavior and growable storage.

The structural follow-up also restores exact checks for the existing BigInt
numeric-update routes and the twelve eager compound-assignment operators.

## Verification

The completed pinned replay passes **373/373**, repairing all **49** failures
reproduced on main and preserving **324** passing controls. There are no Bug,
Crash, NotImplemented or timeout outcomes in the candidate cohort. The
[execution inventory](../../test262/replays/expression-semantics-20260912.executions)
and [verification artifact](../../test262/replays/expression-semantics-20260912.verification.json)
retain exact mode/path identities, paired outcomes and source/result hashes.
Generated full-suite status counts are unchanged by this scoped replay.

The frozen compiler was built from
`e696f13eefb3805e8ee45fe722b578b0b6a35662` with binary SHA-256
`96607b96cb778ef6340c25467822d1e33b0c64e376e92048d28b05b297043b7c`.
The source manifest covers 2,916 inputs, with SHA-256
`2a8e39e976c13b073666400495a7422e0255a6ddaf98e57a9e69734b7af9ddbe`.
Both main and candidate use Test262 tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b` and Rust 1.94.0.

The workspace check, 61 focused IR regressions, 35 focused Wasmtime regressions,
431 backend library tests and 44 structural tests pass. The structural
checkpoint is at `db72df3e2c034f0b3fc1d35b509ebd93ae83ce5c`; only two
integration guards differ from the compiler checkpoint. They repair pre-existing
staleness on main: omitted BigInt numeric-update routes and a compound-assignment
guard that still read the parent module after lowering moved to a child.
All production code, IR/library tests, native fixtures and dependencies are
byte-identical across this test-only follow-up.

The 66 adjacent Wasmtime tests and the SharedArrayBuffer feature-admission
check also pass. The full fake fixture suite passes 191/191 executions from
190 files, separately from the real pinned replay.

The full IR library checkpoint is retained honestly as **1,125 passes and one
failed assertion**. The corrected realm-eval assertion then passed in a separate
exact-test run, with 1,125 tests filtered. A byte-exact source and dependency proof
retains the unchanged 1,125 passes; this is not a newly green full 1,126-test run.
The evidence also preserves earlier failed native/replay attempts and the stale
structural guards, without counting their failures as passes.

Refresh the paired execution cohort with a freshly built compiler:

```sh
cargo build --release --locked -j2 -p lila-cli
LILA_MODULE_MEMORY_CACHE_ENTRIES=1 ./scripts/run-watched.sh \
  --label expression-semantics-replay --stall 900 -- \
  python3 scripts/replay-test262-executions.py \
  test262/replays/expression-semantics-20260912.executions \
  --binary target/release/lila \
  --output-dir target/failure-review/expression-semantics-refresh --workers 4
```

The replay driver preserves strict/sloppy execution identities. Audit a
completed replay with `scripts/audit-test262-replay.py`, passing compiler
metadata with the actual binary SHA-256 and the paired origin report. The
auditor checks source identities, pinned suite contents, native transcripts,
snapshots, outcome counts and exact execution membership.

### CI follow-up

The first PR checks caught two bookkeeping assumptions. The shortcut audit
fingerprint still described the old SharedArrayBuffer admission selector; the
reviewed ledger and generated inventory/status now include the two subclass
paths. Classification and ownership stay unchanged: 112 observations comprise
31 legitimate harness adaptations, 39 diagnostic observations and 42 semantic
shortcuts.

The borrowed eval loop guards also selected the first prepared Script unit.
Finite source discovery now registers the indirect candidate before the direct
candidate, so the guards select the intended kind explicitly and require a
unique match. Both borrowed tests retain every binding-publication assertion;
the owned-storage control checks strict direct eval and sloppy indirect eval
separately. Loop lowering and environment ownership code are unchanged.

These corrections are recorded at
`68e235c63811338097e5620db93393418abda3c6`. Relative to the initial PR head
`b796482b5fca9ed3260f29bdb9ccb29b103082b1`, only this integration test and three
audit artifacts changed.
The compiler and the completed 373-execution replay evidence remain unchanged.
The failed [workspace contract check](https://github.com/mewhhaha/porffor/actions/runs/34744828609)
and [eval guard check](https://github.com/mewhhaha/porffor/actions/runs/34744828587)
remain available as the preceding CI checkpoints.

The full repository contract check passes, including the 15 scanner regressions;
the nine shortcut-status generator regressions and both generated-report checks
also pass. The four additional IR targets below pass 12/12 tests, and the exact
native borrowed-loop regression passes 1/1 with 26 tests filtered. These are
separate follow-up checks, not a rerun of the original replay or broad suites.
Formatting and diff checks pass.

```sh
cargo test --release --locked -j2 --no-fail-fast -p lila-ir \
  --test borrowed_eval_loop_heads --test declaration_completion \
  --test direct_eval_environment --test object_constructor_boxing -- --test-threads=2
LILA_MODULE_MEMORY_CACHE_ENTRIES=1 cargo test --release --locked -j2 -p lila-engine \
  --test aot_declaration_completion \
  borrowed_eval_loop_heads_write_the_selected_caller_bindings -- --exact --test-threads=1
```

## Remaining baseline work

The other 227 newly inventoried executions exercise dynamic import, owned by
T12 in `test262/backlog/ownership-map.tsv`. They need
separate module lifecycle work: retained graphs for computed specifiers, lazy
evaluation, and promise rejection for target parsing, linking and evaluation
errors. Source-phase imports of JavaScript modules also require the specified
rejection. These failures remain visible in the main replay; they are excluded
from this PR's repair cohort, not added to a skip list.

The original full baseline continues against its earlier compiler. Its counters
measure that compiler, while this PR's paired replay measures these changes.
Neither is a completed, current full Test262 conformance result. At the final
counters-only observation, **2026-09-13 07:10 UTC**, the original run remained
active at **79,928/102,043 executions** and **586/744 nodes**: 67,731 Success,
8,663 Bug, 782 Crash and 2,752 NotImplemented. These later counters do not extend
the 70,258-execution identity freeze or this PR's repair cohort.

A new adjacent static-member control also exposed the existing unsupported
private-field `++`/`--` lowering. Direct inspection of its class reports
`private field update target`; optional eval preparation consequently retains
the explicit runtime capability rejection. That separate numeric-update gap
remains open under T09. The private-brand control uses explicit private reads
and writes to verify its setter invocation count.
