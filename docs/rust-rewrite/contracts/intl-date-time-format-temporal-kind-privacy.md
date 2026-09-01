# Intl.DateTimeFormat Temporal-kind privacy

Status: implemented as a source-equivalent T23 invariant closure.

The owner-private `IntlDtfTemporalKind` record binds each supported Temporal
brand to its runtime code, public type name, allowed/default component sets,
rejected style, and time basis. Its fields and the six-row
`INTL_DTF_TEMPORAL_KINDS` table are private to `intl_datetimeformat.rs`.
The owner-private `DtfTimeBasis` exhaustively selects whether the resolved time
zone applies to an exact instant or must leave plain wall-clock fields alone.

The owner-private `DtfBrandedKind` is the total branded dispatch over the six
table rows plus `Temporal.ZonedDateTime`. The owner-private `DtfValueKind`
widens that result with the legacy Number/Date path. Their exhaustive code and
brand projections have no fallback arm. No other backend module can construct
a row, add a competing table, or select the compiler's value-kind code.

Restoring only former visibility across the original 196-line record, basis,
and table region reproduces SHA-256
`d6a9458aa55ec9362cf9fb4481717b58c78f7d4a5bc856d3b377444c853a5d7e`.
Restoring the two former public enum declarations across the original 60-line
branded/value domain reproduces SHA-256
`72754a52a21a06e8db39307da0b08b43ae0d57ec28e5f7cc212e0ca1e032ee40`.

This source-equivalent hardening has no new Intl behavior and does not close T23.
It changes no table row, runtime code, component set, time-zone decision,
emitted instruction, Test262 materialization, or published conformance count.

At the Batch BH checkpoint, `cargo xc` is green and the recursive privacy
target passes `3/3`. The exact PlainDate table/basis and ZonedDateTime branded
rejection leaves pass all four Wasm-AOT executions with every failure bucket at
zero. The separate `temporal-objects-resolved-time-zone.js` leaf is not a green
control: both variants stop at the existing `Unsupported timeZone option`
boundary before this visibility-only seam is observed.
