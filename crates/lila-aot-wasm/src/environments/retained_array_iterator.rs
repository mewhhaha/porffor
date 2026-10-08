//! Suspended array patterns retain the same native IteratorRecord edge.

use super::*;
use crate::gc_types::{InvocationFrame, InvocationFrameSchema, IteratorRecord};
use lila_ir::ArrayIteratorStorageIr;

/// Only an exact owned activation cell can publish or reload this record.
pub(crate) struct RetainedArrayIteratorStorage {
    slot: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn array_iterator_storage(
        &self,
        storage: &ArrayIteratorStorageIr,
    ) -> Result<RetainedArrayIteratorStorage, EmitError> {
        let binding = storage.binding();
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
                "compiler invariant: suspended array iterator requires its exact checked resumable cell",
            ));
        }
        Ok(RetainedArrayIteratorStorage { slot: binding.slot })
    }

    fn retained_array_iterator_cell(
        &mut self,
        storage: &RetainedArrayIteratorStorage,
        function: &mut Function,
    ) -> GcLocal<BindingCell> {
        let schema = self.runtime_schema();
        let frame = self
            .body_entry_locals()
            .and_then(|entry| entry.resume_frame())
            .expect("checked array iterator storage owns its invocation frame");
        // Nested blocks and With records can change the current environment.
        // This cell always belongs to the original invocation environment.
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

    pub(crate) fn emit_publish_retained_array_iterator(
        &mut self,
        storage: &RetainedArrayIteratorStorage,
        record: &GcLocal<IteratorRecord>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let cell = self.retained_array_iterator_cell(storage, function);
        schema
            .struct_type::<BindingCell>()
            .field(BindingCellSchema::DESTRUCTURING_ITERATOR_RECORD)
            .write(
                &cell,
                GcOperand::nullable_reference(record, schema),
                schema,
                function,
            );
        cell.clear(function);
    }

    pub(crate) fn emit_load_retained_array_iterator(
        &mut self,
        storage: &RetainedArrayIteratorStorage,
        function: &mut Function,
    ) -> GcLocal<IteratorRecord> {
        let schema = self.runtime_schema();
        let cell = self.retained_array_iterator_cell(storage, function);
        let record = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<BindingCell>()
                .field(BindingCellSchema::DESTRUCTURING_ITERATOR_RECORD)
                .read(&cell, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        cell.clear(function);
        record
    }

    /// The complete pattern's close path alone retires this edge. Ordinary
    /// Yield, Await, delegated done:false and generic Reference cleanup cannot
    /// do so.
    pub(crate) fn emit_retire_retained_array_iterator(
        &mut self,
        storage: &RetainedArrayIteratorStorage,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let cell = self.retained_array_iterator_cell(storage, function);
        schema
            .struct_type::<BindingCell>()
            .field(BindingCellSchema::DESTRUCTURING_ITERATOR_RECORD)
            .write(&cell, GcOperand::null(schema), schema, function);
        cell.clear(function);
    }
}
