# Temporal date field-read mode

Status: normative for the shared PlainDate/PlainMonthDay property-bag field reader.

## Boundary

`FunctionBuilder::emit_temporal_plain_date_read_fields` implements the common
`PrepareCalendarFields` sweep used by full PlainDate and PlainMonthDay
conversion and by both classes' `with` methods. The named mode identifies the
producer and whether the sweep reads `calendar`. Every producer applies
`ToMonthCode` during field preparation: coerce with a String hint, require a
String primitive, then validate syntax before reading later fields. Calendar
suitability is resolved after field preparation and the overflow option.

The reader now accepts only `TemporalDateFieldReadMode`:

| Mode | Producer | Calendar in the shared sweep |
| --- | --- | --- |
| `DateConversion` | `ToTemporalDate` property-bag path | Read and canonicalize |
| `DateWith` | `Temporal.PlainDate.prototype.with` | Skip |
| `MonthDayConversion` | `ToTemporalMonthDay` property-bag path | Read and canonicalize |
| `MonthDayWith` | `Temporal.PlainMonthDay.prototype.with` | Skip |

The calendar decision is a direct exhaustive match. The mode is never
projected to a Boolean, has no default, and has no wildcard arm. The reader is
visible only to sibling builtin modules, matching its exact two-file ownership.
Month-code validation is unconditional and shares the existing authority with
PlainYearMonth and PlainDateTime.

## Observable witness

`wasm_temporal_date_field_read_modes.js` executes each mode once with a Proxy
field bag and the syntactically invalid month code `"L99M"`.

- PlainDate conversion reads `calendar` once and rejects the month code before
  reading `options.overflow`.
- PlainDate `with` performs no `calendar` `Get` and rejects the month code
  before reading `options.overflow`. Its earlier forbidden-field check is an
  own-property query, not a field-sweep `Get`.
- PlainMonthDay conversion reads `calendar` once and rejects the month code
  before `options.overflow` can be read.
- PlainMonthDay `with` performs the one `calendar` `Get` required by
  `RejectTemporalLikeObject`, skips a second field-sweep read, and rejects the
  month code before `options.overflow`.

Additional PlainDate conversion and `with` controls use syntactically valid
`"M99L"`: its ISO calendar unsuitability is rejected after `options.overflow`.
The native `aot_temporal_month_code` target covers Date and DateTime conversion
and `with`, primitive type rejection, boxed strings, coercion order and thrown
value identity, and syntax-versus-suitability order relative to the year field.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test temporal_date_field_read_mode_structure
cargo test -p lila-engine --test aot_temporal_month_code -- --test-threads=1
cargo test -p lila-cli --test cli date::run_wasm_backend_preserves_temporal_date_field_read_modes -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
cargo fmt --all -- --check
git diff --check
```

The structure target pins the exact four variants, exhaustive calendar
projection, shared month-code validation, the absence of raw Boolean policy at
the reader boundary, and the exact four producers.

The preceding field-mode checkpoint passed the consolidated semantic golden
`2/2` in 707.34 seconds with 664 dumps. That historical run predates the
month-code correction and is not verification of the updated ordering.

## Deferrals

This boundary does not consolidate the duplicated
PlainMonthDay reference-year constants, type other Temporal field readers, add
calendar or time-zone protocols, retire Test262 materialization shortcuts, run
broad Temporal/Test262 trees, or publish conformance status.
