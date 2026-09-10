use super::*;

#[must_use = "prepared Script execution must restore the caller's realm before returning"]
struct PreparedScriptRealmExecution {
    saved_realm_local: u32,
    realm_local: u32,
    environment_local: u32,
    global_object_local: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_prepared_script_dispatch(
        &mut self,
        kind: PreparedScriptKind,
        source_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries = self
            .functions
            .prepared_scripts()
            .iter()
            .filter(|entry| entry.kind == kind)
            .cloned()
            .collect::<Vec<_>>();
        let expected_source_local = self.reserve_temp_local();
        for entry in entries {
            function.instruction(&Instruction::I64Const(self.strings.payload(&entry.source)));
            function.instruction(&Instruction::LocalSet(expected_source_local));
            self.emit_string_payload_equality_i32(source_local, expected_source_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            let execution = self.emit_enter_prepared_script_realm(function);
            match entry.outcome {
                PreparedScriptOutcome::DeferredSyntaxError { message } => {
                    let prototype_local = self.reserve_temp_local();
                    self.load_i64_to_local_from_offset(
                        execution.realm_local,
                        HEAP_REALM_INTRINSICS_OFFSET,
                        prototype_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        prototype_local,
                        HEAP_REALM_INTRINSICS_SYNTAX_ERROR_PROTOTYPE_OFFSET,
                        prototype_local,
                        function,
                    );
                    self.emit_throw_runtime_error_with_prototype_local(
                        SYNTAX_ERROR_NAME,
                        &message,
                        prototype_local,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.release_temp_local(prototype_local);
                }
                PreparedScriptOutcome::Executable(unit) => {
                    let meta = self
                        .functions
                        .get(&unit.id.function_id())
                        .expect("executable prepared Script has a Wasm thunk");
                    function.instruction(&Instruction::LocalGet(execution.environment_local));
                    function.instruction(&Instruction::LocalGet(execution.global_object_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    self.emit_undefined_new_target(function);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Const(0));
                    // Script thunks extend the ordinary seven-parameter call
                    // prefix with explicit variable/private environments and
                    // an absent direct-eval execution context.
                    function.instruction(&Instruction::LocalGet(execution.environment_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::Call(meta.wasm_index));
                    self.store_call_results(self.result_local, self.result_tag_local, function);
                }
            }
            self.emit_restore_prepared_script_realm(execution, function);
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.release_temp_local(expected_source_local);
        Ok(())
    }

    fn emit_enter_prepared_script_realm(
        &mut self,
        function: &mut Function,
    ) -> PreparedScriptRealmExecution {
        let saved_realm_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();
        let environment_local = self.reserve_temp_local();
        let global_object_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(saved_realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(saved_realm_local));
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
            environment_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(environment_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_GLOBAL_OBJECT_OFFSET,
            global_object_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
        PreparedScriptRealmExecution {
            saved_realm_local,
            realm_local,
            environment_local,
            global_object_local,
        }
    }

    fn emit_restore_prepared_script_realm(
        &mut self,
        execution: PreparedScriptRealmExecution,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(execution.saved_realm_local));
        function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
        self.release_temp_local(execution.global_object_local);
        self.release_temp_local(execution.environment_local);
        self.release_temp_local(execution.realm_local);
        self.release_temp_local(execution.saved_realm_local);
    }
}
