use super::*;
use crate::emit::ControlTarget;
use porffor_ir::ObjectDestructuringPatternIr;

fn innermost_target(left: ControlTarget, right: ControlTarget) -> ControlTarget {
    if left.frame >= right.frame {
        left
    } else {
        right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn innermost_target_uses_the_later_control_frame() {
        let outer = ControlTarget {
            frame: 2,
            environment_depth: 1,
        };
        let inner = ControlTarget {
            frame: 5,
            environment_depth: 3,
        };

        assert_eq!(innermost_target(outer, inner), inner);
        assert_eq!(innermost_target(inner, outer), inner);
    }

    #[test]
    fn environment_hops_is_the_depth_difference() {
        assert_eq!(environment_hops(0, 0), 0);
        assert_eq!(environment_hops(4, 1), 3);
    }

    #[test]
    #[should_panic(expected = "control target environment must enclose the current environment")]
    fn environment_hops_rejects_a_deeper_target() {
        environment_hops(1, 2);
    }

    #[test]
    fn finalizer_crosses_only_branches_to_outer_frames() {
        let finalizer = ControlTarget {
            frame: 4,
            environment_depth: 0,
        };
        let outer_branch = ControlTarget {
            frame: 1,
            environment_depth: 0,
        };
        let inner_branch = ControlTarget {
            frame: 6,
            environment_depth: 0,
        };

        assert!(finalizer_crosses_branch(finalizer, outer_branch));
        assert!(!finalizer_crosses_branch(finalizer, finalizer));
        assert!(!finalizer_crosses_branch(finalizer, inner_branch));
    }
}

fn environment_hops(current_depth: u32, target_depth: u32) -> u32 {
    current_depth
        .checked_sub(target_depth)
        .expect("control target environment must enclose the current environment")
}

fn finalizer_crosses_branch(finalizer: ControlTarget, branch_target: ControlTarget) -> bool {
    finalizer.frame > branch_target.frame
}

fn iteration_environment_owns_binding(
    lexical_environment: Option<&ForInOfEnvironmentIr>,
    name: &str,
) -> bool {
    lexical_environment
        .and_then(|environment| environment.iteration_environment.as_ref())
        .is_some_and(|environment| {
            environment
                .bindings
                .iter()
                .any(|binding| binding.name == name)
        })
}

#[derive(Clone, Copy)]
struct DestructuringIteratorLocals {
    iterator_payload: u32,
    iterator_tag: u32,
    next_payload: u32,
    next_tag: u32,
    key: u32,
    result_payload: u32,
    result_tag: u32,
    done_payload: u32,
    done_tag: u32,
    value_payload: u32,
    value_tag: u32,
    return_payload: u32,
    return_tag: u32,
    done: u32,
    close_saved_payload: u32,
    close_saved_tag: u32,
    close_saved_completion: u32,
    close_saved_aux: u32,
}

#[derive(Clone, Copy)]
enum DestructuringIteratorStepKind {
    Elision,
    Value,
}

enum PreparedDestructuringTarget {
    Direct,
    Property {
        target: TypedExpr,
        target_payload: u32,
        target_tag: u32,
        key: DestructuringPropertyKeyIr,
        key_payload: Option<u32>,
        key_tag: Option<u32>,
    },
    Private {
        target_payload: u32,
        target_tag: u32,
        private_name_id: PrivateNameId,
    },
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_statement_result(&self, function: &mut Function, kind: ValueKind) {
        self.emit_undefined_payload(function);
        self.finish_statement_payload(function, kind);
    }

    pub(crate) fn finish_statement_payload(&self, function: &mut Function, kind: ValueKind) {
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
    }

    pub(crate) fn emit_undefined_payload(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
    }

    pub(crate) fn save_current_completion(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalSet(completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(aux_local));
    }

    pub(crate) fn restore_saved_completion(
        &self,
        payload_local: u32,
        tag_local: u32,
        completion_local: u32,
        aux_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalGet(aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    pub(crate) fn set_completion_kind(&self, kind: CompletionKind, function: &mut Function) {
        function.instruction(&Instruction::I64Const(kind.code()));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    pub(crate) fn set_completion_kind_with_aux(
        &self,
        kind: CompletionKind,
        aux: i64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(kind.code()));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::I64Const(aux));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
    }

    pub(crate) fn emit_return_current_completion(&self, function: &mut Function) {
        for _ in 0..self.environment_depth {
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                ENV_PARENT_OFFSET,
                self.current_env_local,
                function,
            );
        }
        match self.return_abi {
            ReturnAbi::MainExport => {
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(RESULT_TAG_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(COMPLETION_KIND_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(COMPLETION_AUX_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::Return);
            }
            ReturnAbi::MultiValue => {
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
                function.instruction(&Instruction::Return);
            }
        }
    }

    pub(crate) fn emit_return_current_completion_if_throw(&mut self, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_propagate_current_completion_if_throw(&mut self, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_propagate_current_throw(&self, function: &mut Function) {
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, 0, function);
        } else {
            self.emit_return_current_completion(function);
        }
    }

    pub(crate) fn emit_break_current_completion_if_throw(
        &self,
        depth: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(depth));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_throw_from_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        Ok(())
    }

    pub(crate) fn emit_propagate_throw_from_locals_if_needed(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            0,
            function,
        )
    }

    pub(crate) fn emit_propagate_throw_from_locals_if_needed_with_extra_depth(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        extra_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_from_locals(payload_local, tag_local, function)?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, 1 + extra_depth, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_resume_after_finally(
        &mut self,
        saved_payload_local: u32,
        saved_tag_local: u32,
        saved_completion_local: u32,
        saved_aux_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
        function.instruction(&Instruction::Else);
        self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_dispatch_branch_completion(
        &self,
        targets: &[(u32, ControlTarget)],
        extra_depth: u32,
        function: &mut Function,
    ) {
        for (target_id, branch_target) in targets {
            function.instruction(&Instruction::LocalGet(self.completion_aux_local));
            function.instruction(&Instruction::I64Const(*target_id as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            let target = self
                .active_finally_target_for_branch(*branch_target)
                .unwrap_or(*branch_target);
            self.emit_branch_to_target(target, extra_depth, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
    }

    pub(crate) fn active_break_targets(&self) -> Vec<(u32, ControlTarget)> {
        let mut targets = Vec::new();
        for target in self.breakable_stack.iter().rev() {
            let target_id = target.frame as u32;
            if !targets.iter().any(|(id, _)| *id == target_id) {
                targets.push((target_id, *target));
            }
        }
        targets
    }

    pub(crate) fn active_continue_targets(&self) -> Vec<(u32, ControlTarget)> {
        let mut targets = Vec::new();
        for target in self.loop_stack.iter().rev() {
            let target_id = target.continue_frame.frame as u32;
            if !targets.iter().any(|(id, _)| *id == target_id) {
                targets.push((target_id, target.continue_frame));
            }
        }
        targets
    }

    pub(crate) fn active_throw_target(&self) -> Option<ControlTarget> {
        match (self.throw_handler_stack.last(), self.finally_stack.last()) {
            (Some(handler), Some(finalizer)) => Some(innermost_target(*handler, *finalizer)),
            (Some(handler), None) => Some(*handler),
            (None, Some(finalizer)) => Some(*finalizer),
            (None, None) => None,
        }
    }

    fn active_finally_target_for_branch(
        &self,
        branch_target: ControlTarget,
    ) -> Option<ControlTarget> {
        self.finally_stack
            .last()
            .copied()
            .filter(|finalizer| finalizer_crosses_branch(*finalizer, branch_target))
    }

    pub(crate) fn emit_dispatch_current_completion(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_dispatch_current_completion_with_extra_depth(0, function)
    }

    pub(crate) fn emit_dispatch_current_completion_with_extra_depth(
        &mut self,
        extra_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, 1 + extra_depth, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.finally_stack.last().copied() {
            self.emit_branch_to_target(target, 2 + extra_depth, function);
        } else {
            self.normalize_derived_constructor_result(function)?;
            self.set_completion_kind(CompletionKind::Normal, function);
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_BREAK));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let targets = self.active_break_targets();
        self.emit_dispatch_branch_completion(&targets, 4 + extra_depth, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let targets = self.active_continue_targets();
        self.emit_dispatch_branch_completion(&targets, 5 + extra_depth, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn push_control(&mut self, kind: ControlFrameKind) -> ControlTarget {
        let target = ControlTarget {
            frame: self.control_stack.len(),
            environment_depth: self.environment_depth,
        };
        self.control_stack.push(kind);
        target
    }

    pub(crate) fn pop_control(&mut self, expected: ControlFrameKind) {
        let actual = self
            .control_stack
            .pop()
            .expect("control stack must not underflow");
        assert!(matches!(
            (actual, expected),
            (ControlFrameKind::If, ControlFrameKind::If)
                | (ControlFrameKind::Block, ControlFrameKind::Block)
                | (ControlFrameKind::Loop, ControlFrameKind::Loop)
        ));
    }

    pub(crate) fn depth_to(&self, target: ControlTarget) -> u32 {
        (self.control_stack.len() - 1 - target.frame) as u32
    }

    fn emit_unwind_environments_to_target(&self, target: ControlTarget, function: &mut Function) {
        for _ in 0..environment_hops(self.environment_depth, target.environment_depth) {
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                ENV_PARENT_OFFSET,
                self.current_env_local,
                function,
            );
        }
    }

    pub(crate) fn emit_branch_to_target(
        &self,
        target: ControlTarget,
        extra_depth: u32,
        function: &mut Function,
    ) {
        self.emit_unwind_environments_to_target(target, function);
        function.instruction(&Instruction::Br(self.depth_to(target) + extra_depth));
    }

    pub(crate) fn emit_branch_if_to_target(
        &self,
        target: ControlTarget,
        extra_depth: u32,
        function: &mut Function,
    ) {
        if target.environment_depth == self.environment_depth {
            function.instruction(&Instruction::BrIf(self.depth_to(target) + extra_depth));
            return;
        }

        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_branch_to_target(target, extra_depth + 1, function);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn push_labels(
        &mut self,
        labels: &[String],
        break_frame: ControlTarget,
        continue_frame: Option<ControlTarget>,
    ) {
        for label in labels {
            self.label_stack.push(LabelTargets {
                name: label.clone(),
                break_frame,
                continue_frame,
            });
        }
    }

    pub(crate) fn pop_labels(&mut self, count: usize) {
        for _ in 0..count {
            self.label_stack.pop();
        }
    }

    pub(crate) fn compile_block_contents(
        &mut self,
        block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(environment) = &block.lexical_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        self.initialize_direct_lexical_bindings(&block.statements, function);
        if block.statements.is_empty() {
            self.emit_statement_result(function, ValueKind::Undefined);
            if block.lexical_environment.is_some() {
                self.emit_leave_lexical_environment(function);
            }
            return Ok(());
        }

        for statement in &block.statements {
            self.compile_statement(statement, function)?;
        }

        if block.lexical_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }

        Ok(())
    }

    fn initialize_direct_lexical_bindings(
        &mut self,
        statements: &[StatementIr],
        function: &mut Function,
    ) {
        for statement in statements {
            match statement {
                StatementIr::Lexical { mode, name, init } => {
                    let storage = self
                        .lookup_current_scope_binding(name)
                        .or_else(|| self.lookup_binding(name))
                        .unwrap_or_else(|| self.allocate_binding(name.clone(), *mode, init.kind));
                    self.initialize_binding_uninitialized(storage, function);
                }
                StatementIr::LexicalBlock(statements) => {
                    self.initialize_direct_lexical_bindings(statements, function);
                }
                StatementIr::Expression(TypedExpr {
                    expr:
                        ExprIr::ArrayDestructure {
                            pattern,
                            assignment: false,
                            ..
                        },
                    ..
                }) => {
                    pattern.visit_bindings(&mut |mode, name| {
                        if mode == BindingMode::Var {
                            return;
                        }
                        let storage = self
                            .lookup_current_scope_binding(name)
                            .or_else(|| self.lookup_binding(name))
                            .unwrap_or_else(|| {
                                self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic)
                            });
                        self.initialize_binding_uninitialized(storage, function);
                    });
                }
                _ => {}
            }
        }
    }

    pub(crate) fn compile_statement(
        &mut self,
        statement: &StatementIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match statement {
            StatementIr::Empty => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Lexical { mode, name, init } => {
                let storage = self
                    .lookup_current_scope_binding(name)
                    .or_else(|| self.lookup_binding(name))
                    .unwrap_or_else(|| self.allocate_binding(name.clone(), *mode, init.kind));
                self.initialize_binding_uninitialized(storage, function);
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(init, value_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(value_local, tag_local, function)?;
                self.write_binding_from_locals(storage, value_local, tag_local, function);
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::AnnexBFunctionCopy {
                source_name,
                block_storage_name,
                variable_storage_name,
            } => {
                let source = self.lookup_binding(block_storage_name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "Annex B declaration `{source_name}` is missing block binding `{block_storage_name}`"
                    ))
                })?;
                let target = self
                    .lookup_owner_binding(variable_storage_name)
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "Annex B declaration `{source_name}` is missing owner binding `{variable_storage_name}`"
                        ))
                    })?;
                self.read_binding_to_locals(
                    source,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.write_binding_from_locals(
                    target,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
                self.mirror_binding_to_global_object(variable_storage_name, target, function)?;
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Expression(expr) => {
                if !expr.possible_kinds.is_singleton() {
                    self.compile_expr_to_locals(
                        expr,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                } else {
                    self.compile_expr_payload(expr, function)?;
                    // A thrown completion always co-homes the thrown value in
                    // `result_local` (every `emit_throw_*` / helper-call
                    // `store_call_results_to` sets it). The statement value left
                    // on the stack by `compile_expr_payload` is the *normal*
                    // result and is unrelated on the throw path, so propagate the
                    // throw from `result_local` — not `scratch_local`, which holds
                    // unrelated scratch state and would replace a real Error
                    // instance (e.g. a strict read-only / setter / Proxy write
                    // TypeError) with garbage.
                    self.emit_propagate_throw_from_locals_if_needed(
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.finish_statement_payload(function, expr.kind);
                }
            }
            StatementIr::Var(declarators) => {
                self.compile_var_declarators(declarators, function)?;
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::LexicalBlock(statements) => {
                for statement in statements {
                    let StatementIr::Lexical { mode, name, init } = statement else {
                        continue;
                    };
                    let storage = self
                        .lookup_current_scope_binding(name)
                        .or_else(|| self.lookup_binding(name))
                        .unwrap_or_else(|| self.allocate_binding(name.clone(), *mode, init.kind));
                    self.initialize_binding_uninitialized(storage, function);
                }
                for statement in statements {
                    self.compile_statement(statement, function)?;
                }
            }
            StatementIr::Block(block) => {
                self.push_scope();
                self.compile_block_contents(block, function)?;
                self.pop_scope();
            }
            StatementIr::Labelled { labels, statement } => {
                self.compile_labelled_statement(labels, statement, function)?;
            }
            StatementIr::Throw(value) => {
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
                    self.uses_heap,
                )));
                self.emit_capture_throw_error_name(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.set_completion_kind(CompletionKind::Throw, function);
                function.instruction(&Instruction::I64Const(-1));
                function.instruction(&Instruction::LocalSet(self.completion_aux_local));
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, 0, function);
                } else {
                    self.emit_return_current_completion(function);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_name,
                catch_source_name,
                catch_parameter_environment,
                catch_block,
            } => {
                self.compile_try_catch(
                    try_block,
                    catch_name,
                    catch_source_name,
                    catch_parameter_environment.as_ref(),
                    catch_block,
                    function,
                )?;
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
            } => {
                self.compile_try_finally(try_block, finally_block, function)?;
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_name,
                catch_source_name,
                catch_parameter_environment,
                catch_block,
                finally_block,
            } => {
                self.compile_try_catch_finally(
                    try_block,
                    catch_name,
                    catch_source_name,
                    catch_parameter_environment.as_ref(),
                    catch_block,
                    finally_block,
                    function,
                )?;
            }
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_truthy_i32(condition, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.push_control(ControlFrameKind::If);
                self.compile_statement(then_branch, function)?;
                function.instruction(&Instruction::Else);
                if let Some(else_branch) = else_branch {
                    self.compile_statement(else_branch, function)?;
                } else {
                    self.emit_statement_result(function, ValueKind::Undefined);
                }
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            StatementIr::While { condition, body } => {
                self.compile_while(condition, body, &[], function)?;
            }
            StatementIr::DoWhile { body, condition } => {
                self.compile_do_while(body, condition, &[], function)?;
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
                lexical_environment,
            } => {
                self.compile_for(
                    init.as_ref(),
                    test.as_ref(),
                    update.as_ref(),
                    body,
                    lexical_environment.as_ref(),
                    &[],
                    function,
                )?;
            }
            StatementIr::ForOfArray {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
            } => self.compile_for_of_array(
                *mode,
                name,
                iterable,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::ForOfString {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
            } => self.compile_for_of_string(
                *mode,
                name,
                iterable,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::ForOfIterator {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
            } => self.compile_for_of_iterator(
                *mode,
                name,
                iterable,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::ForInArray {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_array(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::ForInString {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_string(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::ForInObject {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_object(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                &[],
                function,
            )?,
            StatementIr::Switch {
                discriminant,
                lexical_environment,
                lexical_declarations,
                cases,
            } => {
                self.compile_switch(
                    discriminant,
                    lexical_environment.as_ref(),
                    lexical_declarations,
                    cases,
                    &[],
                    function,
                )?;
            }
            StatementIr::Debugger => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Return(value) => {
                if self.strict
                    && self.throw_handler_stack.is_empty()
                    && self.finally_stack.is_empty()
                    && !self.is_derived_constructor
                {
                    self.compile_return_position_expr(value, function)?;
                    return Ok(());
                }
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.finally_stack.last().copied() {
                    self.set_completion_kind(CompletionKind::Return, function);
                    self.emit_branch_to_target(target, 0, function);
                } else {
                    self.set_completion_kind(CompletionKind::Return, function);
                    self.normalize_derived_constructor_result(function)?;
                    self.set_completion_kind(CompletionKind::Normal, function);
                    self.emit_return_current_completion(function);
                }
            }
            StatementIr::Break { label } => self.compile_break(label.as_deref(), function)?,
            StatementIr::Continue { label } => self.compile_continue(label.as_deref(), function)?,
        }
        Ok(())
    }

    fn compile_return_position_expr(
        &mut self,
        value: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match &value.expr {
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
                static_regexp_compilation: None,
            } if self.emit_tail_indirect_call(callee, this_arg.as_deref(), args, function)? => {}
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_expr_to_locals(
                    condition,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_truthy_tagged_i32(self.result_tag_local, self.result_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.compile_return_position_expr(then_expr, function)?;
                function.instruction(&Instruction::Else);
                self.compile_return_position_expr(else_expr, function)?;
                function.instruction(&Instruction::End);
            }
            ExprIr::LogicalShortCircuit { op, lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                match op {
                    LogicalBinaryOp::Coalesce => {
                        self.compile_nullish_tagged_i32(self.result_tag_local, function)?;
                    }
                    LogicalBinaryOp::And | LogicalBinaryOp::Or => {
                        self.compile_truthy_tagged_i32(
                            self.result_tag_local,
                            self.result_local,
                            function,
                        )?;
                    }
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                match op {
                    LogicalBinaryOp::And | LogicalBinaryOp::Coalesce => {
                        self.compile_return_position_expr(rhs, function)?;
                    }
                    LogicalBinaryOp::Or => self.emit_return_from_result_locals(function),
                }
                function.instruction(&Instruction::Else);
                match op {
                    LogicalBinaryOp::And | LogicalBinaryOp::Coalesce => {
                        self.emit_return_from_result_locals(function);
                    }
                    LogicalBinaryOp::Or => {
                        self.compile_return_position_expr(rhs, function)?;
                    }
                }
                function.instruction(&Instruction::End);
            }
            ExprIr::Comma { lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_return_position_expr(rhs, function)?;
            }
            _ => {
                self.compile_expr_to_locals(
                    value,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_from_result_locals(function);
            }
        }
        Ok(())
    }

    fn emit_return_from_result_locals(&self, function: &mut Function) {
        self.set_completion_kind(CompletionKind::Return, function);
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
    }

    pub(crate) fn compile_labelled_statement(
        &mut self,
        labels: &[String],
        statement: &StatementIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match statement {
            StatementIr::Block(block) => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                let break_frame = self.push_control(ControlFrameKind::Block);
                self.push_labels(labels, break_frame, None);
                self.push_scope();
                self.compile_block_contents(block, function)?;
                self.pop_scope();
                self.pop_labels(labels.len());
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
            }
            StatementIr::While { condition, body } => {
                self.compile_while(condition, body, labels, function)?;
            }
            StatementIr::DoWhile { body, condition } => {
                self.compile_do_while(body, condition, labels, function)?;
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
                lexical_environment,
            } => {
                self.compile_for(
                    init.as_ref(),
                    test.as_ref(),
                    update.as_ref(),
                    body,
                    lexical_environment.as_ref(),
                    labels,
                    function,
                )?;
            }
            StatementIr::ForOfArray {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
            } => self.compile_for_of_array(
                *mode,
                name,
                iterable,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::ForOfString {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
            } => self.compile_for_of_string(
                *mode,
                name,
                iterable,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::ForOfIterator {
                mode,
                name,
                iterable,
                body,
                lexical_environment,
            } => self.compile_for_of_iterator(
                *mode,
                name,
                iterable,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::ForInArray {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_array(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::ForInString {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_string(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::ForInObject {
                mode,
                name,
                target,
                body,
                lexical_environment,
            } => self.compile_for_in_object(
                *mode,
                name,
                target,
                body,
                lexical_environment.as_ref(),
                labels,
                function,
            )?,
            StatementIr::Switch {
                discriminant,
                lexical_environment,
                lexical_declarations,
                cases,
            } => {
                self.compile_switch(
                    discriminant,
                    lexical_environment.as_ref(),
                    lexical_declarations,
                    cases,
                    labels,
                    function,
                )?;
            }
            _ => {
                function.instruction(&Instruction::Block(BlockType::Empty));
                let break_frame = self.push_control(ControlFrameKind::Block);
                self.push_labels(labels, break_frame, None);
                self.compile_statement(statement, function)?;
                self.pop_labels(labels.len());
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
            }
        }
        Ok(())
    }

    pub(crate) fn compile_try_catch(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
        catch_parameter_environment: Option<&LexicalEnvironmentIr>,
        catch_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        let catch_frame = self.push_control(ControlFrameKind::Block);
        self.throw_handler_stack.push(catch_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.push_scope();
        if let Some(environment) = catch_parameter_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let catch_storage = self
            .lookup_current_scope_binding(catch_name)
            .unwrap_or_else(|| self.allocate_dynamic_binding_storage(catch_name));
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        if catch_source_name != catch_name {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(catch_source_name.to_string(), catch_storage);
        }
        self.write_binding_from_locals(
            catch_storage,
            self.result_local,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(catch_block, function)?;
        self.pop_scope();
        if catch_parameter_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_try_finally(
        &mut self,
        try_block: &BlockIr,
        finally_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        let _outer_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let finally_frame = self.push_control(ControlFrameKind::Block);
        self.finally_stack.push(finally_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(finally_block, function)?;
        self.pop_scope();
        self.emit_resume_after_finally(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        Ok(())
    }

    pub(crate) fn compile_try_catch_finally(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
        catch_parameter_environment: Option<&LexicalEnvironmentIr>,
        catch_block: &BlockIr,
        finally_block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        let _outer_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let _finally_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let catch_skip_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let catch_frame = self.push_control(ControlFrameKind::Block);
        self.throw_handler_stack.push(catch_frame);
        // `br` targets exit the selected block. In this layout, branching to
        // `finally_frame` would therefore skip the finalizer itself. The
        // catch-skip block instead ends immediately before the finalizer, so
        // it is the continuation target for abrupt completions from either
        // the try or catch body.
        self.finally_stack.push(catch_skip_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.throw_handler_stack.pop();
        self.emit_branch_to_target(catch_skip_frame, 0, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.push_scope();
        if let Some(environment) = catch_parameter_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let catch_storage = self
            .lookup_current_scope_binding(catch_name)
            .unwrap_or_else(|| self.allocate_dynamic_binding_storage(catch_name));
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(catch_name.to_string(), catch_storage);
        if catch_source_name != catch_name {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(catch_source_name.to_string(), catch_storage);
        }
        self.write_binding_from_locals(
            catch_storage,
            self.result_local,
            self.result_tag_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(catch_block, function)?;
        self.pop_scope();
        if catch_parameter_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.push_scope();
        self.compile_block_contents(finally_block, function)?;
        self.pop_scope();
        self.emit_resume_after_finally(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        Ok(())
    }

    pub(crate) fn compile_while(
        &mut self,
        condition: &TypedExpr,
        body: &StatementIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Loop);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_truthy_i32(condition, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
        self.compile_statement(body, function)?;
        function.instruction(&Instruction::Br(self.depth_to(continue_frame)));
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_do_while(
        &mut self,
        body: &StatementIr,
        condition: &TypedExpr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.compile_truthy_i32(condition, function)?;
        function.instruction(&Instruction::BrIf(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_for(
        &mut self,
        init: Option<&ForInitIr>,
        test: Option<&TypedExpr>,
        update: Option<&TypedExpr>,
        body: &StatementIr,
        lexical_environment: Option<&ForLexicalEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.push_scope();
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        let runtime_environment = lexical_environment.map(|environment| LexicalEnvironmentIr {
            bindings: environment.bindings.clone(),
        });
        if let Some(environment) = &runtime_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        if let Some(init) = init {
            self.compile_for_init(init, function)?;
        }
        if let Some(environment) = lexical_environment {
            self.emit_replace_lexical_environment(environment, function)?;
        }
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        if let Some(test) = test {
            self.compile_truthy_i32(test, function)?;
            function.instruction(&Instruction::I32Eqz);
            self.emit_branch_if_to_target(break_frame, 0, function);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if let Some(environment) = lexical_environment {
            self.emit_replace_lexical_environment(environment, function)?;
        }
        if let Some(update) = update {
            self.compile_expr_payload(update, function)?;
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if runtime_environment.is_some() {
            self.end_lexical_environment_scope();
        }
        self.pop_scope();
        Ok(())
    }

    pub(crate) fn compile_switch(
        &mut self,
        discriminant: &TypedExpr,
        lexical_environment: Option<&LexicalEnvironmentIr>,
        lexical_declarations: &[StatementIr],
        cases: &[SwitchCaseIr],
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let discriminant_payload_local = self.reserve_temp_local();
        let discriminant_tag_local = self.reserve_temp_local();
        let selected_local = self.reserve_temp_local();
        let active_local = self.reserve_temp_local();
        let default_index = cases
            .iter()
            .enumerate()
            .find_map(|(index, case)| case.condition.is_none().then_some(index as i64));

        self.emit_statement_result(function, ValueKind::Undefined);
        self.compile_expr_to_locals(
            discriminant,
            discriminant_payload_local,
            discriminant_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            discriminant_payload_local,
            discriminant_tag_local,
            function,
        )?;
        self.push_scope();
        if let Some(environment) = lexical_environment {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        for case in cases {
            self.initialize_direct_lexical_bindings(&case.body.statements, function);
        }
        for declaration in lexical_declarations {
            self.compile_statement(declaration, function)?;
        }
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(selected_local));

        for (index, case) in cases.iter().enumerate() {
            let Some(condition) = &case.condition else {
                continue;
            };
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.push_control(ControlFrameKind::If);
            self.compile_switch_case_match(
                discriminant,
                discriminant_payload_local,
                discriminant_tag_local,
                condition,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(
                self.scratch_local,
                self.result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(selected_local));
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        if let Some(default_index) = default_index {
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(default_index));
            function.instruction(&Instruction::LocalSet(selected_local));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(active_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        self.push_labels(labels, break_frame, None);
        for (index, case) in cases.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(active_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(selected_local));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(active_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(active_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.push_control(ControlFrameKind::If);
            self.compile_switch_case_body(&case.body, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_labels(labels.len());
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
        self.release_temp_local(active_local);
        self.release_temp_local(selected_local);
        self.release_temp_local(discriminant_tag_local);
        self.release_temp_local(discriminant_payload_local);
        Ok(())
    }

    pub(crate) fn compile_switch_case_body(
        &mut self,
        block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for statement in &block.statements {
            self.compile_statement(statement, function)?;
        }
        Ok(())
    }

    pub(crate) fn compile_switch_case_match(
        &mut self,
        discriminant: &TypedExpr,
        discriminant_payload_local: u32,
        discriminant_tag_local: u32,
        condition: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if discriminant.kind != ValueKind::Dynamic
            && condition.kind != ValueKind::Dynamic
            && discriminant.kind != condition.kind
        {
            self.compile_expr_to_locals(
                condition,
                self.scratch_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_tagged_payload_equality_i32(
                discriminant_tag_local,
                discriminant_payload_local,
                self.result_tag_local,
                self.scratch_local,
                function,
            )?;
            return Ok(());
        }

        if discriminant.kind != ValueKind::Dynamic && condition.kind != ValueKind::Dynamic {
            self.compile_expr_to_locals(
                condition,
                self.scratch_local,
                self.result_tag_local,
                function,
            )?;
            match discriminant.kind {
                ValueKind::Number => {
                    function.instruction(&Instruction::LocalGet(discriminant_payload_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Eq);
                }
                ValueKind::String => {
                    self.emit_string_payload_equality_i32(
                        discriminant_payload_local,
                        self.scratch_local,
                        function,
                    );
                }
                _ => {
                    function.instruction(&Instruction::LocalGet(discriminant_payload_local));
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::I64Eq);
                }
            }
            return Ok(());
        }

        self.compile_expr_to_locals(
            condition,
            self.scratch_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_tagged_payload_equality_i32(
            discriminant_tag_local,
            discriminant_payload_local,
            self.result_tag_local,
            self.scratch_local,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn compile_break(
        &mut self,
        label: Option<&str>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let break_frame = if let Some(label) = label {
            self.label_stack
                .iter()
                .rev()
                .find(|targets| targets.name == label)
                .map(|targets| targets.break_frame)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unknown label `{label}`"
                    ))
                })?
        } else {
            *self.breakable_stack.last().ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: break outside loop or switch",
                )
            })?
        };
        if let Some(target) = self.active_finally_target_for_branch(break_frame) {
            self.set_completion_kind_with_aux(
                CompletionKind::Break,
                break_frame.frame as i64,
                function,
            );
            self.emit_branch_to_target(target, 0, function);
            return Ok(());
        }
        self.emit_branch_to_target(break_frame, 0, function);
        Ok(())
    }

    pub(crate) fn compile_continue(
        &mut self,
        label: Option<&str>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let continue_frame = if let Some(label) = label {
            self.label_stack
                .iter()
                .rev()
                .find(|targets| targets.name == label)
                .and_then(|targets| targets.continue_frame)
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: continue to non-loop label `{label}`"
                    ))
                })?
        } else {
            self.loop_stack
                .last()
                .copied()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: continue outside loop",
                    )
                })?
                .continue_frame
        };
        if let Some(target) = self.active_finally_target_for_branch(continue_frame) {
            self.set_completion_kind_with_aux(
                CompletionKind::Continue,
                continue_frame.frame as i64,
                function,
            );
            self.emit_branch_to_target(target, 0, function);
            return Ok(());
        }
        self.emit_branch_to_target(continue_frame, 0, function);
        Ok(())
    }

    pub(crate) fn compile_for_init(
        &mut self,
        init: &ForInitIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match init {
            ForInitIr::Lexical { mode, name, init } => {
                let storage = self
                    .lookup_current_scope_binding(name)
                    .unwrap_or_else(|| self.allocate_binding(name.clone(), *mode, init.kind));
                self.initialize_binding_uninitialized(storage, function);
                self.compile_expr_to_binding(init, storage, function)?;
            }
            ForInitIr::LexicalBlock(bindings) => {
                for binding in bindings {
                    let storage = self
                        .lookup_current_scope_binding(&binding.name)
                        .unwrap_or_else(|| {
                            self.allocate_binding(
                                binding.name.clone(),
                                binding.mode,
                                binding.init.kind,
                            )
                        });
                    self.initialize_binding_uninitialized(storage, function);
                }
                for binding in bindings {
                    let storage = self
                        .lookup_current_scope_binding(&binding.name)
                        .expect("for lexical binding should be allocated");
                    self.compile_expr_to_binding(&binding.init, storage, function)?;
                }
            }
            ForInitIr::Var(declarators) => {
                self.compile_var_declarators(declarators, function)?;
            }
            ForInitIr::Expression(expr) => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::Drop);
            }
        }
        Ok(())
    }

    pub(crate) fn compile_var_declarators(
        &mut self,
        declarators: &[VarDeclaratorIr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for declarator in declarators {
            let Some(init) = &declarator.init else {
                continue;
            };
            let storage = self.lookup_binding(&declarator.name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound identifier `{}`",
                    declarator.name
                ))
            })?;
            self.compile_expr_to_binding(init, storage, function)?;
            self.mirror_binding_to_global_object(&declarator.name, storage, function)?;
        }
        Ok(())
    }

    pub(crate) fn emit_to_integer_clamped_to_string_len(
        &mut self,
        number_payload_local: u32,
        string_len_local: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_to_slice_index_clamped_to_string_len(
        &mut self,
        number_payload_local: u32,
        string_len_local: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_for_of_array(
        &mut self,
        mode: BindingMode,
        name: &str,
        iterable: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_local = self.reserve_temp_local();
        let array_tag_local = self.reserve_temp_local();
        let len_payload_local = self.reserve_temp_local();
        let len_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(iterable, array_local, array_tag_local, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-of var storage must exist"),
                );
        }
        self.emit_array_length(array_local, len_payload_local, len_tag_local, function);
        function.instruction(&Instruction::LocalGet(len_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(len_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_payload_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
        self.emit_array_read(
            array_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-of lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, value_payload_local, value_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_tag_local);
        self.release_temp_local(len_payload_local);
        self.release_temp_local(array_tag_local);
        self.release_temp_local(array_local);
        Ok(())
    }

    pub(crate) fn compile_for_of_string(
        &mut self,
        mode: BindingMode,
        name: &str,
        iterable: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_payload_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let char_offset_local = self.reserve_temp_local();
        let char_pos_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(iterable, string_payload_local, string_tag_local, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::String))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-of var storage must exist"),
                );
        }
        function.instruction(&Instruction::LocalGet(string_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of target is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(
            string_payload_local,
            buffer_local,
            byte_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(byte_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
        self.emit_load_string_byte(buffer_local, index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            buffer_local,
            index_local,
            byte_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        self.emit_heap_alloc_const(4, function)?;
        function.instruction(&Instruction::LocalSet(char_offset_local));
        function.instruction(&Instruction::LocalGet(char_offset_local));
        function.instruction(&Instruction::LocalSet(char_pos_local));
        self.emit_store_utf8_codepoint(char_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::LocalGet(char_pos_local));
        function.instruction(&Instruction::LocalGet(char_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(byte_local));
        self.emit_pack_string_payload(char_offset_local, byte_local, function);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-of lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, value_payload_local, string_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(value_payload_local);
        self.release_temp_local(char_pos_local);
        self.release_temp_local(char_offset_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(byte_len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(string_tag_local);
        self.release_temp_local(string_payload_local);
        Ok(())
    }

    pub(crate) fn compile_for_of_iterator(
        &mut self,
        mode: BindingMode,
        name: &str,
        iterable: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let iterable_payload_local = self.reserve_temp_local();
        let iterable_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let saved_payload_local = self.reserve_temp_local();
        let saved_tag_local = self.reserve_temp_local();
        let saved_completion_local = self.reserve_temp_local();
        let saved_aux_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(
            iterable,
            iterable_payload_local,
            iterable_tag_local,
            function,
        )?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-of var storage must exist"),
                );
        }
        self.emit_is_heap_object_like_tag_i32(iterable_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of target is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterable_payload_local,
            iterable_tag_local,
            iterable_payload_local,
            iterable_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator method must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((iterable_payload_local, Some(iterable_tag_local))),
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(next_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator next must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);

        self.emit_function_handle_call(
            next_payload_local,
            next_tag_local,
            Some((iterator_payload_local, Some(iterator_tag_local))),
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-of lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, value_payload_local, value_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;

        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        function.instruction(&Instruction::Block(BlockType::Empty));
        let finally_frame = self.push_control(ControlFrameKind::Block);
        self.finally_stack.push(finally_frame);
        self.compile_statement(body, function)?;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_labels(labels.len());
        self.loop_stack.pop();

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(saved_aux_local));
        function.instruction(&Instruction::I64Const(continue_frame.frame as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::LocalSet(saved_completion_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        self.emit_iterator_close_condition_i32(
            saved_completion_local,
            saved_aux_local,
            continue_frame,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_iterator_close_preserving_current_throw(
            IteratorCloseOnThrowLocals {
                iterator_payload_local,
                iterator_tag_local,
                key_local,
                return_payload_local: method_payload_local,
                return_tag_local: method_tag_local,
                result_payload_local,
                result_tag_local,
                saved_payload_local: close_saved_payload_local,
                saved_tag_local: close_saved_tag_local,
                saved_completion_local: close_saved_completion_local,
                saved_aux_local: close_saved_aux_local,
            },
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_iterator_close(
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(saved_completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
        function.instruction(&Instruction::End);

        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(close_saved_aux_local);
        self.release_temp_local(close_saved_completion_local);
        self.release_temp_local(close_saved_tag_local);
        self.release_temp_local(close_saved_payload_local);
        self.release_temp_local(saved_aux_local);
        self.release_temp_local(saved_completion_local);
        self.release_temp_local(saved_tag_local);
        self.release_temp_local(saved_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(result_tag_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(iterable_tag_local);
        self.release_temp_local(iterable_payload_local);
        Ok(())
    }

    pub(crate) fn compile_object_destructure_to_locals(
        &mut self,
        value: &TypedExpr,
        pattern: &ObjectDestructuringPatternIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_payload = self.reserve_temp_local();
        let source_tag = self.reserve_temp_local();
        let source_object_payload = self.reserve_temp_local();
        let source_object_tag = self.reserve_temp_local();
        let property_value_payload = self.reserve_temp_local();
        let property_value_tag = self.reserve_temp_local();
        let mut excluded_keys = Vec::with_capacity(pattern.properties.len());

        self.compile_expr_to_locals(value, source_payload, source_tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(source_payload, source_tag, function)?;
        self.compile_nullish_tagged_i32(source_tag, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot destructure undefined or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_value_to_object_locals(
            source_payload,
            source_tag,
            source_object_payload,
            source_object_tag,
            function,
        )?;

        for property in &pattern.properties {
            let key_payload = self.reserve_temp_local();
            let key_tag = self.reserve_temp_local();
            match &property.key {
                DestructuringPropertyKeyIr::Static(key) => {
                    function.instruction(&Instruction::I64Const(self.strings.payload(key)));
                    function.instruction(&Instruction::LocalSet(key_payload));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(key_tag));
                }
                DestructuringPropertyKeyIr::Computed(key) => {
                    self.compile_expr_to_locals(key, key_payload, key_tag, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        key_payload,
                        key_tag,
                        function,
                    )?;
                    self.emit_value_to_property_key_locals(key_payload, key_tag, function)?;
                }
            }
            let prepared = self.prepare_destructuring_target(&property.target, function)?;
            self.emit_object_read(
                source_object_payload,
                source_object_tag,
                source_object_payload,
                source_object_tag,
                key_payload,
                property_value_payload,
                property_value_tag,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            if let Some(default) = &property.default {
                function.instruction(&Instruction::LocalGet(property_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.push_control(ControlFrameKind::If);
                self.compile_expr_to_locals(
                    default,
                    property_value_payload,
                    property_value_tag,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    property_value_payload,
                    property_value_tag,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            self.put_destructuring_target(
                &property.target,
                prepared,
                property_value_payload,
                property_value_tag,
                function,
            )?;
            excluded_keys.push((key_payload, key_tag));
        }

        if let Some(rest) = &pattern.rest {
            let prepared = self.prepare_destructuring_target(rest, function)?;
            self.emit_copy_data_properties_rest(
                source_object_payload,
                source_object_tag,
                &excluded_keys,
                property_value_payload,
                property_value_tag,
                function,
            )?;
            self.put_destructuring_target(
                rest,
                prepared,
                property_value_payload,
                property_value_tag,
                function,
            )?;
        }

        function.instruction(&Instruction::LocalGet(source_payload));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(source_tag));
        function.instruction(&Instruction::LocalSet(tag_local));

        for (key_payload, key_tag) in excluded_keys.into_iter().rev() {
            self.release_temp_local(key_tag);
            self.release_temp_local(key_payload);
        }
        self.release_temp_local(property_value_tag);
        self.release_temp_local(property_value_payload);
        self.release_temp_local(source_object_tag);
        self.release_temp_local(source_object_payload);
        self.release_temp_local(source_tag);
        self.release_temp_local(source_payload);
        Ok(())
    }

    fn emit_copy_data_properties_rest(
        &mut self,
        source_payload: u32,
        source_tag: u32,
        excluded_keys: &[(u32, u32)],
        target_payload: u32,
        target_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.ownKeys`",
                )
            })?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.getOwnPropertyDescriptor`",
                )
            })?;
        let keys_payload = self.reserve_temp_local();
        let keys_tag = self.reserve_temp_local();
        let keys_length = self.reserve_temp_local();
        let key_index = self.reserve_temp_local();
        let key_payload = self.reserve_temp_local();
        let key_tag = self.reserve_temp_local();
        let descriptor_payload = self.reserve_temp_local();
        let descriptor_tag = self.reserve_temp_local();
        let enumerable_key = self.reserve_temp_local();
        let enumerable_payload = self.reserve_temp_local();
        let enumerable_tag = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(target_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(target_tag));

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(source_payload, source_tag)],
            keys_payload,
            keys_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(keys_payload, keys_tag, function)?;
        self.load_i64_to_local_from_offset(keys_payload, HEAP_LEN_OFFSET, keys_length, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_index));
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key));

        function.instruction(&Instruction::Block(BlockType::Empty));
        let copy_break = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let copy_loop = self.push_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::LocalGet(key_index));
        function.instruction(&Instruction::LocalGet(keys_length));
        function.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(copy_break, 0, function);
        self.emit_array_read(keys_payload, key_index, key_payload, key_tag, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        let skip_key = self.push_control(ControlFrameKind::Block);
        for (excluded_payload, excluded_tag) in excluded_keys {
            self.emit_tagged_payload_same_value_i32(
                key_tag,
                key_payload,
                *excluded_tag,
                *excluded_payload,
                function,
            )?;
            self.emit_branch_if_to_target(skip_key, 0, function);
        }

        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[(source_payload, source_tag), (key_payload, key_tag)],
            descriptor_payload,
            descriptor_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            descriptor_payload,
            descriptor_tag,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(descriptor_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(skip_key, 0, function);

        self.emit_object_read(
            descriptor_payload,
            descriptor_tag,
            descriptor_payload,
            descriptor_tag,
            enumerable_key,
            enumerable_payload,
            enumerable_tag,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.compile_truthy_tagged_i32(enumerable_tag, enumerable_payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(skip_key, 0, function);

        self.emit_object_read(
            source_payload,
            source_tag,
            source_payload,
            source_tag,
            key_payload,
            enumerable_payload,
            enumerable_tag,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.emit_object_define_enumerable_data(
            target_payload,
            key_payload,
            enumerable_payload,
            enumerable_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(key_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(key_index));
        self.emit_branch_to_target(copy_loop, 0, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.release_temp_local(enumerable_tag);
        self.release_temp_local(enumerable_payload);
        self.release_temp_local(enumerable_key);
        self.release_temp_local(descriptor_tag);
        self.release_temp_local(descriptor_payload);
        self.release_temp_local(key_tag);
        self.release_temp_local(key_payload);
        self.release_temp_local(key_index);
        self.release_temp_local(keys_length);
        self.release_temp_local(keys_tag);
        self.release_temp_local(keys_payload);
        Ok(())
    }

    pub(crate) fn compile_array_destructure_to_locals(
        &mut self,
        value: &TypedExpr,
        pattern: &ArrayDestructuringPatternIr,
        assignment: bool,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_payload = self.reserve_temp_local();
        let source_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(value, source_payload, source_tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(source_payload, source_tag, function)?;
        self.compile_array_destructure_from_value_locals(
            value.value_info(),
            source_payload,
            source_tag,
            pattern,
            function,
        )?;
        if assignment {
            function.instruction(&Instruction::LocalGet(source_payload));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(source_tag));
            function.instruction(&Instruction::LocalSet(tag_local));
        } else {
            self.emit_undefined_payload(function);
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        }
        self.release_temp_local(source_tag);
        self.release_temp_local(source_payload);
        Ok(())
    }

    fn compile_array_destructure_from_value_locals(
        &mut self,
        value_info: ValueInfo,
        source_payload: u32,
        source_tag: u32,
        pattern: &ArrayDestructuringPatternIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let method_payload = self.reserve_temp_local();
        let method_tag = self.reserve_temp_local();
        let locals = DestructuringIteratorLocals {
            iterator_payload: self.reserve_temp_local(),
            iterator_tag: self.reserve_temp_local(),
            next_payload: self.reserve_temp_local(),
            next_tag: self.reserve_temp_local(),
            key: self.reserve_temp_local(),
            result_payload: self.reserve_temp_local(),
            result_tag: self.reserve_temp_local(),
            done_payload: self.reserve_temp_local(),
            done_tag: self.reserve_temp_local(),
            value_payload: self.reserve_temp_local(),
            value_tag: self.reserve_temp_local(),
            return_payload: self.reserve_temp_local(),
            return_tag: self.reserve_temp_local(),
            done: self.reserve_temp_local(),
            close_saved_payload: self.reserve_temp_local(),
            close_saved_tag: self.reserve_temp_local(),
            close_saved_completion: self.reserve_temp_local(),
            close_saved_aux: self.reserve_temp_local(),
        };

        self.emit_get_iterator_from_value_locals(
            value_info,
            source_payload,
            source_tag,
            method_payload,
            method_tag,
            locals,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(locals.done));

        function.instruction(&Instruction::Block(BlockType::Empty));
        let exit_target = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let abrupt_target = self.push_control(ControlFrameKind::Block);
        self.finally_stack.push(abrupt_target);
        for element in &pattern.elements {
            self.compile_array_destructuring_element(element, locals, function)?;
        }
        self.finally_stack.pop();

        function.instruction(&Instruction::LocalGet(locals.done));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_iterator_close(
            locals.iterator_payload,
            locals.iterator_tag,
            locals.key,
            locals.return_payload,
            locals.return_tag,
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_branch_to_target(exit_target, 0, function);

        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(locals.done));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_iterator_close_preserving_current_throw(
            IteratorCloseOnThrowLocals {
                iterator_payload_local: locals.iterator_payload,
                iterator_tag_local: locals.iterator_tag,
                key_local: locals.key,
                return_payload_local: locals.return_payload,
                return_tag_local: locals.return_tag,
                result_payload_local: locals.result_payload,
                result_tag_local: locals.result_tag,
                saved_payload_local: locals.close_saved_payload,
                saved_tag_local: locals.close_saved_tag,
                saved_completion_local: locals.close_saved_completion,
                saved_aux_local: locals.close_saved_aux,
            },
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_propagate_current_completion_if_throw(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        for local in [
            locals.close_saved_aux,
            locals.close_saved_completion,
            locals.close_saved_tag,
            locals.close_saved_payload,
            locals.done,
            locals.return_tag,
            locals.return_payload,
            locals.value_tag,
            locals.value_payload,
            locals.done_tag,
            locals.done_payload,
            locals.result_tag,
            locals.result_payload,
            locals.key,
            locals.next_tag,
            locals.next_payload,
            locals.iterator_tag,
            locals.iterator_payload,
            method_tag,
            method_payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_get_iterator_from_value_locals(
        &mut self,
        value_info: ValueInfo,
        source_payload: u32,
        source_tag: u32,
        method_payload: u32,
        method_tag: u32,
        locals: DestructuringIteratorLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if value_info.kind == ValueKind::Array {
            function.instruction(&Instruction::I64Const(
                self.strings.payload("Symbol.iterator"),
            ));
            function.instruction(&Instruction::LocalSet(locals.key));
            self.emit_object_read(
                source_payload,
                source_tag,
                source_payload,
                source_tag,
                locals.key,
                method_payload,
                method_tag,
                function,
            )?;
        } else {
            let source_name = "$array.destructure.source";
            self.push_scope();
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    source_name.to_string(),
                    BindingStorage::Dynamic {
                        tag_local: source_tag,
                        payload_local: source_payload,
                    },
                );
            let source =
                TypedExpr::from_info(value_info, ExprIr::Identifier(source_name.to_string()));
            let method = TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::PropertyRead {
                    target: Box::new(source),
                    key: PropertyKeyIr::StaticString("Symbol.iterator".to_string()),
                },
            );
            self.compile_expr_to_locals(&method, method_payload, method_tag, function)?;
            self.pop_scope();
        }
        self.emit_propagate_throw_from_locals_if_needed(method_payload, method_tag, function)?;
        self.emit_is_callable_i32(method_tag, method_payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "destructuring value is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(method_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_function_handle_call(
            method_payload,
            method_tag,
            Some((source_payload, Some(source_tag))),
            &[],
            locals.iterator_payload,
            locals.iterator_tag,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            method_payload,
            method_tag,
            source_payload,
            source_tag,
            &[],
            locals.iterator_payload,
            locals.iterator_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            locals.iterator_payload,
            locals.iterator_tag,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(locals.iterator_tag, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "destructuring iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(locals.key));
        self.emit_object_read(
            locals.iterator_payload,
            locals.iterator_tag,
            locals.iterator_payload,
            locals.iterator_tag,
            locals.key,
            locals.next_payload,
            locals.next_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            locals.next_payload,
            locals.next_tag,
            function,
        )?;
        Ok(())
    }

    fn compile_array_destructuring_element(
        &mut self,
        element: &ArrayDestructuringElementIr,
        locals: DestructuringIteratorLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match element {
            ArrayDestructuringElementIr::Elision => {
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.push_control(ControlFrameKind::If);
                self.emit_destructuring_iterator_step(
                    locals,
                    DestructuringIteratorStepKind::Elision,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            ArrayDestructuringElementIr::Target { target, default } => {
                let prepared = self.prepare_destructuring_target(target, function)?;
                self.emit_undefined_payload(function);
                function.instruction(&Instruction::LocalSet(locals.value_payload));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(locals.value_tag));
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.push_control(ControlFrameKind::If);
                self.emit_destructuring_iterator_step(
                    locals,
                    DestructuringIteratorStepKind::Value,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                if let Some(default) = default {
                    function.instruction(&Instruction::LocalGet(locals.value_tag));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.push_control(ControlFrameKind::If);
                    self.compile_expr_to_locals(
                        default,
                        locals.value_payload,
                        locals.value_tag,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        locals.value_payload,
                        locals.value_tag,
                        function,
                    )?;
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                }
                self.put_destructuring_target(
                    target,
                    prepared,
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
            }
            ArrayDestructuringElementIr::Rest { target } => {
                let prepared = self.prepare_destructuring_target(target, function)?;
                let rest_payload = self.reserve_temp_local();
                self.compile_array_literal_payload(&[], function)?;
                function.instruction(&Instruction::LocalSet(rest_payload));
                let rest_index = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(rest_index));
                function.instruction(&Instruction::Block(BlockType::Empty));
                let rest_break = self.push_control(ControlFrameKind::Block);
                function.instruction(&Instruction::Loop(BlockType::Empty));
                let rest_loop = self.push_control(ControlFrameKind::Loop);
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                self.emit_branch_if_to_target(rest_break, 0, function);
                self.emit_destructuring_iterator_step(
                    locals,
                    DestructuringIteratorStepKind::Value,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(locals.done));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                self.emit_branch_if_to_target(rest_break, 0, function);
                self.emit_array_write(
                    rest_payload,
                    rest_index,
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(rest_index));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(rest_index));
                self.emit_branch_to_target(rest_loop, 0, function);
                self.pop_control(ControlFrameKind::Loop);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::Block);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(locals.value_tag));
                function.instruction(&Instruction::LocalGet(rest_payload));
                function.instruction(&Instruction::LocalSet(locals.value_payload));
                self.release_temp_local(rest_index);
                self.release_temp_local(rest_payload);
                self.put_destructuring_target(
                    target,
                    prepared,
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
            }
        }
        Ok(())
    }

    fn emit_destructuring_iterator_step(
        &mut self,
        locals: DestructuringIteratorLocals,
        step_kind: DestructuringIteratorStepKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(locals.done));
        function.instruction(&Instruction::LocalGet(locals.next_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_function_handle_call(
            locals.next_payload,
            locals.next_tag,
            Some((locals.iterator_payload, Some(locals.iterator_tag))),
            &[],
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            locals.next_payload,
            locals.next_tag,
            locals.iterator_payload,
            locals.iterator_tag,
            &[],
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_propagate_throw_from_locals_if_needed(
            locals.result_payload,
            locals.result_tag,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.emit_is_heap_object_like_tag_i32(locals.result_tag, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "destructuring iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(locals.key));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.emit_object_read(
            locals.result_payload,
            locals.result_tag,
            locals.result_payload,
            locals.result_tag,
            locals.key,
            locals.done_payload,
            locals.done_tag,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.compile_truthy_tagged_i32(locals.done_tag, locals.done_payload, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(locals.done));
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(locals.value_payload));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(locals.value_tag));
        function.instruction(&Instruction::Else);
        if matches!(step_kind, DestructuringIteratorStepKind::Value) {
            function.instruction(&Instruction::I64Const(self.strings.payload("value")));
            function.instruction(&Instruction::LocalSet(locals.key));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(locals.done));
            self.emit_object_read(
                locals.result_payload,
                locals.result_tag,
                locals.result_payload,
                locals.result_tag,
                locals.key,
                locals.value_payload,
                locals.value_tag,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(locals.done));
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn prepare_destructuring_target(
        &mut self,
        target: &DestructuringTargetIr,
        function: &mut Function,
    ) -> Result<PreparedDestructuringTarget, EmitError> {
        if let DestructuringTargetIr::AssignmentPrivate {
            target,
            private_name_id,
        } = target
        {
            let target_payload = self.reserve_temp_local();
            let target_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(target, target_payload, target_tag, function)?;
            self.emit_propagate_throw_from_locals_if_needed(target_payload, target_tag, function)?;
            return Ok(PreparedDestructuringTarget::Private {
                target_payload,
                target_tag,
                private_name_id: *private_name_id,
            });
        }

        let DestructuringTargetIr::AssignmentProperty { target, key } = target else {
            return Ok(PreparedDestructuringTarget::Direct);
        };

        let target_payload = self.reserve_temp_local();
        let target_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(target, target_payload, target_tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(target_payload, target_tag, function)?;
        let (key_payload, key_tag) = match key {
            DestructuringPropertyKeyIr::Static(_) => (None, None),
            DestructuringPropertyKeyIr::Computed(key) => {
                let key_payload = self.reserve_temp_local();
                let key_tag = self.reserve_temp_local();
                self.compile_expr_to_locals(key, key_payload, key_tag, function)?;
                self.emit_propagate_throw_from_locals_if_needed(key_payload, key_tag, function)?;
                (Some(key_payload), Some(key_tag))
            }
        };
        Ok(PreparedDestructuringTarget::Property {
            target: target.clone(),
            target_payload,
            target_tag,
            key: key.clone(),
            key_payload,
            key_tag,
        })
    }

    fn put_destructuring_target(
        &mut self,
        target: &DestructuringTargetIr,
        prepared: PreparedDestructuringTarget,
        value_payload: u32,
        value_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match target {
            DestructuringTargetIr::Binding { mode, name } => {
                let storage = self
                    .lookup_current_scope_binding(name)
                    .or_else(|| self.lookup_binding(name))
                    .unwrap_or_else(|| {
                        self.allocate_binding(name.clone(), *mode, ValueKind::Dynamic)
                    });
                self.write_binding_from_locals(storage, value_payload, value_tag, function);
                self.mirror_binding_to_global_object(name, storage, function)?;
            }
            DestructuringTargetIr::AssignmentIdentifier {
                name,
                global,
                implicit: _,
                immutable,
            } => {
                if *immutable {
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "assignment to immutable destructuring target",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_propagate_current_throw(function);
                    return Ok(());
                }
                if *global {
                    self.emit_global_property_write(name, value_payload, value_tag, function)?;
                } else {
                    let storage = self.lookup_binding(name).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: unbound destructuring assignment `{name}`"
                        ))
                    })?;
                    self.write_binding_from_locals(storage, value_payload, value_tag, function);
                    self.mirror_binding_to_global_object(name, storage, function)?;
                }
            }
            DestructuringTargetIr::AssignmentProperty { .. } => {
                let PreparedDestructuringTarget::Property {
                    target,
                    target_payload,
                    target_tag,
                    key,
                    key_payload,
                    key_tag,
                } = prepared
                else {
                    unreachable!("property destructuring target must be prepared")
                };
                let target_name = "$array.destructure.target";
                let value_name = "$array.destructure.value";
                let key_name = "$array.destructure.key";
                self.push_scope();
                let scope = self
                    .binding_scopes
                    .last_mut()
                    .expect("binding scope stack must exist");
                scope.insert(
                    target_name.to_string(),
                    BindingStorage::Dynamic {
                        tag_local: target_tag,
                        payload_local: target_payload,
                    },
                );
                scope.insert(
                    value_name.to_string(),
                    BindingStorage::Dynamic {
                        tag_local: value_tag,
                        payload_local: value_payload,
                    },
                );
                if let (Some(key_payload), Some(key_tag)) = (key_payload, key_tag) {
                    scope.insert(
                        key_name.to_string(),
                        BindingStorage::Dynamic {
                            tag_local: key_tag,
                            payload_local: key_payload,
                        },
                    );
                }
                let target_expr = TypedExpr::from_info(
                    target.value_info(),
                    ExprIr::Identifier(target_name.to_string()),
                );
                let property_key = match key {
                    DestructuringPropertyKeyIr::Static(name) => PropertyKeyIr::StaticString(name),
                    DestructuringPropertyKeyIr::Computed(key) => {
                        let raw_key = TypedExpr::from_info(
                            key.value_info(),
                            ExprIr::Identifier(key_name.to_string()),
                        );
                        PropertyKeyIr::StringExpr(Box::new(TypedExpr::spec_to_property_key(
                            raw_key,
                        )))
                    }
                };
                let value_expr = TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::Identifier(value_name.to_string()),
                );
                self.compile_property_write_to_locals(
                    &target_expr,
                    &property_key,
                    &value_expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_current_completion_if_throw(function);
                self.pop_scope();
                if let Some(key_tag) = key_tag {
                    self.release_temp_local(key_tag);
                }
                if let Some(key_payload) = key_payload {
                    self.release_temp_local(key_payload);
                }
                self.release_temp_local(target_tag);
                self.release_temp_local(target_payload);
            }
            DestructuringTargetIr::AssignmentPrivate { .. } => {
                let PreparedDestructuringTarget::Private {
                    target_payload,
                    target_tag,
                    private_name_id,
                } = prepared
                else {
                    unreachable!("private destructuring target must be prepared")
                };
                self.emit_private_write_from_locals(
                    target_payload,
                    target_tag,
                    private_name_id,
                    value_payload,
                    value_tag,
                    function,
                )?;
                self.release_temp_local(target_tag);
                self.release_temp_local(target_payload);
            }
            DestructuringTargetIr::NestedArray(pattern) => {
                self.compile_array_destructure_from_value_locals(
                    ValueInfo {
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    value_payload,
                    value_tag,
                    pattern,
                    function,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn emit_iterator_close_condition_i32(
        &self,
        completion_local: u32,
        aux_local: u32,
        current_continue_target: ControlTarget,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_BREAK));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(aux_local));
        function.instruction(&Instruction::I64Const(current_continue_target.frame as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_iterator_close(
        &mut self,
        iterator_payload_local: u32,
        iterator_tag_local: u32,
        key_local: u32,
        return_payload_local: u32,
        return_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("return")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_undefined_payload(function);
        function.instruction(&Instruction::LocalSet(return_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(return_tag_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            return_payload_local,
            return_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_is_callable_i32(return_tag_local, return_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "IteratorClose return method must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_function_handle_call(
            return_payload_local,
            return_tag_local,
            Some((iterator_payload_local, Some(iterator_tag_local))),
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_leave_throw_completion(
            return_payload_local,
            return_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "IteratorClose return result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_exhaust_static_generator_iterator_if_marked(
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn emit_exhaust_static_generator_iterator_if_marked(
        &mut self,
        iterator_payload_local: u32,
        iterator_tag_local: u32,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let marker_present_local = self.reserve_temp_local();
        let marker_payload_local = self.reserve_temp_local();
        let marker_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings.payload(PORFFOR_STATIC_GENERATOR_ITERATOR_SLOT),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            marker_present_local,
            marker_payload_local,
            marker_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(marker_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_define_bool_data(
            iterator_payload_local,
            "$ArrayIterator.done",
            true,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(marker_tag_local);
        self.release_temp_local(marker_payload_local);
        self.release_temp_local(marker_present_local);
        Ok(())
    }

    pub(crate) fn emit_iterator_close_preserving_current_throw(
        &mut self,
        close: IteratorCloseOnThrowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.save_current_completion(
            close.saved_payload_local,
            close.saved_tag_local,
            close.saved_completion_local,
            close.saved_aux_local,
            function,
        );
        self.emit_iterator_close_preserving_saved_throw(close, function)
    }

    pub(crate) fn emit_iterator_close_preserving_saved_throw(
        &mut self,
        close: IteratorCloseOnThrowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let close_frame = self.push_control(ControlFrameKind::Block);
        self.finally_stack.push(close_frame);
        self.emit_iterator_close(
            close.iterator_payload_local,
            close.iterator_tag_local,
            close.key_local,
            close.return_payload_local,
            close.return_tag_local,
            close.result_payload_local,
            close.result_tag_local,
            function,
        )?;
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.restore_saved_completion(
            close.saved_payload_local,
            close.saved_tag_local,
            close.saved_completion_local,
            close.saved_aux_local,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_iterator_flat_map_close_outer_after_throw(
        &mut self,
        helper_payload_local: u32,
        close: IteratorCloseOnThrowLocals,
        clear_inner_active: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_close_preserving_current_throw(close, function)?;
        self.emit_object_define_bool_data(
            helper_payload_local,
            "$IteratorFlatMapDone",
            true,
            function,
        )?;
        if clear_inner_active {
            self.emit_object_define_bool_data(
                helper_payload_local,
                "$IteratorFlatMapInnerActive",
                false,
                function,
            )?;
        }
        self.emit_object_define_bool_data(
            helper_payload_local,
            "$IteratorFlatMapExecuting",
            false,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn compile_for_in_array(
        &mut self,
        mode: BindingMode,
        name: &str,
        target: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_for_in_object(
            mode,
            name,
            target,
            body,
            lexical_environment,
            labels,
            function,
        )
    }

    pub(crate) fn compile_for_in_string(
        &mut self,
        mode: BindingMode,
        name: &str,
        target: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let string_payload_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let byte_len_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-in var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::String))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-in var storage must exist"),
                );
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(string_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(string_payload_local));
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            string_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_unpack_string_payload(
            string_payload_local,
            buffer_local,
            byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            buffer_local,
            byte_len_local,
            len_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-in lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, key_payload_local, key_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(byte_len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(string_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_array_contains_string_payload(
        &mut self,
        array_local: u32,
        len_local: u32,
        key_payload_local: u32,
        found_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let candidate_payload_local = self.reserve_temp_local();
        let candidate_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            array_local,
            index_local,
            candidate_payload_local,
            candidate_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(candidate_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_string_payload_equality_i32(candidate_payload_local, key_payload_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(candidate_tag_local);
        self.release_temp_local(candidate_payload_local);
        self.release_temp_local(index_local);
    }

    pub(crate) fn emit_for_in_append_unvisited_string_keys(
        &mut self,
        keys_array_local: u32,
        keys_len_local: u32,
        visited_array_local: u32,
        visited_len_local: u32,
        result_array_local: u32,
        result_len_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(keys_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_array_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_contains_string_payload(
            visited_array_local,
            visited_len_local,
            key_payload_local,
            found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            result_array_local,
            result_len_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(result_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
        Ok(())
    }

    pub(crate) fn emit_for_in_mark_visited_string_keys(
        &mut self,
        keys_array_local: u32,
        keys_len_local: u32,
        visited_array_local: u32,
        visited_len_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(keys_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_array_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_contains_string_payload(
            visited_array_local,
            visited_len_local,
            key_payload_local,
            found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            visited_array_local,
            visited_len_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(visited_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(visited_len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
        Ok(())
    }

    pub(crate) fn emit_for_in_object_key_snapshot(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        result_array_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let keys_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.keys`",
                )
            })?;
        let own_names_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyNames.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyNames`",
                )
            })?;
        let current_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let enumerable_keys_local = self.reserve_temp_local();
        let enumerable_keys_tag_local = self.reserve_temp_local();
        let enumerable_keys_len_local = self.reserve_temp_local();
        let own_names_local = self.reserve_temp_local();
        let own_names_tag_local = self.reserve_temp_local();
        let own_names_len_local = self.reserve_temp_local();
        let visited_array_local = self.reserve_temp_local();
        let visited_tag_local = self.reserve_temp_local();
        let result_len_local = self.reserve_temp_local();
        let visited_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_len_local));
        self.emit_alloc_array_payload_with_length(result_len_local, result_array_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(visited_len_local));
        self.emit_alloc_array_payload_with_length(
            visited_len_local,
            visited_array_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(visited_tag_local));

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        let prototype_break_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let prototype_loop_frame = self.push_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(self.depth_to(prototype_break_frame)));

        self.emit_direct_js_call(
            &keys_meta,
            None,
            &[(current_local, current_tag_local)],
            enumerable_keys_local,
            enumerable_keys_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(enumerable_keys_len_local));
        function.instruction(&Instruction::LocalGet(enumerable_keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            enumerable_keys_local,
            HEAP_LEN_OFFSET,
            enumerable_keys_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_for_in_append_unvisited_string_keys(
            enumerable_keys_local,
            enumerable_keys_len_local,
            visited_array_local,
            visited_len_local,
            result_array_local,
            result_len_local,
            function,
        )?;

        self.emit_direct_js_call(
            &own_names_meta,
            None,
            &[(current_local, current_tag_local)],
            own_names_local,
            own_names_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_names_len_local));
        function.instruction(&Instruction::LocalGet(own_names_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            own_names_local,
            HEAP_LEN_OFFSET,
            own_names_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_for_in_mark_visited_string_keys(
            own_names_local,
            own_names_len_local,
            visited_array_local,
            visited_len_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_PROTOTYPE_OFFSET,
            current_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::Br(self.depth_to(prototype_loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.release_temp_local(visited_len_local);
        self.release_temp_local(result_len_local);
        self.release_temp_local(visited_tag_local);
        self.release_temp_local(visited_array_local);
        self.release_temp_local(own_names_len_local);
        self.release_temp_local(own_names_tag_local);
        self.release_temp_local(own_names_local);
        self.release_temp_local(enumerable_keys_len_local);
        self.release_temp_local(enumerable_keys_tag_local);
        self.release_temp_local(enumerable_keys_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_local);
        Ok(())
    }

    pub(crate) fn compile_for_in_object(
        &mut self,
        mode: BindingMode,
        name: &str,
        target: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let entry_tag_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let should_iterate_local = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(target, object_local, object_tag_local, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }

        self.push_scope();
        let storage_without_environment = if mode == BindingMode::Var {
            Some(self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-in var `{name}`"
                ))
            })?)
        } else if !iteration_environment_owns_binding(lexical_environment, name) {
            Some(self.allocate_binding(name.to_string(), mode, ValueKind::String))
        } else {
            None
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("for-in var storage must exist"),
                );
        }
        self.emit_statement_result(function, ValueKind::Undefined);
        if target.kind != ValueKind::Dynamic {
            self.emit_for_in_object_key_snapshot(
                object_local,
                object_tag_local,
                buffer_local,
                entry_tag_local,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(len_local));
            function.instruction(&Instruction::LocalGet(entry_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(buffer_local, HEAP_LEN_OFFSET, len_local, function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::Block(BlockType::Empty));
            let break_frame = self.push_control(ControlFrameKind::Block);
            self.breakable_stack.push(break_frame);
            function.instruction(&Instruction::Loop(BlockType::Empty));
            let loop_frame = self.push_control(ControlFrameKind::Loop);
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
            self.emit_array_read(
                buffer_local,
                index_local,
                key_payload_local,
                key_tag_local,
                function,
            );
            if let Some(environment) = lexical_environment
                .and_then(|environment| environment.iteration_environment.as_ref())
            {
                self.emit_enter_lexical_environment(environment, function)?;
            }
            let storage = self
                .lookup_current_scope_binding(name)
                .or(storage_without_environment)
                .expect("for-in lexical storage must be allocated before assignment");
            self.write_binding_from_locals(storage, key_payload_local, key_tag_local, function);
            self.mirror_binding_to_global_object(name, storage, function)?;
            function.instruction(&Instruction::Block(BlockType::Empty));
            let continue_frame = self.push_control(ControlFrameKind::Block);
            self.loop_stack.push(LoopTargets { continue_frame });
            self.push_labels(labels, break_frame, Some(continue_frame));
            self.compile_statement(body, function)?;
            self.pop_labels(labels.len());
            self.loop_stack.pop();
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
            if lexical_environment
                .and_then(|environment| environment.iteration_environment.as_ref())
                .is_some()
            {
                self.emit_leave_lexical_environment(function);
            }
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
            self.pop_control(ControlFrameKind::Loop);
            function.instruction(&Instruction::End);
            self.breakable_stack.pop();
            self.pop_control(ControlFrameKind::Block);
            function.instruction(&Instruction::End);
            self.pop_scope();
            self.release_temp_local(should_iterate_local);
            self.release_temp_local(index_number_payload_local);
            self.release_temp_local(key_tag_local);
            self.release_temp_local(key_payload_local);
            self.release_temp_local(descriptor_kind_local);
            self.release_temp_local(entry_tag_local);
            self.release_temp_local(entry_local);
            self.release_temp_local(index_local);
            self.release_temp_local(len_local);
            self.release_temp_local(buffer_local);
            self.release_temp_local(object_tag_local);
            self.release_temp_local(object_local);
            return Ok(());
        }
        if target.kind == ValueKind::Dynamic {
            self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.push_control(ControlFrameKind::If);
        }
        self.load_i64_to_local_from_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(object_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        if target.kind == ValueKind::Dynamic {
            function.instruction(&Instruction::LocalGet(object_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
            function.instruction(&Instruction::End);
        } else {
            function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        }
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(should_iterate_local));
        if target.kind == ValueKind::Dynamic {
            function.instruction(&Instruction::LocalGet(object_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(should_iterate_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::F64ConvertI64U);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(index_number_payload_local));
            self.emit_number_to_string_payload(index_number_payload_local, function)?;
            function.instruction(&Instruction::LocalSet(key_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(key_tag_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
        }
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(should_iterate_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
        if target.kind == ValueKind::Dynamic {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(should_iterate_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-in lexical storage must be allocated before assignment");
        self.write_binding_from_locals(storage, key_payload_local, key_tag_local, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_frame = self.push_control(ControlFrameKind::Block);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        self.compile_statement(body, function)?;
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if lexical_environment
            .and_then(|environment| environment.iteration_environment.as_ref())
            .is_some()
        {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if target.kind == ValueKind::Dynamic {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_scope();
        self.release_temp_local(should_iterate_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_tag_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        Ok(())
    }
}
