# DateTimeFormat complete-record pools

The proposed schema-2 DateTimeFormat profile interns complete resolved calendar
and localized zone-name records. Its first cohort retains the genuine existing
four public calendars (`gregory`, `iso8601`, `chinese`, `buddhist`), seven locale
rows, and all 78 positional alphabets. The selector remains the pinned schema-1
CLDR47 input recipe; schema 2 describes the serialized output.

`scripts/generate-intl-datetime-profile.py` resolves the same genuine CLDR leaves,
aliases, alternate policy, defaults, hour cycles, numeric symbols, finite RBNF
fields and zone geography before calling `intl_datetime_pool.pool_profile`.
The helper interns whole canonical records and assigns sorted deterministic pool
indices. Each locale retains four canonical public references. `gregory` and
`iso8601` must reference the same Gregorian pool row, while their public tags and
wire identities remain distinct. A physical record retains its explicit calendar
domain. No formatting name chooses a date calculation algorithm.

The existing materialized profile has 21 physical calendar records and seven
zone-name blocks. They reduce to nine complete calendar records and three
zone-name records. Compact UTF-8 output shrinks from 2,408,085 bytes to 1,113,824
bytes. Expanding the freshly generated pools and applying the previous serializer
reconstructs the previous 6,038,617-byte profile exactly, including provenance.
These are data comparisons, not product execution results.

The native decoder constructs each calendar record once with the existing
pattern/name checks and a closed `CalendarDataKind`. It accepts only canonical,
unique, complete public references with valid indices and matching physical
domains, verifies Gregorian/ISO reference equality and rejects unused pool rows.
An immutable checked `CalendarRecords` association owns `Arc<Calendar>` values.
Its exhaustive public-calendar lookup preserves the rendering call boundary.
Private checked geography and zone-name owners validate the genuine geography
once and each distinct localized pool row once before any locale can accept it.
Their private raw fields prevent an unchecked name block entering the live pool.
Locales share these checked immutable `Arc` owners; rendering remains read-only.
Canonical pool ordering and byte uniqueness are producer/check invariants; the
decoder validates record content and association ownership.

The identity producer includes the pool helper, the new decoder, the exact
generated profile, genuine inputs and the existing calendar dependency sources.
No host formatter, wire revision, Wasm object representation, GC lifecycle,
NumberFormat data, or calendar calculation implementation is added by pooling.

## Verification and admission

Source-only generation/check, Python controls, identity generation/check,
standalone Rust formatting, and materialized byte reconstruction are recorded in
`target/continuation-intl-datetime-profile-pools/runs`. The independent
four-calendar/seven-locale native checkpoint passes 39 controls. Its source
and acceptance receipts retain that exact scope. The expanded native
checkpoint below provides separate evidence; Wasm product verification remains
pending.

The separate native calculation scaffold actually passed eight tests with no
ignored tests, including 48 constructor/code round-trips and 80 raw extreme-date
samples. Its receipt is
`target/continuation-intl-datetime-calendar-native-foundation-codeyear-followup/runs/native-attempt2/ACTUAL-RECEIPT.json`.
The first invocation's genuine seven-pass/one-fail Chinese code-year result is
retained in the predecessor. These samples do not prove the entire date domain,
profile construction, Wasm execution, or conformance. Japanese's required 1873
projection adapter remained open at that scaffold checkpoint; the expanded
successor below supplies its checked projection and native controls.

## Genuine sixteen-calendar and thirteen-locale successor

The source successor contains thirteen locale records: `en`, `en-US`, `ar`,
`ar-EG`, `zh`, `zh-Hans`, `zh-Hans-CN`, `de`, `fr`, `it`, `ja`, `ko` and `hi`.
Each record owns sixteen canonical calendar associations. Gregorian and ISO
share their genuine physical CLDR record, giving 208 public associations and
195 physical associations. Whole-record pooling stores 135 calendar records and
nine localized zone-name records. The generated UTF-8 profile is 9,849,742 bytes.
The existing 78 positional alphabets, including the separately pinned Tolong
Siki supplement, are retained.

