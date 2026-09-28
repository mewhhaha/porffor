use super::*;
use crate::abi::{JsCallParameter, PreparedScriptParameter};

#[must_use = "prepared Script execution must restore the caller's realm before returning"]
struct PreparedScriptRealmExecution {
    saved_realm_local: u32,
    realm_local: u32,
    environment_local: u32,
    global_object_local: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_module_prelude(
        &mut self,
        id: lila_ir::StaticScriptId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_local = self.reserve_temp_local();
        let environment_local = self.reserve_temp_local();
        let global_object_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
            environment_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_GLOBAL_OBJECT_OFFSET,
            global_object_local,
            function,
        );
        let wasm_index = self
            .functions
            .get(&id.function_id())
            .expect("Module prelude has a compiled Script thunk")
            .wasm_index;
        self.emit_prepared_script_thunk_call(
            environment_local,
            global_object_local,
            wasm_index,
            function,
        );
        self.release_temp_local(global_object_local);
        self.release_temp_local(environment_local);
        self.release_temp_local(realm_local);
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_statement_result(function, ValueKind::Undefined);
        Ok(())
    }

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
                    self.emit_prepared_script_thunk_call(
                        execution.environment_local,
                        execution.global_object_local,
                        meta.wasm_index,
                        function,
                    );
                }
            }
            self.emit_restore_prepared_script_realm(execution, function);
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        self.release_temp_local(expected_source_local);
        Ok(())
    }

    /// Both module preludes and prepared dispatch use the schema's complete
    /// parameter order. Extending either prefix forces this match to account
    /// for the new slot before the compiler can emit a mismatched call.
    fn emit_prepared_script_thunk_call(
        &self,
        environment_local: u32,
        global_object_local: u32,
        wasm_index: u32,
        function: &mut Function,
    ) {
        for slot in JsCallParameter::ALL {
            match slot {
                JsCallParameter::Environment => {
                    function.instruction(&Instruction::LocalGet(environment_local));
                }
                JsCallParameter::ThisPayload => {
                    function.instruction(&Instruction::LocalGet(global_object_local));
                }
                JsCallParameter::ThisTag => {
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                }
                JsCallParameter::NewTargetPayload | JsCallParameter::Argc => {
                    function.instruction(&Instruction::I64Const(0));
                }
                JsCallParameter::Argv => crate::gc_types::arg_vector::emit_null(function),
                JsCallParameter::NewTargetTag => {
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                }
            }
        }
        for slot in PreparedScriptParameter::ALL {
            match slot {
                PreparedScriptParameter::VariableEnvironment => {
                    function.instruction(&Instruction::LocalGet(environment_local));
                }
                PreparedScriptParameter::PrivateEnvironment
                | PreparedScriptParameter::DirectEvalContext => {
                    function.instruction(&Instruction::I64Const(0));
                }
            }
        }
        function.instruction(&Instruction::Call(wasm_index));
        self.store_call_results(
            crate::objects::TaggedLocals::new(self.result_local, self.result_tag_local),
            function,
        );
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
