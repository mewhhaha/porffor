//! Statement gates consume the actual typed body activation.

use super::*;
use crate::function_entry::ResumableEntryLocals;
use crate::gc_types::{
    AsyncActivation, AsyncActivationSchema, AsyncGeneratorActivation,
    AsyncGeneratorActivationSchema, GeneratorActivation, GeneratorActivationSchema, I32Local,
};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_resumable_resume_point(
        &self,
        function: &mut Function,
    ) -> Result<I32Local, EmitError> {
        let schema = self.runtime_schema();
        let activation = self
            .body_entry_locals()
            .and_then(|entry| entry.resume_activation())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant: resumable statement has no body activation",
                )
            })?;
        let point = schema.reserve_i32_local(function);
        match activation {
            ResumableEntryLocals::Generator(activation) => schema
                .struct_type::<GeneratorActivation>()
                .field(GeneratorActivationSchema::RESUME_POINT)
                .read(activation, schema, function)
                .store(point, function),
            ResumableEntryLocals::Async(activation) => schema
                .struct_type::<AsyncActivation>()
                .field(AsyncActivationSchema::RESUME_POINT)
                .read(activation, schema, function)
                .store(point, function),
            ResumableEntryLocals::AsyncGenerator(activation) => schema
                .struct_type::<AsyncGeneratorActivation>()
                .field(AsyncGeneratorActivationSchema::RESUME_POINT)
                .read(activation, schema, function)
                .store(point, function),
        }
        Ok(point)
    }

    pub(crate) fn compile_resumable_statement_sequence(
        &mut self,
        statements: &[StatementIr],
        entry_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let generator = match self
            .body_entry_locals()
            .and_then(|entry| entry.resume_activation())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant: resumable sequence has no source activation",
                )
            })? {
            ResumableEntryLocals::Generator(_) => true,
            ResumableEntryLocals::Async(_) | ResumableEntryLocals::AsyncGenerator(_) => false,
        };
        let schema = self.runtime_schema();
        self.emit_generator_statement_list_entry(entry_state, function)?;
        let mut segment_state = entry_state;
        for statement in statements {
            let (statement_entry, statement_exit) = if generator {
                (
                    Self::generator_statement_entry_state(statement),
                    Self::generator_statement_exit_state(statement),
                )
            } else {
                (
                    Self::async_statement_entry_state(statement),
                    Self::async_statement_exit_state(statement),
                )
            };
            if let Some(exit) = statement_exit {
                if statement_entry != Some(segment_state) {
                    return Err(EmitError::unsupported(
                        "compiler invariant: resumable statement does not continue its preceding segment"));
                }
                self.compile_statement(statement, function)?;
                segment_state = exit;
            } else {
                let point = self.emit_resumable_resume_point(function)?;
                point.load(function);
                function.instruction(&Instruction::I32Const(segment_state as i32));
                function.instruction(&Instruction::I32Eq);
                schema.release_i32_local(point, function);
                self.open_frame(ControlFrameKind::If, function);
                self.compile_statement(statement, function)?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        Ok(())
    }
}
