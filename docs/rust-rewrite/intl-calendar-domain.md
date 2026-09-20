# Canonical Chinese/Dangi calendar domain

The pinned ICU calendar 2.0.6 previously extrapolated floating astronomical
calculations across the whole accepted ISO date range. At Temporal's extreme
dates, that produced invalid raw lunar months before the compact year record
was constructed. Debug conversion panicked while packing New Year; release
conversion could expose days 43, 44 or 233 in a 29-day month. Widening the packed
offset alone could not repair those values.

The canonical vendor path now uses one explicit model policy for both Chinese
and Dangi:

| Related ISO years | Calendar model |
| --- | --- |
| Before -3653 | Integer mean-lunar/Gregorian-solar-term approximation |
| -3653 through 4703 | Retained pinned astronomical calculation and bundled cache |
| From 4704 | The same integer approximation |

These are exact New Year joins, shared by both calendars:

| Join year | ISO New Year | Rata Die |
| --- | --- | --- |
| -3653 | -3653-02-01 | -1334565 |
| 4704 | 4704-02-11 | 1717776 |

The model is selected by the related year, before calculation. A failed
invariant does not select another model. The original compiled cache and its
date interpretations remain in place. The retained interval includes every
existing fixed lunar-field and valid-code expectation, whose related years
span -3650 through 4660. Wider existing reversibility and malformed-code
assertions remain active. The new policy does not claim a published-almanac
correction.

## Authority and representation

The distant-date model is adapted from ICU4X 2.1 revision
`38a49da495248dd1ded84cf306e4ca42e64d5bb3`,
`components/calendar/src/cal/east_asian_traditional/simple.rs`, SHA-256
`b362f75297198eaf71b6f84b39991e39705637198ae941ded8bf9a7563ddb867`.
It defines a documented proleptic approximation using mean lunar periods and
Gregorian-anchored solar terms. This is the model approved in the upstream
[proleptic-calendar discussion](https://github.com/unicode-org/icu4x/issues/5778).
All existing dependency versions remain pinned. The Unicode license and
adaptation provenance remain attached to the vendored source.

Calendar observation offsets still come from the canonical Chinese/Dangi
calendar trait. User-selected IANA time zones are a separate input to date/time
formatting; this repair introduces no alternate zone database.

`ChineseBasedYearInfo` has private fields and can be constructed only through
checked cached or computed year constructors. Astronomical month intervals are
checked before conversion into 29/30-day flags, and their exact sum must end at
the next New Year. Computed and cached records must contain July 1 of their
related ISO year. Custom provider caches must also join at every adjacent
record and both calculated endpoints before calendar construction succeeds.

The three-byte serialized layout is unchanged. New Year offsets through
February 22 fit its existing six-bit field and are now validated explicitly.
Leap ordinal 1, ordinals above 13, an unused thirteenth-month length, and invalid
New Year offsets are rejected. Human-readable and byte deserialization use
the same validation. No raw bad record is normalized into a valid date.

RD conversion selects a validated canonical year, then requires the date to
lie inside it. The former invalid-day substitutions are removed. Exact
Euclidean integer arithmetic preserves negative dates and exact-midnight
boundaries. Month codes continue to distinguish the ordinary month number
from a leap month's ordinal.

## Verification boundary

Before this canonical integration, root executed the isolated integer model
in debug and release: four complete scans each checked all 547584 related ISO
years from -271822 through 275761, with zero failures. Both profiles also passed
eight extreme ISO vectors per calendar and both bounded searches for common
New Year joins. There were 16/16 binary runs and four passing arithmetic unit
tests. Those results establish the arithmetic model and candidate joins;
they are not an execution result for this canonical patch.

The retained baseline was then checked in both profiles: all 48 unchanged
assertion groups passed, and all four raw interval scans validated exactly
8359 years from -3654 through 4704, including both join-adjacent years, with
zero failures. The scans checked exact intervals before packing and every
consecutive end/start. Separate searches proved the two shared start days.
These are baseline/model proofs; candidate API verification remains required.

The canonical regression target is:

```sh
cargo test --locked -p lila-intl --test calendar_domain -- --test-threads=1
```

It checks public Chinese/Dangi APIs across the complete year domain, every year
of the retained calculation interval, cached/calculated record agreement,
day-by-day joins and cache edges, ISO/code round trips, adjacent-day arithmetic,
modern published Chinese goldens, and malformed custom-provider rejection for
both calendars. It also runs the 24 retained assertion groups from the vendor
calendar tests, shared continuity tests, AnyCalendar construction cases, and
documentation examples. Private accesses are expressed through public calendar
constructors and getters; original expected values and iteration counts are
unchanged. The target contains 32 tests.
The original isolated 17-case ISO/cached/calculated boundary spike must also be
rerun in both debug and release: 102 executions through the actual patched
calendar API. Root owns those executions and records their outcomes separately.

This is a canonical dependency foundation. Arabic locale patterns, Chinese
Intl rendering, typed date/time format plans, Temporal coercion/Realm behavior,
and the observable AOT codec migration remain separate work. No locale or
calendar is newly advertised by this patch. Existing general ICU calendar
duration behavior and an exhaustive domain beyond Temporal's accepted date
range are not claimed by these checks.
