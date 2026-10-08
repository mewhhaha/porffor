//! Segmenter retains its original GC String and one checked private partition.
use super::super::*;
use super::intl::CanonicalLocaleListLocals;
use super::intl_number::*;
use super::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::gc_types::*;
use lila_intl::number_format::options::LocaleMatcher;
use lila_intl::{SegmenterGranularity, SegmenterWireOperation, SEGMENTER_WIRE_VERSION};
mod construction;
mod iterator;
mod partition;
mod pool;
mod resolved;
mod segments;
pub(crate) use pool::intl_segmenter_pool_strings;
const SEGMENTER_CONSTRUCT_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_SEGMENTER_REQUIRES_NEW;
const SEGMENTER_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_SEGMENTER_METHOD_REQUIRES_A_SEGMENTER_RECEIVER;
const SEGMENTS_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::SEGMENTS_METHOD_REQUIRES_A_SEGMENTS_RECEIVER;
const SEGMENT_ITERATOR_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::SEGMENT_ITERATOR_NEXT_REQUIRES_A_SEGMENT_ITERATOR_RECEIVER;
struct CompletedSegmentInputLocals {
    string: GcLocal<StringValue>,
    unit_count: I32Local,
}
impl CompletedSegmentInputLocals {
    fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        schema.release_i32_local(self.unit_count, f);
        self.string.clear(f);
    }
}
enum SegmentIntrinsic {
    SegmentsPrototype,
    IteratorPrototype,
}
enum SegmentProviderRequest<'a> {
    Resolve {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Supported {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Partition {
        record: &'a GcLocal<IntlSegmenterObject>,
        input: &'a CompletedSegmentInputLocals,
    },
}
struct SegmentProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: SegmenterWireOperation,
}
impl SegmentProviderResponse {
    fn reader(&self, schema: &RuntimeSchema, f: &mut Function) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, f);
        let word = schema.reserve_i64_local(f);
        for expected in [
            SEGMENTER_WIRE_VERSION,
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
    fn emit_segment_brand_guard<T: JavaScriptReference>(
        &mut self,
        error: RuntimeErrorMessage,
        f: &mut Function,
    ) -> Result<GcLocal<T>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("Segmenter-family receiver lacks callable entry")
                })?
                .this_value(),
            f,
        );
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<T>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(error, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let record = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<T>(schema, f), f);
        value.clear(f);
        Ok(record)
    }
    fn emit_segmenter_record(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlSegmenterObject>, EmitError> {
        self.emit_segment_brand_guard(SEGMENTER_RECEIVER_ERROR, f)
    }
    fn emit_segments_record(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlSegmentsObject>, EmitError> {
        self.emit_segment_brand_guard(SEGMENTS_RECEIVER_ERROR, f)
    }
    fn emit_segment_iterator_record(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlSegmentIteratorObject>, EmitError> {
        self.emit_segment_brand_guard(SEGMENT_ITERATOR_RECEIVER_ERROR, f)
    }
    fn emit_segment_intrinsic_header(
        &mut self,
        intrinsic: SegmentIntrinsic,
        f: &mut Function,
    ) -> Result<GcLocal<OrdinaryObject>, EmitError> {
        let slot = match intrinsic {
            SegmentIntrinsic::SegmentsPrototype => {
                NonArrayRealmIntrinsicSlot::IntlSegmentsPrototype
            }
            SegmentIntrinsic::IteratorPrototype => {
                NonArrayRealmIntrinsicSlot::IntlSegmentIteratorPrototype
            }
        };
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let prototype = schema.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(&realm, slot, &prototype, f);
        let header = schema.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        );
        prototype.clear(f);
        realm.clear(f);
        Ok(header)
    }
    fn emit_segment_provider_call(
        &mut self,
        request: SegmentProviderRequest<'_>,
        f: &mut Function,
    ) -> Result<SegmentProviderResponse, EmitError> {
        let schema = self.runtime_schema();
        let operation = match &request {
            SegmentProviderRequest::Resolve { .. } => SegmenterWireOperation::ResolveLocale,
            SegmentProviderRequest::Supported { .. } => SegmenterWireOperation::SupportedLocales,
            SegmentProviderRequest::Partition { .. } => SegmenterWireOperation::SegmentUtf16,
        };
        let message = IntlByteArrayBuilder::with_operation(operation.global_operation(), schema, f);
        message.append_u64_constant(SEGMENTER_WIRE_VERSION, schema, f);
        message.append_u64_constant(u64::from(operation.code()) * 2, schema, f);
        let word = schema.reserve_i64_local(f);
        match request {
            SegmentProviderRequest::Resolve { locales, matcher }
            | SegmentProviderRequest::Supported { locales, matcher } => {
                self.emit_intl_wire_canonical_locales(&message, locales, f)?;
                matcher.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
            }
            SegmentProviderRequest::Partition { record, input } => {
                let st = schema.struct_type::<IntlSegmenterObject>();
                let locale = schema.reserve_gc_local(f).initialize(
                    st.field(IntlSegmenterObjectSchema::LOCALE)
                        .read(record, schema, f)
                        .reference(),
                    f,
                );
                message.append_utf8(&locale, schema, f);
                locale.clear(f);
                let granularity = schema.reserve_i32_local(f);
                st.field(IntlSegmenterObjectSchema::GRANULARITY)
                    .read(record, schema, f)
                    .store(granularity, f);
                granularity.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
                schema.release_i32_local(granularity, f);
                message.append_utf16(&input.string, schema, f);
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
        Ok(SegmentProviderResponse {
            bytes: result,
            operation,
        })
    }
}
