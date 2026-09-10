use super::*;
use lila_ir::DirectEvalContextIr;

impl FunctionBuilder<'_> {
    pub(super) fn emit_direct_eval_call(
        &mut self,
        context: &DirectEvalContextIr,
        callee: &TypedExpr,
        this_arg: Option<&TypedExpr>,
        args: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_payload = self.reserve_temp_local();
        let callee_tag = self.reserve_temp_local();
        let receiver_payload = self.reserve_temp_local();
        let receiver_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(callee, callee_payload, callee_tag, function)?;
        self.emit_propagate_throw_from_locals_if_needed(callee_payload, callee_tag, function)?;
        if let Some(this_arg) = this_arg {
            self.compile_expr_to_locals(this_arg, receiver_payload, receiver_tag, function)?;
            self.emit_propagate_throw_from_locals_if_needed(
                receiver_payload,
                receiver_tag,
                function,
            )?;
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(receiver_payload));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(receiver_tag));
        }
        let (argc, argv) = self.emit_call_args_vector(args, function)?;
        self.emit_direct_eval_or_call_with_argv(
            context,
            callee_payload,
            callee_tag,
            receiver_payload,
            receiver_tag,
            argc,
            argv,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        self.release_temp_local(argv);
        self.release_temp_local(argc);
        self.release_temp_local(receiver_tag);
        self.release_temp_local(receiver_payload);
        self.release_temp_local(callee_tag);
        self.release_temp_local(callee_payload);
        Ok(())
    }

    pub(crate) fn emit_direct_eval_or_call_with_argv(
        &mut self,
        context: &DirectEvalContextIr,
        callee_payload: u32,
        callee_tag: u32,
        receiver_payload: u32,
        receiver_tag: u32,
        argc: u32,
        argv: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm = self.reserve_temp_local();
        let intrinsic = self.reserve_temp_local();
        self.emit_source_execution_realm_to_local(realm, function);
        self.emit_load_realm_eval_intrinsic_to_local(realm, intrinsic, function);
        function.instruction(&Instruction::LocalGet(callee_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(callee_payload));
        function.instruction(&Instruction::LocalGet(intrinsic));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_eval_argument(
            context,
            realm,
            argc,
            argv,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callee_payload,
            callee_tag,
            receiver_payload,
            receiver_tag,
            argc,
            argv,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(intrinsic);
        self.release_temp_local(realm);
        Ok(())
    }

    fn emit_direct_eval_argument(
        &mut self,
        context: &DirectEvalContextIr,
        realm: u32,
        argc: u32,
        argv: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(argc));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_read(argv, self.scratch_local, payload_local, tag_local, function);
        function.instruction(&Instruction::End);
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_eval_script_dispatch(context, realm, payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_direct_eval_script_dispatch(
        &mut self,
        context: &DirectEvalContextIr,
        realm: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries = self
            .functions
            .prepared_scripts()
            .iter()
            .filter(|entry| entry.kind == PreparedScriptKind::DirectEval(context.clone()))
            .cloned()
            .collect::<Vec<_>>();
        let expected_source = self.reserve_temp_local();
        function.instruction(&Instruction::Block(BlockType::Empty));
        for entry in entries {
            function.instruction(&Instruction::I64Const(self.strings.payload(&entry.source)));
            function.instruction(&Instruction::LocalSet(expected_source));
            self.emit_string_payload_equality_i32(payload_local, expected_source, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            let saved_realm = self.reserve_temp_local();
            function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(saved_realm));
            function.instruction(&Instruction::LocalGet(realm));
            function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
            match entry.outcome {
                PreparedScriptOutcome::DeferredSyntaxError { message } => {
                    let prototype = self.reserve_temp_local();
                    self.load_i64_to_local_from_offset(
                        realm,
                        HEAP_REALM_INTRINSICS_OFFSET,
                        prototype,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        prototype,
                        HEAP_REALM_INTRINSICS_SYNTAX_ERROR_PROTOTYPE_OFFSET,
                        prototype,
                        function,
                    );
                    self.emit_throw_runtime_error_with_prototype_local(
                        SYNTAX_ERROR_NAME,
                        &message,
                        prototype,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.release_temp_local(prototype);
                }
                PreparedScriptOutcome::Executable(unit) => {
                    let wasm_index = self
                        .functions
                        .get(&unit.id.function_id())
                        .expect("executable direct eval has a Wasm Script thunk")
                        .wasm_index;
                    let variable_environment = self.reserve_temp_local();
                    let private_environment = self.reserve_temp_local();
                    self.emit_eval_variable_environment_to_local(variable_environment, function);
                    self.emit_current_private_environment_to_local(private_environment, function);
                    let invocation = self.emit_capture_direct_eval_invocation(function)?;
                    function.instruction(&Instruction::LocalGet(self.current_env_local));
                    function.instruction(&Instruction::LocalGet(invocation.this_payload));
                    function.instruction(&Instruction::LocalGet(invocation.this_tag));
                    function.instruction(&Instruction::LocalGet(invocation.new_target_payload));
                    function.instruction(&Instruction::LocalGet(invocation.new_target_tag));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalGet(variable_environment));
                    function.instruction(&Instruction::LocalGet(private_environment));
                    function.instruction(&Instruction::LocalGet(invocation.execution_context));
                    function.instruction(&Instruction::Call(wasm_index));
                    self.store_call_results(payload_local, tag_local, function);
                    self.release_direct_eval_invocation(invocation);
                    self.release_temp_local(private_environment);
                    self.release_temp_local(variable_environment);
                }
            }
            function.instruction(&Instruction::LocalGet(saved_realm));
            function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
            self.release_temp_local(saved_realm);
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        self.emit_reject_dynamic_source(lila_ir::DynamicSourceRuntimeOperation::Eval, function);
        function.instruction(&Instruction::End);
        self.release_temp_local(expected_source);
        Ok(())
    }
}
