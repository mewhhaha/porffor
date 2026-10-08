# RelativeTimeFormat provider contract

This source packet implements pure RelativeTimeFormat operations over genuine
CLDR 47 fields for fourteen captured locales: ar, ar-EG, de, en, en-US, fr,
hi, it, ja, ko, pl, zh, zh-Hans and zh-Hans-CN. It remains a target-only proposal.
MAIN has not admitted this service. The isolated coherent successor connects
the shared host-operation domain, actual compiler builtins and checked public
catalogue admission. See the [compiled consumer contract](intl-display-relative-wasm.md)
and [provider integration](../intl-display-relative-provider.md). Native
controls and pinned tests have not run on this packet.

The producer resolves all eight units (year, quarter, month, week, day, hour,
minute, second) and all three styles. Its six numeric categories use the shared
zero/one/two/few/many/other order. CLDR count inheritance tries the current
locale's other category before moving to a parent, and preserves style aliases.
The immutable producer records the actual source leaf separately from the
requested category. Numeric patterns admit zero or one `{0}` placeholder:
Arabic one/two forms can be complete literals. Optional auto literals retain
the genuine offsets −2 through 2, with no invented translations.

The native constructor checks the exact source recipe and complete 14 × 24
domain, then creates immutable locale owners. Pattern parsing and duplicate
checks occur at construction. Rendering selects a typed slot, rather than
searching strings or interpreting braces repeatedly. Locale resolution selects
from the fourteen relative-time locales before calling the shared NumberFormat
resolver for numbering-system negotiation. Best fit uses the permitted prefix
policy. Unsupported requests fall back to the captured en-US profile.

Finite IEEE binary64 bits retain negative zero. Numeric always uses the past
pattern for negative zero; numeric auto selects the sourced zero literal for
either sign. The absolute magnitude crosses the existing shortest-decimal
NumberFormat boundary, with standard decimal notation, grouping auto and
default fraction precision 0–3. The shared exact cardinal selector uses the same
rounding settings. Numeric parts keep their original localized text and carry
the singular unit, including any NumberFormat bidi literal. Pattern literals
carry no unit. Combined output bytes and part counts are checked separately
from the numeric partition's limits.

These operations follow the finite-value, unit, numeric-auto and part contracts
in [ECMA-402 RelativeTimeFormat](https://402.ecma-international.org/12.0/#sec-partitionrelativetimepattern).
The authored English format/parts vectors are taken from the repository's
pinned Test262 RelativeTimeFormat tests; authored Arabic and French controls
use the actual captured CLDR leaves. These references are source evidence,
not executed conformance results.

The local closed primitive operations reserve codes 30 ResolveRelativeTimeLocale,
31 SupportedRelativeTimeLocales and 32 FormatRelativeTimeParts. Frames use the
existing version-one convention: two little-endian u64 header words, version
and `operation × 2 + direction` (request zero, response one). Resolve requests
carry canonical locales, matcher and optional numbering-system spelling;
supported requests carry locales and matcher. Parts requests carry a checked
resolved/formatted/numbering triple, style, numeric mode, finite IEEE bits and
unit. Parts responses retain kind, text and optional unit. Decoding rejects
nonfinite values, unknown codes, unrelated data locales, truncation, trailing
bytes and invalid numeric part ownership.

The original integration handoff requires exhaustive shared IntlHostOp mappings and
provider dispatch for these three operations, source-bound identity ownership,
and public exports. The compiled Wasm consumer must observe JS arguments and
options in specification order, retain branded constructor state, normalize
singular/plural units and concatenate the returned parts for format. Both
format and formatToParts call the same native parts operation. No JS coercion,
iterator, exception-observation or realm behavior is implemented by this pure
provider. The future coherent batch must compile once, run the 21 authored
native controls, execute compiler/Engine controls for those observations, and run
the complete pinned RelativeTimeFormat scope with every failure owned.


The 2026-10-01 full pinned RelativeTimeFormat run completes all160 modes
with148 passes and12 runtime failures. All six failing physical sources request
Polish output in the long, short or narrow style. The earlier13-locale relative
profile omits `pl`, so those requests resolve to English. This successor
adds genuine pinned CLDR47 Polish fields to the relative-time profile and
its checked locale domain. Polish decimal grouping and cardinal selection
already exist in the consumed shared NumberFormat/PluralRules profiles;
those primary data and algorithms stay byte exact. The original red snapshot
and owned exit remain retained. Fresh native, compiled and pinned verification
of this successor is required before MAIN admission or conformance claims.
