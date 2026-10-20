//! Result header reservation precedes observations; only completed records publish.
use super::*;
#[must_use]
pub(super) struct ReservedIntlLocaleObjectLocal(GcLocal<OrdinaryObject>);
#[must_use]
pub(super) struct InitializedIntlLocaleObjectLocal(GcLocal<IntlLocaleObject>);
impl FunctionBuilder<'_> {
    pub(super) fn emit_reserve_intl_locale_object(
        &mut self,
        target: &ValueLocals,
        function: &mut Function,
    ) -> Result<ReservedIntlLocaleObjectLocal, EmitError> {
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        self.emit_get_prototype_from_constructor(
            target,
            OrdinaryDefaultPrototype::IntlLocale,
            &pending,
            function,
        )?;
        self.emit_intl_number_adopt_completion(&pending, function);
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(pending.value()), function)?,
            function,
        );
        pending.clear(function);
        Ok(ReservedIntlLocaleObjectLocal(header))
    }
    pub(super) fn emit_reserve_intrinsic_intl_locale_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedIntlLocaleObjectLocal, EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::IntlLocalePrototype,
            &prototype,
            function,
        );
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), function)?,
            function,
        );
        prototype.clear(function);
        realm.clear(function);
        Ok(ReservedIntlLocaleObjectLocal(header))
    }
    pub(super) fn emit_initialize_intl_locale_object(
        &self,
        reserved: ReservedIntlLocaleObjectLocal,
        components: &CanonicalLocaleComponents,
        function: &mut Function,
    ) -> InitializedIntlLocaleObjectLocal {
        let schema = self.runtime_schema();
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<IntlLocaleObject>().construct(
                (
                    GcOperand::reference(&reserved.0, schema),
                    GcOperand::reference(&components.tag, schema),
                    GcOperand::reference(&components.language, schema),
                    GcOperand::reference(&components.script, schema),
                    GcOperand::reference(&components.region, schema),
                    GcOperand::reference(&components.base_name, schema),
                ),
                function,
            ),
            function,
        );
        reserved.0.clear(function);
        InitializedIntlLocaleObjectLocal(record)
    }
    pub(super) fn emit_publish_intl_locale_object(
        &self,
        initialized: InitializedIntlLocaleObjectLocal,
        function: &mut Function,
    ) {
        self.completion().initialize(function);
        self.completion()
            .value()
            .set_reference(&initialized.0, self.runtime_schema(), function);
        initialized.0.clear(function);
    }
}
