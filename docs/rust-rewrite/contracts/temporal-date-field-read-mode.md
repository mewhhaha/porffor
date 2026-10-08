# Temporal date field-read ownership

Status: normative for the shared PlainDate/PlainMonthDay property-bag field
reader. Current retargeted source controls await the joined checkpoint.

## Boundary

`FunctionBuilder::emit_temporal_plain_date_read_fields` implements the common
`PrepareCalendarFields` sweep used by full PlainDate and PlainMonthDay
conversion and by both classes' `with` methods. Each producer obtains its actual
checked calendar before passing it to the field reader. Every producer applies
`ToMonthCode` during field preparation: coerce with a String hint, require a
String primitive, then validate syntax before reading later fields. Calendar
suitability is resolved after field preparation and the overflow option.

The reader accepts `&TemporalCalendarSlotLocals`:

| Producer | Calendar ownership |
| --- | --- |
| `ToTemporalDate` property-bag path | Read and canonicalize before field preparation |
| `Temporal.PlainDate.prototype.with` | Use the receiver's checked calendar |
| `ToTemporalMonthDay` property-bag path | Read and canonicalize before field preparation |
| `Temporal.PlainMonthDay.prototype.with` | Use the receiver's checked calendar |

The reader does not choose whether to fetch a calendar and accepts no raw
Boolean policy or obsolete producer-mode enum. It uses the supplied calendar's
actual kernel identity to guard era fields. Month-code validation is
unconditional and shares the existing authority with PlainYearMonth and
PlainDateTime. PlainDate conversion now lives in its consumed private helper
child; the field reader remains the single common implementation. See
[Shared PlainDate conversion](temporal-plain-date-shared-conversion.md).

## Observable witness

`wasm_temporal_date_field_read_modes.js` executes each producer once with a Proxy
field bag and the syntactically invalid month code `"L99M"`.

- PlainDate conversion reads `calendar` once and rejects the month code before
  reading `options.overflow`.
- PlainDate `with` performs the one `calendar` `Get` required by
  `IsPartialTemporalObject`, skips a second field-sweep read, and rejects the
  month code before reading `options.overflow`.
- PlainMonthDay conversion reads `calendar` once and rejects the month code
  before `options.overflow` can be read.
- PlainMonthDay `with` performs the one `calendar` `Get` required by
  `IsPartialTemporalObject`, skips a second field-sweep read, and rejects the
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
cargo test -p lila-cli --test cli date::run_wasm_backend_preserves_temporal_date_field_read_modes -- --exact --test-threads=8
./scripts/check-module-boundaries.sh
cargo fmt --all -- --check
git diff --check
```

The structure target checks the actual checked-calendar parameter, guarded era
reads, shared month-code validation, field acquisition order and all four
producer calls. It also checks the PlainDate helper's field-to-overflow-to-
resolution order. The current retargeted target is unrun.

The preceding field-mode checkpoint passed the consolidated semantic golden
`2/2` in 707.34 seconds with 664 dumps. That historical run predates the
month-code correction and is not verification of the updated ordering.

## Deferrals

This boundary does not consolidate the duplicated
PlainMonthDay reference-year constants, type other Temporal field readers, add
calendar or time-zone protocols, retire Test262 materialization shortcuts, run
broad Temporal/Test262 trees, or publish conformance status.
