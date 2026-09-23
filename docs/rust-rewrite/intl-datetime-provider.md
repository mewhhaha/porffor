# DateTimeFormat provider boundary

Chinese calendar conversion in this provider does not add Chinese calendar
admission to Temporal constructors. `Temporal.PlainDate` still accepts only the
ISO, Gregorian and Buddhist calendar domain implemented by the shared Temporal
operations. A Chinese-calendar PlainDate constructor currently throws before
reaching this formatting boundary; that is a separate implementation gap.

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
ISO and Chinese calendars, and all 77 CLDR 47 positional digit mappings. Named
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
cargo test --release --locked -p lila-engine --test aot_intl_datetime_provider --test aot_intl_datetime_numbering --test aot_intl_named_time_zones --test aot_intl_bound_format_realm --test aot_date_locale -- --test-threads=2
cargo test --release --locked -p lila-aot-wasm --test intl_date_time_format_construction_order_structure --test intl_date_time_format_heap_slot_structure --test intl_bound_format_realm_structure
```

Source references: [CreateDateTimeFormat](https://tc39.es/ecma402/#sec-createdatetimeformat),
[FilterLocales](https://tc39.es/ecma402/#sec-filterlocales), and
[Temporal Intl integration](https://tc39.es/proposal-temporal/#sec-intl.datetimeformat).
