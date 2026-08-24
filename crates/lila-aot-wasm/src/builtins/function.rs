use super::super::*;

mod constructor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FunctionBuiltin {
    Constructor,
    Prototype,
    PrototypeSymbolHasInstance,
    PrototypeCall,
    PrototypeApply,
    PrototypeBind,
    PrototypeToString,
    BoundFunctionInvoker,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_function_builtin(
        &mut self,
        builtin: FunctionBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            FunctionBuiltin::Constructor => self.compile_function_constructor_builtin(function)?,
            FunctionBuiltin::Prototype => {
                // [[Call]] returns undefined; constructability is catalogued.
                self.emit_undefined_payload(function);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            FunctionBuiltin::PrototypeSymbolHasInstance => {
                let missing_receiver = || {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype[Symbol.hasInstance] receiver",
                    )
                };
                let constructor_payload_local =
                    self.this_payload_local.ok_or_else(missing_receiver)?;
                let constructor_tag_local = self.this_tag_local.ok_or_else(missing_receiver)?;
                let object_payload_local = self.reserve_temp_local();
                let object_tag_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(
                    0,
                    object_payload_local,
                    object_tag_local,
                    function,
                );
                self.emit_ordinary_has_instance_from_locals(
                    constructor_payload_local,
                    constructor_tag_local,
                    object_payload_local,
                    object_tag_local,
                    self.result_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(object_tag_local);
                self.release_temp_local(object_payload_local);
            }
            FunctionBuiltin::PrototypeCall => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype.call receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype.call receiver",
                    )
                })?;
                let this_arg_payload_local = self.reserve_temp_local();
                let this_arg_tag_local = self.reserve_temp_local();
                let argc_local = self.reserve_temp_local();
                let argv_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(
                    0,
                    this_arg_payload_local,
                    this_arg_tag_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(argc_local));
                self.emit_rest_array_payload(1, function)?;
                function.instruction(&Instruction::LocalSet(argv_local));

                self.emit_function_or_proxy_call_with_argv_without_throw_propagation(
                    receiver_payload_local,
                    receiver_tag_local,
                    this_arg_payload_local,
                    this_arg_tag_local,
                    argc_local,
                    argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;

                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(this_arg_tag_local);
                self.release_temp_local(this_arg_payload_local);
            }
            FunctionBuiltin::PrototypeApply => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype.apply receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype.apply receiver",
                    )
                })?;
                let this_arg_payload_local = self.reserve_temp_local();
                let this_arg_tag_local = self.reserve_temp_local();
                let apply_args_payload_local = self.reserve_temp_local();
                let apply_args_tag_local = self.reserve_temp_local();
                let argc_local = self.reserve_temp_local();
                let argv_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(
                    0,
                    this_arg_payload_local,
                    this_arg_tag_local,
                    function,
                );
                self.emit_builtin_arg_to_locals(
                    1,
                    apply_args_payload_local,
                    apply_args_tag_local,
                    function,
                );

                self.emit_is_callable_i32(receiver_tag_local, receiver_payload_local, function)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Function.prototype.apply receiver is not callable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);

                self.compile_nullish_tagged_i32(apply_args_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(argc_local));
                self.emit_alloc_array_payload_with_length(argc_local, argv_local, function)?;
                function.instruction(&Instruction::Else);
                self.emit_array_like_snapshot_payload(
                    apply_args_payload_local,
                    apply_args_tag_local,
                    argv_local,
                    "Function.prototype.apply argument list must be array-like",
                    function,
                )?;
                function.instruction(&Instruction::End);
                self.load_i64_to_local_from_offset(
                    argv_local,
                    HEAP_LEN_OFFSET,
                    argc_local,
                    function,
                );

                self.emit_function_or_proxy_call_with_argv_without_throw_propagation(
                    receiver_payload_local,
                    receiver_tag_local,
                    this_arg_payload_local,
                    this_arg_tag_local,
                    argc_local,
                    argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;

                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(apply_args_tag_local);
                self.release_temp_local(apply_args_payload_local);
                self.release_temp_local(this_arg_tag_local);
                self.release_temp_local(this_arg_payload_local);
            }
            FunctionBuiltin::PrototypeBind => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype.bind receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype.bind receiver",
                    )
                })?;
                let bound_args_payload_local = self.reserve_temp_local();

                self.emit_rest_array_payload(1, function)?;
                function.instruction(&Instruction::LocalSet(bound_args_payload_local));

                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_alloc_bound_function_for_bind(
                    receiver_payload_local,
                    receiver_tag_local,
                    bound_args_payload_local,
                    self.result_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Function.prototype.bind receiver is not callable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);

                self.release_temp_local(bound_args_payload_local);
            }
            FunctionBuiltin::PrototypeToString => {
                let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype.toString receiver",
                    )
                })?;
                let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing Function.prototype.toString receiver",
                    )
                })?;
                let proxy_target_payload_local = self.reserve_temp_local();
                let proxy_target_tag_local = self.reserve_temp_local();
                let proxy_handler_payload_local = self.reserve_temp_local();
                let callable_proxy_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    receiver_payload_local,
                    HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
                    self.result_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(callable_proxy_local));
                function.instruction(&Instruction::LocalGet(receiver_payload_local));
                function.instruction(&Instruction::LocalSet(proxy_target_payload_local));
                function.instruction(&Instruction::LocalGet(receiver_tag_local));
                function.instruction(&Instruction::LocalSet(proxy_target_tag_local));
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Br(2));
                function.instruction(&Instruction::End);
                self.load_i64_to_local_from_offset(
                    proxy_target_payload_local,
                    HEAP_OBJECT_BOXED_KIND_OFFSET,
                    proxy_handler_payload_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
                function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
                function.instruction(&Instruction::I64LtU);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Br(2));
                function.instruction(&Instruction::End);
                self.load_i64_to_local_from_offset(
                    proxy_target_payload_local,
                    HEAP_OBJECT_BOXED_TAG_OFFSET,
                    proxy_target_tag_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    proxy_target_payload_local,
                    HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                    proxy_target_payload_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(callable_proxy_local));
                function.instruction(&Instruction::Br(2));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(callable_proxy_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("function () { [native code] }"),
                ));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_current_function_realm_type_error(
                    "Function.prototype.toString receiver is not callable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                self.release_temp_local(callable_proxy_local);
                self.release_temp_local(proxy_handler_payload_local);
                self.release_temp_local(proxy_target_tag_local);
                self.release_temp_local(proxy_target_payload_local);
            }
            FunctionBuiltin::BoundFunctionInvoker => {
                let record_local = self.current_env_local;
                let target_payload_local = self.reserve_temp_local();
                let target_tag_local = self.reserve_temp_local();
                let bound_this_payload_local = self.reserve_temp_local();
                let bound_this_tag_local = self.reserve_temp_local();
                let bound_args_payload_local = self.reserve_temp_local();
                let self_payload_local = self.reserve_temp_local();
                let self_tag_local = self.reserve_temp_local();
                let merged_argv_local = self.reserve_temp_local();
                let merged_argc_local = self.reserve_temp_local();
                let forwarded_new_target_payload_local = self.reserve_temp_local();
                let forwarded_new_target_tag_local = self.reserve_temp_local();

                self.emit_load_bound_function_record(
                    record_local,
                    target_payload_local,
                    target_tag_local,
                    bound_this_payload_local,
                    bound_this_tag_local,
                    bound_args_payload_local,
                    self_payload_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(self_tag_local));
                self.emit_concat_argv_payloads(
                    bound_args_payload_local,
                    self.argv_param_local(),
                    merged_argv_local,
                    function,
                )?;
                self.load_i64_to_local_from_offset(
                    merged_argv_local,
                    HEAP_LEN_OFFSET,
                    merged_argc_local,
                    function,
                );
                self.compile_new_target_to_locals(
                    forwarded_new_target_payload_local,
                    forwarded_new_target_tag_local,
                    function,
                )?;

                function.instruction(&Instruction::LocalGet(forwarded_new_target_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_function_handle_call_with_argv(
                    target_payload_local,
                    target_tag_local,
                    Some((bound_this_payload_local, Some(bound_this_tag_local))),
                    merged_argc_local,
                    merged_argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_tagged_payload_same_value_i32(
                    forwarded_new_target_tag_local,
                    forwarded_new_target_payload_local,
                    self_tag_local,
                    self_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(target_payload_local));
                function.instruction(&Instruction::LocalSet(forwarded_new_target_payload_local));
                function.instruction(&Instruction::LocalGet(target_tag_local));
                function.instruction(&Instruction::LocalSet(forwarded_new_target_tag_local));
                function.instruction(&Instruction::End);
                self.emit_function_handle_construct_with_argv(
                    target_payload_local,
                    target_tag_local,
                    forwarded_new_target_payload_local,
                    forwarded_new_target_tag_local,
                    merged_argc_local,
                    merged_argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);

                self.release_temp_local(forwarded_new_target_tag_local);
                self.release_temp_local(forwarded_new_target_payload_local);
                self.release_temp_local(merged_argc_local);
                self.release_temp_local(merged_argv_local);
                self.release_temp_local(self_tag_local);
                self.release_temp_local(self_payload_local);
                self.release_temp_local(bound_args_payload_local);
                self.release_temp_local(bound_this_tag_local);
                self.release_temp_local(bound_this_payload_local);
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_payload_local);
            }
        }
        Ok(())
    }
}
