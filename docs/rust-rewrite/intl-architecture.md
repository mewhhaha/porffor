# Intl architecture: deterministic ECMA-402 on the Wasm-AOT path

## Status and scope

This document is the source of truth for T23's architecture and data choices.
It describes the target design; the current implementation has not yet reached
it. In particular, naming a crate or a data version here does not make an Intl
service complete. Completion still requires the full pinned `intl402` tree to
pass through Wasm-AOT.

The product boundary is strict:

- JavaScript is parsed and lowered by Lila, and all JavaScript-observable Intl
  semantics execute from the emitted program and its Rust runtime support.
- ICU4X is a deterministic data and pure-algorithm kernel. It is not allowed to
  read JavaScript objects, choose property-access order, construct JavaScript
  results, or source data from the operating system.
- No product path delegates Intl to Boa/spec-exec, the process locale, libc,
  the OS time-zone database, or an embedder's native formatter.
- An Intl runtime capability is not an interpreter or VM: it accepts only
  validated primitive records and cannot receive JavaScript source or objects.

## Current evidence and gaps

The product backend presently has two substantial but deliberately narrow
pieces:

- `crates/lila-aot-wasm/src/builtins/intl.rs` implements structural locale-tag
  validation/casing, `Intl.getCanonicalLocales`, and part of `Intl.Locale`.
  `getCanonicalLocales` now applies the pinned provider's CLDR alias data;
  `Intl.Locale` does not yet share that result and still ignores its options.
- `crates/lila-aot-wasm/src/builtins/intl_datetimeformat.rs` implements much of
  DateTimeFormat's observable option ordering and its formatter/parts/range
  shapes, but its data surface is `en-US`, `gregory`/`iso8601`, `latn`, and
  fixed-offset zones. Handwritten patterns and the fixed-zone catalogue are a
  bootstrap implementation, not the final data layer.

The shared seam now exists in `crates/lila-intl`. A data
identity fixes schema, profile, typed canonical default locale, placement,
digest and upstream version line. The sealed operation catalogue currently has
two deliberately small entries—locale canonicalization and time-zone
canonicalization—each with a distinct validated input, canonical output,
failure type and required capability. An identity-matched `IntlKernel` grants
only an operation-specific handle after checking the selected profile. This is
the real API that consumes the identifier/profile/operation types; they are not
an unattached vocabulary.

One operation is now connected end to end. `lila-intl` directly pins
`icu_locale = 2.0.0` and `icu_locale_data = 2.0.0` and builds a deterministic
Locale-only `LocaleCanonicalizer::new_extended()` provider. The engine shares
that kernel across Wasm stores, and `Intl.getCanonicalLocales` calls it after
observable input processing and structural validation but before list
deduplication. The concrete `lila_host.intl_call` ABI is
`(op: i64, request_span: i64, result_span: i64) -> i64`: spans are distinct
typed offset/length and offset/capacity words, while the result is the closed
domain `Written(u32) | Rejected`. Unknown operation wires and every other
negative result are faults rather than catch-all cases.

This is not two-operation support. `CanonicalizeTimeZone` stays in the closed
catalogue but is explicitly unbound, and `Intl.Locale` is not connected because
only replacing its tag would disagree with its separately stored language,
script, region and base-name slots. There is also no generated artifact data
image or artifact-embedded ICU payload yet. The current provider is compiled
into the Rust host and truthfully declares `External` placement relative to
emitted Wasm; its digest identifies the exact `icu_locale_data-2.0.0.crate`
archive, not data embedded in the artifact.

The provider identity is now carried and enforced independently of that future
data image. A module that imports `lila_host.intl_call` carries exactly one
`lila.intl-data-identity.v1` custom section containing `lila-intl`'s canonical
serialization of the complete `IntlDataIdentity`. A module without the import
carries no such section. Before Wasmtime compilation or instantiation, the
engine checks that import/section relation and compares the section bytes to
the identity of its shared `EmbeddedLocaleProvider`; missing, duplicate,
unexpected, and mismatched identities reject the artifact. Equality needs no
second decoder or field vocabulary in the engine, and caches naturally retain
the contract because it is part of the Wasm bytes.

Time-zone canonicalization is intentionally not the next decorative binding.
The resolved `timezone_provider 0.1.2` normalizer identifies its baked alias
data as tzdb `2025b`, while the selected `jiff-tzdb` transition data reports
`2026a`. Until one generated catalogue gives both layers a coherent identity,
`CanonicalizeTimeZone` must remain unbound rather than mixing data vintages.

