//! PluralRules observes JavaScript before its closed native GC provider seam.
use super::super::*;
use super::intl::CanonicalLocaleListLocals;
use super::intl_number::*;
use super::intl_provider_wire::{
    append_intl_mathematical_value, IntlByteArrayBuilder, IntlByteArrayReader,
};
use crate::gc_types::*;
use lila_intl::number_format::options::*;
use lila_intl::{
    IntlHostOp, NumberPrecisionKind, PluralCategory, PluralCategorySet, PluralConfigurationWord,
    PluralType, PLURAL_WIRE_VERSION,
};
mod construction;
mod pool;
mod resolved;
mod select;
pub(crate) use pool::intl_plural_rules_pool_strings;
const PR_CONSTRUCT_ERROR: RuntimeErrorMessage = RuntimeErrorMessage::INTL_PLURALRULES_REQUIRES_NEW;
const PR_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_PLURALRULES_METHOD_REQUIRES_A_PLURALRULES_RECEIVER;
const PR_RANGE_UNDEFINED: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_PLURALRULES_RANGE_ENDPOINTS_MUST_BE_DEFINED;

enum PluralProviderRequest<'a> {
    Resolve {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Supported {
        locales: &'a CanonicalLocaleListLocals,
        matcher: &'a GcI32DomainLocal<LocaleMatcher>,
    },
    Select {
        record: &'a GcLocal<IntlPluralRulesObject>,
        input: &'a IntlMathematicalValueLocals,
    },
    Range {
        record: &'a GcLocal<IntlPluralRulesObject>,
        start: &'a IntlMathematicalValueLocals,
        end: &'a IntlMathematicalValueLocals,
    },
}
struct PluralProviderResponse {
    bytes: GcLocal<ByteArray>,
    operation: IntlHostOp,
}
impl PluralProviderResponse {
    fn reader(&self, schema: &RuntimeSchema, f: &mut Function) -> IntlByteArrayReader<'_> {
        let reader = IntlByteArrayReader::new(&self.bytes, schema, f);
        let word = schema.reserve_i64_local(f);
        for expected in [
            PLURAL_WIRE_VERSION,
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
fn plural_other_categories() -> PluralCategorySet {
    PluralCategorySet::from_wire_mask(1u64 << PluralCategory::Other.index())
        .expect("Other is a valid category set")
}
impl FunctionBuilder<'_> {
    fn emit_plural_record_from_receiver(
        &mut self,
        f: &mut Function,
    ) -> Result<GcLocal<IntlPluralRulesObject>, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| EmitError::unsupported("PluralRules receiver lacks callable entry"))?
                .this_value(),
            f,
        );
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlPluralRulesObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(PR_RECEIVER_ERROR, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let record = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<IntlPluralRulesObject>(schema, f), f);
        value.clear(f);
        Ok(record)
    }
    fn emit_plural_wire_configuration(
        &self,
        message: &IntlByteArrayBuilder,
        record: &GcLocal<IntlPluralRulesObject>,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let pr = schema.struct_type::<IntlPluralRulesObject>();
        for field in [
            IntlPluralRulesObjectSchema::LOCALE,
            IntlPluralRulesObjectSchema::DATA_LOCALE,
        ] {
            let text = schema
                .reserve_gc_local(f)
                .initialize(pr.field(field).read(record, schema, f).reference(), f);
            message.append_utf8(&text, schema, f);
            text.clear(f);
        }
        let rounding = schema.reserve_gc_local(f).initialize(
            pr.field(IntlPluralRulesObjectSchema::ROUNDING)
                .read(record, schema, f)
                .reference(),
            f,
        );
        let rules = schema.struct_type::<IntlNumberRounding>();
        let code = schema.reserve_i32_local(f);
        let wide = schema.reserve_i64_local(f);
        for word in PluralConfigurationWord::ALL {
            use PluralConfigurationWord as W;
            match word {
                W::Type => pr
                    .field(IntlPluralRulesObjectSchema::TYPE)
                    .read(record, schema, f)
                    .store(code, f),
                W::Notation => pr
                    .field(IntlPluralRulesObjectSchema::NOTATION)
                    .read(record, schema, f)
                    .store(code, f),
                W::CompactDisplay => pr
                    .field(IntlPluralRulesObjectSchema::COMPACT_DISPLAY)
                    .read(record, schema, f)
                    .store(code, f),
                W::MinimumInteger => rules
                    .field(IntlNumberRoundingSchema::MINIMUM_INTEGER)
                    .read(&rounding, schema, f)
                    .store(code, f),
                W::Precision => rules
                    .field(IntlNumberRoundingSchema::PRECISION)
                    .read(&rounding, schema, f)
                    .store(code, f),
                W::MinimumFraction => rules
                    .field(IntlNumberRoundingSchema::MINIMUM_FRACTION)
                    .read(&rounding, schema, f)
                    .store(code, f),
                W::MaximumFraction => rules
                    .field(IntlNumberRoundingSchema::MAXIMUM_FRACTION)
                    .read(&rounding, schema, f)
                    .store(code, f),
                W::MinimumSignificant => rules
                    .field(IntlNumberRoundingSchema::MINIMUM_SIGNIFICANT)
                    .read(&rounding, schema, f)
                    .store(code, f),
                W::MaximumSignificant => rules
                    .field(IntlNumberRoundingSchema::MAXIMUM_SIGNIFICANT)
                    .read(&rounding, schema, f)
                    .store(code, f),
                W::RoundingIncrement => rules
                    .field(IntlNumberRoundingSchema::ROUNDING_INCREMENT)
                    .read(&rounding, schema, f)
                    .store(code, f),
                W::RoundingMode => rules
                    .field(IntlNumberRoundingSchema::ROUNDING_MODE)
                    .read(&rounding, schema, f)
                    .store(code, f),
                W::TrailingZero => rules
                    .field(IntlNumberRoundingSchema::TRAILING_ZERO)
                    .read(&rounding, schema, f)
                    .store(code, f),
            }
            // The native wire uses zero only for inactive fields; GC None is -1.
            code.load(f);
            f.instruction(&Instruction::I32Const(0));
            code.load(f);
            f.instruction(&Instruction::I32Const(0));
            f.instruction(&Instruction::I32GeS);
            f.instruction(&Instruction::Select);
            f.instruction(&Instruction::I64ExtendI32U);
            wide.store(f);
            message.append_u64(wide, schema, f);
        }
        schema.release_i64_local(wide, f);
        schema.release_i32_local(code, f);
        rounding.clear(f);
    }
    fn emit_plural_provider_call(
        &mut self,
        request: PluralProviderRequest<'_>,
        f: &mut Function,
    ) -> Result<PluralProviderResponse, EmitError> {
        let schema = self.runtime_schema();
        let range_rejection = matches!(&request, PluralProviderRequest::Range { .. });
        let operation = match &request {
            PluralProviderRequest::Resolve { .. } => IntlHostOp::ResolvePluralLocale,
            PluralProviderRequest::Supported { .. } => IntlHostOp::SupportedPluralLocales,
            PluralProviderRequest::Select { .. } => IntlHostOp::SelectPlural,
            PluralProviderRequest::Range { .. } => IntlHostOp::SelectPluralRange,
        };
        let message = IntlByteArrayBuilder::with_operation(operation, schema, f);
        message.append_u64_constant(PLURAL_WIRE_VERSION, schema, f);
        message.append_u64_constant(u64::from(operation.code()) * 2, schema, f);
        match request {
            PluralProviderRequest::Resolve { locales, matcher }
            | PluralProviderRequest::Supported { locales, matcher } => {
                self.emit_intl_wire_canonical_locales(&message, locales, f)?;
                let word = schema.reserve_i64_local(f);
                matcher.load(f);
                f.instruction(&Instruction::I64ExtendI32U);
                word.store(f);
                message.append_u64(word, schema, f);
                schema.release_i64_local(word, f);
            }
            PluralProviderRequest::Select { record, input } => {
                self.emit_plural_wire_configuration(&message, record, f);
                append_intl_mathematical_value(&message, input, schema, f);
            }
            PluralProviderRequest::Range { record, start, end } => {
                self.emit_plural_wire_configuration(&message, record, f);
                append_intl_mathematical_value(&message, start, schema, f);
                append_intl_mathematical_value(&message, end, schema, f);
            }
        }
        let bytes = message.finish(schema, f);
        let reply = self.emit_intl_provider_byte_call(&bytes, f)?;
        reply.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        if range_rejection {
            self.emit_intl_number_range_error(
                RuntimeErrorMessage::INTL_PLURALRULES_RANGE_ENDPOINTS_MUST_NOT_BE_NAN,
                f,
            )?;
        } else {
            f.instruction(&Instruction::Unreachable);
        }
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let bytes_out = schema
            .reserve_gc_local(f)
            .initialize(reply.load(schema, f).require_non_null(f), f);
        reply.clear(f);
        bytes.clear(f);
        Ok(PluralProviderResponse {
            bytes: bytes_out,
            operation,
        })
    }
}
