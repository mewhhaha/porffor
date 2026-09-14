use super::*;

pub(crate) const FOR_IN_ENUMERATOR_TEMP_LOCALS: usize = 17;
// The saved StatementList pair stays live through descriptor [[Get]] and its
// generic call paths, whose planning ceiling is 96 temporary locals.
pub(crate) const FOR_IN_INTERNAL_METHOD_TEMP_LOCALS: usize = 2 + 96;
pub(crate) const FOR_IN_INTRINSICS: [StandardBuiltinId; 3] = [
    StandardBuiltinId::ReflectOwnKeys,
    StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
    StandardBuiltinId::ReflectGetPrototypeOf,
];

impl FunctionBuilder<'_> {
    pub(crate) fn compile_for_in(
        &mut self,
        mode: BindingMode,
        name: &str,
        target: &TypedExpr,
        body: &StatementIr,
        lexical_environment: Option<&ForInOfEnvironmentIr>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [own_keys, get_own_descriptor, get_prototype] = FOR_IN_INTRINSICS.map(|builtin| {
            self.functions
                .get(&builtin.function_id())
                .cloned()
                .expect("for-in internal methods are required by its intrinsic plan")
        });
        let current_payload = self.reserve_temp_local();
        let current_tag = self.reserve_temp_local();
        let keys_payload = self.reserve_temp_local();
        let keys_tag = self.reserve_temp_local();
        let keys_length = self.reserve_temp_local();
        let key_index = self.reserve_temp_local();
        let visited_payload = self.reserve_temp_local();
        let visited_length = self.reserve_temp_local();
        let key_payload = self.reserve_temp_local();
        let key_tag = self.reserve_temp_local();
        let visited = self.reserve_temp_local();
        let descriptor_payload = self.reserve_temp_local();
        let descriptor_tag = self.reserve_temp_local();
        let enumerable_key = self.reserve_temp_local();
        let enumerable_payload = self.reserve_temp_local();
        let enumerable_tag = self.reserve_temp_local();
        let should_yield = self.reserve_temp_local();
        let target_payload = self.reserve_temp_local();
        let target_tag = self.reserve_temp_local();

        if let Some(environment) = lexical_environment {
            self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?;
        }
        self.compile_expr_to_locals(target, target_payload, target_tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(target_payload, target_tag, function)?;
        if let Some(environment) = lexical_environment {
            self.emit_leave_for_in_of_tdz_scope(environment, function);
        }
        self.push_scope();
        let storage_without_environment =
            if mode == BindingMode::Var {
                Some(self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!("unbound for-in var `{name}`"))
                })?)
            } else if !iteration_environment_owns_binding(lexical_environment, name) {
                Some(self.allocate_binding(name.to_string(), mode, ValueKind::String))
            } else {
                None
            };
        if mode == BindingMode::Var {
            self.binding_scopes
                .last_mut()
                .expect("for-in binding scope exists")
                .insert(
                    name.to_string(),
                    storage_without_environment.expect("var storage exists"),
                );
        }
        self.emit_statement_result(function, ValueKind::Undefined);
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        self.compile_nullish_tagged_i32(target_tag, function)?;
        self.emit_branch_if_to_target(break_frame, function);
        self.emit_value_to_object_locals(
            target_payload,
            target_tag,
            current_payload,
            current_tag,
            function,
        )?;
        self.release_temp_local(target_tag);
        self.release_temp_local(target_payload);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(visited_length));
        self.emit_alloc_array_payload_with_length(visited_length, visited_payload, function)?;
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key));
        self.emit_statement_result(function, ValueKind::Undefined);

        let prototype_loop = self.open_frame(ControlFrameKind::Loop, function);
        let saved = self.save_statement_list_value(function);
        self.emit_direct_js_call(
            &own_keys,
            None,
            &[(current_payload, current_tag)],
            keys_payload,
            keys_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(keys_payload, keys_tag, function)?;
        self.restore_statement_list_value(saved, function)?;
        self.load_i64_to_local_from_offset(keys_payload, HEAP_LEN_OFFSET, keys_length, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_index));
        let keys_exhausted = self.open_frame(ControlFrameKind::Block, function);
        let key_loop = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(key_index));
        function.instruction(&Instruction::LocalGet(keys_length));
        function.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(keys_exhausted, function);
        self.emit_array_read(keys_payload, key_index, key_payload, key_tag, function);
        function.instruction(&Instruction::LocalGet(key_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(key_index));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(should_yield));

        let saved = self.save_statement_list_value(function);
        let next_property = self.open_frame(ControlFrameKind::Block, function);
        function.instruction(&Instruction::LocalGet(key_tag));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(next_property, function);
        self.emit_for_in_visited_key(
            visited_payload,
            visited_length,
            key_payload,
            visited,
            function,
        );
        function.instruction(&Instruction::LocalGet(visited));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(next_property, function);
        self.emit_direct_js_call(
            &get_own_descriptor,
            None,
            &[(current_payload, current_tag), (key_payload, key_tag)],
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
        self.emit_branch_if_to_target(next_property, function);
        // A missing descriptor does not shadow an inherited property. An
        // existing non-enumerable descriptor does, and never calls its getter.
        self.emit_array_write(
            visited_payload,
            visited_length,
            key_payload,
            key_tag,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(visited_length));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(visited_length));
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
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(should_yield));
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.restore_statement_list_value(saved, function)?;

        function.instruction(&Instruction::LocalGet(should_yield));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        if let Some(environment) =
            lexical_environment.and_then(|environment| environment.iteration_environment.as_ref())
        {
            self.emit_enter_lexical_environment(environment, function)?;
        }
        let storage = self
            .lookup_current_scope_binding(name)
            .or(storage_without_environment)
            .expect("for-in binding is allocated before key publication");
        let saved = self.save_statement_list_value(function);
        self.write_binding_from_locals(storage, key_payload, key_tag, function);
        self.mirror_binding_to_global_object(name, storage, function)?;
        self.restore_statement_list_value(saved, function)?;
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
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
        self.emit_branch_to_target(key_loop, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        let saved = self.save_statement_list_value(function);
        self.emit_direct_js_call(
            &get_prototype,
            None,
            &[(current_payload, current_tag)],
            descriptor_payload,
            descriptor_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            descriptor_payload,
            descriptor_tag,
            function,
        )?;
        self.restore_statement_list_value(saved, function)?;
        function.instruction(&Instruction::LocalGet(descriptor_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_branch_if_to_target(break_frame, function);
        function.instruction(&Instruction::LocalGet(descriptor_payload));
        function.instruction(&Instruction::LocalSet(current_payload));
        function.instruction(&Instruction::LocalGet(descriptor_tag));
        function.instruction(&Instruction::LocalSet(current_tag));
        self.emit_branch_to_target(prototype_loop, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.pop_scope();
        for local in [
            should_yield,
            enumerable_tag,
            enumerable_payload,
            enumerable_key,
            descriptor_tag,
            descriptor_payload,
            visited,
            key_tag,
            key_payload,
            visited_length,
            visited_payload,
            key_index,
            keys_length,
            keys_tag,
            keys_payload,
            current_tag,
            current_payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_for_in_visited_key(
        &mut self,
        visited_payload: u32,
        visited_length: u32,
        key_payload: u32,
        found: u32,
        function: &mut Function,
    ) {
        let index = self.reserve_temp_local();
        let candidate_payload = self.reserve_temp_local();
        let candidate_tag = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        let complete = self.open_frame(ControlFrameKind::Block, function);
        let scan = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(visited_length));
        function.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(complete, function);
        self.emit_array_read(
            visited_payload,
            index,
            candidate_payload,
            candidate_tag,
            function,
        );
        self.emit_string_payload_equality_i32(candidate_payload, key_payload, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found));
        self.emit_branch_to_target(complete, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        self.emit_branch_to_target(scan, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.release_temp_local(candidate_tag);
        self.release_temp_local(candidate_payload);
        self.release_temp_local(index);
    }
}
