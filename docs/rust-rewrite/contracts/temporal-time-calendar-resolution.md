# Temporal time-string calendar resolution

`ParseTemporalTimeString` has two consumers with deliberately different
calendar semantics:

- `ToTemporalTime` parses the wall-clock fields and ignores the value of a
  syntactically valid calendar annotation. Thus
  `Temporal.PlainTime.from("12:34[u-ca=unknown]")` succeeds.
- `ToTemporalCalendarIdentifier` uses the same time-string grammar as one way
  to obtain a calendar identifier. It must canonicalize the first calendar
  annotation, default to `iso8601` when none is present, and throw for an
  unsupported annotation value.

Those policies must be chosen at the parser call site through one closed Rust
domain. A raw optional output local is insufficient: `None` would not say
whether omission is the specified PlainTime behavior or a forgotten calendar
consumer. The exhaustive domain is:

- `Ignore`: validate annotation syntax, but do not interpret its calendar
  value;
- `Resolve`: return a canonical calendar payload/tag or route the RangeError.

The ISO parser owns annotation discovery and the first/critical duplicate
rules. The policy is consumed only after that shared syntax validation and
before parsed locals are released. It must not duplicate a string scan in the
calendar helper, and it must not make the PlainTime path reject unknown
calendar identifiers.

This closes only the calendar-value split for time strings. It does not add a
new calendar implementation, time-zone data, custom calendar protocols, or a
general Temporal parser abstraction.