`icu_normalizer = 2.0.1` with `compiled_data` and
`icu_properties = 2.0.2` remain direct `lila-aot-wasm` dependencies. The locale
provider itself lives in `lila-intl`; AOT and engine depend on that shared crate
instead of owning ICU aliases or protocol ordinals independently.

## Frozen first data line

The first complete conformance profile uses the ICU4X 2.0 family already
resolved in this repository. Product manifests must pin these with exact
requirements when they become direct dependencies; semver-compatible ranges
are insufficient for conformance data.

| Role | Exact package version in the current lock |
|---|---:|
| provider and blob adapter | `icu_provider 2.0.0`, `icu_provider_blob 2.0.0`, `icu_provider_adapters 2.0.0` |
| locale/canonicalization | `icu_locale 2.0.0`, `icu_locale_core 2.0.1` |
| calendars | `icu_calendar 2.0.6` (workspace-patched) |
| collation/casing | `icu_collator 2.0.0`, `icu_casemap 2.0.1` |
| date/time formatting | `icu_datetime 2.0.1`, `icu_time 2.0.1` |
| decimal/plural/list formatting | `icu_decimal 2.0.1`, `icu_plurals 2.0.0`, `icu_list 2.0.1` |
| normalization/properties | `icu_normalizer 2.0.1`, `icu_properties 2.0.2` |
| segmentation | `icu_segmenter 2.0.1` |

The compiled ICU4X data crates currently present in the lock report CLDR
`47.0.0`, ICU data tag `icu4x/2025-05-01/77.x`, and LSTM segmenter data
`v0.1.0`. That ICU data line corresponds to Unicode `16.0.0`; all four values
belong in the generated manifest rather than being inferred at runtime.

Time-zone transitions are shared with T22. The current resolved chain is
`timezone_provider 0.1.2` plus `jiff-tzdb 0.1.6`; the embedded database reports
IANA tzdb `2026a`. The conformance profile freezes that exact database. File
system TZif providers remain useful development tools but are forbidden as the
default product provider.

Any later dependency or data upgrade is one atomic conformance event: update
the exact pins, regenerate the data image and digest, publish size changes, and
run the affected Intl/Temporal trees. Mixing data generated by different ICU,
CLDR, Unicode, segmentation, or tzdb versions is rejected.

ICU4X does not replace ECMA-402. When a stable ICU4X API does not cover a
required service or exact ECMA-402 behavior, Lila owns the missing pure
algorithm and consumes data exported from the same manifest. Adding an
unversioned native library or an English-only implementation is not a fallback.

## Layering decision

Intl is split into four layers with one-way dependencies.

1. **Observable Wasm shell.** Emitted builtins perform `Get`, calls, iterator
   steps, coercions, validation, new-target handling, branding, realm choice,
   bound-function caching, and result-object/parts construction. These use the
   T04/T10 operations and intrinsic registry. No ICU call may happen while an
   observable JavaScript read or call is still pending.
2. **Typed Intl protocol.** A small Rust-owned ABI accepts only canonical
   locale identifiers, validated service options, numbers/BigInts/time values,
   and UTF-8/UTF-16 spans. It returns primitive resolved records or part lists.
   The protocol catalogue is shared by the emitter and engine, so an operation
   name, ordinal, signature, and required capability have one source of truth.
3. **Pure Rust kernel.** ICU4X and Lila-owned pure algorithms implement locale
   negotiation, formatting, collation, and segmentation against an immutable
   provider selected for the Wasm instance. This layer cannot access process
   locale, environment variables, filesystem zoneinfo, JavaScript heap values,
   or realm state.
4. **Versioned data image.** Locale, Unicode, CLDR, and tzdb data are generated
   ahead of execution. The image has a checked schema, manifest, content
   digest, sorted locale/capability indexes, and immutable payloads.

The existing `lila_host` mechanism is the initial transport between layers 1
and 2. It is a runtime support boundary, not permission to call host-native
Intl. Requests and responses live in Wasm memory. The landed locale operation
validates pointers, lengths, UTF-8, operation tags, the kernel capability and
output capacity before constructing or returning Rust domain types. Its
host-external provider now satisfies the artifact/profile identity match, while
still remaining distinct from the future embedded data-image contract.

Each import corresponds to a typed `IntlHostOp`, or is generated from that
closed catalogue. Do not add a stringly `intl_call(name, ...)` escape hatch.
The host receives no JavaScript values and never decides which getter to run or
which exception object to create. Expected kernel failures are a closed enum;
profile/schema/digest mismatch rejects instantiation rather than surfacing as a
surprising JavaScript error halfway through formatting.

