# Date current-time source

ECMAScript gives the Date constructor three distinct current-time consumers:

- `Date.now()` returns the current UTC time value;
- `Date(...)`, called without `new`, ignores its arguments and formats the
  current time with `ToDateString`; and
- zero-argument `new Date()` stores the current time in `[[DateValue]]`.

The normative algorithm is
[`Date ( ...values )`](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-date-constructor).
The function-call branch runs before argument-count dispatch, while only the
zero-argument construct branch reads the clock. Neither branch substitutes the
Unix epoch or applies `TimeClip` to the host reading.

## Closed source domain

Date string formatting accepts one private, exhaustive source domain:

- `ReceiverSlot { payload_local, tag_local }` performs the Date brand check and
  reads the receiver's `[[DateValue]]`; and
- `RealmHostClock` performs one `lila_host.wall_clock_millis` call.

The host-clock arm is also the sole Date-owned clock-import consumer used by
`Date.now()` and zero-argument construction. Adding another Date time-value
source therefore requires extending an exhaustive match instead of silently
inheriting receiver or host semantics. `DateTimeValueSource`, its exhaustive
consumer, the sole clock-import access and all five raw producers live together
in the private `builtins/date/local_string.rs` child. The domain derives no
cloning, copying, debugging, equality or default-construction capability; the
parent and standard dispatcher retain only semantic emitter calls and cannot
name or reconstruct the raw source policy.

The private `builtins/date/local_string.rs` child owns
`DateLocalStringFormat`, its sole consumer, and all four raw policy producers.
The consumer exhaustively projects both decisions together: `Date` includes
only the date half, `Time` includes only the time half, and `DateAndTime`
includes both. A new format therefore cannot inherit either half from a
Boolean default. The private domain derives no cloning, copying, debugging,
equality or default-construction capability. The parent and standard-builtin
dispatcher retain only the existing semantic emitter calls; neither can name
or reconstruct the raw format policy.

The exact five-line domain, ten-line `Date()` producer and 270-line formatter
selection retain SHA-256
`8189b9bba6c4e3c5dbb6f771fdbf23aa7a4f4e96d3898bec520380ac9e7d7916`,
`7736637485e2f8118b32ca75f7eb5e7ca5fc8cf12c826d729dfa6966d14d7cff`
and
`59c1fe7398cca3b5066118019ee541285520fe4a4305ddab4a5a932420ad64b2`.
Those 285 semantic lines retain combined SHA-256
`a155dd53ada727aadd829f02894112843b4187f47c9316710ef932234992493a`;
the formatter method's final structural delimiter moves with them but is not
part of that frozen selection. Including it, the 271-line physical formatter
and complete 286-line physical move have SHA-256
`455adc77784d562e552aadb0ae299a73b532d77e5b9bfab4af5d97d359cd49ec`
and
`45898a09089b25568c753b7990b7fba581fa19222ab8d41810fab80466f8d069`.
The 292-line child has SHA-256
`a75df03ae28a2524c322fa66b028ca50d28d2dcda642334fd0ce0ad73dfce143`
and reduces the current parent from 2,010 to 1,722 lines. The recursive guard
and module policy pin zero parent format names, all nine format mentions and
all five raw consumer/producer sites in the child, plus the unchanged
one/two/two/two semantic dispatcher census. The unchanged Date-function call
line and 25-line prototype dispatch slice retain SHA-256
`45e55826e4ee110a73b8067f64c1555e01b2d418a74fbae785a7d2c73cd597d4`
and
`d3ea28e34b1c66dec63dbdb38d299a5e1d93f2477457bbfa2d0fe6e76be567ee`.

The runtime boundary remains `UtcEpochMilliseconds`. Its range is integral and
strictly below `2^53`, so the existing Wasm `f64` import represents every valid
host result exactly. The backend does not revalidate or clip that already
validated value.

