# Temporal baseline follow-up — 2026-09-10

This batch starts from freshly fetched `origin/main` at
`30c745c7bc165dc6f5513abc58288778288e15c6`, after PR #45 merged. The original
full baseline continues on the frozen older compiler `c5115bf03`.

## Observation and comparison

The frozen observation contains 40,795 of 102,043 exact executions across
309 completed matrix nodes: 36,966 Success, 1,970 Bug, 740 Crash and
1,119 NotImplemented. The pinned Test262 tree is
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. Active partial nodes are excluded.
This is partial progress, not a published full-suite result.

The seven nodes completed since the previous 39,299-execution observation add
1,496 executions and 134 failures. All 302 earlier leaf hashes are unchanged.
The collector reconciles every completed leaf, identity and outcome before
excluding the previous 3,695 failure identities. Its
[portable provenance](../../test262/replays/observed-temporal-20260910.provenance.json)
records input and output hashes and the exact reconciliation.

The [134 newly observed identities](../../test262/replays/observed-temporal-20260910.executions)
comprise 30 PlainYearMonth and 104 ZonedDateTime executions. A separately built
clean main compiler reproduces all 134 as Bug, with no Success,
NotImplemented or Crash outcomes. Its SHA-256 is
`e58d1ff48d75e6ca6fac5f8bbaf8b75a96dc62022993abcd470154a7511962fa`.
The observation list is a repair inventory, never an expected-failure list.

## Changes

- PlainYearMonth parsing preserves whether the source omitted a day and rejects
  a bare year-month with a non-ISO calendar before reading overflow options.
  Addition and subtraction use the first day for either duration sign.
  Differences validate their intermediate date range and both calendar-rounding
  brackets; month rounding retains whole years and uses the increment quotient
  for half-even ties. Expanded month rounding performs the subsequent year
  calculation and validates its date boundary before selecting the result.
- ZonedDateTime publishes static `compare`, calendar-derived getters and
  `toString` through the existing intrinsic and Wasm code-generation paths.
  Comparison converts both operands in order and compares exact epoch values.
  Field conversion uses the existing month-code primitive and syntax checks.
  Explicit-offset interpretation checks the ISO day range at its specified step.
- ZonedDateTime string formatting reads options in specification order and
  rounds the epoch before constructing local fields and annotations. UTC-day
  decomposition keeps epoch arithmetic exact at the supported boundaries,
  including negative nanoseconds and half-even ties. PlainTime, PlainDateTime
  and ZonedDateTime share the same seconds-string precision calculation.
  The shared unit-option reader rejects unknown spellings immediately; known
  units and `auto` reach the algorithm-specific validation after later options.

The rounding phases follow [RoundRelativeDuration and its year-boundary step](https://tc39.es/proposal-temporal/#sec-temporal-roundrelativeduration).
Formatting follows [ZonedDateTime toString](https://tc39.es/proposal-temporal/#sec-temporal.zoneddatetime.prototype.tostring),
including the distinction between option spelling and unit suitability.

The changes use the existing UTC and fixed-offset time-zone implementation;
this batch does not claim named-zone or DST support.

## Verification

All 23 focused Wasmtime regressions pass across four native targets.
Pinned replay and broader verification are in progress. The
[346-execution replay](../../test262/replays/temporal-follow-up-20260910.executions)
contains all 134 observed failures, 68 previously passing neighbors and all
124 executions in the pinned ZonedDateTime `prototype/toString` directory,
plus 20 distinct unit-option controls across the affected Temporal types.
Native Wasmtime regressions cover the repaired behavior and shared formatting
paths. Every raw outcome is retained.

Reproduce the candidate replay after building the CLI:

```sh
cargo build --release --locked -j2 -p lila-cli
python3 scripts/replay-test262-executions.py \
  test262/replays/temporal-follow-up-20260910.executions \
  --binary target/release/lila \
  --output-dir target/failure-review/temporal-follow-up-replay --workers 4
```

The replay tool freezes the compiler and exact list, uses one native execution
per invocation, and retains snapshots and transcripts. The full baseline and
its remaining failures are separate from this focused verification; the
canonical README conformance status must still be refreshed by the publisher.