This design keeps the emitted artifact a real AOT-compiled program. The Rust
kernel is ordinary runtime support analogous to clock and memory imports; it
does not parse or execute JavaScript.

## Rust domains that enforce the boundary

Names may evolve when code lands, but these closed domains and transitions are
required:

```rust
enum IntlService {
    Locale,
    Collator,
    NumberFormat,
    DateTimeFormat,
    PluralRules,
    RelativeTimeFormat,
    ListFormat,
    DisplayNames,
    Segmenter,
    DurationFormat,
}

enum IntlDataProfile {
    Conformance,
    Minimal,
    Custom(CustomProfileId),
}

enum IntlDataPlacement {
    Embedded,
    External,
}

enum IntlDataCapability {
    LocaleAliases,
    LikelySubtags,
    ParentLocales,
    Calendars,
    NumberingSystems,
    DecimalPatterns,
    PluralRules,
    Collation,
    DateTimePatterns,
    TimeZoneNames,
    TimeZoneTransitions,
    RelativeTimePatterns,
    ListPatterns,
    DisplayNames,
    Segmentation,
    UnitsAndCurrencies,
}
```

Every enum has `ALL` and exhaustive matches; there are no `_` arms in service,
operation, or capability dispatch. `LocaleId`, `CanonicalLocaleId`,
`TimeZoneId`, `CanonicalTimeZoneId`, `CustomProfileId`, `IntlDataDigest`, and
every version field are validated newtypes with private fields. The landed
protocol also makes `IntlOperation` sealed and associates its request, response
and failure types; a locale request cannot compile against a time-zone handle.
A complete data profile is constructed only through a validator that proves:

- schema and all upstream versions are the expected exact values;
- every advertised service has all required capabilities;
- every locale has a complete fallback chain ending at the profile default;
- indexes are sorted, unique, in bounds, and point to payloads with the right
  marker/type;
- the shared time-zone catalogue, links, transitions, and localized-name data
  refer to the same canonical identifiers and declared tzdb version; and
- the content digest covers the manifest and all payload bytes.

The observable-to-kernel transition should have an explicit type progression:

```text
ObservedServiceInputs<S>
    -> ValidatedServiceOptions<S>
    -> ResolvedServiceLocale<S>
    -> ServicePlan<S>
```

Only the final three types may cross into the pure kernel. Constructors stay
private, so backend code cannot accidentally format before getters and
coercions finish. Service-specific formatter handles are also distinct types;
a Collator handle cannot be passed to NumberFormat. Handles are scoped to one
Wasm instance and its validated data digest.

## Packaging, profiles, and lazy work

`Conformance` is the default for Test262 and status publication. It contains
all locales, services, Unicode properties, calendars, currencies, units, and
zone data required by the pinned suite, plus the fixed default locale. A
missing mandatory capability makes profile generation fail. It is never
silently repaired with English data.

`Minimal` is an explicitly selected production-size profile. It still carries
complete data for every advertised service at its default/root locale. Locale
negotiation may therefore fall back normally, but an internally incomplete
service cannot be advertised. `Custom` is built from a declarative manifest of
locales, services, calendars, numbering systems, and zones. The generator
computes transitive fallback and capability closure; callers cannot hand-write
the closure bitset.

With `Embedded` placement, the generated image and manifest are carried in
named Wasm custom/data sections. The engine validates and associates them with
the instance before start code runs. With `External` placement, the artifact
carries the complete expected manifest and digest while the embedder supplies
the matching immutable image through an explicit Lila Intl capability. A
missing or non-matching capability is an instantiation error. It never falls
back to host data.

Additional embedder data therefore changes neither JavaScript APIs nor the ABI:
the artifact is compiled for a specific custom profile/digest. An embedder
cannot attach arbitrary extra locales to an artifact whose manifest did not
select them, because doing so would make `supportedLocalesOf` deployment-
dependent.

Lazy loading means lazy validation/index materialization and formatter cache
creation from an already selected immutable image. It never means filesystem,
network, environment, or OS-locale lookup. Shared read-only tables may be
cached across realms; formatter handles and bound function identities remain
instance/realm-owned. Cache lookup starts only after all observable input
processing is complete.

Build and diagnostic output records, at minimum, the profile kind and ID, data
placement, schema version, digest, default locale, ICU4X package line, CLDR,
Unicode, ICU data tag, segmenter model, tzdb version, locale count, capability
set, and compressed/uncompressed byte sizes. Conformance and minimal profile
sizes are reported separately.

## Locale and missing-data behavior

