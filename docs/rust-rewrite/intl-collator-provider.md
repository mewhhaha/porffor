# Collator service, genuine search data and Wasm consumers

This staged service compiles `Intl.Collator` and `String.prototype.localeCompare`
through the actual Wasm backend. Its host kernel consumes primitive requests
and checked pinned ICU data; JavaScript construction, coercion, branded state,
exceptions and cached callable allocation remain in emitted Wasm.

Construction reserves the actual tagged result before observing locales and
options. It then canonicalizes locales, boxes options, reads usage, matcher,
collation, numeric and caseFirst in order, and resolves the native locale/data
association before the final sensitivity and ignorePunctuation reads. The
private record is published only after those reads succeed. Malformed
collation types throw before the numeric property is read. Unsupported
well-formed collations follow actual locale-data resolution.

The native service uses separate resolve, supported-locale and compare
operations. Closed usage, sensitivity, case-first, collation and ordering
domains reject malformed primitive messages. A successful resolve result
retains its request association and actual consumed data profile; final
configuration construction accepts only that proof plus the validated final
options. Compare reconstructs the same checked tuple from immutable native
catalogues. Unknown codes, span overflow, trailing input, malformed UTF-8,
capacity errors and provider faults remain protocol/runtime failures, rather
than catchable JavaScript option errors. Raw UTF-16 operands preserve all code
units until the genuine kernel comparison.

The `compare` getter validates its receiver before accessing private state and
memoizes an anonymous, nonconstructible length-two callable. The first called
getter's Realm owns that callable; later getters reuse it. Calls ignore their
receiver, convert the left operand before the right, and return a closed sign
result with positive zero for equality. `String.prototype.localeCompare`
first checks/coerces its receiver and right operand, then invokes the immutable
Collator construction/compare path. Replacing public Intl constructors or
prototype getters cannot intercept that intrinsic dispatch.

The data is generated from the captured official ICU export release
`icu4x/2025-05-01/77.x`, whose archive SHA-256 is
`e5dae398d77a31ee7fcd86e4b35041cf9bf4643c694794629fc7c5dca76d9c6b`.
The generator uses pinned ICU4X 2.0 kernels and locale fallback data, Implicit
Han roots, Small normalization tries, all locale families and real additional
`search*` payloads. It does not substitute sort data for missing search data.
Its nine generated markers comprise seven collation markers and two NFD
normalization markers; the generated source and actual dependency locks are
hash-bound to the successful export and native admission receipts.

The exact ten generated files live in `lila-intl-collator-data`. Its private
baked module invokes the same nine data-marker macros and exposes the immutable
`PinnedCollatorProvider` to the checked native service. ICU's generated static
constructors use unsafe internally, so this data-only crate defaults to deny
and allows unsafe only for that private generated expansion. Its handwritten
code contains no unsafe operations; `lila-intl` and the other handwritten
product crates retain the workspace forbid lint. The exporter bytes, data
representation, pins and admission evidence are unchanged by this boundary.
The original coherent compile failure is retained; the relocated source still
requires the coordinated real lock refresh and actual product verification.

On 2026-09-30 the isolated generator compiled and exported ten generated
files with 144 metadata identities. Native admission then completed all 7,056
constructor configurations: a default plus 48 explicit configurations per
identity. It checked actual requested/returned marker associations, genuine
German sort/search behavior, all sensitivity mappings, numeric and case
ordering, Thai punctuation defaults/overrides, missing-profile retry rejection,
canonical Latin/Hangul equivalence and UTF-16 comparator controls. These are
native data checks, not JavaScript AvailableLocales counts or full conformance.
The preceding dependency-resolution and admission compilation failures remain
retained separately; the successful candidate uses exact reviewed datatype
pins and the real pinned API conversions.

The selected malformed-UTF-16 policy uses the locked ICU kernel's maximal
ill-formed-subsequence replacement weighting directly, preserving valid
surrogate pairs and adding no lexical tie-break. This choice follows the
recommendation in [Unicode 16 UCA, section 10.1.1](https://www.unicode.org/reports/tr10/tr10-51.html#Handling_Ill_Formed_Code_Unit_Sequences)
and the captured ECMA-402 CompareStrings contract. The finite comparator
controls check symmetry, transitivity, canonical equivalence and positive zero;
they do not prove every possible string ordering.

The separate three-calendar enumeration predecessor records 38/50 pinned
passes across 25 physical files and 16/28 passes in fourteen consumer files.
All 24 failed modes retain owners and reasons, and its 190-file smoke suite
passes all 191 executions. Its transition-boundary control passes both modes
across all 447 primary zones. The later Buddhist predecessor passes 210 native
library tests, eight compiler structure tests and 38 selected Engine tests,
including six new paired Buddhist controls. Its resumed pinned replay also
passes all five Temporal calendar-mismatch files (ten modes) and the complete
transition-boundary file (two modes). The remaining broader pinned scopes have
no completed result here. These are recorded predecessor results and do not establish a JavaScript Collator result.

The checked collation enumeration attachment consumes the real admitted sort
catalogue from the staged service. Actual complete AvailableLocales and public
collation counts remain unmeasured. The original lifecycle correction is
retained, and genuine List identity regeneration binds the complete native
Cargo/lib/protocol/provider source. The separate Buddhist/Tols data join,
candidate-payload provenance refresh and root-owned real lock regeneration
remain explicit prerequisites for coherent product admission.

The staged Engine target has 16 tests and 32 sloppy/strict executions covering
construction and abrupt order, private brands/cache, ordered argument
conversion, true search data/defaults, extensions, numeric/sensitivity/case
options, UTF-16, immutable localeCompare dispatch, called-function Realms and
tagged NewTarget prototypes. Its exact pinned cohort contains 65 Collator files
and ten String.localeCompare files, totaling 75 physical files and 150 modes,
with no exclusions. Product compilation, Engine execution and pinned replay
remain pending. T23 and literal full ECMAScript/Test262 conformance remain open.
