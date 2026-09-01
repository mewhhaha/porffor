# Intl.DateTimeFormat option privacy

Status: implemented as a source-equivalent T23 invariant closure.

The owner-private `IntlDtfOption` record binds each DateTimeFormat option's
property spelling, heap slot, and accepted spelling/code rows. Its fields, the
ten-row component table, and the hour-cycle/date-style/time-style option rows
are private to `intl_datetimeformat.rs`. Other backend modules cannot construct
an option row or introduce a competing table.

Restoring only the former record, field, and four table visibilities reproduces
the exact original 95-line source with SHA-256
`8a430fd40eae20d6975444489d14d9c6d4ef75deaabe9bacde5bd8fce382dc0f`.

This source-equivalent hardening has no new Intl behavior and does not close T23.
It changes no option order, accepted spelling, heap slot, runtime code, emitted
instruction, Test262 materialization, or published conformance count.

At the Batch BK checkpoint, `cargo xc` is green and the recursive privacy
target passes `2/2`. The exact general component-order and date/time-style
order leaves pass all four Wasm-AOT executions with every failure bucket at
zero.
