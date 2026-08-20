# T22 — Date and Temporal

**Status:** In progress — Date and initial Temporal adapters exist; complete APIs/data remain open

**Parallel group:** Feature lane; Date and Temporal can be separate sub-owners  
**Depends on:** T04, T05, T06, T10, T18, T20; locale formatting integrates with T23  
**Blocks:** Time-related T23/T26 closure

## Current repository state

Date has a substantial dedicated backend implementation and focused complete
leaves. Temporal now has a dedicated builtin module with Instant and
ZonedDateTime work and focused snapshots. A realm-owned `HostClock` is the one
product boundary for JavaScript-visible wall-clock and monotonic reads:
`UtcEpochMilliseconds` validates the shared Date/Temporal range, monotonic
instants and durations are separate types, tests may inject a deterministic
clock, and the production `SystemHostClock` is shared by realm clones and agent
workers. `Date.now`, `Temporal.Now` clock reads, the Wasm monotonic-clock import
and the Test262 agent monotonic operation all consume that boundary without
changing their Wasm ABI. Engine compilation, timeout and sleep machinery still
uses real execution-control timers and is deliberately not virtualized.

Date's remaining current-time consumers now share that boundary too. A private,
exhaustive `DateTimeValueSource` distinguishes a branded receiver slot from the
realm host clock; `Date()`, zero-argument `new Date()` and `Date.now()` all take
the clock arm, while prototype string methods take the receiver arm. The Date
constructor's catalog entry carries the load-bearing wall-clock capability, and
an injected nonzero clock regression prevents the function and constructor from
agreeing only because both substituted the Unix epoch. This does not change the
UTC/fixed-offset limitation of local Date formatting.

The default time-zone boundary is not implemented yet. Current AOT Date local
operations and `Temporal.Now` defaults remain UTC/fixed-offset behavior.
`lila-intl` owns the closed `CanonicalTimeZoneId` domain, but no canonical zone
provider or Wasm host ABI connects it to Date/Temporal yet; a realm setting with
no semantic consumer would not satisfy this task. The complete Temporal class
surface, custom calendar/time-zone protocols, pinned deterministic data, Intl
integration and materialization-free full Date/Temporal trees therefore remain
open.

Artifacts that use the existing Intl locale host operation now carry the full
canonical `IntlDataIdentity`, and the engine matches it to the shared provider
before Wasmtime compilation or instantiation. That closes the
artifact/provider identity seam shared with T23, but it does not add a
time-zone provider, transition data, or a default-zone consumer. In particular,
the carried identity's pinned tzdb field is metadata for the selected complete
data line; the current Locale-only external provider still does not claim that
capability.

Temporal time-string parsing now carries an exhaustive calendar-consumer
policy. `PlainTime` consumes the `Ignore` arm required by its grammar, while
`ToTemporalCalendarIdentifier` consumes `Resolve`, so a time string's
`[u-ca=...]` value is canonicalized exactly when it denotes a calendar. This
keeps `PlainTime.from("T11:30[u-ca=unknown]")` valid while making
`withCalendar("T11:30[u-ca=notacal]")` throw instead of silently defaulting to
`iso8601`. Complete calendar coverage and the full Temporal tree remain open.

Date construction now has a closed, required realm-prototype fallback. The
realm-intrinsics record owns `%Date.prototype%`, entry and created realms both
publish that slot, and all zero-, one- and multiple-argument Date construction
select it through the same typed `GetFunctionRealm` policy when
`NewTarget.prototype` is primitive. Missing resolved-realm bootstrap state
traps rather than falling back to the entry global, while object-valued custom
prototypes and the existing branded Date allocation remain on the same
specification path. The prototype read occurs after zero-argument clock acquisition or argument
coercion, and object-valued Object, Function and Array prototypes retain their
payload tag through Date allocation. This targets the three exact
`built-ins/Date/proto-from-ctor-realm-{zero,one,two}.js`
failures from the 2026-08-13 current-pin Wasm-AOT baseline; focused current-SHA
execution remains deferred until that low-RAM matrix releases Cargo/Test262.
The invariant and deferred gates are recorded in
`docs/rust-rewrite/contracts/date-constructor-realm-prototype.md`.

ZonedDateTime differences now have a closed default-largest-unit plan. The
shared PlainDateTime settings reader distinguishes PlainDateTime `until`,
PlainDateTime `since` and ZonedDateTime delegation; the first two resolve an
unset or `"auto"` `largestUnit` from `day`, while the delegate resolves it from
`hour`. A non-copyable resolved-settings witness is consumed directly by the
PlainDateTime arithmetic or materialized as an unreachable normalized options
bag for the existing ZonedDateTime-to-PlainDateTime call. User getters and
conversion hooks are therefore observed once, while the arithmetic body stays
single-sourced. This targets the pinned ZonedDateTime
`defaults-to-returning-hours`, `largestunit-undefined` and
`largestunit-default` cases for both `until` and `since`. Current-SHA execution
remains deferred while the low-RAM matrix owns Cargo/Test262; the invariant and
deferred gates are recorded in
`docs/rust-rewrite/contracts/temporal-zoned-date-time-difference-default.md`.

