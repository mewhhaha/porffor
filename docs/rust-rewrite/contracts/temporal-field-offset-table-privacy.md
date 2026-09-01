# Temporal field-offset table privacy

Status: implemented as a source-equivalent T22 invariant closure.

The owner-private `TEMPORAL_DURATION_FIELD_OFFSETS` table maps the ten Duration
constructor locals to the ten record slots in declaration order. The
owner-private `TEMPORAL_PLAIN_DATE_TIME_FIELD_OFFSETS` table does the same for
the nine PlainDateTime ISO and wall-clock fields. Each table has exactly two
consumers in its defining module: allocation and record loading.

These active codegen tables remain separate from the passive T05 heap-layout
metadata. The latter includes tracing names, widths and pointer classification;
coupling active field-local order to that pre-cutover metadata would give two
independent concerns one lifecycle. Privacy closes the actual owner boundary
without introducing that dependency.

Restoring only the former Duration table visibility reproduces the exact
original 12-line source with SHA-256
`b47f9d79e4e1dc65b91a4ac7a2663a20b54cb5b6aea099266b381e6380e06ab1`.
Restoring only the former PlainDateTime table visibility reproduces the exact
original 11-line source with SHA-256
`f7047424c3fe0d3837f3d5db310d41d2c7a61740badcb97ec606c89c65746123`.

At the Batch BP checkpoint, `cargo xc` is green, the focused structure target
passes `3/3`, and the exact Duration ten-field plus PlainDateTime nine-field
constructor leaves pass all four Wasm-AOT executions with every failure bucket
at zero.

This source-equivalent hardening has no new Temporal behavior and does not close T22.
It changes no field order, record offset, emitted instruction, Test262
materialization, passive heap-layout metadata, or published count.
