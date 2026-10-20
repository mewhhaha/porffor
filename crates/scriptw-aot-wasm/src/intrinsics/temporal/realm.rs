use super::*;
use crate::gc_types::{GcStackReference, OrdinaryObject, ValueLocals};

/// Completed GetPrototypeFromConstructor output, retaining full identity.
#[must_use]
pub(crate) struct TemporalConstructorPrototypeLocals {
    family: TemporalIntrinsicFamily,
    value: ValueLocals,
}
impl TemporalConstructorPrototypeLocals {
    pub(crate) fn release(self, function: &mut Function) {
        self.value.clear(function);
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TemporalPrototypeSource<'a> {
    Intrinsic,
    Constructor(&'a TemporalConstructorPrototypeLocals),
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_load_current_builtin_temporal_prototype(
        &mut self,
        family: TemporalIntrinsicFamily,
        prototype: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            family.prototype_slot(),
            prototype,
            function,
        );
        realm.clear(function);
    }

    pub(crate) fn emit_temporal_constructor_prototype(
        &mut self,
        family: TemporalIntrinsicFamily,
        function: &mut Function,
    ) -> Result<TemporalConstructorPrototypeLocals, EmitError> {
        let schema = self.runtime_schema();
        let constructor = schema.reserve_value_local(function);
        constructor.copy_from(
            self.body_entry_locals()
                .ok_or_else(|| {
                    EmitError::unsupported("Temporal constructor has no callable entry")
                })?
                .new_target(),
            function,
        );
        let completion = schema.reserve_completion(function);
        self.emit_get_prototype_from_constructor(
            &constructor,
            crate::functions::OrdinaryDefaultPrototype::Temporal(family),
            &completion,
            function,
        )?;
        self.completion().copy_from(&completion, function);
        self.emit_propagate_current_throw_if_needed(function);
        let value = schema.reserve_value_local(function);
        value.copy_from(completion.value(), function);
        completion.clear(function);
        constructor.clear(function);
        Ok(TemporalConstructorPrototypeLocals { family, value })
    }

    /// Allocate only the common object header. Each native Temporal allocator
    /// must construct its concrete GC record from completed numeric fields.
    pub(crate) fn emit_alloc_temporal_object_header(
        &mut self,
        family: TemporalIntrinsicFamily,
        source: TemporalPrototypeSource<'_>,
        function: &mut Function,
    ) -> Result<GcStackReference<OrdinaryObject>, EmitError> {
        match source {
            TemporalPrototypeSource::Intrinsic => {
                let prototype = self.runtime_schema().reserve_value_local(function);
                self.emit_load_current_builtin_temporal_prototype(family, &prototype, function);
                let header =
                    self.emit_alloc_plain_object_with_prototype(Some(&prototype), function)?;
                prototype.clear(function);
                Ok(header)
            }
            TemporalPrototypeSource::Constructor(prototype) => {
                if prototype.family != family {
                    return Err(EmitError::unsupported(
                        "Temporal constructor prototype family mismatch",
                    ));
                }
                self.emit_alloc_plain_object_with_prototype(Some(&prototype.value), function)
            }
        }
    }
}
