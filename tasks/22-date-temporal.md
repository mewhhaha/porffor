# T22 — Date and Temporal

**Status:** Blocked on T04-T06/T10/T18/T20  
**Parallel group:** Feature lane; Date and Temporal can be separate sub-owners  
**Depends on:** T04, T05, T06, T10, T18, T20; locale formatting integrates with T23  
**Blocks:** Time-related T23/T26 closure

## Objective

Implement exact Date semantics and the complete Temporal API for the pinned revisions using deterministic clock, calendar and time-zone interfaces. Reuse vendored `temporal_rs` where appropriate, but preserve JavaScript-observable coercion, property access, branding, realm and descriptor behavior in Porffor's own runtime/compiler layers.

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
- Porffor object branding/internal slots and prototype dispatch;
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
cargo test -p porffor-runtime time_ --quiet
cargo test -p porffor-aot-wasm date_ --quiet
cargo test -p porffor-aot-wasm temporal_ --quiet
cargo test -p porffor-cli wasm_date --quiet
./target/debug/porf test262 run built-ins/Date --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/porf test262 run built-ins/Temporal --execution-backend wasm --timeout-ms 240000 --threads 4
```

Run with several injected zones—including UTC and zones with DST gaps/folds—and compare deterministic outputs against the spec-exec backend.