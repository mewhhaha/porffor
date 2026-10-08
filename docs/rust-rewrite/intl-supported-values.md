# Intl.supportedValuesOf: consumed data and remaining conformance work

The Wasm builtin implements all six keys: `calendar`, `collation`, `currency`,
`numberingSystem`, `timeZone` and `unit`. It converts its argument with `ToString`
once before dispatch. A Symbol conversion throws TypeError; an unrecognized
string throws RangeError. Argument-conversion TypeErrors, invalid-key RangeErrors
and fresh result Arrays belong to the called function's Realm. Attempting `new`
with the nonconstructible function fails in the evaluating caller's Realm,
before entering the builtin. Public constructor replacement cannot change those intrinsic
choices. Argument conversion, JavaScript exceptions and allocation execute in
the compiled user program.

The compiler accepts only a checked native catalogue from `lila-intl`.
Its private list fields are initialized once after validating the actual
embedded provider, reachable number profiles and closed calendar/unit domains.
The compiler pools the resulting primitive strings and binds the catalogue's
provider identity in the Wasm artifact. Enumeration adds no provider wire
operation, object representation or execution fallback.

This is a partial ECMA-402 capability, with truthful available data. The
coherent staged source consumes a DateTimeFormat domain with four calendars,
number rendering has 78 positional digit systems, including the checked
CLDR48/UCD17 `tols` supplement to the unchanged base47 profiles, reachable currency labels
cover 307 codes, the checked IANA catalogue has 447 primary identifiers, and
the sanctioned single-unit domain has 45 units. The four calendars are
`buddhist`, `chinese`, `gregory` and `iso8601`; Buddhist is admitted only with
its actual validated pattern and conversion profile. Aliases such as `US/Eastern`
are excluded from the time-zone result. The staged Collator service forms its
catalogue only after the native owner certifies real sort profiles. The private
list constructor requires that consumed service and catalogue together. Raw ICU
attributes, standard/search purpose tags and unused payloads never establish
availability. Complete product AvailableLocales and public collation counts
remain unmeasured until this joined source passes its runtime gates.

The isolated source successor supplies all sixteen calendar projections and
compiled DisplayNames and RelativeTimeFormat consumers. Its immutable catalogue
admission additionally requires real fallback-none DisplayNames names for the
307 reachable currency codes, and real RelativeTimeFormat resolution/formatting
for all 78 positional numbering systems. Those checks happen once before any
public list is minted. Their source contract is
[the provider integration](intl-display-relative-provider.md); complete
consumer verification remains pending. The four-calendar counts above describe
the earlier composition and its retained receipts. T23 owns the complete
calendar, Collator, numbering and new-consumer product verification, Segmenter,
DurationFormat and full ECMA-402 closure. These gaps remain observable
in the complete pinned enumeration cohort; no files or execution modes are
excluded. T22 separately owns Chinese Temporal calendar construction:
Chinese is actually supported by DateTimeFormat and must remain enumerated,
although the current Temporal calendar domain has only five calendars.
The preceding three-calendar enumeration checkpoint exposed that gap in five
pinned Temporal calendar-mismatch files, all ten modes. Chinese remains admitted
by DateTimeFormat and remains owned Temporal capability work.

The preceding Temporal attempt passed all 50 selected Engine tests and its
first 133 pinned physical files (266 modes). It then stopped on the next
transition-boundary file because `Intl.supportedValuesOf` was missing, before
entering the transition loop. That failed run and its 171 command receipts
remain retained. The subsequent enumeration checkpoint passes the complete
transition-boundary loop in both modes across all 447 listed primary zones.
This does not complete the full Temporal checkpoint.

On 2026-10-01 the preceding three-calendar source checkpoint passes all-target
checking, five native catalogue tests, nine compiler structure tests, one IR
control, six Engine tests (12 sloppy/strict scripts) and three neighboring Intl
CLI controls. The Engine scripts cover metadata, coercion and abrupt identity,
reentrant calls, all six result domains, fresh mutable Arrays and called-function
Realm ownership. The complete pinned `intl402/Intl/supportedValuesOf` cohort
records 38/50 passes across 25 physical files. Its twelve Runtime Bugs cover
missing calendars, `tols`, Collator, two DisplayNames consumers and
RelativeTimeFormat. Fourteen consumer files record 16/28 passes: the zone
consumers pass, while Chinese Temporal construction and the absent DurationFormat
service account for twelve more Runtime Bugs. All 24 real failed modes retain
concrete T22/T23 owners and reasons, with zero exclusions. The separate smoke
suite passes 191/191 executions from 190 physical files. These results precede
the Buddhist addition. Its integrated predecessor passes all 210 native library
tests, eight compiler structure tests, 38 Engine tests, the five exact Temporal
calendar-mismatch files (ten modes), and the transition-boundary file (two
modes). The completed predecessor records 518/558 real modes: DateTimeFormat
468/496, enumeration 38/50 and the six retained Temporal files 12/12. All 40
Runtime Bugs retain T22/T23 ownership, without exclusions. Its separate smoke
suite passes 191/191. Those results do not admit this future joined source.

Refresh the complete API cohort with
`./target/debug/lila test262 run intl402/Intl/supportedValuesOf --execution-backend wasm-aot --threads 2 --jobs 1`.
Exact snapshots, selected execution IDs, actual command exits and failure reasons
remain in the local `target/continuation-intl-supported-values-pinned-verification`
checkpoint artifacts. Fake-suite and full
pinned-suite truth remain separate; the generated README status block is
updated only by the status publisher.

The normative API and data requirements are
[Intl.supportedValuesOf](https://tc39.es/ecma402/#sec-intl.supportedvaluesof)
and its linked Available* operations. The implementation audit uses the
captured ECMA-402 revision `e463f3c8b62e5c67f4846cd05eec40d8d5947ed0`
and Test262 content pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`.
