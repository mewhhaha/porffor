use super::*;

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
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1 + extra_depth));
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
        targets: &[(u32, usize)],
        extra_depth: u32,
        function: &mut Function,
    ) {
        for (target_id, frame) in targets {
            function.instruction(&Instruction::LocalGet(self.completion_aux_local));
            function.instruction(&Instruction::I64Const(*target_id as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Br(self.depth_to(*frame) + extra_depth));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
    }

    pub(crate) fn active_break_targets(&self) -> Vec<(u32, usize)> {
        let mut targets = Vec::new();
        for frame in self.breakable_stack.iter().rev() {
            let target_id = *frame as u32;
            if !targets.iter().any(|(id, _)| *id == target_id) {
                targets.push((target_id, *frame));
            }
        }
        targets
    }

    pub(crate) fn active_continue_targets(&self) -> Vec<(u32, usize)> {
        let mut targets = Vec::new();
        for target in self.loop_stack.iter().rev() {
            let target_id = target.continue_frame as u32;
            if !targets.iter().any(|(id, _)| *id == target_id) {
                targets.push((target_id, target.continue_frame));
            }
        }
        targets
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
        if let Some(target) = self.throw_handler_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1 + extra_depth));
        } else if let Some(target) = self.finally_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 1 + extra_depth));
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_RETURN));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.finally_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 2 + extra_depth));
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
        if let Some(target) = self.finally_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 3 + extra_depth));
        } else {
            let targets = self.active_break_targets();
            self.emit_dispatch_branch_completion(&targets, 4 + extra_depth, function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_CONTINUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(target) = self.finally_stack.last() {
            function.instruction(&Instruction::Br(self.depth_to(*target) + 4 + extra_depth));
        } else {
            let targets = self.active_continue_targets();
            self.emit_dispatch_branch_completion(&targets, 5 + extra_depth, function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn push_control(&mut self, kind: ControlFrameKind) -> usize {
        let index = self.control_stack.len();
        self.control_stack.push(kind);
        index
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

    pub(crate) fn depth_to(&self, target_index: usize) -> u32 {
        (self.control_stack.len() - 1 - target_index) as u32
    }

    pub(crate) fn push_labels(
        &mut self,
        labels: &[String],
        break_frame: usize,
        continue_frame: Option<usize>,
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
        if block.statements.is_empty() {
            self.emit_statement_result(function, ValueKind::Undefined);
            return Ok(());
        }

        for statement in &block.statements {
            self.compile_statement(statement, function)?;
        }

        Ok(())
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
                let storage = self.allocate_binding(name.clone(), *mode, init.kind);
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(init, value_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(value_local, tag_local, function)?;
                self.write_binding_from_locals(storage, value_local, tag_local, function);
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
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
                if let Some(target) = self.throw_handler_stack.last() {
                    function.instruction(&Instruction::Br(self.depth_to(*target)));
                } else if let Some(target) = self.finally_stack.last() {
                    function.instruction(&Instruction::Br(self.depth_to(*target)));
                } else {
                    self.emit_return_current_completion(function);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_name,
                catch_source_name,
                catch_block,
            } => {
                self.compile_try_catch(
                    try_block,
                    catch_name,
                    catch_source_name,
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
                catch_block,
                finally_block,
            } => {
                self.compile_try_catch_finally(
                    try_block,
                    catch_name,
                    catch_source_name,
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
            } => {
                self.compile_for(
                    init.as_ref(),
                    test.as_ref(),
                    update.as_ref(),
                    body,
                    &[],
                    function,
                )?;
            }
            StatementIr::ForOfArray {
                mode,
                name,
                iterable,
                body,
            } => self.compile_for_of_array(*mode, name, iterable, body, &[], function)?,
            StatementIr::ForOfString {
                mode,
                name,
                iterable,
                body,
            } => self.compile_for_of_string(*mode, name, iterable, body, &[], function)?,
            StatementIr::ForOfIterator {
                mode,
                name,
                iterable,
                body,
            } => self.compile_for_of_iterator(*mode, name, iterable, body, &[], function)?,
            StatementIr::ForInArray {
                mode,
                name,
                target,
                body,
            } => self.compile_for_in_array(*mode, name, target, body, &[], function)?,
            StatementIr::ForInString {
                mode,
                name,
                target,
                body,
            } => self.compile_for_in_string(*mode, name, target, body, &[], function)?,
            StatementIr::ForInObject {
                mode,
                name,
                target,
                body,
            } => self.compile_for_in_object(*mode, name, target, body, &[], function)?,
            StatementIr::Switch {
                discriminant,
                cases,
            } => {
                self.compile_switch(discriminant, cases, &[], function)?;
            }
            StatementIr::Debugger => {
                self.emit_statement_result(function, ValueKind::Undefined);
            }
            StatementIr::Return(value) => {
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
                if let Some(target) = self.finally_stack.last() {
                    self.set_completion_kind(CompletionKind::Return, function);
                    function.instruction(&Instruction::Br(self.depth_to(*target)));
                } else {
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
            } => {
                self.compile_for(
                    init.as_ref(),
                    test.as_ref(),
                    update.as_ref(),
                    body,
                    labels,
                    function,
                )?;
            }
            StatementIr::ForOfArray {
                mode,
                name,
                iterable,
                body,
            } => self.compile_for_of_array(*mode, name, iterable, body, labels, function)?,
            StatementIr::ForOfString {
                mode,
                name,
                iterable,
                body,
            } => self.compile_for_of_string(*mode, name, iterable, body, labels, function)?,
            StatementIr::ForOfIterator {
                mode,
                name,
                iterable,
                body,
            } => self.compile_for_of_iterator(*mode, name, iterable, body, labels, function)?,
            StatementIr::ForInArray {
                mode,
                name,
                target,
                body,
            } => self.compile_for_in_array(*mode, name, target, body, labels, function)?,
            StatementIr::ForInString {
                mode,
                name,
                target,
                body,
            } => self.compile_for_in_string(*mode, name, target, body, labels, function)?,
            StatementIr::ForInObject {
                mode,
                name,
                target,
                body,
            } => self.compile_for_in_object(*mode, name, target, body, labels, function)?,
            StatementIr::Switch {
                discriminant,
                cases,
            } => {
                self.compile_switch(discriminant, cases, labels, function)?;
            }
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: label on unsupported statement kind",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn compile_try_catch(
        &mut self,
        try_block: &BlockIr,
        catch_name: &str,
        catch_source_name: &str,
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
        let catch_storage = self.allocate_dynamic_binding_storage(catch_name);
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
        self.compile_block_contents(catch_block, function)?;
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
        let finally_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let catch_skip_frame = self.push_control(ControlFrameKind::Block);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let catch_frame = self.push_control(ControlFrameKind::Block);
        self.throw_handler_stack.push(catch_frame);
        self.finally_stack.push(finally_frame);
        self.push_scope();
        self.compile_block_contents(try_block, function)?;
        self.pop_scope();
        self.throw_handler_stack.pop();
        function.instruction(&Instruction::Br(self.depth_to(catch_skip_frame)));
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.push_scope();
        let catch_storage = self.allocate_dynamic_binding_storage(catch_name);
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
        self.compile_block_contents(catch_block, function)?;
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
        self.pop_labels(labels.len());
        self.loop_stack.pop();
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
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.push_scope();
        if let Some(init) = init {
            self.compile_for_init(init, function)?;
        }
        self.emit_statement_result(function, ValueKind::Undefined);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let break_frame = self.push_control(ControlFrameKind::Block);
        self.breakable_stack.push(break_frame);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame = self.push_control(ControlFrameKind::Loop);
        if let Some(test) = test {
            self.compile_truthy_i32(test, function)?;
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::BrIf(self.depth_to(break_frame)));
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
        self.pop_scope();
        Ok(())
    }

    pub(crate) fn compile_switch(
        &mut self,
        discriminant: &TypedExpr,
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
        self.push_scope();
        self.compile_expr_to_locals(
            discriminant,
            discriminant_payload_local,
            discriminant_tag_local,
            function,
        )?;
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
            self.compile_switch_case_match(
                discriminant,
                discriminant_payload_local,
                discriminant_tag_local,
                condition,
                function,
            )?;
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::LocalSet(selected_local));
            function.instruction(&Instruction::End);
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
            self.compile_expr_payload(condition, function)?;
            function.instruction(&Instruction::LocalSet(self.scratch_local));
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
        if let Some(target) = self.finally_stack.last() {
            self.set_completion_kind_with_aux(CompletionKind::Break, break_frame as i64, function);
            function.instruction(&Instruction::Br(self.depth_to(*target)));
            return Ok(());
        }
        function.instruction(&Instruction::Br(self.depth_to(break_frame)));
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
        if let Some(target) = self.finally_stack.last() {
            self.set_completion_kind_with_aux(
                CompletionKind::Continue,
                continue_frame as i64,
                function,
            );
            function.instruction(&Instruction::Br(self.depth_to(*target)));
            return Ok(());
        }
        function.instruction(&Instruction::Br(self.depth_to(continue_frame)));
        Ok(())
    }

    pub(crate) fn compile_for_init(
        &mut self,
        init: &ForInitIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match init {
            ForInitIr::Lexical { mode, name, init } => {
                let storage = self.allocate_binding(name.clone(), *mode, init.kind);
                self.compile_expr_to_binding(init, storage, function)?;
            }
            ForInitIr::LexicalBlock(bindings) => {
                for binding in bindings {
                    let storage = self.allocate_binding(
                        binding.name.clone(),
                        binding.mode,
                        binding.init.kind,
                    );
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

        self.push_scope();
        let storage = if mode == BindingMode::Var {
            self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?
        } else {
            self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic)
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(name.to_string(), storage);
        }
        self.compile_expr_to_locals(iterable, array_local, array_tag_local, function)?;
        let iteration_env_source_local = self.capture_iteration_env_source(mode, name, function);
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
        self.emit_enter_iteration_env(iteration_env_source_local, function)?;
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
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
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
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
        self.pop_scope();

        if let Some(iteration_env_source_local) = iteration_env_source_local {
            self.release_temp_local(iteration_env_source_local);
        }
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

        self.push_scope();
        let storage = if mode == BindingMode::Var {
            self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?
        } else {
            self.allocate_binding(name.to_string(), mode, ValueKind::String)
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(name.to_string(), storage);
        }

        self.compile_expr_to_locals(iterable, string_payload_local, string_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(string_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of target is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let iteration_env_source_local = self.capture_iteration_env_source(mode, name, function);
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
        self.emit_enter_iteration_env(iteration_env_source_local, function)?;
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
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
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
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
        self.pop_scope();

        if let Some(iteration_env_source_local) = iteration_env_source_local {
            self.release_temp_local(iteration_env_source_local);
        }
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

        self.push_scope();
        let storage = if mode == BindingMode::Var {
            self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-of var `{name}`"
                ))
            })?
        } else {
            self.allocate_binding(name.to_string(), mode, ValueKind::Dynamic)
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(name.to_string(), storage);
        }

        self.compile_expr_to_locals(
            iterable,
            iterable_payload_local,
            iterable_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(iterable_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of target is not iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
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
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator method must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
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
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
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
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(next_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator next must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let iteration_env_source_local = self.capture_iteration_env_source(mode, name, function);
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
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "for-of iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
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
        self.emit_return_current_completion_if_throw(function);
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
        self.emit_return_current_completion_if_throw(function);
        self.emit_enter_iteration_env(iteration_env_source_local, function)?;
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

        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_iterator_close_condition_i32(
            saved_completion_local,
            saved_aux_local,
            continue_frame,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
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
        function.instruction(&Instruction::End);
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
        function.instruction(&Instruction::End);

        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(self.depth_to(loop_frame)));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
        self.pop_scope();

        if let Some(iteration_env_source_local) = iteration_env_source_local {
            self.release_temp_local(iteration_env_source_local);
        }
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

    pub(crate) fn emit_iterator_close_condition_i32(
        &self,
        completion_local: u32,
        aux_local: u32,
        current_continue_frame: usize,
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
        function.instruction(&Instruction::I64Const(current_continue_frame as i64));
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
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "IteratorClose return method must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_function_handle_call(
            return_payload_local,
            return_tag_local,
            Some((iterator_payload_local, Some(iterator_tag_local))),
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "IteratorClose return result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
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
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::I64Const(self.strings.payload("return")));
        function.instruction(&Instruction::LocalSet(close.key_local));
        self.emit_object_read(
            close.iterator_payload_local,
            close.iterator_tag_local,
            close.iterator_payload_local,
            close.iterator_tag_local,
            close.key_local,
            close.return_payload_local,
            close.return_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(close.return_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_without_throw_propagation(
            close.return_payload_local,
            close.return_tag_local,
            Some((close.iterator_payload_local, Some(close.iterator_tag_local))),
            &[],
            close.result_payload_local,
            close.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_exhaust_static_generator_iterator_if_marked(
            close.iterator_payload_local,
            close.iterator_tag_local,
            close.key_local,
            function,
        )?;
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
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_for_in_object(mode, name, target, body, labels, function)
    }

    pub(crate) fn compile_for_in_string(
        &mut self,
        mode: BindingMode,
        name: &str,
        target: &TypedExpr,
        body: &StatementIr,
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

        self.push_scope();
        let storage = if mode == BindingMode::Var {
            self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-in var `{name}`"
                ))
            })?
        } else {
            self.allocate_binding(name.to_string(), mode, ValueKind::String)
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(name.to_string(), storage);
        }

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        let iteration_env_source_local = self.capture_iteration_env_source(mode, name, function);
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
        self.emit_enter_iteration_env(iteration_env_source_local, function)?;
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
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
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
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
        self.pop_scope();

        if let Some(iteration_env_source_local) = iteration_env_source_local {
            self.release_temp_local(iteration_env_source_local);
        }
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
        self.emit_return_current_completion_if_throw(function);
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
        self.emit_return_current_completion_if_throw(function);
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

        self.push_scope();
        let storage = if mode == BindingMode::Var {
            self.lookup_binding(name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: unbound for-in var `{name}`"
                ))
            })?
        } else {
            self.allocate_binding(name.to_string(), mode, ValueKind::String)
        };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(name.to_string(), storage);
        }

        self.compile_expr_to_locals(target, object_local, object_tag_local, function)?;
        let iteration_env_source_local = self.capture_iteration_env_source(mode, name, function);
        self.emit_statement_result(function, ValueKind::Undefined);
        if target.kind != ValueKind::Dynamic {
            self.emit_for_in_object_key_snapshot(
                object_local,
                object_tag_local,
                buffer_local,
                entry_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
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
            self.emit_enter_iteration_env(iteration_env_source_local, function)?;
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
            self.emit_exit_iteration_env(iteration_env_source_local, name, function);
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
            self.emit_exit_iteration_env(iteration_env_source_local, name, function);
            self.pop_scope();

            if let Some(iteration_env_source_local) = iteration_env_source_local {
                self.release_temp_local(iteration_env_source_local);
            }
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
        self.emit_enter_iteration_env(iteration_env_source_local, function)?;
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
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
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
            function.instruction(&Instruction::End);
        }
        self.emit_exit_iteration_env(iteration_env_source_local, name, function);
        self.pop_scope();

        if let Some(iteration_env_source_local) = iteration_env_source_local {
            self.release_temp_local(iteration_env_source_local);
        }
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
