# DurationFormat compiled consumer

This packet proposes the real Wasm AOT consumer for Intl.DurationFormat and
Temporal.Duration.prototype.toLocaleString. It adds the constructor,
supportedLocalesOf, resolvedOptions, format and formatToParts emitters, a private
112-byte formatter record, and fourteen Engine controls in both sloppy and
strict modes. Compilation and all twenty-eight script modes remain unexecuted.
The whole joined candidate must pass Root's verification before admission.

The constructor checks NewTarget and resolves its prototype before observing
locales or options. CanonicalizeLocaleList precedes strict GetOptionsObject;
localeMatcher and numberingSystem precede native locale resolution. The returned
TwoDigitHours value is a checked Boolean associated with the resolved native
profile. Each unit style and its display option complete their Get and coercion
before validation, data-driven width promotion, and the next unit. Fractional
precision is read last. The twelve closed words are stored only after the
complete sequence. supportedLocalesOf uses the separate CoerceOptionsToObject
operation, preserving its primitive boxing semantics.

Formatting checks the formatter brand before observing its argument. A private
ten-state sequence owns the alphabetical property reads: days, hours,
microseconds, milliseconds, minutes, months, nanoseconds, seconds, weeks, years.
Each present field completes ToNumber and integral finite validation before the
next Get. The completed owner carries ten original IEEE Number fields, including
negative zero and integers wider than i64. Missing all ten fields throws
TypeError. Genuine Temporal.Duration instances load their stored fields without
ordinary property getters. The pinned Temporal extension also accepts ISO
duration strings through the existing emitted Wasm grammar parser. No JavaScript
source parser, interpreter or JavaScript runtime is embedded in the artifact.

The checked native service remains responsible for uniform sign and exact
aggregate bounds. Its semantic rejection becomes a called-function-Realm
RangeError; malformed frames and resource failures remain host failures. The
request borrows both the private formatter record proof and the completed input
proof through its associated response. format concatenates the actual returned
text. formatToParts creates fresh called-function-Realm arrays and objects with
type, value and optional unit in that order, with ordinary data-property
attributes. The response accepts only the seven actual NumberFormat part kinds,
ten unit labels, and valid presence words; only literal separators may lack a
unit. A valid all-auto zero configuration can return an empty partition.

Temporal.Duration.prototype.toLocaleString checks its own brand before locales
or options. It loads the same stored Number field owner, uses the shared private
DurationFormat initializer, and borrows the same native partition path. It does
not read the public Intl.DurationFormat property or the duration's own field
properties. Replacing the public constructor therefore cannot intercept it.
toString and toJSON retain the ISO serialization path. The shared StandardBuiltin
dispatch, side-effect catalogue and dependency planning changes belong to Root.

The formatter owns two traceable string references and twelve scalar words;
the layout marks only the locale and numbering-system fields as pointers.
Temporary owners remain below their scratch locals and release in exact reverse
order. The initializer returns only its completed record; the renderer borrows
that record and input without releasing either caller-owned proof.

The finite native proposal is the fifteen captured locales ar, ar-EG, de, en,
en-US, es, fr, hi, it, ja, ko, sr, zh, zh-Hans and zh-Hans-CN. It consumes actual
CLDR47 digital patterns and the checked existing NumberFormat and unit
ListFormat owners, including all seventy-eight admitted positional alphabets.
This packet adds no generated data and makes no global locale coverage claim.
Original native44 and wire7 source packets, the bounded TwoDigitHours/native
host successors, and all source/runtime failures remain separately retained.

The fourteen controls cover constructor observation and strict options,
effective option order and abrupt transitions, immediate field coercions,
integrality failures, complete aggregate validation, stored Temporal fields and
ISO strings, native Number/List parts, sign and exact fractions, finite locale
selection and Serbian separators, fresh results and descriptors, cross-Realm
and NewTarget behavior, zero extents and numeric bridges, and both Temporal
locale-string integration and its brand/options order. Assertions derive from
the pinned bodies, captured primary data, exact mathematical bounds, and real
shared service comparisons. No tests have been skipped or declared green.

The normative sources are [ECMA-402 2025 DurationFormat](https://tc39.es/ecma402/2025/#sec-intl.durationformat),
[GetDurationUnitOptions](https://tc39.es/ecma402/2025/#sec-getdurationunitoptions),
and the [Temporal locale-sensitive Duration method](https://tc39.es/proposal-temporal/#sup-temporal.duration.prototype.tolocalestring).
The pinned format and formatToParts temporal-duration-string-arg.js bodies supply
the concrete string-extension vectors; the pinned invalid-arguments bodies
require RangeError for malformed strings.
