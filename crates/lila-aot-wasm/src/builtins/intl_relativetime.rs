//! RelativeTimeFormat publishes only completed GC constructor and input owners.
use super::super::*;
use super::intl::CanonicalLocaleListLocals;
use super::intl_number::*;
use super::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::gc_types::*;
use lila_intl::number_format::{options::LocaleMatcher, NumberPartKind};
use lila_intl::{
    IntlHostOp, RelativeNumeric, RelativeStyle, RelativeTimeConfigurationWord as RtWord,
    RelativeUnit, RELATIVE_WIRE_VERSION,
};
mod construction;
mod pool;
mod render;
mod resolved;
pub(crate) use pool::intl_relative_time_pool_strings;
const RT_CONSTRUCT_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_RELATIVETIMEFORMAT_REQUIRES_NEW;
const RT_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_RELATIVETIMEFORMAT_METHOD_REQUIRES_A_RELATIVETIMEFORMAT_RECEIVER;
const RT_FINITE_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_RELATIVETIMEFORMAT_VALUE_MUST_BE_FINITE;
const RT_UNIT_ERROR: RuntimeErrorMessage = RuntimeErrorMessage::INVALID_RELATIVETIMEFORMAT_UNIT;
/// Only completed ToNumber/ToString and both checks can mint these inputs.
struct CompletedRelativeTimeFormatInputsLocals {
    value_bits: I64Local,
    unit: GcI32DomainLocal<RelativeUnit>,
}
impl CompletedRelativeTimeFormatInputsLocals {
    fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        self.unit.clear(schema, f);
        schema.release_i64_local(self.value_bits, f);
    }
}
#[derive(Clone, Copy)]
pub(in crate::builtins) enum RelativeTimeFormatOutput {
    String,
    Parts,
}
enum RelativeProviderRequest<'a> {
    Resolve {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
        numbering: &'a GcLocal<StringValue>,
    },
    Supported {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Parts {
        record: &'a GcLocal<IntlRelativeTimeFormatObject>,
        input: &'a CompletedRelativeTimeFormatInputsLocals,
    },
}
struct RelativeProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: IntlHostOp,
}
impl RelativeProviderResponse {
    fn reader(&self, schema: &RuntimeSchema, f: &mut Function) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, f);
        let word = schema.reserve_i64_local(f);
        for expected in [
            RELATIVE_WIRE_VERSION,
            u64::from(self.operation.code()) * 2 + 1,
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
    fn emit_relative_record_from_receiver(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlRelativeTimeFormatObject>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("RelativeTimeFormat receiver lacks callable entry")
                })?
                .this_value(),
            f,
        );
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlRelativeTimeFormatObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(RT_RECEIVER_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let record = schema.reserve_gc_local(f).initialize(
            value.cast_reference::<IntlRelativeTimeFormatObject>(schema, f),
            f,
        );
        value.clear(f);
        Ok(record)
    }
    fn emit_relative_provider_call(
        &mut self,
        request: RelativeProviderRequest<'_>,
        f: &mut Function,
    ) -> Result<RelativeProviderResponse, EmitError> {
        let schema = self.runtime_schema();
        let operation = match &request {
            RelativeProviderRequest::Resolve { .. } => IntlHostOp::ResolveRelativeTimeLocale,
            RelativeProviderRequest::Supported { .. } => IntlHostOp::SupportedRelativeTimeLocales,
            RelativeProviderRequest::Parts { .. } => IntlHostOp::FormatRelativeTimeParts,
        };
        let message = IntlByteArrayBuilder::with_operation(operation, schema, f);
        message.append_u64_constant(RELATIVE_WIRE_VERSION, schema, f);
        message.append_u64_constant(u64::from(operation.code()) * 2, schema, f);
        let word = schema.reserve_i64_local(f);
        match request {
            RelativeProviderRequest::Resolve {
                locales,
                matcher,
                numbering,
            } => {
                self.emit_intl_wire_canonical_locales(&message, locales, f)?;
                matcher.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
                let units = schema.reserve_gc_local(f).initialize(
                    schema
                        .struct_type::<StringValue>()
                        .field(StringValueSchema::CODE_UNITS)
                        .read(numbering, schema, f)
                        .reference(),
                    f,
                );
                schema
                    .array_type::<CodeUnitArray>()
                    .length(&units, schema, f);
                f.instruction(&Instruction::I32Eqz);
                f.instruction(&Instruction::I32Eqz);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                units.clear(f);
                message.append_u64(word, schema, f);
                word.load(f);
                f.instruction(&Instruction::I64Eqz);
                f.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, f);
                message.append_utf8(numbering, schema, f);
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            RelativeProviderRequest::Supported { locales, matcher } => {
                self.emit_intl_wire_canonical_locales(&message, locales, f)?;
                matcher.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
            }
            RelativeProviderRequest::Parts { record, input } => {
                let rt = schema.struct_type::<IntlRelativeTimeFormatObject>();
                for field in [
                    IntlRelativeTimeFormatObjectSchema::LOCALE,
                    IntlRelativeTimeFormatObjectSchema::FORMATTING_LOCALE,
                    IntlRelativeTimeFormatObjectSchema::NUMBERING_SYSTEM,
                ] {
                    let text = schema
                        .reserve_gc_local(f)
                        .initialize(rt.field(field).read(record, schema, f).reference(), f);
                    message.append_utf8(&text, schema, f);
                    text.clear(f);
                }
                let code = schema.reserve_i32_local(f);
                for field in RtWord::ALL {
                    match field {
                        RtWord::Style => rt
                            .field(IntlRelativeTimeFormatObjectSchema::STYLE)
                            .read(record, schema, f)
                            .store(code, f),
                        RtWord::Numeric => rt
                            .field(IntlRelativeTimeFormatObjectSchema::NUMERIC)
                            .read(record, schema, f)
                            .store(code, f),
                    }
                    code.load(f);
                    f.instruction(&Instruction::I64ExtendI32U);
                    word.store(f);
                    message.append_u64(word, schema, f);
                }
                schema.release_i32_local(code, f);
                message.append_u64(input.value_bits, schema, f);
                input.unit.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
            }
        }
        schema.release_i64_local(word, f);
        let bytes = message.finish(schema, f);
        let reply = self.emit_intl_provider_byte_call(&bytes, f)?;
        reply.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let result = schema
            .reserve_gc_local(f)
            .initialize(reply.load(schema, f).require_non_null(f), f);
        reply.clear(f);
        bytes.clear(f);
        Ok(RelativeProviderResponse {
            bytes: result,
            operation,
        })
    }
}
