# Intl.DateTimeFormat time-zone authority privacy

Status: implemented as a source-equivalent T23 invariant closure.

The owner-private `TzOffsetMinutes` newtype is the only representation of an
accepted fixed UTC offset. Its sole constructor enforces the complete
`UTCOffset` grammar range, while the emitted parser reads the same hour and
minute bounds. The owner-private `IntlDtfNamedZone` record and private
`INTL_DTF_NAMED_ZONES` catalogue pair every accepted constant-offset name with
one such checked offset. The compile-time case-insensitive uniqueness check
continues to reject ambiguous catalogue rows.

The owner-private `DtfCanonicalTimeZone` and the owner-private `DtfResolvedTimeZone`
preserve the existing move-only lifecycle. Reserving creates three unwritten
locals; only the option reader consumes that state and
returns a resolved state; only the resolved state can store all three record
slots and release all three locals. No other backend module can construct or
publish either half of that lifecycle.

Restoring only the former visibility of the offset and named-zone family
reproduces the exact original 150-line source with SHA-256
`4f284353c06da9e135d8ff5e863a7310b4abc94a7e1fcc57076be898e54cf641`.
Restoring only the former visibility of the two lifecycle states reproduces
the exact original 54-line source with SHA-256
`6a1a4427fae20803f35d9b3a35c62f12a1ee3224259e2a620365326b358c7513`.
At the Batch BM checkpoint, `cargo xc` is green, the focused structure target
passes `4/4`, and the exact UTC-name and fixed-offset equivalence leaves pass
all four Wasm-AOT executions with every failure bucket at zero.

This source-equivalent hardening has no new Intl behavior and does not close T23.
It changes no accepted time zone, canonical spelling, offset, emitted
instruction, record slot, Test262 materialization, or published count.