The Batch Z source-owner move preserves the exact four-line source domain and
41-line exhaustive-consumer/current-time-wrapper selections at SHA-256
`71ba6e635d162f63abcd7a35eb6cf7e66ae2e53b02eb43d038ec79febc0d3492`
and
`4ff02aca4eca4f2bc447c380079b9c9a6e01182b072de4366364dd9947dfa6dc`;
their combined 45-line selection retains SHA-256
`4c570b074e898f3a4b9930d42b12cb0694ce3c64e3469266c78d4acf7c6afe61`.
The resulting 1,675-line parent and 339-line child have SHA-256
`afe18d7006f8d8ffde380e8d667c56837cdef6d19ebe0e845c9434b41e0609c0`
and
`ae6d62f5ac5586704695a77839582e3a2fe8dc3fdb0b95e50b3be5157f4ec435`.
The recursive guard and module policy pin zero parent source names, raw
consumer calls and clock-import accesses, and exact child censuses of ten
source names, seven qualified variants, three consumer/caller sites and one
clock access. The retained six-line `Date.now()` wrapper and standard
constructor call retain SHA-256
`026e7208baa5402ac0bc15e098caf04c72b69f8cffb480d058c6759287666e63`
and
`c15bb6f7dbebf9fd006c56abfd49faf9b78e6bdc660dcc1efa81283d2bd03afa`.
This is a source-equivalent owner move. At the 2026-08-28 Batch Z checkpoint,
`cargo xc` is green, the recursive structure target passes `7/7`, both exact
backend source-oracle tests pass `2/2`, and the exact injected-clock engine and
locale-string CLI witnesses each pass `1/1`. Emitted-Wasm golden verification
remains deferred.

## Import authority

The standard-builtin catalog is the sole authority for optional host imports.
Both `DateNow` and `DateConstructor` carry `WALL_CLOCK`; removing either flag
causes emission of that builtin body to lack the import its exhaustive source
consumer requires. `DateConstructor` retains `CONSTRUCTABLE` as an independent
flag.

Because one generic Date builtin body contains call, zero-argument construct,
and explicit-argument construct paths, an artifact that uses `new Date(0)` may
conservatively carry the clock import even though that invocation does not read
it at runtime.

## Observable regression

The durable regression uses a realm with the injected time
`1640995200123` (`2022-01-01T00:00:00.123Z`) and checks all consumers against
that nonzero value. It also checks a Date subclass and calls `Date` with an
object whose conversion hooks would throw: argument expressions are evaluated
by the language, but the Date function must not coerce or inspect their values.

The pinned `S15.9.2.1_A2.js` case is not sufficient by itself. It compares
`Date()` with `(new Date()).toString()`, so two implementations that both
substitute the epoch agree with each other while disagreeing with the host
clock.

The existing `wasm_date_locale_strings.js` CLI witness covers the receiver-slot
source and the `Date`, `Time` and `DateAndTime` projections through the three
locale aliases and their direct Date-method counterparts. The injected-clock
engine witness covers the host source through `Date.now()`, `Date()`, ordinary
and subclass construction. The bounded recursive structure regression owns
both private declarations, the private child boundary, their absent
capabilities, both exhaustive projections, the exact source and format
producer censuses and every producer mapping:

```console
cargo test -p lila-aot-wasm --test date_local_string_format_structure
cargo test -p lila-engine --lib tests::wasm_backend_uses_one_injected_clock_for_date_temporal_and_monotonic_reads -- --exact --test-threads=1
cargo test -p lila-cli --test cli date::run_wasm_backend_succeeds_for_date_locale_strings_fixture -- --exact --test-threads=1
```

This source-equivalent invariant migration is expected to leave emitted Wasm
byte-identical. At the 2026-08-28 Batch Y checkpoint, the structure target
passes `7/7`; the exact injected-clock engine witness and the exact existing CLI
receiver witness each pass `1/1`. Independent review is clean. The shared
`cargo xc`, full formatter, diff, module-boundary and task-plan checks are
green. Emitted-Wasm goldens and broad conformance suites remain deferred.

## Non-claims

This contract does not add a default-time-zone provider. Date local operations
and `ToDateString` therefore retain the backend's documented UTC/fixed-offset
behavior. It also does not change explicit-argument construction, parsing,
`TimeClip`, DST handling, Temporal APIs, or the monotonic execution clock.
