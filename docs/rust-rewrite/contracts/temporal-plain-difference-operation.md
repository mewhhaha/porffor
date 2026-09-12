# Temporal plain difference operation

Status: normative for the shared `until` / `since` dispatch of the four plain
Temporal types.

## Boundary

`Temporal.PlainDate`, `Temporal.PlainYearMonth`, `Temporal.PlainTime` and
`Temporal.PlainDateTime` each have one shared difference emitter for their
`until` and `since` prototype methods. The closed
`TemporalPlainDifferenceOperation::{Until, Since}` domain owns two correlated
decisions:

| Operation | Rounding mode | Final duration |
| --- | --- | --- |
| `Until` | Use the requested mode directly | Preserve the computed sign |
| `Since` | Negate the requested mode | Negate the computed result |

All eight standard-builtin producers construct this operation directly. The
domain is visible only within the builtin module tree and has no default,
wildcard or Boolean projection. PlainDate, PlainYearMonth and PlainTime match
it exhaustively at their rounding and final-sign decisions.

As of 2026-09-10, PlainDateTime maps the operation into
`TemporalDateTimeDifferenceSettingsPlan::{PlainUntil, PlainSince}`. The shared
settings reader applies rounding-mode negation once; the direct arithmetic
consumer `emit_temporal_difference_date_time` owns final-result negation.
ZonedDateTime selects the corresponding `ZonedUntil` or `ZonedSince` settings
plan and maps its method direction to the same operation domain. It no longer
passes an options object to a PlainDateTime difference builtin.

The shared arithmetic lives in `builtins/temporal_difference.rs` and receives
already converted fields plus a borrowed
`ResolvedTemporalDateTimeDifferenceSettings` witness. The entry emitter owns
the four settings locals until arithmetic finishes. A separate closed
`TemporalDifferenceContext::{Plain, Zoned { offset_seconds_local }}` determines
calendar-candidate range checks. This keeps operation direction separate from
the receiver's range rules. The
[ZonedDateTime difference contract](temporal-zoned-date-time-difference-default.md)
defines the hour/day fallback and time-zone boundary.

## Observable witness

`wasm_temporal_plain_difference_operation.js` executes all eight producers with
`roundingMode: "ceil"`. Its vectors are deliberately non-integral at the
selected unit, so a wrong rounding mapping changes the magnitude as well as the
sign:

- PlainDate, PlainYearMonth and PlainDateTime return `3` years from `until` and
  `-2` years from `since`;
- PlainTime returns `5` hours from `until` and `-4` hours from `since`.

The values come from the pinned Test262 `roundingmode-ceil.js` witnesses for
both methods on all four receiver families. Every assertion identifies the
receiver and operation whose producer mapping failed.

## Current verification requirements

The 2026-09-10 direct-arithmetic batch requires a new native checkpoint. Source
review and the historical results below do not establish its runtime result.

```sh
cargo test -p lila-aot-wasm --test temporal_plain_difference_operation_structure
cargo test -p lila-aot-wasm --test temporal_zoned_date_time_difference_defaults_structure
cargo test -p lila-aot-wasm --test temporal_plain_arithmetic_operation_structure
cargo test -p lila-cli --test cli date::run_wasm_backend_distinguishes_plain_temporal_until_and_since -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
cargo fmt --all -- --check
git diff --check
```

The source targets must retain the exact two-variant operation domain, four
typed plain entry emitters, four-plus-four standard producer census, and
rounding/final-sign ownership across the shared arithmetic call. The coupled
ZonedDateTime target must pin all four settings plans and direct witness use,
including the absence of the retired options transport.

## Historical verification: 2026-09-01 checkpoint

The original operation-domain migration replaced six Boolean call arguments
in three emitters; PlainDateTime already named its operation. At that
checkpoint, ZonedDateTime used a separate `ZonedDelegate` settings state and
passed unnegated settings through an internal options object to the selected
PlainDateTime builtin. That transport state and its serializers were removed
by the 2026-09-10 direct-arithmetic batch.

The original structure target passed `3/3`; the then-current ZonedDateTime and
arithmetic targets passed `5/5` and `3/3`; and the exact CLI witness passed
`1/1`. The module boundary policy, scoped Rust and fixture formatting, scoped
diff check and `cargo xc` passed for that checkpoint.

Its shared semantic golden passed `2/2` in 697.36 seconds and
contained 671 dumps. Relative to the preceding 669-dump checkpoint it added only
this fixture and the independent `Array.fromAsync` Promise-Realm fixture,
removed none and left all 669 retained dumps equal after accounting
normalization. This confirmed the intended source-equivalent retained output.
No Test262 tree was run.

## Scope

The original domain-only closure did not change arithmetic or observable
option access. The 2026-09-10 batch does change difference arithmetic and
ZonedDateTime range/option ordering, so the old source-equivalence result does
not carry forward to those changes. Neither checkpoint establishes named-zone
or DST support, completion of the broad Date/Temporal ladder, or a new pinned
Test262 status publication.

## Direct-arithmetic checkpoint completed 2026-09-12

The [ZonedDateTime baseline follow-up](../zoned-date-time-baseline-follow-up.md)
records 408/408 pinned executions, 49/49 focused Wasmtime regressions (including
all ten difference tests), 1,122 IR tests, 428 backend tests, 102 Temporal
structural tests, the workspace check, and 191/191 fake-fixture executions.
The pinned replay repairs 136 reproduced main failures and retains the other
46 observed passes. These results do not publish a new full-suite baseline.
