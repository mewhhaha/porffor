# Temporal calendar-carrier privacy

Status: implemented as a source-equivalent T22 invariant closure.

The owner-private `TemporalCalendarCarrier` is the sole pairing between each
Temporal brand that carries a `[[Calendar]]` slot and that slot's record
offset. Its private complete list drives the only raw slot fast path. Exhaustive
brand and offset projections mean that adding a carrier requires both halves of
the representation to be selected before the backend builds.

The raw fast path is private too. Only the existing calendar canonicalization
and `ToTemporalCalendarIdentifier` paths can invoke it, and neither can supply
a substitute calendar payload. A branded Temporal object therefore yields its
own stored calendar without observable `calendar` or `calendarId` property
access.

Restoring only the former visibility of the carrier and its projections
reproduces the exact original 45-line source with SHA-256
`1726881c45223f008814169edef8a3066c23b8733d86714d63570535ba3dd831`.
Restoring only the former visibility of the raw fast path reproduces the exact
original 54-line source with SHA-256
`a74006922ea5018cd1d001421de4f83b70c23db9b73924ab24627415c642765c`.
At the Batch BN checkpoint, `cargo xc` is green, the focused structure target
passes `3/3`, and the exact five-carrier getter-suppression leaf passes both
Wasm-AOT executions with every failure bucket at zero.

This source-equivalent hardening has no new Temporal behavior and does not close T22.
It changes no accepted calendar, getter observation, stored payload, emitted
instruction, Test262 materialization, or published count.
