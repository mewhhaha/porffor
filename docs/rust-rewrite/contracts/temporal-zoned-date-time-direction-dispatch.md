# Temporal ZonedDateTime direction dispatch

Status: current Wasm-AOT direction contract as of 2026-09-25 (named time zones). The structural
assertion refresh passed the [coordinated verification checkpoint](../../../test262/replays/zoned-date-time-follow-up-20260910.verification.json).

## Invariant

`ZonedDateTimeArithmetic::{Add, Subtract}` and
`ZonedDateTimeDifference::{Until, Since}` are private, non-derived domains in
`builtins/temporal_zoned_date_time_methods.rs`. Arithmetic runs
`AddZonedDateTime` through the time-zone kernel in the ZonedDateTime emitter
itself. Its one exhaustive match decides whether the duration is negated first.
It no longer delegates to the PlainDateTime `add` or `subtract` builtins,
because that wall-clock path cannot follow a named zone's transitions.
Difference dispatch uses three exhaustive matches inside its private shared
emitter:

| Direction | Settings plan | Result |
| --- | --- | --- |
| `Until` | `TemporalDateTimeDifferenceSettingsPlan::ZonedUntil` | as computed |
| `Since` | `TemporalDateTimeDifferenceSettingsPlan::ZonedSince` | negated |

The settings plan owns the hour fallback and rounding-mode direction. The
time-unit path (`DifferenceInstant`) and the date-unit path
(`DifferenceZonedDateTimeWithRounding`) each negate a `since` result in their
own exhaustive match.

The shared catalog dispatcher can call only four fixed entries:
`emit_temporal_zoned_date_time_add_builtin`, `subtract_builtin`,
`until_builtin` and `since_builtin`. It cannot import either direction domain,
select a variant, or call either raw emitter. `builtins/mod.rs` does not
re-export the domains.

The module audit requires the exact private domains, four fixed entries and
four fixed catalog routes, rejects raw emitter calls and escaping domains, and
budgets the family owner independently. The structural target pins the exact
variants, the arithmetic negation match, the settings-plan mapping, both result
negations, fixed entry-to-variant mapping, fixed catalog routes, private raw
emitters and absent re-export.

## Historical source-equivalence witnesses: 2026-09-01 checkpoint

The original direction-privacy closure changed no instruction-emitting
statement. Reconstructing only the former
derive attributes and visibility of the two direction domains produces the
exact original 36-line selection with SHA-256
`82f3f206759543894d9ec36a278938c4a17e3f0db2602df13f9c9e7c1f1756a0`.
Reconstructing only former visibility on the 122-line arithmetic emitter and
217-line difference emitter produces their exact original SHA-256 values
`0df4c7b1b768c8520b30f505c8d5c5f6e18d1a8dbee0dff7b08149f2aa3bbde2`
and
`8c95229bd602e45445a7c6ad5e2a89b3d120b903be74b73ac185782859d73cdf`.

These exact witnesses describe that original closure. They do not assert
source equivalence for PR47's replacement of difference arithmetic or its
observable ordering repairs.

## Historical verification

- `cargo xc` passed with the then-existing workspace warnings.
- `temporal_zoned_date_time_dispatch_structure` passed `3/3`.
- Four neighboring ZonedDateTime structure targets passed `15/15`.
- The exact arithmetic/era and difference-default CLI controls each passed
  `1/1`.
- Formatting, module-boundary, task-plan and exact Test262 shortcut gates passed.

The 2026-09-12 refresh replaces the stale `impl ZonedDateTimeDifference`
assertion with exact operation/settings-plan mappings. It retains privacy,
non-derived domains and all four fixed routes. Its only production-source edit
updates the obsolete difference-method comment; the coordinated structural
rerun supplies current verification separately from these historical results.

## Nonclaims

The original source-equivalent compiler hardening introduced no new Temporal behavior,
Test262 pass or published-status change. PR47 subsequently repaired the option
ordering and shared difference arithmetic; its evidence has its own scope.
The current assertion/comment refresh adds no runtime behavior. Named-zone and
DST arithmetic remain outside the supported UTC/fixed-offset domain, and this
does not close T22.
