//! Class continuation environments stay outside the JavaScript value domain.

use super::*;
use crate::gc_types::{
    BindingCell, BindingCellSchema, Environment, GcLocal, GcOperand, InvocationFrame,
    InvocationFrameSchema, Nullable,
};

/// Only an owned activation cell can retain a suspended class-name scope.
pub(super) struct ClassEnvironmentCapture {
    slot: u32,
}

impl FunctionBuilder<'_> {
    pub(super) fn class_environment_capture(
        &self,
        name: &str,
    ) -> Result<ClassEnvironmentCapture, EmitError> {
        let slot = self.owned_env_slot(name).ok_or_else(|| {
            EmitError::unsupported(
                "compiler invariant: class environment capture has no owned activation cell",
            )
        })?;
        if self
            .body_entry_locals()
            .and_then(|entry| entry.resume_frame())
            .is_none()
        {
            return Err(EmitError::unsupported(
                "compiler invariant: class environment capture has no resumable frame",
            ));
        }
        Ok(ClassEnvironmentCapture { slot })
    }

    fn class_environment_capture_cell(
        &mut self,
        capture: &ClassEnvironmentCapture,
        function: &mut Function,
    ) -> GcLocal<BindingCell> {
        let schema = self.runtime_schema();
        let frame = self
            .body_entry_locals()
            .and_then(|entry| entry.resume_frame())
            .expect("validated class capture owns its invocation frame");
        // ENVIRONMENT may be inside the class or another suspended block.
        // INVOCATION_ENVIRONMENT is the stable owner of this hidden cell.
        let environment = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<InvocationFrame>()
                .field(InvocationFrameSchema::INVOCATION_ENVIRONMENT)
                .read(frame, schema, function)
                .reference(),
            function,
        );
        let cell = self.emit_environment_cell_local(&environment, capture.slot, function);
        environment.clear(function);
        cell
    }

    pub(super) fn emit_store_class_environment_capture(
        &mut self,
        capture: &ClassEnvironmentCapture,
        environment: &GcLocal<Environment, Nullable>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let cell = self.class_environment_capture_cell(capture, function);
        schema
            .struct_type::<BindingCell>()
            .field(BindingCellSchema::CAPTURED_ENVIRONMENT)
            .write(
                &cell,
                GcOperand::reference(environment, schema),
                schema,
                function,
            );
        cell.clear(function);
    }

    pub(super) fn emit_load_class_environment_capture(
        &mut self,
        capture: &ClassEnvironmentCapture,
        function: &mut Function,
    ) -> GcLocal<Environment, Nullable> {
        let schema = self.runtime_schema();
        let cell = self.class_environment_capture_cell(capture, function);
        let environment = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<BindingCell>()
                .field(BindingCellSchema::CAPTURED_ENVIRONMENT)
                .read(&cell, schema, function)
                .reference(),
            function,
        );
        cell.clear(function);
        environment
    }

    pub(super) fn emit_clear_class_environment_capture(
        &mut self,
        capture: &ClassEnvironmentCapture,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let cell = self.class_environment_capture_cell(capture, function);
        schema
            .struct_type::<BindingCell>()
            .field(BindingCellSchema::CAPTURED_ENVIRONMENT)
            .write(&cell, GcOperand::null(schema), schema, function);
        cell.clear(function);
    }
}