The default locale is an artifact/profile input represented by a validated
`LocaleId`; it is never read from the process. The conformance profile fixes it
to the value selected by the Test262 configuration.

Fallback is split deliberately:

- structurally invalid language tags or invalid option values produce the
  ECMA-402-required JavaScript error in the observable shell;
- a valid but unavailable requested locale follows LookupMatcher/BestFitMatcher
  and the profile's complete parent chain to its fixed default;
- a valid but unsupported Unicode extension value is ignored or replaced by
  the resolved default exactly where `ResolveLocale` requires it;
- an unknown time-zone identifier is handled by the ECMA-402 time-zone
  validation rules, not locale fallback; and
- missing data promised by the manifest is an invalid artifact/provider and is
  rejected before JavaScript execution.

Best-fit matching is a named, versioned algorithm choice backed by the pinned
data. It must not alias lookup matching merely because that passes a small
fixture.

## Shared boundary with Date and Temporal

T22 and T23 consume one validated time-zone catalogue. Its Rust face exposes
canonical identifier lookup, primary/link resolution, possible instants,
offset-at-instant, next/previous transition, and the tzdb version. The
`temporal_rs::TimeZoneProvider` adapter and Intl.DateTimeFormat use that same
catalogue and the same `CanonicalTimeZoneId` newtype.

Localized time-zone names are an Intl/CLDR capability layered over canonical
zone IDs; they do not own a second transition database. The host default zone
is a separate explicit clock/host setting validated against the catalogue. It
may select a zone but may not provide transitions or localized names.

Calendar identifiers and aliases likewise come from one generated catalogue.
Temporal and Intl adapters may expose different spec surfaces, but the same
spelling cannot canonicalize differently in the two subsystems.

## Code seams and staged delivery

The intended ownership seams are:

- a small `lila-intl` crate for the manifest, generated catalogues, pure kernel,
  service/operation enums, typed requests/results, and profile validation;
- `lila-aot-wasm::builtins::intl_common` for observable abstract operations and
  the staged input builders;
- one backend module per service for JavaScript object/realm semantics and ABI
  adaptation;
- `lila-aot-wasm` artifact planning/emission for data sections, required Intl
  imports, and metadata; and
- `lila-engine` for instantiation checks and bindings from typed Intl imports to
  an instance's immutable `lila-intl` provider.

The new crate is justified by the dependency direction: neither the public
engine nor the backend should own a second copy of the data schema or operation
catalogue. It must not depend on parser, IR, Wasmtime, or JavaScript object
representations.

Implementation proceeds in dependency order:

1. Finish the manifest/provider foundation. Manifest/profile/capability types,
   deterministic identifier domains, exact locale pins, a host-embedded locale
   provider, and one consumed typed kernel/engine binding have landed. The
   canonical artifact ABI metadata and pre-instantiation identity match have
   landed. The deterministic artifact generator/image remains.
2. Replace locale canonicalization and options helpers with the shared locale
   layer; finish `Intl.Locale`, matching, extension negotiation, and
   `supportedValuesOf`.
3. Implement NumberFormat and PluralRules over shared decimal/rounding records;
   then RelativeTimeFormat and DurationFormat.
4. Move DateTimeFormat from handwritten locale/zone tables to the shared data
   and time-zone provider; integrate Date and Temporal locale methods.
5. Implement Collator and Segmenter, keeping segment indices in UTF-16 code
   units at the Wasm boundary.
6. Implement ListFormat and DisplayNames, then close descriptors, realms,
   subclassing, bound functions, ranges, parts, and cross-service option reuse.
7. Remove bootstrap-only tables only after their general replacement is in the
   product call graph, measure both data profiles, and run the full pinned
   `intl402` and adjacent locale-sensitive trees.

After step 1, service lanes can be dry-written concurrently because each owns a
service module and closed request/result types. Shared locale/options, the
operation catalogue, manifest schema, intrinsic registry, and artifact ABI are
integrator-owned files and change in coordinated batches.

## Completion evidence

T23 remains incomplete until all of the following are true:

- every selected dependency and upstream data version is exact and emitted in
  artifact diagnostics;
- the conformance image is reproducibly generated and its digest stable across
  supported build hosts;
- the engine rejects absent/mismatched Intl capabilities before execution;
- no Intl/locale-sensitive product code reads host locale or OS locale/tz data;
- all services and adjacent String, Number, BigInt, Date, and Temporal methods
  use the shared observable shell and data/kernel boundary;
- conformance and minimal data/code size are measured separately; and
- the full pinned `intl402` tree is green under Wasm-AOT with no exact-result
  materializations or silent skips.
