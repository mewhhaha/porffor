//! A complete With owns one actual object environment through suspension.

use super::*;
use lila_ir::{AsyncFunctionWithIr, AsyncGeneratorWithIr, OrdinaryGeneratorWithIr};

/// These checked source owners share the original physical With lifetime.
/// A foreign execution kind cannot enter the pipeline through any facade.
enum ResumableWith<'a> {
    Generator(&'a OrdinaryGeneratorWithIr),
    Async(&'a AsyncFunctionWithIr),
    AsyncGenerator(&'a AsyncGeneratorWithIr),
}
impl ResumableWith<'_> {
    fn execution_kind(&self) -> FunctionExecutionKind {
        match self {
            Self::Generator(_) => FunctionExecutionKind::Generator,
            Self::Async(_) => FunctionExecutionKind::Async,
            Self::AsyncGenerator(_) => FunctionExecutionKind::AsyncGenerator,
        }
    }
    fn entry_state(&self) -> u32 {
        match self {
            Self::Generator(plan) => plan.entry_state(),
            Self::Async(plan) => plan.entry_state(),
            Self::AsyncGenerator(plan) => plan.entry_state(),
        }
    }
    fn exit_state(&self) -> u32 {
        match self {
            Self::Generator(plan) => plan.exit_state(),
            Self::Async(plan) => plan.exit_state(),
            Self::AsyncGenerator(plan) => plan.exit_state(),
        }
    }
    fn head(&self) -> &BlockIr {
        match self {
            Self::Generator(plan) => plan.head().region().block(),
            Self::Async(plan) => plan.head(),
            Self::AsyncGenerator(plan) => plan.head().region().block(),
        }
    }
    fn head_entry_state(&self) -> u32 {
        match self {
            Self::Generator(plan) => plan.head().region().entry_state(),
            Self::Async(plan) => plan.entry_state(),
            Self::AsyncGenerator(plan) => plan.head().region().entry_state(),
        }
    }
    fn head_end_state(&self) -> u32 {
        match self {
            Self::Generator(plan) => plan.head().region().end_state(),
            Self::Async(plan) => plan.head_ready_state(),
            Self::AsyncGenerator(plan) => plan.head().region().end_state(),
        }
    }
    fn body(&self) -> &BlockIr {
        match self {
            Self::Generator(plan) => plan.body().block(),
            Self::Async(plan) => plan.body(),
            Self::AsyncGenerator(plan) => plan.body().block(),
        }
    }
    fn body_entry_state(&self) -> u32 {
        match self {
            Self::Generator(plan) => plan.body().entry_state(),
            Self::Async(plan) => plan.body_entry_state(),
            Self::AsyncGenerator(plan) => plan.body().entry_state(),
        }
    }
    fn body_end_state(&self) -> u32 {
        match self {
            Self::Generator(plan) => plan.body().end_state(),
            Self::Async(plan) => plan.body_end_state(),
            Self::AsyncGenerator(plan) => plan.body().end_state(),
        }
    }
    fn head_binding(&self) -> &OwnedEnvBindingIr {
        match self {
            Self::Generator(plan) => plan.head_binding(),
            Self::Async(plan) => plan.head_binding(),
            Self::AsyncGenerator(plan) => plan.head_binding(),
        }
    }
    fn object_binding(&self) -> &OwnedEnvBindingIr {
        match self {
            Self::Generator(plan) => plan.object_binding(),
            Self::Async(plan) => plan.object_binding(),
            Self::AsyncGenerator(plan) => plan.object_binding(),
        }
    }
    fn lexical_environment(&self) -> &lila_ir::LexicalEnvironmentIr {
        match self {
            Self::Generator(plan) => plan.lexical_environment(),
            Self::Async(plan) => plan.lexical_environment(),
            Self::AsyncGenerator(plan) => plan.lexical_environment(),
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn compile_ordinary_generator_with(
        &mut self,
        plan: &OrdinaryGeneratorWithIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_with(ResumableWith::Generator(plan), function)
    }

    pub(super) fn compile_async_function_with(
        &mut self,
        plan: &AsyncFunctionWithIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_with(ResumableWith::Async(plan), function)
    }

    pub(super) fn compile_async_generator_with(
        &mut self,
        plan: &AsyncGeneratorWithIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous = self.checked_async_generator_environment_owner;
        self.checked_async_generator_environment_owner =
            Some(CheckedAsyncGeneratorEnvironmentOwner::for_with(plan));
        let result = self.compile_resumable_with(ResumableWith::AsyncGenerator(plan), function);
        self.checked_async_generator_environment_owner = previous;
        result
    }

    fn compile_resumable_with(
        &mut self,
        plan: ResumableWith<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == plan.execution_kind())
            || !self
                .owned_env_bindings
                .iter()
                .any(|binding| binding == plan.head_binding())
            || self.owned_env_slot(&plan.head_binding().name) != Some(plan.head_binding().slot)
        {
            return Err(EmitError::unsupported(
                "compiler invariant: complete With requires its exact resumable head cell",
            ));
        }
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        self.emit_generator_statement_list_entry(plan.entry_state(), function)?;

        self.emit_resumable_state_in_range(
            plan.head_entry_state(),
            plan.head_end_state(),
            true,
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        // Operand captures and yields are not source StatementList results.
        // The existing completion owner suppresses only this actual head.
        self.compile_resumable_operand_region(plan.head(), plan.head_entry_state(), function)?;
        self.emit_set_resumable_resume_point(plan.body_entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_resumable_state_in_range(
            plan.body_entry_state(),
            plan.body_end_state(),
            true,
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        // Allocate at fresh body entry, or reattach the same saved child of
        // the current outer environment. Closures keep the original cells.
        self.emit_enter_resumable_lexical_environment(
            plan.lexical_environment(),
            plan.body_entry_state(),
            function,
        )?;
        let object = plan.object_binding();
        let object_storage = match self.lookup_current_scope_binding(&object.name) {
            Some(storage @ BindingStorage::EnvSlot { slot, hops: 0 }) if slot == object.slot => {
                storage
            }
            Some(BindingStorage::Local(_) | BindingStorage::EnvSlot { .. }) | None => {
                return Err(EmitError::unsupported(
                    "compiler invariant: With object must use its original analyzed child cell",
                ));
            }
        };
        self.emit_resumable_state_equals(plan.body_entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let value = self.runtime_schema().reserve_value_local(function);
        self.read_generator_statement_list_binding(plan.head_binding(), &value, function);
        self.write_binding_from_locals(object_storage, &value, function);
        value.clear(function);
        // With's UpdateEmpty supplies Undefined, never the preceding outer V.
        self.emit_statement_result(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Rebuild before the body can inject a resumed Return or Throw. A
        // nested finally/iterator close runs first; outward transfers meet here.
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(cleanup);
        self.compile_resumable_block_contents(
            plan.body(),
            plan.body_entry_state(),
            true,
            function,
        )?;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.emit_leave_lexical_environment(function);
        self.emit_save_resumable_environment(function)?;
        // Leaving and saving are pure physical operations. Preserve the whole
        // body Completion before its enclosing catch/finally/branch sees it.
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
