//! ListFormat retains a completed GC String List at its native byte boundary.
use super::super::*;
use super::intl::CanonicalLocaleListLocals;
use super::intl_number::*;
use super::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::gc_types::*;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::{
    IntlHostOp, ListFormatConfigurationWord, ListPartKind, ListStyle, ListType, LIST_WIRE_VERSION,
};
mod construction;
mod iterable;
mod pool;
mod render;
mod resolved;
use iterable::ObservedStringListLocals;
pub(crate) use pool::intl_list_format_pool_strings;
const LF_CONSTRUCT_ERROR: RuntimeErrorMessage = RuntimeErrorMessage::INTL_LISTFORMAT_REQUIRES_NEW;
const LF_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_LISTFORMAT_METHOD_REQUIRES_A_LISTFORMAT_RECEIVER;
const LF_STRING_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_LISTFORMAT_ITERABLE_VALUE_MUST_BE_A_STRING;
#[derive(Clone, Copy)]
pub(in crate::builtins) enum ListFormatOutput {
    String,
    Parts,
}
enum ListProviderRequest<'a> {
    Resolve {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Supported {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Parts {
        record: &'a GcLocal<IntlListFormatObject>,
        input: &'a ObservedStringListLocals,
    },
}
struct ListProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: IntlHostOp,
}
impl ListProviderResponse {
    fn reader(&self, schema: &RuntimeSchema, f: &mut Function) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, f);
        let word = schema.reserve_i64_local(f);
        for expected in [LIST_WIRE_VERSION, u64::from(self.operation.code()) * 2 + 1] {
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
    fn emit_list_record_from_receiver(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlListFormatObject>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| EmitError::unsupported("ListFormat receiver lacks callable entry"))?
                .this_value(),
            f,
        );
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlListFormatObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(LF_RECEIVER_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let record = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<IntlListFormatObject>(schema, f), f);
        value.clear(f);
        Ok(record)
    }
    fn emit_list_provider_call(
        &mut self,
        request: ListProviderRequest<'_>,
        f: &mut Function,
    ) -> Result<ListProviderResponse, EmitError> {
        let schema = self.runtime_schema();
        let operation = match &request {
            ListProviderRequest::Resolve { .. } => IntlHostOp::ResolveListLocale,
            ListProviderRequest::Supported { .. } => IntlHostOp::SupportedListLocales,
            ListProviderRequest::Parts { .. } => IntlHostOp::FormatListParts,
        };
        let message = IntlByteArrayBuilder::with_operation(operation, schema, f);
        message.append_u64_constant(LIST_WIRE_VERSION, schema, f);
        message.append_u64_constant(u64::from(operation.code()) * 2, schema, f);
        match request {
            ListProviderRequest::Resolve { locales, matcher }
            | ListProviderRequest::Supported { locales, matcher } => {
                self.emit_intl_wire_canonical_locales(&message, locales, f)?;
                let word = schema.reserve_i64_local(f);
                matcher.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
                schema.release_i64_local(word, f);
            }
            ListProviderRequest::Parts { record, input } => {
                let lf = schema.struct_type::<IntlListFormatObject>();
                let locale = schema.reserve_gc_local(f).initialize(
                    lf.field(IntlListFormatObjectSchema::LOCALE)
                        .read(record, schema, f)
                        .reference(),
                    f,
                );
                message.append_utf8(&locale, schema, f);
                locale.clear(f);
                let code = schema.reserve_i32_local(f);
                let word = schema.reserve_i64_local(f);
                for field in ListFormatConfigurationWord::ALL {
                    match field {
                        ListFormatConfigurationWord::Type => lf
                            .field(IntlListFormatObjectSchema::TYPE)
                            .read(record, schema, f)
                            .store(code, f),
                        ListFormatConfigurationWord::Style => lf
                            .field(IntlListFormatObjectSchema::STYLE)
                            .read(record, schema, f)
                            .store(code, f),
                    }
                    code.load(f);
                    f.instruction(&Instruction::I64ExtendI32U);
                    word.store(f);
                    message.append_u64(word, schema, f);
                }
                input.count().load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
                let index = schema.reserve_i32_local(f);
                set_i32(index, 0, f);
                let done = self.open_frame(ControlFrameKind::Block, f);
                let next = self.open_frame(ControlFrameKind::Loop, f);
                index.load(f);
                input.count().load(f);
                f.instruction(&Instruction::I32GeU);
                self.emit_branch_if_to_target(done, f);
                let value = schema.reserve_value_local(f);
                self.emit_argument_vector_entry_to_value(input.values(), index, &value, f);
                let text = schema
                    .reserve_gc_local(f)
                    .initialize(value.cast_reference::<StringValue>(schema, f), f);
                message.append_utf16(&text, schema, f);
                text.clear(f);
                value.clear(f);
                index.load(f);
                f.instruction(&Instruction::I32Const(1));
                f.instruction(&Instruction::I32Add);
                index.store(f);
                self.emit_branch_to_target(next, f);
                self.pop_control(ControlFrameKind::Loop);
                f.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::Block);
                f.instruction(&Instruction::End);
                schema.release_i32_local(index, f);
                schema.release_i64_local(word, f);
                schema.release_i32_local(code, f);
            }
        }
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
        Ok(ListProviderResponse {
            bytes: result,
            operation,
        })
    }
}
