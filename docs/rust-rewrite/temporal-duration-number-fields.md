# Canonical Temporal.Duration Number fields

Temporal.Duration retains the complete valid integer Number domain in every
field. Nanoseconds such as `1.728e22` and microseconds such as `1.728e19` remain
unchanged in constructor arguments, property bags, existing instances, getters,
`with`, sign transforms, arithmetic and the results of Temporal differences.
There is no Instant-specific storage or conversion path.

## Storage and validation

The existing 80-byte record still has ten untraced 8-byte slots. Each contains
binary64 bits for a finite integral Number; zero is canonical +0. The private
`TemporalDurationFields` bundle distinguishes those locals from integer scratch
and presence masks. It deliberately has no indexing or dereference trait;
callers name `number_bits(unit)` or `number_bits_locals()` explicitly.

`ToIntegerIfIntegral` still rejects an abrupt conversion, NaN, infinity or a
fraction immediately. Constructors convert arguments in declaration order.
Property bags convert all present properties in alphabetical order before sign
and range validation. Thus an oversized early field does not hide a later
getter's thrown value. Arguments objects use the shared Object-kind predicate,
as do Duration options.

Validation bounds years, months and weeks below 2^32, rejects mixed signs, and
requires the exact magnitude of the day/time total to stay below
`2^53 * 10^9` nanoseconds. Necessary per-field bounds precede integer projection.
For milliseconds, microseconds and nanoseconds, normalization decodes the Number
significand/exponent and shifts an exact quotient/remainder pair. It never casts
a wide field to i64 and never divides the value in floating point. At most seven
bounded contributions accumulate below 2^56 seconds, within i64; the signed
subsecond remainder remains smaller than 10^9 after normalization.

These rules follow [IsValidDuration](https://tc39.es/proposal-temporal/#sec-temporal-isvalidduration)
and [ToTemporalPartialDurationRecord](https://tc39.es/proposal-temporal/#sec-temporal-totemporalpartialdurationrecord).

## Exact arithmetic and Number projection

Temporal arithmetic continues to use signed integer seconds and nanosecond
remainders. Calendar operations explicitly project only the four bounded date
fields. Whole components parsed from strings stay in separate integer scratch
until their final Number conversion; leading zeros do not consume an arbitrary
digit budget.

Balancing subsecond largest units forms the exact two-word integer before
converting it to Number. The same conversion boundary supports fractional
subsecond `total` results: binary long division produces 53 significant bits,
a guard bit and exact sticky information, then performs one ties-to-even
rounding. The product is below 2^84 for the supported arithmetic inputs; the
bounded binary division uses no allocation or host calls. The closed
`TemporalDurationSubsecondUnit` and `TemporalDurationNumberProjection` domains
supply the scale/divisor pair; compile-time assertions pin their products and
bounds. Callers cannot pass arbitrary factors. The final floating
multiplication is only by an exact power of two. It is not epoch arithmetic.

For example, balancing 2^64 nanoseconds into microseconds stores the Number
`18446744073709552` and the independent nanosecond remainder `616`. Subsequent
operations use that stored Number precision. Balancing a value just below the
canonical total limit can round a wide Number field up to the limit and must
throw; it is not saturated. See [TemporalDurationFromInternal](https://tc39.es/proposal-temporal/#sec-temporal-temporaldurationfrominternal)
and [CreateTemporalDuration](https://tc39.es/proposal-temporal/#sec-temporal-createtemporalduration).

Number bits never become pointer fields. Record size, collector layout and
root ownership are unchanged. The shared Duration allocator and constructor
retain the intrinsic Realm selected by the active builtin and NewTarget; the
corresponding Realm slots are provided by the accompanying Instant-method batch.

## Verification

The native fixture target exercises storage and sign, coercion order and abrupt
identity, exact wide normalization, balancing and rational rounding, canonical
bounds, calendar consumers, Arguments objects and borrowed intrinsic Realms:

```sh
cargo test --locked -p lila-aot-wasm --test temporal_duration_number_fields_structure --test temporal_duration_field_transform_structure --test temporal_duration_arithmetic_operation_structure --test temporal_duration_heap_slot_structure
cargo test --locked -p lila-engine --test aot_temporal_duration_wide_fields --test aot_temporal_instant_methods --test aot_temporal_zoned_date_time_difference --test aot_temporal_plain_date_zoned -- --test-threads=1
```

The change passes a deterministic independent integer/Fraction audit of the
arithmetic algorithms. Integrated checkpoint seventeen revision one passes all
eight native wide-Duration tests and all twelve Instant method tests. Its
selected pinned Instant replay passes 227/229; the two remaining failures concern
option-read order, not wide-field arithmetic. Compiler identities and full
limitations are in the [completed-baseline notes](completed-baseline-follow-up.md).
No published full-suite conformance count is changed here.
