# Native calendar projection and source names

This proposal joins one private sixteen-calendar calculation domain to genuine
schema-2 DateTimeFormat profile names. Its source gate is separate from native
and product execution. The proposal requires the separately owned closed sixteen-calendar public
request, wire, catalogue, and caller changes. MAIN remains at the currently
admitted four until the complete coherent expansion is verified and integrated.

`CalendarId` is the only canonical native identity. It chooses a real ICU calendar,
not a Gregorian calculation labeled with a different identifier. Physical CLDR
records may be shared; calculation identity is preserved through `Fields`.
`CalendarYear` preserves signed era years or cyclic year plus related ISO year.
`Month` separately retains ordinal, standard code, and formatting code.

The checked projection preserves the ICU constructor distinction for Chinese and
Dangi: their `from_codes` year is the related ISO year, while other calendars use
extended years. Their cyclical names and leap-month placeholders remain separate
from Hebrew leap-month names and from the genuine thirteenth Coptic/Ethiopian
months. Hebrew M05L maps to CLDR month6, ordinary M06 to month7, and formatting
M06L to month7 with the explicit leap-year discriminator. Later Hebrew month
codes map to CLDR8 through13; calendar ordinals remain available for arithmetic.
The Unicode LDML month-name contract documents the ordinary-year skipped month6:
https://unicode.org/reports/tr35/tr35-dates.html#Months_and_Days

Era codes are translated once into closed native labels, then into the exact
source record key. Islamic AH is CLDR48 index0 and BH is index1. Japanese modern
era indices232 through236 remain in the Japanese namespace. Before1873 the
adapter selects Gregorian eras and years, using separately sourced Gregorian
name keys; it does not expose ICU's1868 Meiji boundary as the product contract.

All profile associations are validated against their physical calendar domain.
Required era, month13, Hebrew leap-name, cyclic-name, and leap-template records
must exist before a profile can be used. Numeric e/c weekdays require a sourced
first day of week and retain their numbering-override context. Japanese year
numbering uses the exact pinned RBNF first-year label and decimal fallback; it
checks the full year before two-digit reduction so year101 does not become 元.

The public request path and native expanded-profile acceptance call the same
field renderer. The public-to-native bridge is exhaustive across all sixteen
canonical identifiers. Its source seal does not claim public product admission. Eight rendering/schema
controls and the predecessor's ten projection controls are authored and remain
UNEXECUTED in this source-only package. They require the joined genuine208 data
and native profile schema. No Cargo or product command is executed by the source
producer, and no source check is reported as a native or runtime pass.

The predecessor retains actual native foundation8/8 and80 sampled raw boundary
facts. Samples do not prove every TimeClip or Temporal bound for every calendar.
Input-kind bounds remain owned by the checked request constructors; this adapter
validates ISO days and clocks rather than inventing a universal calendar range.
