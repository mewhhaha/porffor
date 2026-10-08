//! Collator's ordered JavaScript observations and completed native GC record.
use super::super::*;
use super::intl::CanonicalLocaleListLocals;
use super::intl_number::*;
use super::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::gc_types::*;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::{
    CollatorCaseFirst, CollatorCollationKind, CollatorOrdering, CollatorSensitivity, CollatorUsage,
    IntlHostOp, COLLATOR_WIRE_VERSION,
};
mod compare;
mod construction;
mod options;
mod pool;
mod resolved;
use compare::CompletedCollatorCompareStringsLocals;
use options::ObservedCollatorResolutionInputsLocals;
pub(crate) use pool::intl_collator_pool_strings;
const CO_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_COLLATOR_METHOD_REQUIRES_A_COLLATOR_RECEIVER;

enum CollatorProviderRequest<'a> {
    Resolve(&'a ObservedCollatorResolutionInputsLocals),
    Supported {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Compare {
        record: &'a GcLocal<IntlCollatorObject>,
        input: &'a CompletedCollatorCompareStringsLocals,
    },
}
struct CollatorProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: IntlHostOp,
}
impl CollatorProviderResponse {
    fn reader(&self, schema: &RuntimeSchema, function: &mut Function) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, function);
        let word = schema.reserve_i64_local(function);
        for expected in [
            COLLATOR_WIRE_VERSION,
            u64::from(self.operation.code()) * 2 + 1,
        ] {
            reader.read_u64(word, schema, function);
            word.load(function);
            function.instruction(&Instruction::I64Const(expected as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        schema.release_i64_local(word, function);
        reader
    }
    fn clear(self, function: &mut Function) {
        self.bytes.clear(function);
    }
}
fn collator_append_domain<V: GcI32Constant>(
    message: &IntlByteArrayBuilder,
    domain: &GcI32DomainLocal<V>,
    schema: &RuntimeSchema,
    function: &mut Function,
) {
    let word = schema.reserve_i64_local(function);
    domain.load(function);
    function.instruction(&Instruction::I64ExtendI32U);
    word.store(function);
    message.append_u64(word, schema, function);
    schema.release_i64_local(word, function);
}
fn collator_append_boolean(
    message: &IntlByteArrayBuilder,
    flag: I32Local,
    schema: &RuntimeSchema,
    function: &mut Function,
) {
    let word = schema.reserve_i64_local(function);
    flag.load(function);
    function.instruction(&Instruction::I64ExtendI32U);
    word.store(function);
    message.append_u64(word, schema, function);
    schema.release_i64_local(word, function);
}
impl FunctionBuilder<'_> {
    fn emit_collator_record_from_receiver(
        &mut self,
        function: &mut Function,
    ) -> Result<GcLocal<IntlCollatorObject>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        value.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("Collator receiver lacks actual callable entry")
                })?
                .this_value(),
            function,
        );
        value.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlCollatorObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(CO_RECEIVER_ERROR, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let record = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<IntlCollatorObject>(schema, function),
            function,
        );
        value.clear(function);
        Ok(record)
    }
    fn emit_collator_read_domain<V: GcI32Constant + Copy>(
        &self,
        reader: &IntlByteArrayReader<'_>,
        out: &GcI32DomainLocal<V>,
        variants: impl IntoIterator<Item = (V, u64)>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let word = schema.reserve_i64_local(function);
        let recognized = schema.reserve_i32_local(function);
        reader.read_u64(word, schema, function);
        set_i32(recognized, 0, function);
        for (variant, code) in variants {
            word.load(function);
            function.instruction(&Instruction::I64Const(code as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            out.set_constant(variant, function);
            set_i32(recognized, 1, function);
            function.instruction(&Instruction::End);
        }
        recognized.load(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        schema.release_i32_local(recognized, function);
        schema.release_i64_local(word, function);
    }
    fn emit_collator_read_boolean(
        &self,
        reader: &IntlByteArrayReader<'_>,
        out: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let word = schema.reserve_i64_local(function);
        reader.read_u64(word, schema, function);
        word.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        word.load(function);
        function.instruction(&Instruction::I32WrapI64);
        out.store(function);
        schema.release_i64_local(word, function);
    }
    fn emit_collator_provider_call(
        &mut self,
        request: CollatorProviderRequest<'_>,
        function: &mut Function,
    ) -> Result<CollatorProviderResponse, EmitError> {
        let schema = self.runtime_schema();
        let operation = match &request {
            CollatorProviderRequest::Resolve(_) => IntlHostOp::ResolveCollatorLocale,
            CollatorProviderRequest::Supported { .. } => IntlHostOp::SupportedCollatorLocales,
            CollatorProviderRequest::Compare { .. } => IntlHostOp::CompareCollator,
        };
        let message = IntlByteArrayBuilder::with_operation(operation, schema, function);
        message.append_u64_constant(COLLATOR_WIRE_VERSION, schema, function);
        message.append_u64_constant(u64::from(operation.code()) * 2, schema, function);
        match request {
            CollatorProviderRequest::Resolve(input) => {
                self.emit_intl_wire_canonical_locales(&message, input.locales(), function)?;
                collator_append_domain(&message, input.matcher(), schema, function);
                collator_append_domain(&message, input.usage(), schema, function);
                collator_append_boolean(&message, input.collation_present(), schema, function);
                input.collation_present().load(function);
                self.open_frame(ControlFrameKind::If, function);
                message.append_utf8(input.collation(), schema, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                collator_append_boolean(&message, input.numeric_present(), schema, function);
                input.numeric_present().load(function);
                self.open_frame(ControlFrameKind::If, function);
                collator_append_boolean(&message, input.numeric(), schema, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                emit_domain_is(input.case_first(), None, function);
                function.instruction(&Instruction::I32Eqz);
                let present = schema.reserve_i32_local(function);
                present.store(function);
                collator_append_boolean(&message, present, schema, function);
                present.load(function);
                self.open_frame(ControlFrameKind::If, function);
                collator_append_domain(&message, input.case_first(), schema, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                schema.release_i32_local(present, function);
            }
            CollatorProviderRequest::Supported { locales, matcher } => {
                self.emit_intl_wire_canonical_locales(&message, locales, function)?;
                collator_append_domain(&message, matcher, schema, function);
            }
            CollatorProviderRequest::Compare { record, input } => {
                let co = schema.struct_type::<IntlCollatorObject>();
                let locale = schema.reserve_gc_local(function).initialize(
                    co.field(IntlCollatorObjectSchema::LOCALE)
                        .read(record, schema, function)
                        .reference(),
                    function,
                );
                message.append_utf8(&locale, schema, function);
                locale.clear(function);
                let word = schema.reserve_i32_local(function);
                let wide = schema.reserve_i64_local(function);
                co.field(IntlCollatorObjectSchema::USAGE)
                    .read(record, schema, function)
                    .store(word, function);
                word.load(function);
                function.instruction(&Instruction::I64ExtendI32U);
                wide.store(function);
                message.append_u64(wide, schema, function);
                co.field(IntlCollatorObjectSchema::COLLATION_KIND)
                    .read(record, schema, function)
                    .store(word, function);
                word.load(function);
                function.instruction(&Instruction::I64ExtendI32U);
                wide.store(function);
                message.append_u64(wide, schema, function);
                word.load(function);
                function.instruction(&Instruction::I32Const(
                    CollatorCollationKind::UnicodeType.encode(),
                ));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                let text = schema.reserve_gc_local(function).initialize(
                    co.field(IntlCollatorObjectSchema::COLLATION)
                        .read(record, schema, function)
                        .reference(),
                    function,
                );
                message.append_utf8(&text, schema, function);
                text.clear(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                co.field(IntlCollatorObjectSchema::NUMERIC)
                    .read(record, schema, function)
                    .store(word, function);
                collator_append_boolean(&message, word, schema, function);
                co.field(IntlCollatorObjectSchema::CASE_FIRST)
                    .read(record, schema, function)
                    .store(word, function);
                word.load(function);
                function.instruction(&Instruction::I64ExtendI32U);
                wide.store(function);
                message.append_u64(wide, schema, function);
                co.field(IntlCollatorObjectSchema::SENSITIVITY)
                    .read(record, schema, function)
                    .store(word, function);
                word.load(function);
                function.instruction(&Instruction::I64ExtendI32U);
                wide.store(function);
                message.append_u64(wide, schema, function);
                co.field(IntlCollatorObjectSchema::IGNORE_PUNCTUATION)
                    .read(record, schema, function)
                    .store(word, function);
                collator_append_boolean(&message, word, schema, function);
                message.append_utf16(input.left(), schema, function);
                message.append_utf16(input.right(), schema, function);
                schema.release_i64_local(wide, function);
                schema.release_i32_local(word, function);
            }
        }
        let bytes = message.finish(schema, function);
        let reply = self.emit_intl_provider_byte_call(&bytes, function)?;
        reply.load(schema, function).is_null(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let response = schema.reserve_gc_local(function).initialize(
            reply.load(schema, function).require_non_null(function),
            function,
        );
        reply.clear(function);
        bytes.clear(function);
        Ok(CollatorProviderResponse {
            bytes: response,
            operation,
        })
    }
}
