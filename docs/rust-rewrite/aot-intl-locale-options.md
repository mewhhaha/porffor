# Intl.Locale constructor options and provider integration

The Wasm-AOT constructor reads `language`, `script`, `region`, `variants`,
`calendar`, `collation`, `firstDayOfWeek`, `hourCycle`, `caseFirst`, `numeric`
and `numberingSystem` in specification order. Each Get, conversion and
validation completes before the next property read. Getter and conversion
exceptions propagate unchanged. Invalid option values use the called
constructor's Realm.

## Observable contract

The existing reserved/initialized result lifecycle resolves the result
prototype before observing the tag. A branded Locale input supplies its
internal tag without calling user `toString`; other accepted inputs use the
ordinary conversion path. Options are coerced before tag validation, so null
options throw even for an invalid tag, while invalid tags prevent option reads.
Undefined options act as an empty null-prototype object without consulting
Object.prototype. Defined primitives are boxed once.

The pinned provider canonicalizes the original tag before core overrides and
canonicalizes the final tag after extension overrides. Every provider result
refreshes tag, baseName, language, script and region together. Variants replace
the old variant sequence while retaining extensions and private use. They use
the structural grammar and case-insensitive duplicate detection. Unicode
options replace only their own keyword, preserving unrelated attributes,
keywords, extensions and private use. Numeric uses ToBoolean; the remaining
options use ToString. First-day numeric strings 0 through 7 map to weekdays.

Eight new getters expose calendar, collation, firstDayOfWeek, hourCycle,
caseFirst, numeric, numberingSystem and variants. They read the immutable
canonical tag through fixed branded entrypoints, so no additional object
representation or duplicate mutable option state is needed. Absent string
fields return undefined, present empty extension values return the empty
string, and numeric returns a Boolean. Main and created realms install the
same catalog identities in their own function realms.

## Provider domain and host-call contract

Locale identifiers have no 255-byte syntactic limit. A pure zero-capacity
provider call returns `RequiredCapacity(u32)`, encoded as `-2 - capacity`,
before an exact-size allocation and a second writing call. No JavaScript
operation occurs between these calls. `Written(u32)` and `Rejected` remain
closed alternatives; invalid host responses are ABI faults. The engine checks
request memory bounds before allocation and reports allocation failure.
Time-zone identifiers keep their independent 255-byte wire limit.

Artifact identity now includes `host-call-abi=2`. Hosts reject artifacts with
incompatible serialized identities before instantiation; the pinned data
schema, digest and ICU versions are unchanged.

ICU's language field accepts only two or three letters. A private parsed
reserved-language domain accepts the additional five-to-eight-letter
ECMAScript domain without treating an ICU parse rejection as invalid input.
Its alias operation consumes the same pinned alias tables. Provider setup
checks that unconstrained language rules are absent and that wildcard variant
rules preserve the language. Narrow language-key schemas cannot match a
reserved language; failed likely-subtag lookup therefore selects the first
complex territory replacement. `abcde-Armn-SU` becomes `abcde-Armn-RU`, while
`und-Armn-SU` becomes `und-Armn-AM`. The parser's temporary `und` spelling never
enters a semantic canonicalizer or likely-subtag lookup for a reserved field.
Base and transform-extension languages each receive their own alias pass.

## Verification boundary

The staged batch adds 12 direct Wasm-AOT regressions, two IR regressions and
five provider-domain tests, and updates protocol and structural census tests.
The provider tests inspect the pinned alias/likely-subtag schemas and every
complex territory entry, including distinct base and transform languages.
Existing core-options and neighboring authority tests remain required.

```sh
cargo test -p lila-intl -- --test-threads=1
cargo test -p lila-ir --test intl_locale_getters -- --test-threads=1
cargo test -p lila-aot-wasm --test intl_canonical_locale_tag_invocation_structure --test intl_namespace_plan_structure -- --test-threads=1
cargo test -p lila-engine --test aot_intl_locale_options --test aot_intl_locale_constructor -- --test-threads=1
```

Compilation, runtime regressions and exact Test262 replays are pending at
staging handoff. The evidence directory records 204 exact constructor/getter
executions and 76 shared-provider executions with pinned source hashes. These
are verification inputs, not passing counts. Published conformance counts are
unchanged. Follow the repository batch verification ladder after integration.

## Remaining provider and service work

The pinned ICU canonicalizer lacks BCP47 keyword-value alias tables. In
particular, `islamicc` and `ethiopic-amete-alem` still require canonical calendar
aliases; the exact `constructor-options-canonicalized.js` case remains open
pending real provider data integration. This batch does not guess alias
outputs or weaken that test. Locale maximize/minimize, locale information
services, complete matching and broader Intl services remain open.

References: [ECMA-402 Locale construction](https://tc39.es/ecma402/#sec-intl.locale),
[UTS35 canonicalization](https://unicode.org/reports/tr35/#Annex_C_LocaleId_Canonicalization),
[UTS35 likely subtags](https://unicode.org/reports/tr35/#Likely_Subtags), and the
[pinned ICU alias schema](https://raw.githubusercontent.com/unicode-org/icu4x/icu@2.0.0/components/locale/src/provider.rs).
