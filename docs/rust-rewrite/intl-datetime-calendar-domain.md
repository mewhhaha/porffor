# DateTimeFormat calendar and locale domain

The proposed coherent208-profile batch admits sixteen canonical DateTimeFormat
calendar names only after its genuine CLDR data, checked native conversion and
parts renderer are joined. The names are buddhist, chinese, coptic, dangi,
ethioaa, ethiopic, gregory, hebrew, indian, islamic-civil, islamic-tbla,
islamic-umalqura, iso8601, japanese, persian and roc. The thirteen locale records
are en, en-US, ar, ar-EG, zh, zh-Hans, zh-Hans-CN, de, fr, it, ja, ko and hi.
Generic islamic and physical CLDR aliases such as gregorian are not extra public
calendar values.

Options reuse the sole pinned CLDR47 complete-value keyword alias authority:
`islamicc` canonicalizes to `islamic-civil` and `ethiopic-amete-alem` to `ethioaa`.
Aliases remain absent from the public catalogue. A compound keyword is matched
as its complete value; an alias prefix is never rewritten on its own.

The exhaustive wire encoder keeps existing calendar codes1–4 and appends
codes5–16. Gregorian and ISO retain distinct public tags while sharing genuine
Gregorian data. The same public enum drives the AOT response reader, compiler
string pool and checked supported-values catalogue; these consumers have no
separate calendar list. The catalogue selects every calendar through the actual
provider and rejects any fallback before publishing its immutable sorted list.

The independent four-calendar/seven-locale pooling package and its 39 native
passes remain scoped to that source. The complete sixteen-calendar successor,
including genuine range endpoint companions, passes all 283 native library
tests and all three public API tests on 2026-10-01. The public tests select and
format every one of the 208 associations, pin independent calendar dates and
genuine names, round-trip the wire data, and check complete-value aliases.
The initial default-locale and invalid PlainDate reference-time failures are
retained; the constructor still requires noon.

The actual combined receipt is
`target/continuation-intl-datetime-range-native-checkpoint-attempt1/ROOT-ACTUAL-NATIVE.json`.
Focused Wasm AOT controls and pinned-suite replay remain required before
product admission. Native results do not establish JavaScript conformance.
