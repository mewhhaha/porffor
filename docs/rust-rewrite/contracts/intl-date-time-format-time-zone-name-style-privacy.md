# Intl.DateTimeFormat time-zone-name style privacy

Status: implemented as a source-equivalent T23 invariant closure.

The owner-private `TimeZoneNameStyle` is the sole six-style authority for
DateTimeFormat's `timeZoneName` option. Its complete list, resolved spelling,
UTC-family name, and heap code projections remain exhaustive and have no
fallback arm. Other backend modules cannot construct a style or select one of
those projections independently.

Restoring only the former enum visibility reproduces the exact original
66-line domain and projection source with SHA-256
`ee5ac6a3396cdf58e102796ca82dbc6c75bf2799fe8c93bc3c42f17d091ea117`.

This source-equivalent hardening has no new Intl behavior and does not close T23.
It changes no accepted style, spelling, localized name, heap code, emitted
instruction, Test262 materialization, or published count.

At the Batch BJ checkpoint, `cargo xc` is green, the recursive style target
passes `3/3`, and the exact six-style constructor leaf passes both Wasm-AOT
executions with every failure bucket at zero.
