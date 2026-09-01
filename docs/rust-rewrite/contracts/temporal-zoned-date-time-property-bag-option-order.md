# Temporal.ZonedDateTime property-bag option order

Status: implemented and focused verification complete on 2026-08-29.

## Boundary

The property-bag branch of `Temporal.ZonedDateTime.from` performs its final
field conversion for `year`, then reads and string-coerces the three options in
this order:

1. `disambiguation`
2. `offset`
3. `overflow`

The required `timeZone` check and conversion run at that field's sorted read
position, before the final `year` conversion and before options. Only after the
option reads does the path resolve the calendar era and reject missing `year`
or `day`.

The move keeps the existing `ZonedDateTimeOptionKey` authority and
`emit_temporal_zoned_date_time_options` consumer intact. That helper reserves
five scratch locals and releases them in reverse order before returning. The
property-bag emitter keeps its non-copyable `TemporalEraLocals` alive across the
option read, then moves it into `emit_temporal_resolve_era_to_year`, whose
destructuring and reverse releases remain the only consuming path.

## Source invariant

`temporal_zoned_date_time_property_bag_option_order_structure.rs` pins:

- one option-reader call after the complete `year` conversion and before era
  resolution;
- the required `timeZone` check and conversion after its Get but before `year`
  and options;
- the era-field read before options and era resolution before the required
  `year` and `day` failures;
- the option reader's exact five scratch reservations and reverse releases;
  and
- the non-`Clone`, non-`Copy` era witness staying live across the option reader
  before the resolver consumes and releases it.

## Runtime evidence

`wasm_temporal_zoned_date_time_era.js` reuses one observed options factory for
three property bags. Every case requires this exact nine-event log:

```text
get disambiguation|get disambiguation.toString|call disambiguation.toString|get offset|get offset.toString|call offset.toString|get overflow|get overflow.toString|call overflow.toString
```

The conversion hooks return the valid spellings `compatible`, `reject` and
`constrain`. The complete log must exist before each later failure:

| Property bag | Later failure |
| --- | --- |
| invalid gregory `era: "xyz"` with `eraYear` and `year` | `RangeError` |
| neither `year` nor an `era`/`eraYear` pair | `TypeError` |
| valid `year` with no `day` | `TypeError` |

A missing `timeZone` and an invalid present time-zone object each observe only
the `timeZone` getter. Neither reaches the later `year` getter or any option
getter, pinning both required-field validation and conversion at the sorted
field position.

The fixture prints
`temporal-zdt-option-order:invalid-era|missing-year|missing-day|missing-time-zone|invalid-time-zone`, and its
existing CLI test requires that marker in the Wasm-AOT output.

## Verification

```sh
cargo test -p lila-aot-wasm --test temporal_zoned_date_time_property_bag_option_order_structure
cargo test -p lila-cli --test cli run_wasm_backend_succeeds_for_temporal_zoned_date_time_era_fixture -- --test-threads=1
```

The structure target passes `3/3` and the CLI fixture passes `1/1` through the
Wasm-AOT backend. `node --check` accepts the fixture, `cargo fmt --all --
--check` is clean and `cargo check -p lila-aot-wasm` is green with only the
pre-existing vendored parser warning.

## Nonclaims

This boundary does not change the field-read order that precedes the options,
the string or branded `ZonedDateTime` conversion branches, option validation,
fixed-offset time-zone resolution or custom calendar and time-zone protocols.
The runtime cases do not claim abrupt option completion coverage or complete
time-zone identifier semantics. This change does not complete Temporal or T22
and does not change a published conformance count.
