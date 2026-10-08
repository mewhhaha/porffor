# Temporal ZonedDateTime direction dispatch

## Source replacement — 2026-10-05

The GC source retains the shared closed direction domains and four fixed entries. Each entry now passes complete argument and options values to the branded GC receiver authority. The former payload/tag source recipe has been retired; the unchanged fixed-route and historical receipt checks remain.

The pre-retirement source is identified exactly:

- `temporal_zoned_date_time_dispatch_structure.rs`: SHA-256 `c877c582ca0e3f45733a8d23c330cc2eeae9b01ac33313d2ce74f113e8c508af`.

The earlier verification checkpoints below do not verify the GC replacement. The atomic GC source and its finite controls are authored and unexecuted; compilation, Wasm validation and runtime conformance remain unverified.

Status: staged source contract for the T22 named-zone authority batch, 2026-09-30.
The new leaves, shared arithmetic and named provider are staged and unexecuted;
this document records no new runtime result or published-suite count.

## Current invariant

The shared closed direction domains
`TemporalZonedArithmeticOperation::{Add, Subtract}` and
`TemporalZonedDifferenceOperation::{Until, Since}` belong to
`builtins/temporal_zoned_arithmetic.rs`. The four fixed entries in
`temporal_zoned_date_time_methods.rs` select exactly one direction and pass the
opaque branded receiver plus complete original argument/options values to that
single authority. They do not convert through PlainDateTime or re-coerce values.

The authority owns duration conversion, calendar arithmetic, exact elapsed
arithmetic and zoned difference settings. Add/subtract read duration and overflow
before local projection. Until/since convert the other operand and compare actual
calendars before reading settings; date-unit comparisons use PrimaryIdentifier,
while time-unit differences use exact epochs. Retained proof handles keep zone,
calendar and normalized epoch tied to their actual construction and allocation.

The shared catalog dispatcher still calls only four fixed entries:
`emit_temporal_zoned_date_time_add_builtin`, `subtract_builtin`, `until_builtin`
and `since_builtin`. It cannot select either operation domain or call either
private raw leaf wrapper. `builtins/mod.rs` does not re-export the domains.
The module guard locates the shared authority and checks four fixed routes;
the retained structure target checks entry-to-direction mappings and
fixed catalog routes into that authority. Runtime meaning belongs to the
forthcoming named-zone native and exact pinned controls, not those source checks.

## Retired owner distinction

The former private, non-derived domains `ZonedDateTimeArithmetic` and
`ZonedDateTimeDifference` and their PlainDateTime arithmetic projection are
retired by this coherent batch. The old fixed-zone difference machinery is
replaced by the shared zone-aware authority. These historical ownership claims
and source-equivalence receipts below remain evidence of their original
checkpoint; they are not claims about the replacement source.

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

The historical closure introduced no new Temporal behavior, Test262 pass or
published-status change. PR47's shared fixed-zone difference/ordering work has
separate evidence. The staged T22 replacement requires the full named consumer
graph and its actual verification checkpoint before named admission; it does
not establish full Temporal conformance and does not close T22.
