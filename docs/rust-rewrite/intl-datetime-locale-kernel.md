# DateTimeFormat locale, calendar and parts boundary

DateTimeFormat sends validated primitive requests to the pure Intl provider. The
Wasm builtin owns JavaScript coercion, option observation, receiver checks,
calendar compatibility and Realm allocation. The provider resolves locale data,
selects patterns and renders one parts list; string formatting concatenates that
list. A range has independently prepared endpoint snapshots and explicit part
sources. There is no host-side JavaScript evaluator or opaque mutable host object.

## Pinned profiles

The admitted locale inventory is en, en-US, ar, ar-EG, zh, zh-Hans and zh-Hans-CN.
Unsupported requests use the specified locale lookup and default en-US selection;
supportedLocalesOf returns only requested tags that match the same inventory.
Gregorian, ISO8601 and Chinese calendars use the selected locale's inherited
patterns and names. Arabic ar defaults to latn; ar-EG defaults to arab according
to the pinned data. Calendar, numbering-system and hour-cycle Unicode extensions
and explicit options follow ResolveLocale precedence.

The CLDR input is release 47.0.0, commit
`2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`. The vendored manifest checks every input's
length and SHA-256 before parsing. The selector records supported locales,
calendars, draft threshold and alternate-pattern/name policy. LDML distinguishing/value
attribute roles come from the pinned DTD. Inheritance consumes `pattern@numbers`
with its pattern text, and rejects unresolved required leaves, cycles and
unconsumed value attributes. An absent decimal symbol never substitutes a
symbol from another numbering system.

All 77 positional numbering systems have exactly ten distinct Unicode scalars.
The renderer emits each selected digit directly, so variable UTF-8 byte lengths
are valid. Names and literal punctuation are never subjected to whole-string
digit replacement. The selected Chinese `d=hanidays` pattern override is compiled
from the pinned RBNF rules to a checked finite day table; other numeric fields
retain their own numbering selection. This is a pure generated field table,
not an RBNF interpreter in the product.

CLDR47 supplies month, weekday, era, cyclic-year and day-period names, date/time
styles, available skeletons, interval patterns, connectors and localized zone
names. The provider validates every selected field/name dependency before
publishing its immutable profile. Basic and best-fit format matchers currently
use BasicFormatMatcher scoring over the same profile. Pattern widths respect
LDML skeleton matching; resolvedOptions exposes the actual selected legacy
component widths. Styles remain distinct from component selections.

## Pattern selection policy

The profile prefers a supplied CLDR `alt="ascii"` date/time pattern at each
source locale before continuing inheritance. This applies uniformly to styles,
available formats, intervals, fallback connectors and append patterns; a missing
alternate retains the default. A child default precedes a parent alternate, and
aliases retain the selected leaf's source and value attributes. Names keep their
separate declared policy. CLDR47 currently supplies 21 such English source
leaves (17 available formats and four style patterns), and no ASCII interval
alternates in these inputs. Interval literals therefore retain their original
punctuation, including narrow no-break spaces where supplied.

