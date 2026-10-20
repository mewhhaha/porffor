use super::*;
use crate::gc_types::{InvocationFrame, InvocationFrameSchema};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_enter_resumable_lexical_environment(
        &mut self,
        environment: &LexicalEnvironmentIr,
        entry_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self
            .current_function_meta()
            .map(|meta| meta.protocol().execution_kind())
        {
            Some(FunctionExecutionKind::Async | FunctionExecutionKind::Generator) => {}
            Some(FunctionExecutionKind::AsyncGenerator)
                if self.checked_async_generator_environment_owner.is_some() => {}
            Some(FunctionExecutionKind::Ordinary | FunctionExecutionKind::AsyncGenerator)
            | None => {
                return Err(EmitError::unsupported(
                    "saved lexical environment requires a plain async function, generator or checked mixed region",
                ));
            }
        }
        let source_scope = self
            .checked_async_generator_environment_owner
            .is_some_and(|owner| !owner.is_region());
        if source_scope {
            self.emit_checked_async_generator_scope_resume(function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.emit_enter_resumable_lexical_environment_record(
                environment,
                entry_state,
                function,
            )?;
            function.instruction(&Instruction::Else);
            // Unscoped Saved-chain states keep their original entry.
            self.emit_allocate_lexical_environment_record(environment, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        } else {
            self.emit_enter_resumable_lexical_environment_record(
                environment,
                entry_state,
                function,
            )?;
        }
        self.begin_existing_lexical_environment_scope(environment);
        Ok(())
    }

    pub(crate) fn emit_enter_checked_async_generator_catch_environment(
        &mut self,
        environment: &LexicalEnvironmentIr,
        entry_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_checked_async_generator_scope_resume(function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_enter_resumable_lexical_environment_record(environment, entry_state, function)?;
        function.instruction(&Instruction::Else);
        // The original unscoped Saved path enters its catch record only on Throw;
        // a normal catch resume already owns the saved lexical chain.
        self.completion().kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_allocate_lexical_environment_record(environment, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.begin_existing_lexical_environment_scope(environment);
        Ok(())
    }

    fn emit_enter_resumable_lexical_environment_record(
        &mut self,
        environment: &LexicalEnvironmentIr,
        entry_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let resume_point = self.emit_resumable_resume_point(function)?;
        resume_point.load(function);
        function.instruction(&Instruction::I32Const(entry_state as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_allocate_lexical_environment_record(environment, function)?;
        function.instruction(&Instruction::Else);
        self.emit_reattach_saved_child_lexical_environment(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(resume_point, function);
        Ok(())
    }

    /// Both checked source scopes and a retained foreign iteration use the
    /// original saved record whose parent is the reconstructed current scope.
    pub(crate) fn emit_reattach_saved_child_lexical_environment(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let entry = self
            .body_entry_locals()
            .expect("a resumable scope owns a callable entry");
        let frame = schema
            .reserve_gc_local::<InvocationFrame, NonNullable>(function)
            .initialize(
                entry
                    .resume_frame()
                    .expect("a resumable scope owns a frame")
                    .load(schema, function),
                function,
            );

        // An inner block may own the saved record. Reattach this block's
        // existing child of the enclosing record before restoring its bindings.
        let saved = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<InvocationFrame>()
                    .field(InvocationFrameSchema::LEXICAL_ENVIRONMENT)
                    .read(&frame, schema, function)
                    .reference(),
                function,
            );
        let parent = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize_null(schema, function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        saved.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        parent.replace(
            schema
                .struct_type::<Environment>()
                .field(EnvironmentSchema::PARENT)
                .read(&saved, schema, function)
                .reference(),
            function,
        );
        parent.load(schema, function);
        self.current_environment().load(schema, function);
        function.instruction(&Instruction::RefEq);
        function.instruction(&Instruction::BrIf(1));
        saved.replace(parent.load(schema, function), function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.replace_current_environment(saved.load(schema, function), function);
        parent.clear(function);
        saved.clear(function);
        frame.clear(function);
        Ok(())
    }
}
