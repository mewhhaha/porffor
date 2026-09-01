# Temporal ZonedDateTime direction dispatch

Status: implemented and verified for the current Wasm-AOT ZonedDateTime
arithmetic and difference surface.

## Invariant

`ZonedDateTimeArithmetic::{Add, Subtract}` and
`ZonedDateTimeDifference::{Until, Since}` are private, non-derived domains in
`builtins/temporal_zoned_date_time_methods.rs`. Their exhaustive projections
to the corresponding PlainDateTime builtin identities remain private with the
two shared emitter bodies.

The shared catalog dispatcher can call only four fixed entries:
`emit_temporal_zoned_date_time_add_builtin`, `subtract_builtin`,
`until_builtin` and `since_builtin`. It cannot import either direction domain,
select a variant, or call either raw emitter. `builtins/mod.rs` does not
re-export the domains.

The module audit requires the exact private domains, four fixed entries and
four fixed catalog routes, rejects raw emitter calls and escaping domains, and
budgets the family owner independently. The structural target pins the exact
variants, exhaustive projections, fixed entry-to-variant mapping, fixed
catalog routes, private raw emitters and absent re-export.

## Source-equivalence witnesses

No instruction-emitting statement changed. Reconstructing only the former
derive attributes and visibility of the two direction domains produces the
exact original 36-line selection with SHA-256
`82f3f206759543894d9ec36a278938c4a17e3f0db2602df13f9c9e7c1f1756a0`.
Reconstructing only former visibility on the 122-line arithmetic emitter and
217-line difference emitter produces their exact original SHA-256 values
`0df4c7b1b768c8520b30f505c8d5c5f6e18d1a8dbee0dff7b08149f2aa3bbde2`
and
`8c95229bd602e45445a7c6ad5e2a89b3d120b903be74b73ac185782859d73cdf`.

## Verification

- `cargo xc` passes; existing workspace warnings remain.
- `temporal_zoned_date_time_dispatch_structure` passes `3/3`.
- Four neighboring ZonedDateTime structure targets pass `15/15`.
- The exact arithmetic/era and difference-default CLI controls each pass
  `1/1`.
- Formatting, module-boundary, task-plan and exact Test262 shortcut gates pass.

## Nonclaims

This is source-equivalent compiler hardening with no new Temporal behavior,
Test262 pass or published-status change. The documented ZonedDateTime DST and
observable ordering gaps remain open, and this does not close T22.
