//! The captured for-await head has one environment per iteration, not per
//! invocation of its resumable body. These consuming carriers keep reattachment
//! and cleanup paired without adding another activation layout or binding cell.
use super::*;
use lila_ir::LexicalEnvironmentIr;

#[must_use = "a saved iteration environment must be reattached before its body"]
pub(super) struct SavedForAwaitIterationEnvironment {
    environment_local: u32,
    state_local: u32,
    value_resume_state: u32,
    activation_local: u32,
    environment_offset: u64,
}

#[must_use = "an active iteration environment must reach its single cleanup"]
pub(super) struct ActiveForAwaitIterationEnvironment {
    cleanup: ControlTarget,
    activation_local: u32,
    environment_offset: u64,
}

impl FunctionBuilder<'_> {
    pub(super) fn detach_suspended_for_await_iteration_environment(
        &mut self,
        state_local: u32,
        plan: &AsyncForOfIteratorPlanIr,
        activation_local: u32,
        layout: &ForAwaitActivationLayout,
        function: &mut Function,
    ) -> SavedForAwaitIterationEnvironment {
        let environment_local = self.reserve_temp_local();
        // Entry and await-next/close resumes already carry the parent. Only a
        // body resume carries a child, published before the preceding yield.
        Self::emit_state_in_inclusive_range_i32(
            state_local,
            plan.value_resume_state + 1,
            plan.close_resume_state - 1,
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(environment_local));
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            self.current_env_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // The activation continues rooting the child while iterator bookkeeping
        // uses the parent's layout. No user-body instruction runs detached.
        SavedForAwaitIterationEnvironment {
            environment_local,
            state_local,
            value_resume_state: plan.value_resume_state,
            activation_local,
            environment_offset: layout.environment_offset(),
        }
    }

    pub(super) fn enter_suspended_for_await_iteration_environment(
        &mut self,
        saved: SavedForAwaitIterationEnvironment,
        environment: &LexicalEnvironmentIr,
        function: &mut Function,
    ) -> Result<ActiveForAwaitIterationEnvironment, EmitError> {
        function.instruction(&Instruction::LocalGet(saved.state_local));
        function.instruction(&Instruction::I64Const(i64::from(saved.value_resume_state)));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        // This registers the binding layout once at compile time. Both runtime
        // arms finish with exactly that layout, but the resume arm never creates
        // or initializes a second cell behind pre-suspension closures.
        self.emit_enter_lexical_environment(environment, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(saved.environment_local));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.release_temp_local(saved.environment_local);
        self.store_i64_local_at_offset(
            saved.activation_local,
            saved.environment_offset,
            self.current_env_local,
            function,
        );
        // The child depth is intentional: an abrupt branch must not unwind the
        // child on its way to the cleanup that owns that same leave.
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(cleanup);
        Ok(ActiveForAwaitIterationEnvironment {
            cleanup,
            activation_local: saved.activation_local,
            environment_offset: saved.environment_offset,
        })
    }

    pub(super) fn leave_suspended_for_await_iteration_environment(
        &mut self,
        active: ActiveForAwaitIterationEnvironment,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        assert_eq!(self.finally_stack.pop(), Some(active.cleanup));
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // Yield/await returned directly with the child still rooted. Normal and
        // abrupt exits converge here, leave once, and publish the parent before
        // IteratorClose (which can itself suspend) or another next() request.
        self.emit_leave_lexical_environment(function);
        self.store_i64_local_at_offset(
            active.activation_local,
            active.environment_offset,
            self.current_env_local,
            function,
        );
        self.emit_dispatch_current_completion(function)
    }
}
