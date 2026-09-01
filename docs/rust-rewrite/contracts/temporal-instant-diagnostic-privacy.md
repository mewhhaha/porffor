# Temporal.Instant diagnostic privacy

Status: implemented as a source-equivalent T22 invariant closure.

The owner-private `TEMPORAL_INSTANT_NON_INTEGRAL_EPOCH_MILLISECONDS_MESSAGE`
belongs only to the `fromEpochMilliseconds` non-integral Number RangeError
path. The owner-private `TEMPORAL_INSTANT_VALUE_OF_MESSAGE` belongs only to the
unconditional `valueOf` TypeError path. Each name has one declaration and one
consumer in `temporal_instant.rs`; neither diagnostic is a cross-module API.

The string pool continues to own identical literal rows independently. Making
the codegen names private does not change interning, emitted bytes, current-realm
error construction, or the observable diagnostic text.

Restoring only the former non-integral diagnostic visibility reproduces the
exact original three-line source with SHA-256
`783be630ab0b186ca6e47d703313d37314540e71454c1d1ec5f994b93f4a249d`.
Restoring only the former `valueOf` diagnostic visibility reproduces the exact
original two-line source with SHA-256
`e50fae0dab7f68f5d12df521f40cea34c2d47ddf0e71078e02871ef30b754b11`.

At the Batch BQ checkpoint, `cargo xc` is green, the focused structure target
passes `3/3`, and the exact non-integral plus implicit-conversion leaves pass
all four Wasm-AOT executions with every failure bucket at zero.

This source-equivalent hardening has no new Temporal behavior and does not close T22.
It changes no throw kind, evaluation order, interning row, emitted
instruction, Test262 materialization, or published count.
