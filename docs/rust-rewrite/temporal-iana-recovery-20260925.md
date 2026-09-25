# Temporal IANA lane recovery, 2026-09-25

The interrupted lane implemented named Temporal time zones through the existing
pinned IANA 2026a owner in `lila-intl`. The compiled JavaScript operations issue
closed time-zone queries; the host has no JavaScript parser or interpreter.
See the [time-zone contract](intl-named-time-zones.md#temporal-time-zones).

## Recovered changes and provenance

The saved lane contained `488ec24fb` (named zones) and `a1dd07c08` (compare ISO
date-until candidates before clamping their day), plus an uncommitted README
correction. Recovery preserved that correction, rebased onto `62c4df308`, and
resolved one catalog test conflict by retaining both the upstream Intl entries
and the new Temporal entries. The corresponding commits are `aaad4dd3c`,
`b5cd6c490` and `700c16608`.

Review found that `i64::MIN.abs()` could panic or overflow through an unchecked
Temporal time-zone wire value. `eedb2425e` uses unsigned magnitude checks and
adds decoded-wire coverage for both integer extremes, every query kind, and
named/fixed zones. The request encoder now derives the identifier from its
stored zone, making an inconsistent second identifier impossible to supply.

Saved runs use upstream Test262 `7ab7faf`. Their snapshot
`pinned_revisions.test262` value is the vendored **tree** hash
`91b2052adad1f066ae031e2ff3a1e9bd6d732886`, matching `git ls-tree 62c4df308
test262/vendor/test262`; it is not the upstream commit ID. Both sides used the
same manifest hash for each selection. The baseline executable was saved as
`tziana/base2-lila` after rebuilding the then-current `b2537c088` baseline;
`tziana/after4-lila` includes the final named-zone and ISO-date fixes before
recovery. These runs do not verify the subsequently integrated branch.

The recovered machine-local evidence is under:

```text
/tmp/claude-1000/-home-mewhhaha-src-porffor/8e1032a6-2542-412b-9d45-28a78003b16f/scratchpad/
```

The prior agent transcript is under the matching Claude project/session path,
`subagents/agent-ae21a10df4f3e4818.jsonl`. These paths are provenance references,
not checked-in test dependencies.

## Completed Test262 evidence recovered from disk

These are complete **directory selections**, separate from the published full
pinned-suite status. No global status block or authoritative counts were changed.
Each baseline/after comparison uses completed execution IDs and failure sets.

| Selection | Baseline passes | After passes | Fixed | Regressed |
|---|---:|---:|---:|---:|
| `intl402/Temporal` | 880/4058 | 1190/4058 | 310 | 0 |
| `intl402/DateTimeFormat` | 462/490 | 466/490 | 4 | 0 |

The Temporal after counts by family are: Duration 40/42 (baseline 8), Instant
32/34 (26), Now 6/6 (0), PlainDate 212/986 (200), PlainDateTime 222/966 (202),
PlainMonthDay 98/180 (98), PlainTime 24/24 (24), PlainYearMonth 148/654 (148),
and ZonedDateTime 408/1166 (174).

Completed snapshot files and their SHA-256 hashes:

| Snapshot directory / file | SHA-256 |
|---|---|
| `snap-b2-intl-temporal/x-14224850054162780710.json` | `0e94d0f29d6f170b09d2f58e1c8ea300812946ee830682188bb8dfe4ba7100ce` |
| `snap-a4-intl-temporal/x-14224850054162780710.json` | `e9ce30be34fd607a740e3f30a1f81afbdfe4054847b4e37bac0c3bd9aa40c58b` |
| `snap-b2-dtf/x-10516745908827578212.json` | `ead79c22d4667d6a987a57f98097da5edbebc9343116d39949ea5ff8895f8ec8` |
| `snap-a4-dtf/x-10516745908827578212.json` | `39b194931c8e3f1e2a54df6ead9e06fc421b1a15fadd1c97eb2df7a0d6644934` |

The recorded invocation, replacing `<dir>` and `<scratch>` for each selection:

```sh
TZ=UTC LC_ALL=C.UTF-8 LILA_TEST262_FORCE_CASE_RUNNER=1 \
  systemd-run --scope --user --quiet --slice=lila-build.slice \
  -p MemoryMax=10G -p MemorySwapMax=0 -- nice -n 5 \
  ./target/release/lila --jobs 1 test262 run <dir> \
  --execution-backend wasm-aot --suite-root test262/vendor/test262 \
  --snapshot-dir <scratch> --snapshot-name x --threads 3 --timeout-ms 60000
```

## Incomplete built-ins sweep

The original `built-ins/Temporal` baseline and after processes survived the API
failure and were still running during recovery. Their selection contains 9,210
executions. At 2026-09-25 17:05:46 UTC, the saved checkpoints showed baseline
6,660 passes out of 6,810 completed, and after 5,934 passes out of 5,940 completed.
Among the 5,940 common completed IDs, 132 were repaired and none regressed.
This is partial evidence: neither the missing executions nor the final
integrated branch have a complete verdict in these checkpoints.

The six after failures observed at that checkpoint are the strict/sloppy pairs
for `PlainDate/prototype/with/order-of-operations.js`,
`PlainTime/from/argument-plaindatetime.js`, and
`ZonedDateTime/timezone-iso-string.js`. They already failed on the baseline and
remain T22 Temporal semantics work. Their full exception details remain in
`snap-a4-builtins-temporal/x-12616516935041329389.json`.

## Other saved verification

All results below were recovered from the prior session's `tziana/` logs;
they were not rerun in recovery unless listed in the next section.

| Log | Result |
|---|---|
| `test-engine2.log` | 147 passed across 18 unfiltered Temporal/Intl integration targets |
| `test-engine4.log` | 28 selected engine library tests passed |
| `test-cli.log` | `date::` CLI tests: 30 passed |
| `test-intl-ir.log` | 1,804 passed, no failures, one existing ignored test |
| `test-aot4.log` | 2,067 passed, one descriptor-owner census failure |
| `test-engine5.log` | 61 passed, two compilation-pool configuration failures |

The descriptor census failure is
`every_descriptor_object_producer_routes_through_the_owner`, fixed separately
on the integration branch. The engine failures report `Wasm compilation pool
already configured for 8 jobs` in
`array_present_index_helper_preserves_growth_holes_values_and_updates` and
`emitted_pattern_compiler_roundtrips_owned_descriptors_without_candidate_lookup_or_host_calls`.
They require integration verification; they are not reported as passing here.

## Fresh recovery verification

At recovered commit `eedb2425e`:

- `cargo test --release --locked -j3 -p lila-intl temporal_time_zone -- --test-threads=3`:
  14 passed, including the integer-boundary regression and every-zone transition
  consistency checks.
- `cargo test --release --locked -j3 -p lila-engine --lib intl_time_zone_host::tests::temporal_query_answers_named_and_offset_zones_and_faults_on_bad_requests -- --test-threads=3`:
  one passed.
- `bash scripts/check-module-boundaries.sh`: passed.
- `git diff --check`: passed.

Both Cargo commands ran in `systemd-run --scope --user` with `MemoryMax=10G`,
`MemorySwapMax=0`, and `nice -n 5`. Broad final integration tests belong to the
integrator; recovery did not duplicate the active full directory sweeps.

## Remaining domains and ownership

Non-ISO Temporal calendar arithmetic remains T22 work. The failed
ZonedDateTime roundtrip/extreme-date fixtures also exercise non-ISO calendars;
their failure is not evidence that named zones are treated as UTC. Other known
remaining Intl failures include DurationFormat construction and non-English
calendar month names, owned by the Intl implementation.

`intl402/Temporal/ZonedDateTime/links.js` expects Pacific/Johnston to compare
equal to Pacific/Honolulu. The shared named-zone provider retains Johnston as
primary under its country-preserving backzone rule. This existing data/spec
edge is documented in the [pinned data notes](../../crates/lila-intl/data/iana-tzdb-2026a/README.md)
and remains owned by that provider; it is not suppressed or special-cased.

The directory runs are not green, no failures are silently skipped, and this
work does not establish full ECMAScript or Test262 conformance.
