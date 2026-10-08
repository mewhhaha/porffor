//! JavaScript observations and complete GC configurations; pinned native parts.
use super::super::*;
use super::intl_number::*;
use crate::functions::{NonArrayRealmIntrinsicSlot, OrdinaryDefaultPrototype};
use crate::gc_types::*;
use lila_intl::number_format::options::*;
use lila_intl::number_format::{NumberPartKind, RangePartSource};
use lila_intl::NumberPrecisionKind;

mod construction_lifecycle;
mod initialization;
mod options;
mod pool;
pub(crate) use pool::intl_number_format_pool_strings;
mod primitive_locale;
mod render;
mod resolved;
mod validation;

const NF_RECEIVER_ERROR: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_NUMBERFORMAT_METHOD_REQUIRES_A_NUMBERFORMAT_RECEIVER;
const NF_RANGE_UNDEFINED: RuntimeErrorMessage =
    RuntimeErrorMessage::INTL_NUMBERFORMAT_RANGE_ENDPOINTS_MUST_BE_DEFINED;
const NF_RANGE_NAN: &str = "Intl.NumberFormat range endpoints must not be NaN";
const NF_CURRENCY_REQUIRED: RuntimeErrorMessage =
    RuntimeErrorMessage::CURRENCY_STYLE_REQUIRES_A_CURRENCY_OPTION;
const NF_UNIT_REQUIRED: RuntimeErrorMessage =
    RuntimeErrorMessage::UNIT_STYLE_REQUIRES_A_UNIT_OPTION;

#[derive(Clone, Copy)]
pub(crate) enum NfFormatMode {
    String,
    Parts,
}

struct NfOptionsLocals {
    style: GcI32DomainLocal<StyleOption>,
    style_text: GcLocal<StringValue>,
    currency_display: GcI32DomainLocal<Option<CurrencyDisplay>>,
    currency_sign: GcI32DomainLocal<Option<CurrencySign>>,
    unit_display: GcI32DomainLocal<Option<UnitDisplay>>,
    notation: GcI32DomainLocal<NotationOption>,
    compact_display: GcI32DomainLocal<Option<CompactDisplay>>,
    grouping: GcI32DomainLocal<Grouping>,
    sign: GcI32DomainLocal<SignDisplay>,
}
impl NfOptionsLocals {
    fn new(builder: &mut FunctionBuilder<'_>, function: &mut Function) -> Result<Self, EmitError> {
        let schema = builder.runtime_schema();
        Ok(Self {
            style: GcI32DomainLocal::new(schema, StyleOption::Decimal, function),
            style_text: schema.reserve_gc_local(function).initialize(
                builder.emit_interned_string_reference("", function)?,
                function,
            ),
            currency_display: GcI32DomainLocal::new(
                schema,
                Some(CurrencyDisplay::Symbol),
                function,
            ),
            currency_sign: GcI32DomainLocal::new(schema, Some(CurrencySign::Standard), function),
            unit_display: GcI32DomainLocal::new(schema, Some(UnitDisplay::Short), function),
            notation: GcI32DomainLocal::new(schema, NotationOption::Standard, function),
            compact_display: GcI32DomainLocal::new(schema, Some(CompactDisplay::Short), function),
            grouping: GcI32DomainLocal::new(schema, Grouping::Auto, function),
            sign: GcI32DomainLocal::new(schema, SignDisplay::Auto, function),
        })
    }
    fn clear(self, schema: &RuntimeSchema, function: &mut Function) {
        self.sign.clear(schema, function);
        self.grouping.clear(schema, function);
        self.compact_display.clear(schema, function);
        self.notation.clear(schema, function);
        self.unit_display.clear(schema, function);
        self.currency_sign.clear(schema, function);
        self.currency_display.clear(schema, function);
        self.style_text.clear(function);
        self.style.clear(schema, function);
    }
}
impl FunctionBuilder<'_> {
    fn emit_nf_record_from_receiver(
        &mut self,
        function: &mut Function,
    ) -> Result<GcLocal<IntlNumberFormatObject>, EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        receiver.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| EmitError::unsupported("NumberFormat method lacks callable entry"))?
                .this_value(),
            function,
        );
        receiver.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<IntlNumberFormatObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_type_error(NF_RECEIVER_ERROR, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let record = schema.reserve_gc_local(function).initialize(
            receiver.cast_reference::<IntlNumberFormatObject>(schema, function),
            function,
        );
        receiver.clear(function);
        Ok(record)
    }
}
