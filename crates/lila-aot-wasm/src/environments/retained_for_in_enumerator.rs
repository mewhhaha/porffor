//! A suspended ForIn cursor occupies one exact original invocation cell.

use super::*;
use crate::gc_types::{ForInEnumerationRecord, InvocationFrame, InvocationFrameSchema};

pub(crate) struct RetainedForInEnumeratorStorage {
    slot: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn for_in_enumerator_storage(
        &self,
        binding: &OwnedEnvBindingIr,
    ) -> Result<RetainedForInEnumeratorStorage, EmitError> {
        if !self.current_function_meta().is_some_and(|meta| {
            match meta.protocol().execution_kind() {
                FunctionExecutionKind::Generator | FunctionExecutionKind::Async => true,
                FunctionExecutionKind::AsyncGenerator => self
                    .checked_async_generator_environment_owner
                    .is_some_and(|owner| owner.is_region()),
                FunctionExecutionKind::Ordinary => false,
            }
        }) || self
            .body_entry_locals()
            .and_then(|entry| entry.resume_frame())
            .is_none()
            || !self.owned_env_bindings.iter().any(|owned| owned == binding)
            || self.owned_env_slot(&binding.name) != Some(binding.slot)
        {
            return Err(EmitError::unsupported(
                "compiler invariant: ForIn enumeration requires its exact owned resumable cell",
            ));
        }
        Ok(RetainedForInEnumeratorStorage { slot: binding.slot })
    }

    fn retained_for_in_enumerator_cell(
        &mut self,
        storage: &RetainedForInEnumeratorStorage,
        function: &mut Function,
    ) -> GcLocal<BindingCell> {
        let schema = self.runtime_schema();
        let frame = self
            .body_entry_locals()
            .and_then(|entry| entry.resume_frame())
            .expect("checked ForIn storage owns its invocation frame");
        // With and nested lexical bodies change current hops. The retained
        // enumeration cell always belongs to the original invocation instead.
        let environment = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<InvocationFrame>()
                .field(InvocationFrameSchema::INVOCATION_ENVIRONMENT)
                .read(frame, schema, function)
                .reference(),
            function,
        );
        let cell = self.emit_environment_cell_local(&environment, storage.slot, function);
        environment.clear(function);
        cell
    }

    pub(crate) fn emit_publish_retained_for_in_enumerator(
        &mut self,
        storage: &RetainedForInEnumeratorStorage,
        record: &GcLocal<ForInEnumerationRecord>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let cell = self.retained_for_in_enumerator_cell(storage, function);
        schema
            .struct_type::<BindingCell>()
            .field(BindingCellSchema::FOR_IN_ENUMERATION_RECORD)
            .write(
                &cell,
                GcOperand::nullable_reference(record, schema),
                schema,
                function,
            );
        cell.clear(function);
    }

    pub(crate) fn emit_load_retained_for_in_enumerator(
        &mut self,
        storage: &RetainedForInEnumeratorStorage,
        function: &mut Function,
    ) -> GcLocal<ForInEnumerationRecord> {
        let schema = self.runtime_schema();
        let cell = self.retained_for_in_enumerator_cell(storage, function);
        let record = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<BindingCell>()
                .field(BindingCellSchema::FOR_IN_ENUMERATION_RECORD)
                .read(&cell, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        cell.clear(function);
        record
    }

    pub(crate) fn emit_retire_retained_for_in_enumerator(
        &mut self,
        storage: &RetainedForInEnumeratorStorage,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let cell = self.retained_for_in_enumerator_cell(storage, function);
        schema
            .struct_type::<BindingCell>()
            .field(BindingCellSchema::FOR_IN_ENUMERATION_RECORD)
            .write(&cell, GcOperand::null(schema), schema, function);
        cell.clear(function);
    }
}
