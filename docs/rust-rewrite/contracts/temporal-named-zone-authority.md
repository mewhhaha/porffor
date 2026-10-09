# Temporal named-zone authority

The Wasm Temporal path must preserve the input zone's meaning. It cannot trust a
supplied numeric offset for an arbitrary slash-containing annotation, replace
the named zone with that offset, or accept an unavailable identifier.

## Replacement authority and preparation limits

The T22 replacement retires `builtins/temporal_zone_authority.rs` and its blanket
available-name semantic-gap producer. Its complete pure identifier grammar moves
to the private `temporal_zone_provider/identity.rs` owner; only the visibility
changes to preserve access from other builtins. The validated identity/data/policy
factories recover Identifier and PrimaryIdentifier, retain an exact epoch and
actual calendar, and attach real projection/inverse/day/transition/arithmetic
consumers. Unknown identifiers remain intrinsic `RangeError` completions.
Ordinary available named construction must return actual named semantics.

The existing `aot_temporal_zone_authority` target retains seven tests and thirty
planned sloppy/strict Wasm observations. Fixed/UTC and unavailable-name controls
remain. The other tests now require exact epoch/offset/wall/Identifier/calendar
results for matched, critical, Z, missing-time and missing-offset annotations;
all four mismatched-offset policies; constructor/property-bag names; a nonslash
alias with PrimaryIdentifier equality; and relative Duration day totals of 23/25
hours across actual Paris transitions. No blanket expected-gap assertion remains.
The separately owned contextual rounding-window gap does not replace ordinary
named construction with a rejection.

