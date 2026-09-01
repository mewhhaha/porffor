# Temporal plain difference operation

Status: normative for the shared `until` / `since` dispatch of the four plain
Temporal types.

## Boundary

`Temporal.PlainDate`, `Temporal.PlainYearMonth`, `Temporal.PlainTime` and
`Temporal.PlainDateTime` each have one shared difference emitter for their
`until` and `since` prototype methods. PlainDateTime already named the
direction, but the other three emitters received six raw Boolean arguments.
Those Booleans controlled two correlated decisions inside every emitter:

| Operation | Rounding mode | Final duration |
| --- | --- | --- |
| `Until` | Use the requested mode directly | Preserve the computed sign |
| `Since` | Negate the requested mode | Negate the computed result |

All eight standard-builtin producers now construct only
`TemporalPlainDifferenceOperation::{Until, Since}`. Each of the four emitters
matches the operation exhaustively for both decisions. The domain is visible
only within the builtin module tree and has no default, wildcard or Boolean
projection.

PlainDateTime maps the operation exhaustively into its existing difference
settings plan. `TemporalDateTimeDifferenceSettingsPlan::ZonedDelegate` remains
a distinct internal state: ZonedDateTime passes an unnegated normalized mode to
the selected PlainDateTime builtin, which still owns the final operation
direction. Folding that transport state into `Since` would negate the mode
twice.

## Observable witness

`wasm_temporal_plain_difference_operation.js` executes all eight producers with
`roundingMode: "ceil"`. Its vectors are deliberately non-integral at the
selected unit, so a wrong rounding mapping changes the magnitude as well as the
sign:

- PlainDate, PlainYearMonth and PlainDateTime return `3` years from `until` and
  `-2` years from `since`;
- PlainTime returns `5` hours from `until` and `-4` hours from `since`.

The values come from the pinned Test262 `roundingmode-ceil.js` witnesses for
both methods on all four receiver families. Every assertion identifies the
receiver and operation whose producer mapping failed.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test temporal_plain_difference_operation_structure
cargo test -p lila-aot-wasm --test temporal_zoned_date_time_difference_defaults_structure
cargo test -p lila-aot-wasm --test temporal_plain_arithmetic_operation_structure
cargo test -p lila-cli --test cli date::run_wasm_backend_distinguishes_plain_temporal_until_and_since -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
cargo fmt --all -- --check
git diff --check
```

The bounded source target owns the exact two-variant domain, four typed
consumers, two exhaustive decisions per consumer, exact four-plus-four producer
census and absence of raw operation Booleans. The two neighboring structure
targets retain the ZonedDateTime settings-plan contract and the arithmetic
emitter boundaries after the shared type rename.

The new structure target passes `3/3`; the updated ZonedDateTime and arithmetic
targets pass `5/5` and `3/3`; and the exact CLI witness passes `1/1`. The module
boundary policy, scoped Rust and fixture formatting, scoped diff check and
`cargo xc` are green.

The following shared semantic golden passes `2/2` in 697.36 seconds and
contains 671 dumps. Relative to the preceding 669-dump checkpoint it adds only
this fixture and the independent `Array.fromAsync` Promise-Realm fixture,
removes none and leaves all 669 retained dumps equal after accounting
normalization. This confirms the intended source-equivalent retained output.
No Test262 tree was run.

## Deferrals

This closure does not change Temporal difference arithmetic, option access
order, calendar or time-zone semantics. It does not merge ZonedDateTime's
operation domain with the plain domain, type the remaining Temporal conversion
field-reader policies, run the broad Date/Temporal ladder or publish conformance
status.
