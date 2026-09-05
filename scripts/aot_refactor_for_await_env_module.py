from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(text.replace(old, new, 1))


parent = "crates/lila-aot-wasm/src/control_flow.rs"

replace_once(
    parent,
    "mod async_function_for_of_iterator;\nmod for_await_iterator_symbol;\n",
    "mod async_function_for_of_iterator;\nmod for_await_iteration_environment;\nmod for_await_iterator_symbol;\n",
)

replace_once(
    parent,
    '''
    const fn lexical_environment_offset(&self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_ENV_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET,
        }
    }
''',
    "\n",
)

replace_once(
    parent,
    '''        // `done` is read from the loop's own suspension-owned binding, not from
        // a local, so this test is meaningful on a body resume too: the
        // `next()` that produced the in-flight iteration wrote `false` there,
        // and the iteration is not finished, so the loop correctly does not
        // break out from under a half-run body.
        if body_suspends && iteration_environment.is_some() {
            // A body-resume invocation already owns an in-flight iteration, so
            // `done` is known false. Reading the parent-owned slot before the
            // child environment is attached would use the wrong runtime base;
            // perform the observable done test only on the value-resume path.
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.read_binding_to_locals(
                done_storage,
                done_payload_local,
                done_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(done_payload_local));
            function.instruction(&Instruction::I32WrapI64);
            function.branch_if_to_label(break_frame.label);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        } else {
            self.read_binding_to_locals(
                done_storage,
                done_payload_local,
                done_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(done_payload_local));
            function.instruction(&Instruction::I32WrapI64);
            function.branch_if_to_label(break_frame.label);
        }

        // A captured `let`/`const` head owns one fresh Environment Record per
        // iteration. The value-resume invocation allocates it and publishes its
        // exact pointer in the activation. A body-resume invocation starts with
        // that pointer already restored by function entry, so both runtime arms
        // converge before the compiler attaches one binding view.
        let iteration_cleanup_frame = if body_suspends && iteration_environment.is_some() {
            Some(self.open_frame(ControlFrameKind::Block, function))
        } else {
            None
        };
        if let Some(environment) = iteration_environment {
            if body_suspends {
                function.instruction(&Instruction::LocalGet(state_local));
                function.instruction(&Instruction::I64Const(async_plan.value_resume_state as i64));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_allocate_lexical_environment_record(environment, function)?;
                self.store_i64_local_at_offset(
                    activation_local,
                    activation_environment_offset,
                    self.current_env_local,
                    function,
                );
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.push_scope();
                self.begin_existing_lexical_environment_scope(environment);
                self.finally_stack.push(ControlTarget {
                    environment_depth: self.environment_depth,
                    ..iteration_cleanup_frame
                        .expect("resumable iteration environment needs a cleanup frame")
                });
            } else {
                self.emit_enter_lexical_environment(environment, function)?;
            }
        }
''',
    '''        self.emit_for_await_iteration_done_check(
            body_suspends,
            iteration_environment.is_some(),
            state_local,
            async_plan.value_resume_state,
            done_storage,
            done_payload_local,
            done_tag_local,
            break_frame,
            function,
        )?;
        self.emit_enter_for_await_iteration_environment(
            iteration_environment,
            body_suspends,
            activation_local,
            activation_environment_offset,
            state_local,
            async_plan.value_resume_state,
            function,
        )?;
''',
)

replace_once(
    parent,
    '''        self.compile_statement(body, function)?;
        if body_suspends && iteration_environment.is_some() {
            self.finally_stack.pop();
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
            self.emit_leave_lexical_environment(function);
            self.pop_scope();
            self.store_i64_local_at_offset(
                activation_local,
                activation_environment_offset,
                self.current_env_local,
                function,
            );
            // Normal fallthrough continues into the ordinary iterator-close
            // decision. Abrupt completions are routed only after the parent
            // environment is again authoritative in the activation.
            self.emit_dispatch_async_completion(function)?;
        }
''',
    '''        self.compile_statement(body, function)?;
        self.emit_leave_for_await_iteration_environment(
            iteration_environment.is_some(),
            body_suspends,
            activation_local,
            activation_environment_offset,
            function,
        )?;
''',
)

module = Path("crates/lila-aot-wasm/src/control_flow/for_await_iteration_environment.rs")
if module.exists():
    raise SystemExit(f"{module}: already exists")
module.write_text(r'''use super::*;

impl ForAwaitActivationLayout {
    pub(super) const fn lexical_environment_offset(&self) -> u64 {
        match self {
            Self::AsyncFunction => HEAP_ASYNC_ENV_OFFSET,
            Self::AsyncGenerator => HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_for_await_iteration_done_check(
        &mut self,
        body_suspends: bool,
        iteration_environment_present: bool,
        state_local: u32,
        value_resume_state: u32,
        done_storage: BindingStorage,
        done_payload_local: u32,
        done_tag_local: u32,
        break_frame: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_path_only = body_suspends && iteration_environment_present;
        if value_path_only {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(value_resume_state as i64));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
        }
        self.read_binding_to_locals(
            done_storage,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(done_payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.branch_if_to_label(break_frame.label);
        if value_path_only {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    pub(super) fn emit_enter_for_await_iteration_environment(
        &mut self,
        environment: Option<&LexicalEnvironmentIr>,
        body_suspends: bool,
        activation_local: u32,
        activation_environment_offset: u64,
        state_local: u32,
        value_resume_state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(environment) = environment else {
            return Ok(());
        };
        if !body_suspends {
            return self.emit_enter_lexical_environment(environment, function);
        }

        let cleanup_frame = self.open_frame(ControlFrameKind::Block, function);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(value_resume_state as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_allocate_lexical_environment_record(environment, function)?;
        self.store_i64_local_at_offset(
            activation_local,
            activation_environment_offset,
            self.current_env_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.push_scope();
        self.begin_existing_lexical_environment_scope(environment);
        self.finally_stack.push(ControlTarget {
            environment_depth: self.environment_depth,
            ..cleanup_frame
        });
        Ok(())
    }

    pub(super) fn emit_leave_for_await_iteration_environment(
        &mut self,
        iteration_environment_present: bool,
        body_suspends: bool,
        activation_local: u32,
        activation_environment_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !body_suspends || !iteration_environment_present {
            return Ok(());
        }
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_leave_lexical_environment(function);
        self.pop_scope();
        self.store_i64_local_at_offset(
            activation_local,
            activation_environment_offset,
            self.current_env_local,
            function,
        );
        self.emit_dispatch_async_completion(function)
    }
}
''')
