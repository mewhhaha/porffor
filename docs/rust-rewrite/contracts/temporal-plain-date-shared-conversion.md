# Shared PlainDate conversion

Status: source authored; the joined formatting, type, emitted-size and runtime
checkpoint has not run for this change.

`TemporalPlainDateConvert` emits the complete `ToTemporalDate` conversion once.
The public builtin emitter and Duration's `relativeTo` conversion call that
typed helper through `emit_temporal_to_temporal_date`. Its physical algorithm
is private to `temporal_plain_date_methods/convert.rs` and is called only by the
helper compiler.

The call transports the input, the actual called `FunctionContext`, the caller
Environment and the closed overflow-options choice. The helper retains that
context while nested property reads or coercions run. Intrinsic prototypes and
conversion errors therefore continue to use the called function's Realm,
including after a user hook calls a function from another Realm. The converter
does not read a public Temporal constructor or infer a Realm from the current
job.

The existing `TemporalConversionOverflowOptions::Read(&ValueLocals)` and
`Omit` boundary remains intact. The registered `OptionalValue` operand carries
presence separately from the value: present `undefined` follows the original
option-validation path, while omission skips that path. Its private decoded
parameter exposes the value only through `emit_if_present`. The original five
overflow-read locations remain in the complete conversion body; callers do not
allocate an undefined placeholder or choose between duplicated bodies.

Branded PlainDate, PlainDateTime and ZonedDateTime paths precede property-bag
conversion. A property bag still obtains its calendar before the ordered field
sweep, validates month-code syntax before reading the year, then reads overflow
before calendar resolution. String parsing and range checks retain their
original order. Abrupt property reads and coercions preserve the whole thrown
value.

The helper returns a whole Completion. After every conversion and option check
succeeds, it allocates one temporary genuine `TemporalPlainDateObject` with the
called Realm's intrinsic prototype to transport the checked ISO fields and
calendar. This adds one helper-local allocation to a successful conversion and
uses the existing GC layout. The caller projects that record only under a
Normal completion and promptly clears both record and pending Completion roots.
On Throw it copies the whole Completion and uses the existing caller throw
propagation, preserving any enclosing handler. The helper releases its retained
FunctionContext at the normal epilogue; an early Return drops the Wasm frame.

The focused source controls are
`temporal_conversion_overflow_options_structure` and
`temporal_date_field_read_mode_structure`. The extended Duration observation
fixture checks slot fast paths, real option reads, field order, month-code
failure order and abrupt identity. The created-Realm test
`shared_plain_date_conversion_keeps_called_realm_slots_order_and_errors`
checks foreign result/error prototypes, public constructor replacement, nested
hooks, present-undefined options and borrowed Duration conversion. These
controls are authored and unrun. The shared calendar size control owns unique
helper emission, consumed calls and the unchanged body limits.