This is an explicit data selection allowed by [LDML47 Overriding
Data](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35.md#overriding-data).
The previous default-pattern U+202F output was valid CLDR output. Choosing the
supplied alternate permits matching the pinned expected ASCII space without
changing digits or replacing characters after rendering. Both string and parts
formatting consume the same selected literal tokens.

Intl callers supply no context distinguishing midnight at the start of a day
from midnight at its end. The provider uses the context-free choice described by
[LDML47 Day Period Rules](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-dates.md#day-period-rules):
`b` uses AM at midnight, and `B` uses the enclosing flexible period. Exact noon
and its nanosecond boundary remain distinct. The policy applies to the closed
Midnight value in every locale, including interval comparison; it does not
remove the pinned rules or names.

Era is optional when selecting a Temporal format and does not by itself prevent
numeric date defaults. For a calendar with era names, candidate closure uses its
inherited `appendItems/Era` pattern to add a requested era to an otherwise exact
pattern. This preserves numeric year/month fields instead of choosing an
unrequested month name or day. The checked append pattern retains locale order;
a calendar with no era names has no such candidate. See [LDML47 Missing Skeleton
Fields](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-dates.md#missing-skeleton-fields).

Requested skeletons and output fields have different domains. The ECMA year
option requests numeric or two-digit width. Explicit LDML `U` skeletons request
a cyclic name and are recorded outside that request inventory, while `U` output
inside numeric-year skeletons or styles remains supported. The selected output
may contain `year`, `relatedYear`, `yearName`, or a combination. Interval matching
preserves that actual year-field set. An interval that would replace related
years with calendar years or discard cyclic names uses the declared interval
fallback and the selected pattern for both endpoints. These rules follow the
[LDML47 field domains and interval fallback](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35-dates.md#element-intervalformats);
they do not rewrite field tokens or synthesize calendar names.

## Exact calendar and time-zone inputs

The canonical patched ICU4X calendar conversion is shared by all provider
consumers. Chinese/Dangi preserve the proven astronomical interval from related
year -3653 through 4703 and use the exact arithmetic calendar model outside it.
The boundaries are fixed proved joins, not error-triggered fallbacks. The
calendar patch retains the existing golden fixtures and diagnoses invalid packed
cache records. See [the canonical calendar domain](intl-calendar-domain.md) and its tests for the
model policy and full-domain evidence.

Legacy dates reach the provider after TimeClip; Temporal.Instant supplies exact
floor seconds and normalized nanoseconds. Plain dates and times supply exact ISO
fields with the specified reference date/noon anchoring. They are not converted
through a floating-point epoch or clipped. The typed input constructor rejects
invalid dates and terminal-domain records. The shell checks calendar compatibility
before dropping the source calendar from the wire record.

Named offsets, Standard/Daylight classification and standard-time stability all
come from the same existing IANA2026a snapshot. There is no second time-zone
database. Localized zone-name selection consumes that snapshot, including the
complete 184-day stability proof required for generic-to-standard fallback.
Each endpoint resolves its own transition. Plain inputs bypass zone conversion.

## Plans and observable order

The opaque plan is `LILADTF1`, the 32-byte component identity, and the canonical
DateTimePlanRequest encoding. Each format call validates that envelope, decodes
the primitive recipe and constructs the private selected plan. No host address,
cache index or JavaScript value crosses the boundary. The identity includes the
codec source, generated profile, source manifests, generators, provider algorithms
and canonical calendar patch/package identities; the containing Intl provider
also binds the named-zone provider identity.

The recipe retains the original component selection and the caller's required
and defaults contexts. The Intl constructor is Any/Date; Date locale methods
retain their date/time distinctions. PlainDate, PlainYearMonth and PlainMonthDay
locale methods use Date/Date, PlainTime uses Time/Time, and PlainDateTime uses
Any/All. Per-kind selection happens before legacy defaults can erase whether an
option was originally present.

SelectDateTimeFormat publishes a checked DateTimeFormatAvailability bitset.
Legacy and Instant formats are mandatory, while five Plain kinds may have no
format. During range handling, the shell validates the left calendar and checks
left availability before handling the right calendar and availability. Thus an
unavailable left PlainDate format raises TypeError before a mismatched right
calendar raises RangeError. The provider independently rejects unavailable
formats and mismatched range kinds.

Range equality compares calendar fields at visible precision, rather than
formatted strings; colliding narrow month names cannot collapse distinct values.
The greatest differing field selects a validated CLDR interval. Shared calendar
fields, endpoint fields and interval literals retain explicit source ownership.
The locale interval fallback is used only when no applicable interval pattern
exists, never to suppress malformed input or provider-data errors. Reversed valid
inputs retain caller order, as required by the pinned current range algorithms.

## Refresh and verification

Regenerate the profile with:

```
python3 scripts/generate-intl-datetime-profile.py --output crates/lila-intl/src/provider/datetime/generated/profile.json
python3 scripts/generate-intl-datetime-identity.py
```

`--report <path>` optionally retains the complete consumed-leaf provenance. Run
identity generation after final formatting because it hashes source bytes.
Normal CI checks the generated files without changing them:

```
python3 scripts/generate-intl-datetime-profile.py --output crates/lila-intl/src/provider/datetime/generated/profile.json --check
python3 scripts/generate-intl-datetime-identity.py --check
python3 -m unittest discover -s scripts/tests -p test_intl_cldr_profile.py
python3 -m unittest discover -s scripts/tests -p test_generate_intl_datetime_profile.py
cargo test -p lila-intl --lib provider::datetime
```

The private provider tests cover exact localized parts, modern independent
Chinese calendar dates and leap months, extreme Plain inputs, selected widths,
numbering overrides, named transition boundaries, range source ownership,
subprecision equality, supplied ASCII alternates and unchanged interval literals,
context-free midnight and exact noon, era-only Temporal defaults, Chinese scalar
and range year families, and malformed profiles/plans. The engine native tests and
pinned Test262 replay must additionally exercise the real emitted Wasm consumers,
including coercion order and foreign-Realm allocation. Source generation or a
private-kernel test pass alone does not establish compiler or Test262 conformance.