## Objective

Implement exact Date semantics and the complete Temporal API for the pinned revisions using deterministic clock, calendar and time-zone interfaces. Reuse vendored `temporal_rs` where appropriate, but preserve JavaScript-observable coercion, property access, branding, realm and descriptor behavior in Lila's own runtime/compiler layers.

## Host time contract

Define typed host services for:

- current UTC epoch time used by `Date.now` and Temporal's clock-facing operations;
- default time-zone identifier and offset-transition queries;
- pinned time-zone database/version metadata;
- monotonic time for Test262 agent APIs, kept distinct from wall-clock time.

Tests must inject deterministic clocks/zones. Production defaults may use the host, but behavior must not depend on process locale or undocumented OS parsing.

## Date

Implement all Date constructor/function and prototype semantics:

- call vs construct behavior, zero/one/multiple arguments and custom new target;
- `TimeClip`, MakeTime/MakeDay/MakeDate, year 0-99 adjustment and invalid dates;
- ISO date-time string parsing required by ECMAScript, legacy implementation-defined forms only where explicitly supported/documented;
- local/UTC getters and setters, overflow normalization and argument coercion order;
- DST gaps/folds and historical offset handling through the pinned zone provider;
- `Date.parse`, `Date.UTC`, `Date.now`;
- `toISOString`, `toJSON`, `toString`, `toUTCString`, date/time/localized variants, `valueOf`, `getTime` and `@@toPrimitive`;
- exact descriptors, branding, realm errors and subclass/custom-prototype behavior.

Do not use Rust/OS date parsers for ECMAScript ISO parsing unless wrapped by exhaustive compatibility tests.

## Temporal integration architecture

Create a JavaScript-facing adapter around `temporal_rs` or the selected semantic kernel. The adapter must own:

- ordered option/property access and conversion through T04;
- Lila object branding/internal slots and prototype dispatch;
- realm-specific constructors/prototypes/errors;
- conversion to/from ECMAScript strings, Numbers, BigInts and objects;
- calendar/time-zone protocol calls, including user-defined objects and abrupt completion;
- iterable/record construction and property descriptors.

Do not expose Rust library structs directly as JavaScript objects or let the library bypass proxies/getters.

## Temporal API scope

Implement every class/function in the pinned suite, including as applicable:

- `Temporal.Instant`;
- `PlainDate`, `PlainTime`, `PlainDateTime`, `PlainYearMonth`, `PlainMonthDay`;
- `ZonedDateTime`;
- `Duration`;
- `Now` operations;
- calendar and time-zone handling required by the current proposal/revision;
- parsing/formatting, arithmetic, comparison, rounding, balancing, total, since/until and field preparation;
- offset options, disambiguation, overflow, smallest/largest unit, rounding increment/mode and relative-to behavior;
- protocol interactions with custom calendars/time zones if present in the pin.

Use generated tables/enums for option names and units to prevent inconsistent validation among methods.

## Correctness and data requirements

- Pin calendar, Unicode and IANA time-zone data versions in reproducible build metadata.
- Add vectors around leap years, negative epoch values, extreme valid ranges, nanosecond boundaries, DST transitions and calendar eras.
- Preserve arbitrary-precision nanosecond arithmetic where required; do not pass through f64.
- Avoid host-dependent formatting except through the explicitly pinned Intl layer.
- Distinguish invalid input errors and range boundaries exactly.

## Acceptance criteria

- Full pinned `built-ins/Date` and Temporal trees are green.
- Runs are deterministic under an injected clock/time zone and reproducible across supported hosts.
- Date parsing, setters and formatting pass DST/extreme-range/coercion-order tests.
- Temporal option/property access order works with proxies/getters and abrupt completions.
- All Temporal classes enforce branding, descriptors, subclassing and cross-realm error behavior.
- Time-zone data version is pinned and surfaced in developer diagnostics.
- No exact Test262 date/time materialization remains.

## Required tests

```sh
cargo test -p lila-runtime time_ --quiet
cargo test -p lila-aot-wasm date_ --quiet
cargo test -p lila-aot-wasm temporal_ --quiet
cargo test -p lila-cli wasm_date --quiet
./target/debug/lila test262 run built-ins/Date --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/lila test262 run built-ins/Temporal --execution-backend wasm --timeout-ms 240000 --threads 4
```

Run with several injected zones—including UTC and zones with DST gaps/folds—and compare deterministic outputs against the spec-exec differential oracle (diagnostic comparison only; the Wasm-AOT results are the product evidence).
