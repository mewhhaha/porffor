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
inheriting receiver or host semantics.

`DateLocalStringFormat` separately owns the closed date/time projection. Its
`includes_date` and `includes_time` decisions are exhaustive matches, so a new
format cannot accidentally omit both halves.

The runtime boundary remains `UtcEpochMilliseconds`. Its range is integral and
strictly below `2^53`, so the existing Wasm `f64` import represents every valid
host result exactly. The backend does not revalidate or clip that already
validated value.

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

## Non-claims

This contract does not add a default-time-zone provider. Date local operations
and `ToDateString` therefore retain the backend's documented UTC/fixed-offset
behavior. It also does not change explicit-argument construction, parsing,
`TimeClip`, DST handling, Temporal APIs, or the monotonic execution clock.
