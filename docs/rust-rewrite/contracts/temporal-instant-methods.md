# Temporal.Instant arithmetic and differences

The authoritative built-in catalog adds add, subtract, round, until and since
with one required argument, ordinary method attributes and synchronous user
code effects. Shapes, result kinds, exhaustive dispatch, dependency planning,
entry bootstrap and created-Realm bootstrap consume those identities.

Receiver validation precedes argument coercion. Add/subtract complete canonical
ToTemporalDuration conversion and validation before rejecting nonzero date
units. Time is normalized through the shared Duration seconds/subseconds
algorithm; no complete epoch nanosecond value is narrowed to i64 or f64.

The exact BigInt splitter owns both Instant and ZonedDateTime record offsets
through the exhaustive TemporalEpochNanosecondsRecord domain. Arithmetic
carries signed remainders before making a floor pair. BigInt reconstruction
then passes through the existing private, non-Copy EpochNanoseconds proof;
new method children cannot bypass validation or manufacture that proof.

Instant round follows RoundNumberToIncrementAsIfPositive, including negative
epochs. A floor day plus a within-day nanosecond value bounds all intermediate
products. Its positive rounding quantum must divide a full day inclusively;
GetRoundingIncrementOption still caps the input at one billion. Half-even uses
both the day and within-day quotient parity. The separate difference path
uses the existing next-unit exclusive increment rule and signed rounding.

Until/since convert the other Instant with the canonical intrinsic from body.
After branded copies, that body calls the shared ToPrimitive boundary with a
string hint for every object kind, including Arguments, and retains the actual
primitive tag. Non-string primitives throw TypeError before ISO parsing;
primitive-hook abrupt completions retain their identity and precede option
reads. The conversion token fixes the executing function's Realm for errors
and is consumed by a tagged projection without primitive ToString.

Differences then read and coerce largestUnit, roundingIncrement, roundingMode
and smallestUnit once in that order. Recognized date-category unit values are
rejected only after those four reads complete; a later getter or conversion
throw takes precedence. Unknown unit spellings, invalid rounding-mode names
and out-of-range increments still fail during their own independent option
validation. The reader retains the converted values across later callbacks.
Default largestUnit is second and smallestUnit is nanosecond. Since negates
the rounding mode before computing other-minus-this and negates the rounded
result afterward. Canonical Duration balance/create owns Number precision.

The two needed Temporal prototype slots are written during both Realm
bootstraps. New method results use the executing intrinsic function's Realm;
receiver prototypes and mutable public constructors are not prototype
sources. Entry and created Realm members share one property authority, and
created callable materialization couples the defining Realm with callable
Function.prototype. Retained namespace locals publish in reverse allocation
order. Other created-Realm Temporal families remain separately unimplemented.

Required verification includes `aot_temporal_instant_methods`, the IR
`temporal_instant_methods` target, the epoch proof and new method/Realm source
contracts, existing ZonedDateTime difference regressions, and the exact
227-failure plus two-control canonical replay. Wide Number-valued Duration
fields are a canonical prerequisite: the implementation must not narrow valid
nanosecond/microsecond durations or use an Instant-specific representation.

Primary algorithm authority: [Temporal.Instant methods and abstract operations](https://tc39.es/proposal-temporal/#sec-temporal-instant-objects).