Current source authority is [tc39/ecma262 Stage4 PR3966](https://github.com/tc39/ecma262/pull/3966) head
`3d4a6e7124a6878cb5af3132af7e01e01a88317f`, including
`temporal/zoneddatetime.emu`, `timezone.emu` and `formatting.emu`. Current
GetStartOfDay interval-checks its gap result; both day endpoints are checked
before rounding selection. Day rounding clamps progress to end−1ns when a
post-midnight backward change revisits a prior date. Serialization uses
RoundEpochNanoseconds before the new offset/local projection. The former
rendered proposal is not the authority for this replacement.

These source changes are staged only. Rustfmt and isolated patch/source checks
are preparation evidence. The stable Node26.10.0 reference passes all thirty
rewritten authority observations; those observations are not Lila execution or
Test262 conformance. The complete compiler graph, focused actual Wasm controls,
pinned cohort, broad verification and generated identity refresh remain required
before named admission or published status changes. No full T22 acceptance is
claimed.

## Historical boundary checkpoint

The following record describes the earlier blanket-gap checkpoint and its then
executed checks. Its available-name rejection, old wire census, fixture gap
expectations and rendered-proposal links are historical receipts. They are not
current ownership or coverage claims for the replacement. Root's task/status
update records actual new-batch results after verification.

### Historical runtime boundary

The shared `lila-intl` provider already contains the pinned IANA 2026a Zone/Link
catalogue, normalized Identifier/PrimaryIdentifier pairs, and TZif transitions
for offset-at-instant lookup. Temporal inverse civil-time resolution, gap/overlap
selection, and the named-zone consumer wiring remain compiler work owned by
T22. T23 owns the provider/data boundary. This is separate from the required
experimental Wasmtime capabilities.

Raw identifier strings first take the complete IANA identifier grammar;
remaining strings retain the existing ISO parse goals. Case-insensitive UTC and
numeric fixed zones retain their ordinary Temporal semantics. All other named
identifiers reach `IntlHostOp::LookupNamedTimeZone` before construction:

- An unavailable name throws the current function realm's intrinsic `RangeError`.
- An available name reports `RuntimeSemanticGap::TemporalNamedTimeZone` outside
  JavaScript completion. No partly functioning named-zone object is returned.

Parse goals that ignore a time-zone annotation, including PlainDate and Instant
string conversions, retain that behavior. They do not manufacture a named-zone
object and therefore do not take this semantic-gap boundary.

`RuntimeSemanticRejection::{DynamicSource, Gap}` owns the mandatory
`lila_host.reject_runtime_semantics(i64) -> ()` import's sole wire domain. Dynamic
operations retain codes 0–5; the Temporal gap is code 6. The import's unary ABI
and function index remain unchanged. Decode rejects every unassigned code.
Wasmtime returns the typed reason before generic trap formatting. Engine
classification distinguishes `DynamicSource` from `SemanticGap`, preserves each
projection through worker aggregates, and keeps unrelated real failures.
Test262 classifies a standalone gap as Unsupported/NotImplemented. Catching an
exception or expecting runtime `RangeError` cannot make that gap pass.

### Historical verification evidence

The `aot_temporal_zone_authority` engine target defines seven tests with 30 fresh
sloppy/strict Wasm observations. Fixed/UTC and unavailable-name fixtures require
exact normal completion. Available-name controls cover matching and mismatched
offsets with all four offset policies, critical annotation, `Z`, omitted time or
offset, raw constructor/property-bag names, Duration `relativeTo`, and a nonslash
IANA Link. Two Engine unit proofs pin gap projection and aggregate precedence;
one Test262 runner test adds four catch/runtime-negative controls.

On 2026-09-30 all-target checking and the full IR suite pass. The Engine target
passes all seven tests and 30 Wasm-AOT observations; the execution-failure target
passes all nine tests, and the runner test passes all four catch/runtime-negative
controls. Node's Intl provider independently confirmed the Paris epoch/wall-time
reference; Node exposes no Temporal here. Pinned replay and broad verification
remain pending. No Test262 aggregate or T22 completion is claimed.

```sh
cargo test -p lila-ir --lib runtime_semantics::tests
cargo test -p lila-ir --lib builtins::catalog::tests::intl_provider_callers_declare_the_host_import
cargo test -p lila-aot-wasm --test runtime_link -- host_import_function_indices_structure::
cargo test -p lila-engine --lib execution_failure::tests
cargo test -p lila-engine --test aot_temporal -- aot_temporal_zone_authority:: --test-threads=1
cargo test -p lila-test262 --lib temporal_named_zone_semantic_gap_cannot_pass_runtime_negative_or_catch
```

### Historical remaining acceptance and primary controls

Full T22 acceptance requires inverse possible-instant lookup, gap/overlap
selection, start-of-day semantics, all named-zone consumers, and preserved named
identity. Replace the gap expectations with those real semantics once the
complete consumer path is implemented. In particular, reject must throw for a
mismatched offset, use must select the supplied offset, prefer must select a
matching zone candidate or fall back to zone resolution, and ignore must use
zone resolution. Exact `Z` still preserves the named identity.

The official algorithms are
[ToTemporalZonedDateTime](https://tc39.es/proposal-temporal/#sec-temporal-totemporalzoneddatetime),
[ToTemporalTimeZoneIdentifier](https://tc39.es/proposal-temporal/#sec-temporal-totemporaltimezoneidentifier),
and [InterpretISODateTimeOffset](https://tc39.es/proposal-temporal/#sec-temporal-interpretisodatetimeoffset).

The pinned cohort below contains six physical files with no flags: 12 independent
sloppy/strict executions. The five `intl402` files require named-zone consumers;
their typed gaps remain failures. The last file uses the supported UTC zone and
checks offset policies with a critical annotation. Its ordinary conformance
replay is separate from the named-zone gap evidence:

- `intl402/Temporal/ZonedDateTime/from/argument-string-dst-option-offset.js`
- `intl402/Temporal/ZonedDateTime/from/zoneddatetime-sub-minute-offset.js`
- `intl402/Temporal/ZonedDateTime/from/do-not-canonicalize-iana-identifiers.js`
- `intl402/Temporal/ZonedDateTime/from/etc-timezone.js`
- `intl402/Temporal/ZonedDateTime/from/argument-string-with-Z-and-timezone.js`
- `built-ins/Temporal/ZonedDateTime/from/offset-overrides-critical-flag.js`

The last pinned file accidentally checks `useResult` again in its prefer
assertion; it proves the use/ignore critical-flag controls, while the first file
provides the independent prefer behavior. The authored unavailable slash-name
fixture covers the former explicit-offset bypass directly. No provider or
compiler implementation is used as its own semantic oracle.
