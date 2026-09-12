use super::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_indirect_call(
        &mut self,
        callee: &TypedExpr,
        this_arg: Option<&TypedExpr>,
        args: &[TypedExpr],
        static_regexp_compilation: Option<&StaticRegExpCompilation>,
        direct_eval: Option<&lila_ir::DirectEvalContextIr>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(context) = direct_eval {
            return self.emit_direct_eval_call(
                context,
                callee,
                this_arg,
                args,
                &CallContinuation::Continue,
                payload_local,
                tag_local,
                function,
            );
        }
        if args.is_empty()
            && static_regexp_compilation.is_none()
            && self
                .current_function_meta()
                .is_some_and(|meta| meta.protocol.class_kind() == ClassFunctionKind::Constructor)
        {
            if let (ExprIr::FunctionValue(function_id), Some(this_arg)) = (&callee.expr, this_arg) {
                if matches!(this_arg.expr, ExprIr::This) {
                    let initializer_meta =
                        self.functions.get(function_id).cloned().filter(|meta| {
                            meta.class_element_execution_kind
                                == ClassElementExecutionKind::InstanceFieldInitializer
                        });
                    if let Some(initializer_meta) = initializer_meta {
                        let this_payload_local = self.reserve_temp_local();
                        let this_tag_local = self.reserve_temp_local();
                        self.compile_expr_to_locals(
                            this_arg,
                            this_payload_local,
                            this_tag_local,
                            function,
                        )?;
                        self.emit_direct_class_element_js_call(
                            &initializer_meta,
                            self.class_function_context_local,
                            Some((this_payload_local, Some(this_tag_local))),
                            &[],
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.release_temp_local(this_tag_local);
                        self.release_temp_local(this_payload_local);
                        return Ok(());
                    }
                }
            }
        }

        let string_match_function_id = StandardBuiltinId::StringPrototypeMatch.function_id();
        let string_split_function_id = StandardBuiltinId::StringPrototypeSplit.function_id();
        let string_slice_function_id = StandardBuiltinId::StringPrototypeSlice.function_id();
        if let (Some(function_id), Some(this_arg)) =
            (callee.function_targets.exact_single_target(), this_arg)
        {
            if function_id == &string_match_function_id
                || function_id == &string_split_function_id
                || function_id == &string_slice_function_id
            {
                if function_id == &string_match_function_id {
                    return self.emit_string_match_method_call(
                        this_arg,
                        args,
                        payload_local,
                        tag_local,
                        function,
                    );
                }
                if function_id == &string_slice_function_id {
                    return self.emit_string_slice_method_call(
                        this_arg,
                        args,
                        payload_local,
                        tag_local,
                        function,
                    );
                }
                return self.emit_string_split_method_call(
                    this_arg,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        if let (
            ExprIr::PropertyRead {
                key: PropertyKeyIr::StaticString(name),
                ..
            },
            Some(this_arg),
        ) = (&callee.expr, this_arg)
        {
            let string_or_undefined = KindSet::from_kind(ValueKind::String)
                .union(KindSet::from_kind(ValueKind::Undefined));
            if name == "split" && this_arg.possible_kinds.is_subset_of(string_or_undefined) {
                return self.emit_string_split_method_call(
                    this_arg,
                    args,
                    payload_local,
                    tag_local,
                    function,
                );
            }
        }
        let reflect_define_property_function_id =
            StandardBuiltinId::ReflectDefineProperty.function_id();
        let object_define_property_function_id =
            StandardBuiltinId::ObjectDefineProperty.function_id();
        let is_reflect_define_property_access = matches!(
            &callee.expr,
            ExprIr::PropertyRead {
                target,
                key: PropertyKeyIr::StaticString(name),
            } if name == "defineProperty"
                && matches!(
                    &target.expr,
                    ExprIr::GlobalPropertyRead { name } | ExprIr::Identifier(name)
                        if name == REFLECT_NAME
                )
        );
        if is_reflect_define_property_access
            && callee
                .function_targets
                .exact_single_target()
                .is_some_and(|function_id| {
                    function_id == &reflect_define_property_function_id
                        || function_id == &object_define_property_function_id
                })
        {
            let callee_payload_local = self.reserve_temp_local();
            let callee_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;
            let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
            let meta = self
                .functions
                .get(&reflect_define_property_function_id)
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                    )
                })?;
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            self.emit_default_this_for_known_strictness(meta.strict, function);
            self.emit_undefined_new_target(function);
            function.instruction(&Instruction::LocalGet(argc_local));
            function.instruction(&Instruction::LocalGet(argv_local));
            function.instruction(&Instruction::Call(meta.wasm_index));
            self.store_call_results(payload_local, tag_local, function);
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
            self.set_completion_kind(CompletionKind::Normal, function);
            self.release_temp_local(argv_local);
            self.release_temp_local(argc_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            return Ok(());
        }

        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();
        let default_this_payload_local = self.reserve_temp_local();
        let default_this_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;

        let this_locals = if let Some(this_arg) = this_arg {
            let this_payload_local = self.reserve_temp_local();
            let this_tag_local = self.reserve_temp_local();
            self.compile_expr_to_locals(this_arg, this_payload_local, this_tag_local, function)?;
            Some((this_payload_local, this_tag_local))
        } else {
            None
        };
        let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

        if let Some(StaticRegExpCompilation::InvalidSyntax { message }) = static_regexp_compilation
        {
            self.emit_throw_runtime_error(
                SYNTAX_ERROR_NAME,
                message,
                payload_local,
                tag_local,
                function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
            self.release_temp_local(argv_local);
            self.release_temp_local(argc_local);
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                self.release_temp_local(this_tag_local);
                self.release_temp_local(this_payload_local);
            }
            self.release_temp_local(default_this_tag_local);
            self.release_temp_local(default_this_payload_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            return Ok(());
        }

        let (this_payload_local, this_tag_local) =
            if let Some((this_payload_local, this_tag_local)) = this_locals {
                (this_payload_local, this_tag_local)
            } else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(default_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(default_this_tag_local));
                (default_this_payload_local, default_this_tag_local)
            };

        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            callee_payload_local,
            callee_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        if let Some(StaticRegExpCompilation::Program(program)) = static_regexp_compilation {
            if callee.function_targets.exact_single_target()
                == Some(&StandardBuiltinId::RegExpPrototypeCompile.function_id())
            {
                if let Some((this_payload_local, _)) = this_locals {
                    self.emit_regexp_program_slots(this_payload_local, Some(program), function);
                }
            } else {
                function.instruction(&Instruction::LocalGet(callee_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(callee_payload_local));
                function.instruction(&Instruction::GlobalGet(REGEXP_CONSTRUCTOR_GLOBAL_INDEX));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_regexp_program_slots(payload_local, Some(program), function);
                function.instruction(&Instruction::End);
            }
        }

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        if let Some((this_payload_local, this_tag_local)) = this_locals {
            self.release_temp_local(this_tag_local);
            self.release_temp_local(this_payload_local);
        }
        self.release_temp_local(default_this_tag_local);
        self.release_temp_local(default_this_payload_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        Ok(())
    }
}
