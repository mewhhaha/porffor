# `Temporal.ZonedDateTime` field-result ownership

Status: T22 ownership invariant extended to the complete 21-field dispatch,
2026-09-10. All six tests in the coordinated native surface target pass.

## Boundary

`emit_temporal_zoned_date_time_iso_field` emits one of 21 accessor bodies.
Nine bodies leave one Number payload on the Wasm stack. The other twelve write
the complete result payload/tag pair themselves. The private domain naming
that distinction is:

```rust
enum ZdtFieldResult {
    NumberOnStack,
    WrittenByCallee,
}
```

Every invocation creates one result value from the exhaustive field match and
immediately consumes it in the sole result-publication match. The domain has no
clone, copy, debug, equality or default capability. A second consuming
observation therefore fails to compile instead of duplicating or moving result
publication silently.

## Result routes

The nine existing numeric stack arms publish their value as Number. The
self-written routes preserve the callee's complete result pair:

- `monthCode` publishes String; `era` and `eraYear` retain the shared calendar
  emitter's String, Number or Undefined result pair.
- `dayOfWeek`, `dayOfYear`, `daysInWeek`, `daysInMonth`, `daysInYear` and
  `monthsInYear` project the receiver's local ISO date through the shared
  PlainDate calculation and publish Number.
- `weekOfYear` and `yearOfWeek` publish the ISO week projection as Number or
  Undefined for a calendar without the ISO week convention.
- `inLeapYear` publishes Boolean.

The calendar projection accepts the private closed
`ZonedDateTimeCalendarField` domain. Its exhaustive mapping names one shared
PlainDate calculation for each numeric calendar accessor. Local date components
are converted from Number payload bits to integer fields before that call.

The type does not prove that an arm selected the correct result variant. That
mapping remains guarded structurally: the exact complete 21-arm block and the
final consuming match are pinned together with no unchecked gap.

## Durable source guard

The Rust-lexical structure target pins:

- an attribute-free private two-variant declaration with no manual capability,
  alias, representation or cast route;
- exactly 24 source-wide `ZdtFieldResult` identifiers: one declaration, 21
  producers and two consumer arms;
- exactly ten qualified `NumberOnStack` routes and thirteen qualified
  `WrittenByCallee` routes, including their final consumer arms;
- exactly two `delivery` identifiers, its inferred binding and its consuming
  match;
- all 21 complete field bodies in order, each bound to its exact result
  variant; and
- the exact final Number publication, empty self-written arm and publication
  before temporary-local release, with no wildcard or secondary observation.

The lexical normalizer ignores comments and formatting, preserves literal
contents, canonicalizes raw identifiers and is exercised by normal, byte, C,
raw, raw-byte and raw-C strings plus character, byte-character and lifetime
syntax.

## Historical focused evidence: 2026-08-27

The original capability removal was source-equivalent: it changed no field
arm, emitted instruction, helper call, result tag or temporary-local order.
That version had twelve producers, 15 source-wide result identifiers, ten
qualified `NumberOnStack` routes and four qualified `WrittenByCallee` routes.

```sh
cargo test -p lila-aot-wasm \
  --test temporal_zoned_date_time_field_result_structure -- --test-threads=1
cargo test -p lila-aot-wasm \
  --test temporal_zoned_date_time_calendar_coercion_structure -- --test-threads=1
cargo test -p lila-cli --test cli -- \
  --exact date::run_wasm_backend_succeeds_for_temporal_zoned_date_time_era_fixture
```

At that checkpoint, the dedicated and neighboring structure targets passed
`3/3` each, the exact CLI fixture passed `1/1`, and workspace formatting plus
the owned diff check passed. The CLI fixture covered the numeric tail and
self-written era path, including ISO Undefined results, gregory String/Number
pairs, the BCE boundary, fixed offsets and sub-millisecond components. These
results describe the historical twelve-field implementation.

## Current verification scope

The 2026-09-10 repair batch adds nine accessors, so its emitted behavior is no
longer source-equivalent to the historical checkpoint. The new
`aot_temporal_zoned_date_time_surface` native target covers local-date calendar
values, ISO week-year boundaries, Boolean/Undefined result tags, accessor
metadata and brand rejection. All six tests pass through Wasmtime in the
[2026-09-10 batch checkpoint](../temporal-baseline-follow-up.md), independently
of the historical evidence above.

The ownership invariant does not supply time-zone data, add another calendar,
prove general Temporal conformance or make result-variant selection a Rust type
proof. This contract publishes no conformance count and does not close T22.
