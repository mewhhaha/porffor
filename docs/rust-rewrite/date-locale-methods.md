# Date locale methods

`Date.prototype.toLocaleString`, `toLocaleDateString` and `toLocaleTimeString`
use the same DateTimeFormat initialization and field renderer as the Intl
constructor. They previously called the plain Date string formatters and ignored
locale and option arguments. The shared initialization has a closed purpose
domain: constructor, Date date, Date time or Date date-and-time. Each purpose
selects its required components, default numeric fields and rejected styles.

The method validates the Date receiver and saves its numeric value before any
locale or option observation. An invalid Date returns `Invalid Date` immediately.
Callbacks may mutate the Date or throw; they cannot change the saved value or
replace the intrinsic constructor/formatter used by this operation. The options
object is read once in the shared specification order and is never mutated.
Definition errors and primitive options wrappers use the executing method's Realm.
Locale-list canonicalization uses the intrinsic operation shared with
`Intl.getCanonicalLocales`: it handles `Intl.Locale` identifiers, rejects null,
skips absent indices and converts each present entry before reading the next.

The formatter keeps existing locale, calendar and time-zone support boundaries.
This change does not supply additional locale patterns or calendars.

Twelve native Wasm-AOT regressions cover independent Date-only dependency roots,
all three default field sets, explicit numbering and components, required-field
selection, observable read order, null and sparse locale lists, `Intl.Locale`
identifiers, invalid receiver and Date behavior, reentrant Date mutation,
replaced public Intl properties, foreign-Realm errors, primitive options
wrappers and thrown-value identity. The construction lifecycle guard still pins
reserve-before-observation, one initialization and one publication. Checkpoint
thirteen passes all twelve native regressions and all 24 pinned Date locale
executions on 2026-09-18, repairing four failures from checkpoint twelve and
retaining twenty successes, with zero crashes or timeouts. The exact compiler
and source identities are retained in the
[completed-baseline evidence](completed-baseline-follow-up.md). These bounded
results do not refresh the full-suite status.

The algorithms are specified by [ECMA-402 Date locale methods](https://tc39.es/ecma402/#sup-date.prototype.tolocalestring)
and [CreateDateTimeFormat](https://tc39.es/ecma402/#sec-createdatetimeformat).
