# DateTimeFormat positional numbering

The completed frozen-checkpoint-9 Intl replay contained 24 failing executions
under `DateTimeFormat/prototype/format`, representing 12 physical files. The
`numbering-system.js` pair failed because constructor negotiation only accepted
`latn` and every numeric display producer emitted ASCII. This change addresses
that shared cause without adding locale or calendar patterns.

`Intl.DateTimeFormat` now accepts all 77 positional numbering systems in CLDR
47. The constructor's supported `nu` values are derived from the same table
that supplies digits and fractional decimal separators. A resolved numbering
system therefore has a renderer. Algorithmic systems such as `roman` remain
unsupported and use the existing locale negotiation fallback.

The numeric formatter covers year, numeric month, day, hour, minute, second,
and fractional second fields. The existing single field walk feeds `format`,
`formatToParts`, `formatRange`, and `formatRangeToParts`; range attribution and
practical equality continue to operate on the original field values. The
`2-digit` width is applied to decimal digits before localization, which also
preserves whole supplementary-plane code points. Non-contiguous digits such as
`hanidec` use the table directly rather than assuming a code-point offset.

The table constructor checks that each mapping contains ten equal-width UTF-8
scalars. The formatter selects that mapping once per shared formatting body
and translates generated ASCII fields through byte-aligned scalar slices. GMT
display labels use the same numeric translation; canonical `timeZone` slot
identifiers retain the separate ASCII formatter required by
`FormatOffsetTimeZoneIdentifier`.

The behavior follows [FormatDateTimePattern](https://tc39.es/ecma402/#sec-formatdatetimepattern),
which uses the formatter's selected numbering system for numeric fields and
fractional seconds. Pinned [CLDR numbering systems](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/common/supplemental/numberingSystems.xml)
provide the digit mappings. Decimal symbols are resolved through the `en` and
`root` symbol tables, including their explicit aliases. The generator rejects
unresolved symbols, unknown alias forms, changed checksums, and mappings that
would require a different UTF-8 layout.

The vendored XML, Unicode license, source URLs, and hashes are under
`crates/lila-aot-wasm/data/cldr-47-numbering`. They use the same CLDR 47 commit as
the Locale keyword-alias source. This is compiled AOT formatter data; it does
not change the host Locale provider identity or advertise additional provider
services. Regenerate and verify with:

```sh
python3 scripts/generate-intl-datetime-numbering.py --report crates/lila-aot-wasm/data/cldr-47-numbering/generated-report.json
python3 scripts/generate-intl-datetime-numbering.py --check --report crates/lila-aot-wasm/data/cldr-47-numbering/generated-report.json
python3 -m unittest scripts/tests/test_generate_intl_datetime_numbering.py
cargo test -p lila-engine --test aot_intl_datetime_numbering
```

The six native regressions exercise all ten digits of every positional system,
option/keyword precedence, padding, fractional separators, parts/range
agreement, extended Temporal values, and separation of localized GMT labels
from canonical identifiers. Compilation and native/canonical execution of
this staged change are pending; no published conformance count is changed.

The other 11 physical format failures retain separate causes and owners in the
Intl follow-up backlog:

| Files | Demonstrated boundary | Remaining implementation owner |
| --- | --- | --- |
| Eight Temporal formatting files | Construction rejects geographic time-zone names before formatting begins | Shared named-zone transition provider and DateTimeFormat caller |
| `temporal-objects-format-with-era.js` | `Date.prototype.toLocaleString` still dispatches to the legacy Date string formatter | Complete Date locale-method/CreateDateTimeFormat defaults family |
| `temporal-objects-no-time-clip-non-latin-numerals.js` | `ar-EG` falls back to English patterns and the default `latn` system | Arabic locale patterns and locale-default numbering data |
| `related-year-zh.js` | Chinese calendar and Chinese year-name/related-year patterns are unavailable | Chinese calendar and locale-pattern provider |

The named-zone failures do not establish whether later Temporal assertions
pass. Those assertions must be rerun after the constructor boundary is fixed.
Explicit `en-US-u-nu-arab` support does not establish Arabic locale support.
