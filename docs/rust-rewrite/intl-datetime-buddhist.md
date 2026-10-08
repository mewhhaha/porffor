# Buddhist DateTimeFormat capability and verification boundary

The integrated Buddhist predecessor adds actual formatting to the pure
DateTimeFormat provider. Its all-target check, all 210 native library tests,
eight compiler structure tests and 38 selected Engine tests pass, including
six new paired Buddhist controls. Its exact pinned checkpoint is recorded
below. The joined successor source still requires coherent verification. Four calendars are
available after native setup validates their consumed data: `buddhist`,
`chinese`, `gregory` and `iso8601`. The remaining twelve required calendars and broader Intl services remain
T23 work; the joined successor authors the checked `tols` digit supplement.
Chinese Temporal admission remains T22 work. This addition makes no full-conformance claim.

The closed `DateTimeCalendar::Buddhist` member has wire code 4, preserving codes
1–3 and the existing request layout/operations. The real Wasm locale decoder
and pool iterate the same closed domain. Enumeration accepts only the native
checked list after actual locale selection proves no calendar fallback.
No new host operation, record, locale inventory or JavaScript execution path
is introduced. The generated provider identity changes with the actual
calendar data, codec and renderer source and remains attached to artifacts.

The selected CLDR47 input is commit
`2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`. Its checksum-bound Buddhist leaves
supply localized names, styles, component skeletons, interval patterns and
connectors through the existing inheritance generator. All prior raw source
XML and Gregorian/Chinese profile entries remain unchanged. The selector
adds Buddhist to the same seven locales; this does not advertise Thai locale
support. Gregorian and ISO share the existing Gregorian profile.

The vendored `icu_calendar 2.0.6` Buddhist calendar supplies actual solar
conversion. Its months/days are Gregorian-equivalent, 1970-01-02 projects to
2513-01-02, and its only era is `be`, selecting CLDR era index 0. It permits
year zero and negative era years. The captured ECMA-402
[FormatDateTimePattern](https://github.com/tc39/ecma402/blob/e463f3c8b62e5c67f4846cd05eec40d8d5947ed0/spec/datetimeformat.html#L1404)
maps a year at or below zero to `1 - year` before numeric/two-digit formatting.
The provider preserves the signed converted calendar year and applies this
operation only during formatting. Existing exact Instant and Plain input
limits and range policies remain unchanged; legacy Number/Date values still
undergo TimeClip in Wasm.

Five new native provider controls cover solar/era conversion, localized
patterns and nonpositive years, extension/option precedence, range endpoint
sources, exact terminal epochs and rejection of incomplete Buddhist data.
One codec control covers the new closed response member. The existing complete
pattern closure now visits every admitted calendar. Six new Engine controls
each execute sloppy and strict scripts, observing exact localized output,
negative fractional Instants, both Instant limits, called-method Realm
arrays/objects, range coercion, actual Temporal calendar matching and checked
enumeration. The existing ZonedDateTime locale fixture retains its option
ordering and unsupported-calendar rejection, now using ROC while requiring
Buddhist to succeed. These controls pass in the integrated predecessor.

Source generation/check commands:

```sh
python3 scripts/generate-intl-datetime-profile.py --output crates/lila-intl/src/provider/datetime/generated/profile.json --report target/buddhist-consumed-leaves.json
python3 scripts/generate-intl-datetime-profile.py --output crates/lila-intl/src/provider/datetime/generated/profile.json --report crates/lila-intl/src/provider/datetime/generated/report.json --check
python3 -m unittest discover -s scripts/tests -p test_generate_intl_datetime_profile.py
python3 scripts/generate-intl-datetime-identity.py
python3 scripts/generate-intl-datetime-identity.py --check
```

Refresh the focused controls with:

```sh
cargo xc
cargo test --locked -p lila-intl --lib provider::datetime -- --test-threads=2
cargo test --locked -p lila-intl --lib datetime_protocol -- --test-threads=2
cargo test --locked -p lila-intl --lib supported_values -- --test-threads=2
cargo test --locked -p lila-engine --test aot_intl_datetime_buddhist --test aot_temporal_zoned_locale --test aot_intl_datetime_provider --test aot_intl_datetime_numbering --test aot_intl_supported_values -- --test-threads=2
```

The completed 2026-10-01 Buddhist predecessor checkpoint records 518/558
real modes across 279 physical files, with all 40 Runtime Bugs retained and no
exclusions. The five exact Temporal calendar-mismatch files pass 10/10 modes,
and the complete transition-boundary loop passes 2/2 across 447 primary zones.
The whole DateTimeFormat cohort records 468/496 passes (248 physical files);
the whole enumeration cohort records 38/50 passes (25 physical files). The
separate smoke suite passes 191/191 executions from 190 physical files.
Calendar and locale coverage, Chinese Temporal construction, range formatting
and missing Intl consumers retain T22/T23 ownership. These results apply to the
Buddhist predecessor, before the joined PluralRules, ListFormat, Collator,
system-zone and Tolong Siki source. That composition requires its own product
verification; the complete T22 checkpoint and full conformance remain open.

The joined successor must replay the complete scopes with the original mode
inventory. Its independent full workspace and fake checkpoint remain required. Source generation and profile
checks do not establish native or Wasm conformance. The canonical README
status block can change only through the verified publisher, never by this
source patch.
