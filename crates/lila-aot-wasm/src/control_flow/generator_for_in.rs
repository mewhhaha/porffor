//! Checked Generator, Async and AsyncGenerator ForIn share one native cursor.

use super::*;
use lila_ir::AsyncGeneratorForInIr;

impl FunctionBuilder<'_> {
    pub(super) fn for_in_execution_kind(plan: &AsyncGeneratorForInIr) -> FunctionExecutionKind {
        match plan.execution() {
            lila_ir::ResumableRegionProtocolIr::Generator => FunctionExecutionKind::Generator,
            lila_ir::ResumableRegionProtocolIr::Async => FunctionExecutionKind::Async,
            lila_ir::ResumableRegionProtocolIr::AsyncGenerator => {
                FunctionExecutionKind::AsyncGenerator
            }
        }
    }

    pub(super) fn compile_async_generator_for_in(
        &mut self,
        plan: &AsyncGeneratorForInIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous = self.checked_async_generator_environment_owner;
        if plan.execution() == lila_ir::ResumableRegionProtocolIr::AsyncGenerator {
            self.checked_async_generator_environment_owner =
                Some(CheckedAsyncGeneratorEnvironmentOwner::for_for_in(plan));
        }
        let result = self.compile_resumable_for_in(plan, labels, function);
        self.checked_async_generator_environment_owner = previous;
        result
    }

    fn compile_resumable_for_in(
        &mut self,
        plan: &AsyncGeneratorForInIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.current_function_meta().is_some_and(|meta| {
            meta.protocol().execution_kind() == Self::for_in_execution_kind(plan)
        }) {
            return Err(EmitError::unsupported(
                "compiler invariant: complete ForIn requires its exact resumable execution owner",
            ));
        }
        let storage = self.for_in_enumerator_storage(plan.enumerator_binding())?;
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_checkpoint_generator_statement_list_value(function);
        self.push_scope();
        // This whole scope restores the original outer environment and retires
        // the cursor before an enclosing catch/finally receives any abrupt.
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(cleanup);
        let break_target = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_target);
        self.begin_resumable_iteration_value(plan, break_target)?;
        let schema = self.runtime_schema();
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let initial = schema.reserve_value_local(function);
        initial.set_undefined(function);
        self.write_generator_statement_list_binding(plan.value_binding(), &initial, function);
        self.emit_statement_result(function);
        initial.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Head TDZ records can themselves span Yield or Await. Rebuild their original
        // scope and cleanup destination before the injected completion occurs.
        self.emit_resumable_state_in_range(
            plan.entry_state(),
            plan.head().region().end_state(),
            true,
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        if let Some(environment) = plan
            .lexical_environment()
            .and_then(|e| e.tdz_environment.as_ref())
        {
            self.emit_enter_resumable_lexical_environment(
                environment,
                plan.entry_state(),
                function,
            )?;
        }
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        if let Some(environment) = plan.lexical_environment() {
            for name in &environment.tdz_binding_names {
                let binding = self.lookup_current_scope_binding(name).unwrap_or_else(|| {
                    self.allocate_binding(
                        name.clone(),
                        plan.head_mode(),
                        ValueKind::Dynamic,
                        function,
                    )
                });
                self.initialize_binding_uninitialized(binding, function);
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // The paused Iteration value context makes the entire source head an
        // operand, rather than a series of fictitious StatementList results.
        self.compile_resumable_operand_region(
            plan.head().region().block(),
            plan.entry_state(),
            function,
        )?;
        if plan
            .lexical_environment()
            .and_then(|e| e.tdz_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.emit_save_resumable_environment(function)?;
        let source = schema.reserve_value_local(function);
        self.read_generator_statement_list_binding(plan.head_binding(), &source, function);
        let record = self.emit_create_for_in_enumerator(&source, function)?;
        self.emit_publish_retained_for_in_enumerator(&storage, &record, function);
        record.clear(function);
        source.set_undefined(function);
        self.write_generator_statement_list_binding(plan.head_binding(), &source, function);
        source.clear(function);
        self.emit_set_resumable_resume_point(plan.advance_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let enumeration_loop = self.open_frame(ControlFrameKind::Loop, function);
        self.emit_resumable_state_equals(plan.advance_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let value = schema.reserve_value_local(function);
        self.read_generator_statement_list_binding(plan.value_binding(), &value, function);
        self.completion().set_normal(&value, function);
        value.clear(function);
        let record = self.emit_load_retained_for_in_enumerator(&storage, function);
        let next = schema.reserve_completion(function);
        self.emit_advance_for_in_enumerator(&record, &next, function)?;
        record.clear(function);
        next.value().tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(break_target, function);
        self.write_generator_statement_list_binding(plan.key_binding(), next.value(), function);
        next.clear(function);
        self.emit_set_resumable_resume_point(plan.initialization_region().entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_resumable_state_in_range(
            plan.initialization_region().entry_state(),
            plan.body().end_state(),
            true,
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        if let Some(environment) = plan
            .lexical_environment()
            .and_then(|e| e.iteration_environment.as_ref())
        {
            self.emit_enter_resumable_lexical_environment(
                environment,
                plan.initialization_region().entry_state(),
                function,
            )?;
        }
        self.emit_resumable_state_in_range(
            plan.initialization_region().entry_state(),
            plan.initialization_region().end_state(),
            true,
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        // The original per-key Reference/binding operation has its own mixed
        // range. A resumed initializer keeps this key and this fresh record;
        // a resumed body never advances or assigns again.
        self.compile_resumable_operand_region_in_current_scope(
            plan.initialization(),
            plan.initialization_region().entry_state(),
            function,
            |_, _| Ok(()),
        )?;
        self.emit_set_resumable_resume_point(plan.body().entry_state(), function)?;
        let value = schema.reserve_value_local(function);
        self.read_generator_statement_list_binding(plan.value_binding(), &value, function);
        self.completion().set_normal(&value, function);
        value.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let continue_target = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets {
            continue_frame: continue_target,
        });
        self.push_labels(labels, break_target, Some(continue_target));
        self.activate_generator_statement_list_value();
        self.emit_restore_generator_statement_list_value(function);
        self.compile_resumable_block_contents(
            plan.body().block(),
            plan.body().entry_state(),
            true,
            function,
        )?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        // Normal and the matching Continue keep the exact current body V.
        self.emit_checkpoint_generator_statement_list_value(function);
        if plan
            .lexical_environment()
            .and_then(|e| e.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.emit_save_resumable_environment(function)?;
        self.emit_set_resumable_resume_point(plan.continue_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_branch_to_target(enumeration_loop, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        // Every exit has unwound to this original outer environment. ForIn
        // performs no IteratorClose and publishes no JavaScript iterator.
        self.emit_retire_retained_for_in_enumerator(&storage, function);
        let retired = schema.reserve_value_local(function);
        retired.set_undefined(function);
        for binding in [
            plan.head_binding(),
            plan.key_binding(),
            plan.value_binding(),
        ] {
            self.write_generator_statement_list_binding(binding, &retired, function);
        }
        retired.clear(function);
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.emit_save_resumable_environment(function)?;
        self.end_generator_statement_list_value();
        self.emit_dispatch_current_completion(function)?;
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
