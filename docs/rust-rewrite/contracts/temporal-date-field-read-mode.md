# Temporal date field-read mode

Status: normative for the shared PlainDate/PlainMonthDay property-bag field reader.

## Boundary

`FunctionBuilder::emit_temporal_plain_date_read_fields` implements the common
`PrepareCalendarFields` sweep used by full PlainDate and PlainMonthDay
conversion and by both classes' `with` methods. Two independent positional
Booleans previously selected whether that sweep read `calendar` and whether it
validated `monthCode` syntax immediately. Every Boolean combination happened
to be live, but nothing at the call boundary named the combined specification
mode; transposing the arguments compiled and changed observable property and
option access order.

The reader now accepts only `TemporalDateFieldReadMode`:

| Mode | Producer | Calendar in the shared sweep | `monthCode` handling |
| --- | --- | --- | --- |
| `DateConversion` | `ToTemporalDate` property-bag path | Read and canonicalize | Convert to String; suitability is checked by `CalendarResolveFields` after the overflow option |
| `DateWith` | `Temporal.PlainDate.prototype.with` | Skip | Convert to String; suitability is checked after the overflow option |
| `MonthDayConversion` | `ToTemporalMonthDay` property-bag path | Read and canonicalize | Perform `ToMonthCode` syntax validation during the field sweep |
| `MonthDayWith` | `Temporal.PlainMonthDay.prototype.with` | Skip | Perform `ToMonthCode` syntax validation during the field sweep |

Both decisions are direct exhaustive matches. The mode is never projected to
a Boolean, has no default, and has no wildcard arm. The reader is visible only
to sibling builtin modules, matching its exact two-file ownership.

## Observable witness

`wasm_temporal_date_field_read_modes.js` executes each mode once with a Proxy
field bag and the syntactically invalid month code `"L99M"`.

- PlainDate conversion reads `calendar` once and then reads `options.overflow`
  before `CalendarResolveFields` rejects the month code.
- PlainDate `with` performs no `calendar` `Get` and reads `options.overflow`
  before rejecting the month code. Its earlier forbidden-field check is an
  own-property query, not a field-sweep `Get`.
- PlainMonthDay conversion reads `calendar` once and rejects the month code
  before `options.overflow` can be read.
- PlainMonthDay `with` performs the one `calendar` `Get` required by
  `RejectTemporalLikeObject`, skips a second field-sweep read, and rejects the
  month code before `options.overflow`.

Those counts distinguish all four modes: changing either exhaustive mapping
makes the fixture throw even though every branch still ends in a RangeError.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test temporal_date_field_read_mode_structure
cargo test -p lila-cli --test cli date::run_wasm_backend_preserves_temporal_date_field_read_modes -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
cargo fmt --all -- --check
git diff --check
```

The structure target pins the exact four variants, both exhaustive projections,
the absence of raw Boolean policy at the reader boundary, and the exact four
producers.

The consolidated semantic golden passes `2/2` in 707.34 seconds and contains
664 dumps. This fixture is its only addition to the preceding checkpoint, and
no fixture is removed.

## Deferrals

This source-equivalent type closure does not consolidate the duplicated
PlainMonthDay reference-year constants, type other Temporal field readers, add
calendar or time-zone protocols, retire Test262 materialization shortcuts, run
broad Temporal/Test262 trees, or publish conformance status.
