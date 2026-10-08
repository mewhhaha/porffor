use super::*;

/// An unreachable fresh header, allocated before any locales/options hooks.
#[must_use]
pub(super) struct ReservedIntlNumberFormatObjectLocal(GcLocal<OrdinaryObject>);
/// Only the completed branded configuration can cross constructor publication.
#[must_use]
pub(super) struct InitializedIntlNumberFormatObjectLocal(GcLocal<IntlNumberFormatObject>);

impl FunctionBuilder<'_> {
    pub(super) fn emit_reserve_intl_number_format_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedIntlNumberFormatObjectLocal, EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        let prototype = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("NumberFormat constructor lacks callable entry")
                })?
                .new_target(),
            function,
        );
        target.tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        // A plain call selects the active intrinsic constructor's immutable
        // prototype, without consulting mutable public Intl properties.
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::IntlNumberFormatPrototype,
            &prototype,
            function,
        );
        realm.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_get_prototype_from_constructor(
            &target,
            OrdinaryDefaultPrototype::IntlNumberFormat,
            &pending,
            function,
        )?;
        self.emit_intl_number_adopt_completion(&pending, function);
        prototype.copy_from(pending.value(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), function)?,
            function,
        );
        pending.clear(function);
        prototype.clear(function);
        target.clear(function);
        Ok(ReservedIntlNumberFormatObjectLocal(header))
    }
    pub(super) fn emit_initialize_intl_number_format_object(
        &self,
        reserved: ReservedIntlNumberFormatObjectLocal,
        locale: &GcLocal<StringValue>,
        data_locale: &GcLocal<StringValue>,
        numbering_system: &GcLocal<StringValue>,
        selected: &NfOptionsLocals,
        rounding: &GcLocal<IntlNumberRounding>,
        function: &mut Function,
    ) -> InitializedIntlNumberFormatObjectLocal {
        let schema = self.runtime_schema();
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IntlNumberFormatObject>().construct(
                (
                    GcOperand::reference(&reserved.0, schema),
                    GcOperand::reference(locale, schema),
                    GcOperand::reference(data_locale, schema),
                    GcOperand::reference(numbering_system, schema),
                    selected.style.operand(),
                    GcOperand::reference(&selected.style_text, schema),
                    selected.currency_display.operand(),
                    selected.currency_sign.operand(),
                    selected.unit_display.operand(),
                    selected.notation.operand(),
                    selected.compact_display.operand(),
                    GcOperand::reference(rounding, schema),
                    selected.grouping.operand(),
                    selected.sign.operand(),
                    GcOperand::null(schema),
                ),
                function,
            ),
            function,
        );
        reserved.0.clear(function);
        InitializedIntlNumberFormatObjectLocal(record)
    }
    pub(super) fn emit_publish_intl_number_format_object(
        &self,
        initialized: InitializedIntlNumberFormatObjectLocal,
        function: &mut Function,
    ) {
        self.completion().initialize(function);
        self.completion()
            .value()
            .set_reference(&initialized.0, self.runtime_schema(), function);
        initialized.0.clear(function);
    }
}
