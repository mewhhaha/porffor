use super::*;
#[must_use]
pub(super) struct ReservedIntlDateTimeFormatObjectLocal(pub(super) GcLocal<OrdinaryObject>);
impl FunctionBuilder<'_> {
    pub(super) fn emit_reserve_intl_date_time_format_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedIntlDateTimeFormatObjectLocal, EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        let prototype = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        target.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("DateTimeFormat constructor lacks callable entry")
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
            NonArrayRealmIntrinsicSlot::IntlDateTimeFormatPrototype,
            &prototype,
            function,
        );
        realm.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_get_prototype_from_constructor(
            &target,
            OrdinaryDefaultPrototype::IntlDateTimeFormat,
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
        Ok(ReservedIntlDateTimeFormatObjectLocal(header))
    }

    pub(super) fn emit_reserve_intrinsic_date_time_format_object(
        &mut self,
        f: &mut Function,
    ) -> Result<ReservedIntlDateTimeFormatObjectLocal, EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let prototype = schema.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::IntlDateTimeFormatPrototype,
            &prototype,
            f,
        );
        let header = schema.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        );
        prototype.clear(f);
        realm.clear(f);
        Ok(ReservedIntlDateTimeFormatObjectLocal(header))
    }
}
