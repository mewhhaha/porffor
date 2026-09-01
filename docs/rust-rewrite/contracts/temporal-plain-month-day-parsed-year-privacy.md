# Temporal.PlainMonthDay parsed-year privacy

Status: implemented as a source-equivalent T22 invariant closure.

The owner-private `TemporalParsedMonthDayYear` binds the parsed ISO year local
to the local that records whether the source actually contained a year. The
private raw month-day parser is its only producer. The private reference-year
step consumes it by value, performs the non-ISO year-presence and date-limit
checks, and then stores the fixed ISO reference year.

The sole string caller keeps the existing observable order: parse first, read
the overflow option only after a successful parse, then consume the parsed-year
state through the reference-year step. No sibling backend module can mint the
state or invoke the raw parser.

Restoring only the former carrier visibility reproduces the exact original
five-line source with SHA-256
`edd8d04d5cf6ec69edd44225d78506a09d49e857a028ad52071a39d78417a4be`.
Restoring only the former raw-parser visibility reproduces the exact original
46-line source with SHA-256
`a6f4eeae8728f7f922afac564ea96b845164c0115682f0821fabdb76d0cac6ff`.
At the Batch BO checkpoint, `cargo xc` is green, the focused structure target
passes `3/3`, and the exact valid/invalid string plus reference-year leaf passes
both Wasm-AOT executions with every failure bucket at zero.

This source-equivalent hardening has no new Temporal behavior and does not close T22.
It changes no accepted string, option observation, reference year, emitted
instruction, Test262 materialization, or published count.
