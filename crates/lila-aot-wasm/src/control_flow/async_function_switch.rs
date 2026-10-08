use super::*;
use lila_ir::{AsyncFunctionSwitchIr, AsyncFunctionSwitchSelectorIr};

impl FunctionBuilder<'_> {
    pub(super) fn compile_async_function_switch(
        &mut self,
        plan: &AsyncFunctionSwitchIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == FunctionExecutionKind::Async)
        {
            return Err(EmitError::unsupported(
                "async switch requires its plain async activation",
            ));
        }
        let schema = self.runtime_schema();
        let discriminant = schema.reserve_value_local(function);
        let selected = schema.reserve_i32_local(function);
        let selected_state = schema.reserve_i32_local(function);
        self.emit_async_state_in_range(plan.entry_state(), plan.exit_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        // Initial discrimination is outside the shared CaseBlock environment.
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.compile_expr_to_value(plan.discriminant(), &discriminant, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.push_scope();
        if let Some(environment) = plan.lexical_environment() {
            self.emit_enter_resumable_lexical_environment(
                environment,
                plan.entry_state(),
                function,
            )?;
        }
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        for case in plan.cases() {
            self.initialize_direct_lexical_bindings(&case.body().statements, function);
        }
        for declaration in plan.lexical_declarations() {
            self.compile_statement(declaration, function)?;
        }
        if plan.selection_fallback_state().is_some() {
            self.emit_set_resumable_resume_point(plan.entry_state() + 1, function)?;
        } else {
            function.instruction(&Instruction::I32Const(-1));
            selected.store(function);
            // Default is committed only after every preceding/following selector
            // failed; later selector effects stop immediately after a match.
            for (index, case) in plan.cases().iter().enumerate() {
                let Some(condition) = case.condition() else {
                    continue;
                };
                selected.load(function);
                function.instruction(&Instruction::I32Const(-1));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                self.compile_switch_case_match(&discriminant, condition, function)?;
                self.open_frame(ControlFrameKind::If, function);
                function.instruction(&Instruction::I32Const(
                    i32::try_from(index)
                        .map_err(|_| EmitError::unsupported("too many switch cases"))?,
                ));
                selected.store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            if let Some(index) = plan
                .cases()
                .iter()
                .position(|case| case.condition().is_none())
            {
                selected.load(function);
                function.instruction(&Instruction::I32Const(-1));
                function.instruction(&Instruction::I32Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I32Const(
                    i32::try_from(index)
                        .map_err(|_| EmitError::unsupported("too many switch cases"))?,
                ));
                selected.store(function);
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::I32Const(plan.exit_state() as i32));
            selected_state.store(function);
            for (index, case) in plan.cases().iter().enumerate() {
                selected.load(function);
                function.instruction(&Instruction::I32Const(
                    i32::try_from(index)
                        .map_err(|_| EmitError::unsupported("too many switch cases"))?,
                ));
                function.instruction(&Instruction::I32Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I32Const(case.entry_state() as i32));
                selected_state.store(function);
                function.instruction(&Instruction::End);
            }
            self.emit_set_resumable_resume_point_local(selected_state, function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if let Some(fallback_state) = plan.selection_fallback_state() {
            for case in plan.cases() {
                let owner = match case.selector() {
                    Some(AsyncFunctionSwitchSelectorIr::Resumable(owner)) => owner,
                    Some(AsyncFunctionSwitchSelectorIr::Eager(_)) => {
                        return Err(EmitError::unsupported(
                            "unowned selector in retained switch selection",
                        ))
                    }
                    None => continue,
                };
                self.emit_async_state_in_range(owner.entry_state(), owner.next_state(), function)?;
                self.open_frame(ControlFrameKind::If, function);
                self.compile_resumable_statement_sequence(
                    owner.prefix(),
                    owner.entry_state(),
                    function,
                )?;
                self.emit_resumable_state_equals(owner.ready_state(), function)?;
                self.open_frame(ControlFrameKind::If, function);
                // Reload the retained discriminant only after this prefix completes.
                self.compile_expr_to_value(plan.discriminant(), &discriminant, function)?;
                self.emit_propagate_current_throw_if_needed(function);
                self.compile_switch_case_match(&discriminant, owner.condition(), function)?;
                self.open_frame(ControlFrameKind::If, function);
                self.emit_set_resumable_resume_point(case.entry_state(), function)?;
                function.instruction(&Instruction::Else);
                self.emit_set_resumable_resume_point(owner.next_state(), function)?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            self.emit_resumable_state_equals(fallback_state, function)?;
            self.open_frame(ControlFrameKind::If, function);
            let target = plan
                .cases()
                .iter()
                .find(|case| case.selector().is_none())
                .map_or(plan.exit_state(), |case| case.entry_state());
            self.emit_set_resumable_resume_point(target, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_statement_result(function);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        self.push_labels(labels, break_frame, None);
        for case in plan.cases() {
            self.emit_async_state_in_range(case.entry_state(), case.next_entry_state(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.compile_resumable_statement_sequence(
                &case.body().statements,
                case.entry_state(),
                function,
            )?;
            self.emit_set_resumable_resume_point(case.next_entry_state(), function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_labels(labels.len());
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        if plan.lexical_environment().is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i32_local(selected_state, function);
        schema.release_i32_local(selected, function);
        discriminant.clear(function);
        Ok(())
    }
}
