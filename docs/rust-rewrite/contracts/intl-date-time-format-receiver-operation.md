# Intl.DateTimeFormat receiver operation domain

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

## Closed receiver policy

The five DateTimeFormat entry points that require an initialized receiver use
the private `IntlDateTimeFormatReceiverOperation` domain:

- `ResolvedOptions`;
- `FormatGetter`;
- `FormatToParts`;
- `FormatRange`; and
- `FormatRangeToParts`.

The domain derives no capabilities. Its borrowed exhaustive `full_message`
projection owns each complete incompatible-receiver diagnostic. The receiver
reader accepts only a borrowed operation, so an arbitrary method spelling or a
separately assembled message cannot reach the brand check.

The ordered `ALL` list matches the former string-pool insertion order. The
DateTimeFormat pool walks `ALL` and `full_message`, making the semantic
projection the only message authority without changing data-segment ordering.

## Producers and ordering

`resolvedOptions`, the `format` getter and `formatToParts` each construct their
named operation directly. The shared range body maps its existing exhaustive
`DtfFormatMode` to `FormatRange` or `FormatRangeToParts`; callers still select
only `String` or `Parts` output mode.

For ranges, receiver validation remains before either argument conversion.
The operation projection is Rust emission policy and adds no Wasm instruction.
All receiver tag/brand loads, the invalid-receiver TypeError path, range
conversion order, formatter selection and result-tag instructions remain in
their previous order.

## Durable evidence

`intl_dtf_receiver_operation_structure.rs` pins the five rows, lack of
capabilities, ordered `ALL`, exact full-message projection, pool loop, typed
reader, three direct producers and exhaustive range mapping.
`intl_dtf_range_mode_structure.rs` separately pins that the range brand check
precedes argument conversion and that output mode still selects both formatter
and result representation.

At the 2026-08-27 focused checkpoint, the receiver structure target passes
`4/4`, the neighboring range-mode structure target passes `3/3`, and the
retained DateTimeFormat construction-order CLI fixture passes `1/1`. The six
exact vendored receiver leaves pass all `12/12` sloppy/strict Wasm-AOT
executions:

- `prototype/format/no-instanceof.js`;
- `prototype/resolvedOptions/no-instanceof.js`;
- `prototype/formatToParts/this-has-not-internal-throws.js`;
- `prototype/formatToParts/this-is-not-object-throws.js`;
- `prototype/formatRange/this-bad-object.js`; and
- `prototype/formatRangeToParts/this-bad-object.js`.

Every Parser, EarlyError, Lowering, Runtime, WasmBackend, HostHarness and
Unsupported bucket is zero. All twelve outcomes are `Success`, with
`NotImplemented`, `Crash` and `Bug` also zero. Each leaf used its own snapshot
with one compiler job and one Test262 worker.

This boundary adds no fixture or source rewrite and does not change supported
locales, calendars, time zones, formatting data, conformance counts, or the
open created-Realm Intl work. The focused cohort is not a full DateTimeFormat,
Intl402 or T23 closure claim.
