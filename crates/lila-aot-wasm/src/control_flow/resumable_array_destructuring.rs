//! A complete Array-pattern scope closes the same iterator on resumed abrupts.

use super::*;
use lila_ir::{
    ArrayIteratorStorageIr, AsyncFunctionArrayDestructuringIr, AsyncGeneratorArrayDestructuringIr,
    OrdinaryGeneratorArrayDestructuringIr,
};

/// Every checked source carrier consumes one actual retained iterator pipeline.
enum ResumableArrayDestructuring<'a> {
    Generator(&'a OrdinaryGeneratorArrayDestructuringIr),
    Async(&'a AsyncFunctionArrayDestructuringIr),
    AsyncGenerator(&'a AsyncGeneratorArrayDestructuringIr),
}

impl ResumableArrayDestructuring<'_> {
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

    fn body_entry_state(&self) -> u32 {
        match self {
            Self::Generator(plan) => plan.body().entry_state(),
            Self::Async(plan) => plan.body_entry_state(),
            Self::AsyncGenerator(plan) => plan.body_entry_state(),
        }
    }

    fn exit_state(&self) -> u32 {
        match self {
            Self::Generator(plan) => plan.exit_state(),
            Self::Async(plan) => plan.exit_state(),
            Self::AsyncGenerator(plan) => plan.exit_state(),
        }
    }

    fn raw_source(&self) -> &TypedExpr {
        match self {
            Self::Generator(plan) => plan.raw_source(),
            Self::Async(plan) => plan.raw_source(),
            Self::AsyncGenerator(plan) => plan.raw_source(),
        }
    }

    fn storage(&self) -> &ArrayIteratorStorageIr {
        match self {
            Self::Generator(plan) => plan.storage(),
            Self::Async(plan) => plan.storage(),
            Self::AsyncGenerator(plan) => plan.storage(),
        }
    }

    fn body(&self) -> &BlockIr {
        match self {
            Self::Generator(plan) => plan.body().block(),
            Self::Async(plan) => plan.body(),
            Self::AsyncGenerator(plan) => plan.body().block(),
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn compile_ordinary_generator_array_destructuring(
        &mut self,
        plan: &OrdinaryGeneratorArrayDestructuringIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_array_destructuring(
            ResumableArrayDestructuring::Generator(plan),
            function,
        )
    }

    pub(super) fn compile_async_function_array_destructuring(
        &mut self,
        plan: &AsyncFunctionArrayDestructuringIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_array_destructuring(
            ResumableArrayDestructuring::Async(plan),
            function,
        )
    }

    pub(super) fn compile_async_generator_array_destructuring(
        &mut self,
        plan: &AsyncGeneratorArrayDestructuringIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous = self.checked_async_generator_environment_owner;
        self.checked_async_generator_environment_owner =
            Some(CheckedAsyncGeneratorEnvironmentOwner::for_array_destructuring(plan));
        let result = self.compile_resumable_array_destructuring(
            ResumableArrayDestructuring::AsyncGenerator(plan),
            function,
        );
        self.checked_async_generator_environment_owner = previous;
        result
    }

    fn compile_resumable_array_destructuring(
        &mut self,
        plan: ResumableArrayDestructuring<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == plan.execution_kind())
        {
            return Err(EmitError::unsupported(
                "compiler invariant: retained array owner must match its actual execution kind",
            ));
        }
        let storage = self.array_iterator_storage(plan.storage())?;
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        self.emit_generator_statement_list_entry(plan.entry_state(), function)?;
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let schema = self.runtime_schema();
        let saved = self.save_statement_list_value(function);
        let source = schema.reserve_value_local(function);
        self.compile_expr_to_value(plan.raw_source(), &source, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        // Acquisition is outside this pattern's own close scope. An enclosing
        // pattern remains active and therefore owns an acquisition failure.
        let acquired = self.emit_get_sync_iterator(
            &source,
            SyncIteratorConsumer::ArrayDestructuring,
            function,
        )?;
        self.emit_publish_retained_array_iterator(&storage, acquired.record(), function);
        acquired.clear(function);
        source.clear(function);
        self.restore_statement_list_value(saved, function)?;
        self.emit_set_resumable_resume_point(plan.body_entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Fresh and resumed execution use the exact same retained record.
        // The close destination exists before a resumed Yield or rejected
        // Await completion is injected into the body.
        let iterator = OwnedSyncIterator {
            record: self.emit_load_retained_array_iterator(&storage, function),
            consumer: SyncIteratorConsumer::ArrayDestructuring,
        };
        let pending = schema.reserve_completion(function);
        let closed = schema.reserve_completion(function);
        let close = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(close);
        self.compile_resumable_block_contents(
            plan.body(),
            plan.body_entry_state(),
            true,
            function,
        )?;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        pending.copy_from(self.completion(), function);
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        // Step/protocol/value errors already set DONE in the shared native
        // step owner. Target/default/Put and injected Return/Throw leave it
        // false. Existing IteratorClose preserves the complete pending record.
        self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?;
        self.completion().copy_from(&closed, function);
        self.emit_retire_retained_array_iterator(&storage, function);
        closed.clear(function);
        pending.clear(function);
        iterator.clear(function);
        // The own finalizer has been popped; nested abrupts now reach the next
        // enclosing close scope. Pending Yield and Await exited before this
        // point, retaining the record for the next actual invocation.
        self.emit_dispatch_current_completion(function)?;
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
