# DateTimeFormat provider boundary

The complete source successor proposes sixteen DateTimeFormat calendars and
thirteen locale profiles through this same product boundary. Its public enum,
closed wire mapping, catalogue and native profile must be admitted together
after actual native and Wasm verification. The four-calendar observations below
retain their historical scope. See the [calendar domain](intl-datetime-calendar-domain.md)
and [genuine profile contract](intl-datetime-profile-pools.md).

DateTimeFormat conversion does not add calendar admission to Temporal
constructors. The shared Temporal domain admits `iso8601`, `gregory`,
`buddhist`, `roc` and `japanese`; it does not admit `chinese`. The complete
DateTimeFormat successor selects sixteen canonical calendars independently.
Unsupported valid keywords such as generic `islamic` still resolve to an
available profile calendar under ECMA-402. A non-ISO ZonedDateTime must match
that resolved calendar after every formatter option observation. Matching ROC
formatting joins the expanded source while generic-Islamic fallback retains
its mismatch coverage. ISO exemption, explicit-calendar precedence and late
getter abrupt completion remain covered.

The Buddhist predecessor passes its focused native/Engine regressions and
records 468/496 passes in the whole pinned DateTimeFormat cohort, with all
28 Runtime Bugs owned. Those observations belong to that earlier source. The
expanded successor needs its own native and product verification; full Temporal
or Intl completion is not inferred. See the [Buddhist contract](intl-datetime-buddhist.md).

The Wasm compiler owns all observable ECMA-402 behavior: locale list conversion,
option getters and coercion, branded receivers, calendar compatibility,
exception Realm, result objects and the cached bound formatter. The Rust host
receives only copied primitive records. It never receives a JavaScript object,
function, source program or execution handle.

ABI4 adds five closed operations: resolve date/time locale, filter supported
locales, select a formatter recipe, partition one value, and partition a range.
The operation marker fixes its request, response, error and required data
capabilities. Every message includes the schema version and an operation-derived
tag. Decoders reject truncation, invalid codes, extra bytes and impossible counts.
Capacity queries write no memory; a retry may overlap the original request
because the host copies it before writing its response.

The object stores a traced opaque recipe carrying the provider identity. There
is no host formatter registry or mutable handle. Host partitions carry typed
parts and explicit range ownership; generated Wasm either joins their values or
creates ordinary current-function-Realm arrays and objects. The provider retains
the original component selection and caller defaults so later Temporal kind
selection cannot mistake a legacy default for a requested component.

Legacy values undergo TimeClip in Wasm. Instants retain floor epoch seconds and
nanoseconds, including negative sub-millisecond values. Plain Temporal values
retain validated ISO fields and bypass legacy TimeClip. Wasm checks calendar
compatibility before sending those fields. Chinese conversion uses the
[documented canonical calendar domain](intl-calendar-domain.md), including its
retained astronomical interval and distant integer approximation.

The profile supplies English, Arabic and simplified Chinese data, Gregorian,
ISO, Chinese and Buddhist calendars, the 77 CLDR47 positional digit mappings
and the checked [CLDR48/UCD17 Tolong Siki supplement](intl-numbering-tols.md). Named
zones use the same pinned IANA transition authority as other Intl operations.
Only complete locale/calendar profile entries participate in negotiation;
unsupported valid keywords follow ECMA-402 resolution. This profile is bounded
and does not establish full Intl or Test262 conformance.

The obsolete English pattern renderer, extension-key tables and range-local
carriers in the AOT backend are removed. Their seven source-spelling/privacy
tests are retired with those implementations. The private construction lifecycle,
heap pointer inventory, time-zone domains and bound-function Realm checks remain.
Observable regressions cover conversion order, inherited option access, primitive
option boxing, exact Temporal inputs, localized fields, parts/range agreement,
calendar errors, Realm ownership and intrinsic locale-method entry points.

Focused verification commands (results are recorded in the completed-baseline
checkpoint notes, not inferred from the existence of these tests):

```sh
cargo test --release --locked -p lila-intl
cargo test --release --locked -p lila-engine --lib intl_datetime_host::tests -- --test-threads=2
cargo test --release --locked -p lila-engine --test aot_intl -- aot_intl_datetime_provider:: aot_intl_datetime_numbering:: aot_intl_named_time_zones:: aot_intl_bound_format_realm:: aot_date_locale:: --test-threads=2
cargo test --release --locked -p lila-aot-wasm --test intl_date_time_format_construction_order_structure --test intl_date_time_format_heap_slot_structure --test intl_bound_format_realm_structure
```

Source references: [CreateDateTimeFormat](https://tc39.es/ecma402/#sec-createdatetimeformat),
[FilterLocales](https://tc39.es/ecma402/#sec-filterlocales), and
[Temporal Intl integration](https://tc39.es/proposal-temporal/#sec-intl.datetimeformat).
