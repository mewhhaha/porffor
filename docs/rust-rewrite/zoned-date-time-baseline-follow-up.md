# ZonedDateTime baseline follow-up — 2026-09-10

This batch starts from freshly fetched `origin/main` at
`46aabdad0c4976dfe4e41ffd982bcc26049fe153`, after PR #46 merged.
The original full baseline continues on the frozen compiler `c5115bf03`.

## Frozen observation

At task start, the baseline had completed 41,545 of 102,043 executions across
312 matrix nodes: 37,534 Success, 2,152 Bug, 740 Crash and 1,119 NotImplemented.
The pinned Test262 tree is `aa55200d1310384c5cf69ea95b2a2ecba457007b`.
Active partial nodes are excluded; this is not a published full-suite result.

All 309 previously completed leaf hashes remain unchanged. The three additional
ZonedDateTime nodes contain 750 executions: 568 Success and 182 Bug on the old
compiler. The collector reconciles every completed leaf before excluding the
3,829 previous failure identities.

The [observed inventory](../../test262/replays/observed-zoned-date-time-20260910.executions)
and [portable provenance](../../test262/replays/observed-zoned-date-time-20260910.provenance.json)
retain the exact inputs. These observations are not expected-failure entries.
A separately built main compiler reproduces 136 Bug outcomes; the other 46
executions already pass after PR #46 (42 calendar-getter cases, two equality
cases and two addition cases). No replay timed out. Its compiler SHA-256 is
`b76632b6c8a150252352f7a97db037e556894366cd232574fd726aebb826c974`.
Every native snapshot and transcript is reconciled with its exact execution
identity and pinned source hash.

## Repair scope

- Add ZonedDateTime rounding, transition queries, hours-in-day and start-of-day
  through builtin registration, spec IR and Wasm code generation.
- Add PlainDate-to-ZonedDateTime conversion required by an observed arithmetic
  boundary test, preserving conversion order and optional time semantics.
- Validate property-bag UTC offset syntax at its conversion boundary, retaining
  fractional offset precision.
- Preserve exact seconds/subseconds in wide time differences and complete
  relative calendar rounding and boundary validation. Plain and Zoned operations
  share arithmetic through a closed context that selects their distinct range
  and rounding rules; no intermediate public options bag or method call is needed.
- Round wall-clock fields from the selected field’s origin. For example,
  01:10 rounded to 20 minutes with half-even becomes 01:00. Higher fields must
  not change tie parity in PlainTime, PlainDateTime or ZonedDateTime.
  Elapsed-duration rounding retains its separate scalar authority.

Time-zone operations use the existing UTC/fixed-offset domain. This batch does
not claim named-zone transition data or DST support. Transition queries still
validate required options before returning no transition for supported zones;
day operations must validate their actual endpoint instants.

## Verification scope

All 49 focused Wasmtime regressions pass on the committed implementation
`39a0aeca0ae36f70381b5b45cfa90e2835c3e16a`. Candidate replay and broad
verification are in progress. The
[408-execution replay](../../test262/replays/zoned-date-time-follow-up-20260910.executions)
contains 182 observations, 44 historical passing neighbors, all 92 executions
from the pinned PlainDate `prototype/toZonedDateTime` surface, and 90 related
until/subtract, shared difference, time-rounding and remaining start-of-day
controls. Twelve historical neighbors are
exception-only controls for formerly missing methods; they are not standalone
evidence that those methods work. Positive native and pinned cases cover the
new behavior. No historical Success was available for hoursInDay/startOfDay in
the frozen observation.

Reproduce the replay with a newly built candidate and an unused output directory:

```sh
cargo build --release --locked -j2 -p lila-cli
python3 scripts/replay-test262-executions.py \
  test262/replays/zoned-date-time-follow-up-20260910.executions \
  --binary target/release/lila \
  --output-dir target/failure-review/zoned-date-time-follow-up --workers 6
```

The tool freezes its compiler and exact list and retains one native snapshot
and transcript per execution. The per-execution timeout remains 60 seconds.
The full baseline and generated README status remain separate from this cohort.

The first native attempt exposed a missing `plainTime` compiler-pool registration
when compiling programs that did not themselves contain that property name.
The registration was corrected before the candidate replay; the existing
month-code and PlainYearMonth regressions exercise this initialization path.
A subsequent native run passed 48 of 49 tests and exposed two shared ISO-date
diagnostics gated on Plain-family builtin bodies. They now reside in the shared
Temporal pool. A production pool regression verifies these literals using an
empty source and individually selected builtin bodies. Both failed attempts
retain their source manifests and native transcripts locally.
