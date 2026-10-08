//! DurationFormat preserves ordered observations and completed GC publication.
use super::super::*;
use super::intl::CanonicalLocaleListLocals;
use super::intl_number::*;
use super::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::functions::{ArgumentListConstruction, NonArrayRealmIntrinsicSlot};
use crate::gc_types::*;
use lila_intl::number_format::{options::LocaleMatcher, NumberPartKind};
use lila_intl::{
    DurationDisplay, DurationFractionalDigits, DurationHostOp, DurationStyle, DurationUnit,
    DurationUnitStyle, DURATION_WIRE_VERSION,
};
mod construction;
mod inputs;
mod options;
mod pool;
mod render;
mod resolved;
mod temporal;
pub(crate) use pool::intl_durationformat_pool_strings;
const DU_CONSTRUCT_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_DURATIONFORMAT_REQUIRES_NEW;
const DU_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_DURATIONFORMAT_METHOD_REQUIRES_A_DURATIONFORMAT_RECEIVER;
const DU_INPUT_ERROR: RuntimeErrorMessage = RuntimeErrorMessage::INTL_DURATIONFORMAT_DURATION_MUST_BE_AN_OBJECT_WITH_AT_LEAST_ONE_DURATION_FIELD;
const DU_FIELD_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_DURATIONFORMAT_DURATION_FIELD_MUST_BE_AN_INTEGER;
const DU_NATIVE_ERROR: &str = "Invalid Intl.DurationFormat duration";
/// Only a complete bag sequence, the intrinsic Temporal parser, or stored
/// Temporal brand fields can publish these integral Number bits.
struct CompletedDurationFormatInputsLocals {
    number_bits: [I64Local; 10],
}
impl CompletedDurationFormatInputsLocals {
    fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        for local in self.number_bits.into_iter().rev() {
            schema.release_i64_local(local, f);
        }
    }
}
#[derive(Clone, Copy)]
enum DurationFormatOutput {
    String,
    Parts,
}
enum DurationProviderRequest<'a> {
    Resolve {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
        numbering: &'a ValueLocals,
    },
    Supported {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Parts {
        record: &'a GcLocal<IntlDurationFormatObject>,
        input: &'a CompletedDurationFormatInputsLocals,
    },
}
struct DurationProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: DurationHostOp,
}
impl DurationProviderResponse {
    fn reader(&self, schema: &RuntimeSchema, f: &mut Function) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, f);
        let word = schema.reserve_i64_local(f);
        for expected in [
            DURATION_WIRE_VERSION,
            u64::from(self.operation.proposed_global_tag()) * 2 + 1,
        ] {
            reader.read_u64(word, schema, f);
            word.load(f);
            f.instruction(&Instruction::I64Const(expected as i64));
            f.instruction(&Instruction::I64Ne);
            f.instruction(&Instruction::If(BlockType::Empty));
            f.instruction(&Instruction::Unreachable);
            f.instruction(&Instruction::End);
        }
        schema.release_i64_local(word, f);
        reader
    }
    fn clear(self, f: &mut Function) {
        self.bytes.clear(f);
    }
}
impl FunctionBuilder<'_> {
    fn emit_duration_record_from_receiver(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlDurationFormatObject>, EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        receiver.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("DurationFormat receiver lacks callable entry")
                })?
                .this_value(),
            f,
        );
        receiver.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlDurationFormatObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(DU_RECEIVER_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let record = schema.reserve_gc_local(f).initialize(
            receiver.cast_reference::<IntlDurationFormatObject>(schema, f),
            f,
        );
        receiver.clear(f);
        Ok(record)
    }
    fn emit_duration_unit_record(
        &self,
        record: &GcLocal<IntlDurationFormatObject>,
        unit: DurationUnit,
        f: &mut Function,
    ) -> GcLocal<IntlDurationUnitOptions> {
        let schema = self.runtime_schema();
        let field = match unit {
            DurationUnit::Year => IntlDurationFormatObjectSchema::YEARS,
            DurationUnit::Month => IntlDurationFormatObjectSchema::MONTHS,
            DurationUnit::Week => IntlDurationFormatObjectSchema::WEEKS,
            DurationUnit::Day => IntlDurationFormatObjectSchema::DAYS,
            DurationUnit::Hour => IntlDurationFormatObjectSchema::HOURS,
            DurationUnit::Minute => IntlDurationFormatObjectSchema::MINUTES,
            DurationUnit::Second => IntlDurationFormatObjectSchema::SECONDS,
            DurationUnit::Millisecond => IntlDurationFormatObjectSchema::MILLISECONDS,
            DurationUnit::Microsecond => IntlDurationFormatObjectSchema::MICROSECONDS,
            DurationUnit::Nanosecond => IntlDurationFormatObjectSchema::NANOSECONDS,
        };
        schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<IntlDurationFormatObject>()
                .field(field)
                .read(record, schema, f)
                .reference(),
            f,
        )
    }
    fn emit_duration_provider_call(
        &mut self,
        request: DurationProviderRequest<'_>,
        f: &mut Function,
    ) -> Result<DurationProviderResponse, EmitError> {
        let schema = self.runtime_schema();
        let operation = match &request {
            DurationProviderRequest::Resolve { .. } => DurationHostOp::Resolve,
            DurationProviderRequest::Supported { .. } => DurationHostOp::SupportedLocales,
            DurationProviderRequest::Parts { .. } => DurationHostOp::Parts,
        };
        let message = IntlByteArrayBuilder::with_operation(operation.global_operation(), schema, f);
        message.append_u64_constant(DURATION_WIRE_VERSION, schema, f);
        message.append_u64_constant(u64::from(operation.proposed_global_tag()) * 2, schema, f);
        let word = schema.reserve_i64_local(f);
        match request {
            DurationProviderRequest::Resolve {
                locales,
                matcher,
                numbering,
            } => {
                self.emit_intl_wire_canonical_locales(&message, locales, f)?;
                matcher.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
                emit_tag_is(numbering, WasmRuntimeValueTag::Undefined, f);
                self.open_frame(ControlFrameKind::If, f);
                message.append_u64_constant(0, schema, f);
                f.instruction(&Instruction::Else);
                message.append_u64_constant(1, schema, f);
                let string = schema
                    .reserve_gc_local(f)
                    .initialize(numbering.cast_reference::<StringValue>(schema, f), f);
                message.append_utf8(&string, schema, f);
                string.clear(f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            DurationProviderRequest::Supported { locales, matcher } => {
                self.emit_intl_wire_canonical_locales(&message, locales, f)?;
                matcher.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
            }
            DurationProviderRequest::Parts { record, input } => {
                let st = schema.struct_type::<IntlDurationFormatObject>();
                for field in [
                    IntlDurationFormatObjectSchema::LOCALE,
                    IntlDurationFormatObjectSchema::NUMBERING_SYSTEM,
                ] {
                    let string = schema
                        .reserve_gc_local(f)
                        .initialize(st.field(field).read(record, schema, f).reference(), f);
                    message.append_utf8(&string, schema, f);
                    string.clear(f);
                }
                let scalar = schema.reserve_i32_local(f);
                st.field(IntlDurationFormatObjectSchema::STYLE)
                    .read(record, schema, f)
                    .store(scalar, f);
                scalar.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
                // The private GC optional encoding is -1/0..9; native wire is 0/1..10.
                st.field(IntlDurationFormatObjectSchema::FRACTIONAL_DIGITS)
                    .read(record, schema, f)
                    .store(scalar, f);
                scalar.load(f);
                f.instruction(&Instruction::I64ExtendI32S);
                f.instruction(&Instruction::I64Const(1));
                f.instruction(&Instruction::I64Add);
                word.store(f);
                message.append_u64(word, schema, f);
                let display = schema.reserve_i32_local(f);
                for &unit in DurationUnit::ALL {
                    let row = self.emit_duration_unit_record(record, unit, f);
                    let row_type = schema.struct_type::<IntlDurationUnitOptions>();
                    row_type
                        .field(IntlDurationUnitOptionsSchema::STYLE)
                        .read(&row, schema, f)
                        .store(scalar, f);
                    row_type
                        .field(IntlDurationUnitOptionsSchema::DISPLAY)
                        .read(&row, schema, f)
                        .store(display, f);
                    scalar.load(f);
                    f.instruction(&Instruction::I64ExtendI32U);
                    display.load(f);
                    f.instruction(&Instruction::I64ExtendI32U);
                    f.instruction(&Instruction::I64Const(8));
                    f.instruction(&Instruction::I64Shl);
                    f.instruction(&Instruction::I64Or);
                    word.store(f);
                    message.append_u64(word, schema, f);
                    row.clear(f);
                }
                schema.release_i32_local(display, f);
                schema.release_i32_local(scalar, f);
                for &bits in &input.number_bits {
                    message.append_u64(bits, schema, f);
                }
            }
        }
        schema.release_i64_local(word, f);
        let bytes = message.finish(schema, f);
        let reply = self.emit_intl_provider_byte_call(&bytes, f)?;
        reply.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        match operation {
            DurationHostOp::Parts => self.emit_intl_number_range_error(
                RuntimeErrorMessage::INVALID_INTL_DURATIONFORMAT_DURATION,
                f,
            )?,
            DurationHostOp::Resolve | DurationHostOp::SupportedLocales => {
                f.instruction(&Instruction::Unreachable);
            }
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let output = schema
            .reserve_gc_local(f)
            .initialize(reply.load(schema, f).require_non_null(f), f);
        reply.clear(f);
        bytes.clear(f);
        Ok(DurationProviderResponse {
            bytes: output,
            operation,
        })
    }
}