The six additional language XML files come from the checked CLDR47 archive.
Japanese first-year rules use the actual pinned CLDR47 `common/rbnf/ja.xml`.
An explicitly bounded CLDR48 supplement supplies only the missing era subtrees
for Coptic, Ethiopian and Islamic physical calendars. Its eleven source files
match actual Git-tree blob identities at commit
`acd6d88ae493633240e19a87a721076a8a75c310`; its manifest SHA256 is
`d551300d46e7a64558d60bb2732b702902aa68ae60ee62284d0a67b314c25932`.
All old four-calendar consumed leaves remain byte-exact. Missing names,
patterns, algorithms or provenance fail extraction.

Hebrew names retain all thirteen CLDR month indices and the separate leap
qualifier on month seven. Native month codes choose those keys rather than a
month ordinal. Japanese modern era indices 232–236 and the genuinely sourced
Gregorian BCE/CE names before 1873 have distinct typed namespaces. Coptic era
one and all calendar-specific era domains are checked against the exported
rows. Chinese and Dangi retain their related ISO year and cyclic fields.
Japanese `jpanyear` is a complete integer kernel: full era year one prints the
source `元`, and every other year uses the genuine Latin decimal fallback.
The first-year decision precedes two-digit reduction. It is not a finite table
of possible years.

Genuine Korean interval patterns retain numeric `e`/`c` weekday fields and
locale first-weekday data. The native field keeps its format/standalone context;
its constructor requires the interval's genuine textual-weekday skeleton.
Numeric weekdays do not introduce a public DateTimeFormat option. Localized
zone owners capture and validate actual offset branch prefixes once, including
the French Unicode minus, before the renderer can consume them.

The future native profile accepts only the complete exact sixteen/thirteen
recipe. One immutable `[Arc<Calendar>; 16]` association and exhaustive public,
wire and native-calendar matches prevent a sixteen-calendar catalogue from
being paired with four records. The same public calendar domain drives the
existing Wasm result reader and compiler string pool; the provider remains the
actual formatter called by Date and Temporal locale methods.

Actual source generation and profile/report reproduction both exit zero in
`target/continuation-intl-datetime-208-profile-export/runs`. The source proof
reconstructs the old materialized profile and all 16,799 old provenance leaves
exactly. The three failed producer attempts remain retained: French offset
signs, Korean numeric weekdays and Japanese year rules each required explicit
source handling. Source observations remain distinct from runtime results.
The combined calendar/range successor passes all 283 native library tests and
all three public API tests on 2026-10-01, including profile construction,
calendar rendering and public wire controls. The actual receipt is
`target/continuation-intl-datetime-range-native-checkpoint-attempt1/ROOT-ACTUAL-NATIVE.json`.
The isolated coherent successor also passes complete workspace/all-target
checking, 27 affected compiler structure tests and the retained string-seed
regression. Three Engine tests pass six strict/sloppy scripts: two test the
complete 208-association calendar domain, and one tests genuine range endpoint
patterns. The receipts are recorded in
`target/continuation-intl-calendar-complete-focused-attempt1/ROOT-ACTUAL-FOCUSED.json`
and the corrected compiler command log is bound by
`target/continuation-intl-calendar-complete-compiler-foundation-attempt4/ROOT-COMMAND-LOG-CORRECTION.json`.
MAIN admission, the complete workspace checkpoint and pinned replay remain
pending. The
independent four-calendar/seven-locale pooling checkpoint retains its original
scope; the expanded native result is measured separately.

The checked localized zone owner captures the positive and negative prefixes
from complete genuine English/French CLDR47 `hourFormat` branches. Construction
admits only `+HH:mm;-HH:mm` and `+HH:mm;−HH:mm`; rendering consumes the captured
text. Separate native controls use independently reconstructed English/French
pattern scalars and exercise zero, half-hour, hour and second offsets at both
widths. Those controls pass in the expanded native checkpoint above.
