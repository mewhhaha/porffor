//! A captured for-await head retains one actual environment per iteration.
use super::*;
use lila_ir::LexicalEnvironmentIr;

#[must_use = "a saved iteration environment must be reattached before its body"]
pub(super) struct SavedForAwaitIterationEnvironment {
    environment: GcLocal<Environment, Nullable>,
    point: I32Local,
    value_resume_state: u32,
}
#[must_use = "an active iteration environment must reach its single cleanup"]
pub(super) struct ActiveForAwaitIterationEnvironment {
    cleanup: ControlTarget,
}

impl FunctionBuilder<'_> {
    pub(super) fn detach_suspended_for_await_iteration_environment(
        &mut self,
        point: I32Local,
        plan: ForAwaitIteratorPlan<'_>,
        function: &mut Function,
    ) -> SavedForAwaitIterationEnvironment {
        let schema = self.runtime_schema();
        let environment = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize_null(schema, function);
        Self::emit_state_in_inclusive_range_i32(
            point,
            plan.value_resume_state() + 1,
            plan.close_resume_state() - 1,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        environment.replace(self.current_environment().load(schema, function), function);
        let parent = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<Environment>()
                .field(EnvironmentSchema::PARENT)
                .read(self.current_environment(), schema, function)
                .reference(),
            function,
        );
        self.replace_current_environment(parent.load(schema, function), function);
        parent.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        SavedForAwaitIterationEnvironment {
            environment,
            point,
            value_resume_state: plan.value_resume_state(),
        }
    }
    pub(super) fn enter_suspended_for_await_iteration_environment(
        &mut self,
        saved: SavedForAwaitIterationEnvironment,
        environment: &LexicalEnvironmentIr,
        function: &mut Function,
    ) -> Result<ActiveForAwaitIterationEnvironment, EmitError> {
        let schema = self.runtime_schema();
        saved.point.load(function);
        function.instruction(&Instruction::I32Const(saved.value_resume_state as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_allocate_lexical_environment_record(environment, function)?;
        function.instruction(&Instruction::Else);
        self.replace_current_environment(saved.environment.load(schema, function), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.begin_existing_lexical_environment_scope(environment);
        saved.environment.clear(function);
        if self.current_function_meta().is_some_and(|meta| {
            meta.protocol().execution_kind() == FunctionExecutionKind::AsyncGenerator
        }) && self
            .async_generator_resume_environment_plan
            .as_ref()
            .is_some_and(|plan| !plan.enclosing_scope_resume_states().is_empty())
        {
            saved.point.load(function);
            function.instruction(&Instruction::I32Const(saved.value_resume_state as i32));
            function.instruction(&Instruction::I32Eq);
            self.emit_checked_async_generator_enclosing_saved_resume(function)?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            // Fresh entry publishes its new record. A checked body resume must
            // keep the deepest saved child until that child is reconstructed.
            self.emit_save_resumable_environment(function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        } else {
            self.emit_save_resumable_environment(function)?;
        }
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(cleanup);
        Ok(ActiveForAwaitIterationEnvironment { cleanup })
    }
    pub(super) fn leave_suspended_for_await_iteration_environment(
        &mut self,
        active: ActiveForAwaitIterationEnvironment,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        assert_eq!(self.finally_stack.pop(), Some(active.cleanup));
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_leave_lexical_environment(function);
        self.emit_save_resumable_environment(function)?;
        self.emit_dispatch_current_completion(function)
    }
}
