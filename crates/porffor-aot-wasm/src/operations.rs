use super::*;
use porffor_ir::StaticRegExpCompilation;

/// A binary operator that wants both operands as Numbers.
///
/// It exists to carry one bit that is easy to get wrong by hand:
/// `ApplyStringOrNumericBinaryOperator` (13.15.3) applies ToPrimitive to *both*
/// operands before ToNumeric to *either*, but only for `+` - step 1 is guarded
/// on the operator. Every other operator runs ToNumeric(lhs) to completion and
/// only then ToNumeric(rhs). The difference shows up whenever ToNumeric(lhs)
/// throws and the rhs has a hook: `obj1 + obj2` runs `obj2.valueOf()` before
/// the Symbol TypeError, `obj1 ^ obj2` does not.
///
/// Callers pass the operator they already hold rather than an order flag, so
/// there is no way to select the wrong order for an operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NumericBinaryOperator {
    Arithmetic(ArithmeticBinaryOp),
    Bitwise(BitwiseBinaryOp),
}

impl NumericBinaryOperator {
    fn applies_to_primitive_before_numeric(self) -> bool {
        match self {
            Self::Arithmetic(op) => match op {
                ArithmeticBinaryOp::Add => true,
                ArithmeticBinaryOp::Sub
                | ArithmeticBinaryOp::Mul
                | ArithmeticBinaryOp::Div
                | ArithmeticBinaryOp::Mod
                | ArithmeticBinaryOp::Exp => false,
            },
            Self::Bitwise(_) => false,
        }
    }
}

fn spec_operation_property_key_operand(key: &TypedExpr) -> PropertyKeyIr {
    match &key.expr {
        ExprIr::String(value) if key.kind == ValueKind::String => {
            PropertyKeyIr::StaticString(value.clone())
        }
        _ => PropertyKeyIr::StringExpr(Box::new(key.clone())),
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_construct(
        &mut self,
        callee: &TypedExpr,
        args: &[TypedExpr],
        static_regexp_compilation: Option<&StaticRegExpCompilation>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callee_payload_local = self.reserve_temp_local();
        let callee_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;
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
            self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                payload_local,
                tag_local,
                0,
                function,
            )?;
            self.release_temp_local(argv_local);
            self.release_temp_local(argc_local);
            self.release_temp_local(callee_tag_local);
            self.release_temp_local(callee_payload_local);
            return Ok(());
        }
        self.emit_function_or_proxy_construct_with_argv(
            callee_payload_local,
            callee_tag_local,
            callee_payload_local,
            callee_tag_local,
            argc_local,
            argv_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            0,
            function,
        )?;
        if let Some(StaticRegExpCompilation::Program(program)) = static_regexp_compilation {
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

        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.release_temp_local(callee_tag_local);
        self.release_temp_local(callee_payload_local);
        Ok(())
    }

    pub(crate) fn emit_instanceof_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let static_rhs_prototype_global = rhs
            .function_targets
            .iter()
            .next()
            .filter(|_| rhs.function_targets.len() == 1)
            .and_then(|function_id| StandardBuiltinId::from_function_id(function_id))
            .and_then(standard_builtin_prototype_global_index)
            .or_else(|| match &rhs.expr {
                ExprIr::GlobalPropertyRead { name } | ExprIr::Identifier(name) => {
                    match name.as_str() {
                        OBJECT_NAME => Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                        FUNCTION_NAME => Some(FUNCTION_PROTOTYPE_GLOBAL_INDEX),
                        ARRAY_NAME => Some(ARRAY_PROTOTYPE_GLOBAL_INDEX),
                        NUMBER_NAME => Some(NUMBER_PROTOTYPE_GLOBAL_INDEX),
                        STRING_NAME => Some(STRING_PROTOTYPE_GLOBAL_INDEX),
                        BOOLEAN_NAME => Some(BOOLEAN_PROTOTYPE_GLOBAL_INDEX),
                        ERROR_NAME => Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
                        EVAL_ERROR_NAME => Some(EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX),
                        AGGREGATE_ERROR_NAME => Some(AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX),
                        SUPPRESSED_ERROR_NAME => Some(SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX),
                        RANGE_ERROR_NAME => Some(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX),
                        SYNTAX_ERROR_NAME => Some(SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX),
                        TYPE_ERROR_NAME => Some(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX),
                        URI_ERROR_NAME => Some(URI_ERROR_PROTOTYPE_GLOBAL_INDEX),
                        REFERENCE_ERROR_NAME => Some(REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX),
                        _ => None,
                    }
                }
                _ => None,
            });
        let lhs_payload_local = self.reserve_temp_local();
        let lhs_tag_local = self.reserve_temp_local();
        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();
        let proto_key_local = self.reserve_temp_local();
        let rhs_proto_payload_local = self.reserve_temp_local();
        let rhs_proto_tag_local = self.reserve_temp_local();
        let search_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let next_proto_local = self.reserve_temp_local();
        let next_proto_tag_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();

        self.compile_expr_to_locals(lhs, lhs_payload_local, lhs_tag_local, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(proto_key_local));
        if let Some(prototype_global) = static_rhs_prototype_global {
            function.instruction(&Instruction::GlobalGet(prototype_global));
            function.instruction(&Instruction::LocalSet(rhs_proto_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(rhs_proto_tag_local));
        } else {
            // InstanceofOperator step 1 / OrdinaryHasInstance step 1: the right-hand
            // side must be an object (and, once we reach OrdinaryHasInstance, callable).
            // A primitive right-hand side throws a TypeError about the `instanceof`
            // operand here, rather than reading `prototype` off a non-object and
            // surfacing an unrelated error message.
            function.instruction(&Instruction::LocalGet(rhs_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            self.emit_is_heap_object_like_tag_i32(rhs_tag_local, function);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Right-hand side of 'instanceof' is not callable",
                rhs_proto_payload_local,
                rhs_proto_tag_local,
                function,
            )?;
            // The throw sits inside the guard `If` opened above, so the dispatch
            // has to skip that extra frame when it branches to an active catch
            // handler; without it the branch lands on the guard's own `End` and
            // execution falls through as if nothing had been thrown.
            self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(rhs_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                rhs_payload_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                rhs_proto_payload_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                rhs_payload_local,
                HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                rhs_proto_tag_local,
                function,
            );
            function.instruction(&Instruction::Else);
            self.emit_object_read(
                rhs_payload_local,
                rhs_tag_local,
                rhs_payload_local,
                rhs_tag_local,
                proto_key_local,
                rhs_proto_payload_local,
                rhs_proto_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        }

        self.emit_is_heap_object_like_tag_i32(rhs_proto_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Function has non-object prototype in instanceof check",
            rhs_proto_payload_local,
            rhs_proto_tag_local,
            function,
        )?;
        // Same extra frame as the callability guard above: the dispatch is
        // nested inside this `If`, so an active catch handler is one level
        // further away than the default depth assumes.
        self.emit_dispatch_current_completion_with_extra_depth(1, function)?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        for kind in [ValueKind::Array, ValueKind::Function, ValueKind::Arguments] {
            function.instruction(&Instruction::LocalGet(lhs_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalSet(search_local));
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalSet(search_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(search_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_object_get_prototype_of_with_depth(
                search_local,
                search_tag_local,
                next_proto_local,
                next_proto_tag_local,
                3,
                function,
            )?;
        } else {
            self.emit_ordinary_get_prototype_of(
                search_local,
                search_tag_local,
                next_proto_local,
                next_proto_tag_local,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(next_proto_local));
        function.instruction(&Instruction::LocalGet(rhs_proto_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_proto_local));
        function.instruction(&Instruction::LocalSet(search_local));
        function.instruction(&Instruction::LocalGet(next_proto_tag_local));
        function.instruction(&Instruction::LocalSet(search_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);

        self.release_temp_local(found_local);
        self.release_temp_local(next_proto_tag_local);
        self.release_temp_local(next_proto_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_local);
        self.release_temp_local(rhs_proto_tag_local);
        self.release_temp_local(rhs_proto_payload_local);
        self.release_temp_local(proto_key_local);
        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.release_temp_local(lhs_tag_local);
        self.release_temp_local(lhs_payload_local);
        Ok(())
    }

    pub(crate) fn emit_update_delta(
        &self,
        op: NumericUpdateOp,
        value_kind: ValueKind,
        function: &mut Function,
    ) {
        match value_kind {
            ValueKind::Number => {
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                match op {
                    NumericUpdateOp::Increment => function.instruction(&Instruction::F64Add),
                    NumericUpdateOp::Decrement => function.instruction(&Instruction::F64Sub),
                };
                function.instruction(&Instruction::I64ReinterpretF64);
            }
            ValueKind::BigInt => {
                function.instruction(&Instruction::I64Const(1));
                match op {
                    NumericUpdateOp::Increment => function.instruction(&Instruction::I64Add),
                    NumericUpdateOp::Decrement => function.instruction(&Instruction::I64Sub),
                };
            }
            _ => unreachable!("update delta only supports Number and BigInt"),
        }
    }

    pub(crate) fn emit_update_delta_from_locals(
        &self,
        op: NumericUpdateOp,
        value_kind: ValueKind,
        value_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        if value_kind != ValueKind::Dynamic {
            function.instruction(&Instruction::LocalGet(value_local));
            self.emit_update_delta(op, value_kind, function);
            return;
        }

        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Const(1));
        match op {
            NumericUpdateOp::Increment => function.instruction(&Instruction::I64Add),
            NumericUpdateOp::Decrement => function.instruction(&Instruction::I64Sub),
        };
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        match op {
            NumericUpdateOp::Increment => function.instruction(&Instruction::F64Add),
            NumericUpdateOp::Decrement => function.instruction(&Instruction::F64Sub),
        };
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_truthy_i32(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let truthy_local = self.reserve_temp_local();
        if !expr.possible_kinds.is_singleton() || expr_result_tag_is_runtime_dynamic(&expr.expr) {
            self.compile_expr_to_locals(expr, self.scratch_local, self.result_tag_local, function)?;
            self.compile_truthy_tagged_i32(self.result_tag_local, self.scratch_local, function)?;
        } else {
            self.compile_expr_payload(expr, function)?;
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.compile_truthy_local_i32(expr.kind, self.scratch_local, function)?;
        }
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(truthy_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(truthy_local));
        function.instruction(&Instruction::I32WrapI64);
        self.release_temp_local(truthy_local);
        Ok(())
    }

    pub(crate) fn compile_spec_operation_payload(
        &mut self,
        operation: SpecOperationIr,
        operands: &[TypedExpr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match operation {
            SpecOperationIr::IsCallable => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                self.emit_is_callable_i32(tag_local, payload_local, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::IsConstructor => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                self.emit_is_constructor_i32(tag_local, payload_local, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::IsPropertyKey => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                self.emit_is_property_key_i32(tag_local, function);
                function.instruction(&Instruction::I64ExtendI32U);
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::ToBoolean => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.emit_to_boolean_payload_from_expr(operand, function)
            }
            SpecOperationIr::ToPrimitive(_) => {
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_spec_operation_to_locals(
                    operation,
                    operands,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(payload_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::ToNumeric => {
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_spec_operation_to_locals(
                    operation,
                    operands,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(payload_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::ToNumber => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_expr_to_number_payload(operand, function)
            }
            SpecOperationIr::ToBigInt => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                self.emit_value_to_bigint_locals(
                    tag_local,
                    payload_local,
                    false,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(payload_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::ToString => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                // `emit_value_to_string_payload` routes Object/Array/Arguments
                // through the shared outlined ToString helper, which
                // hard-returns the whole function on a Symbol/ToPrimitive
                // throw instead of reaching an enclosing in-function
                // try/catch. This is the static `String(x)`/template-literal
                // ToString-coercion site (see `SpecOperationIr::ToString`'s
                // lowering) and sits at tracked block depth, so dispatch
                // Object/Array/Arguments ourselves via
                // `emit_tagged_to_primitive_locals` (which leaves an abrupt
                // completion in locals without branching) and propagate
                // correctly here.
                let primitive_payload_local = self.reserve_temp_local();
                let primitive_tag_local = self.reserve_temp_local();
                self.emit_tagged_to_primitive_locals(
                    ToPrimitiveHint::String,
                    payload_local,
                    tag_local,
                    primitive_payload_local,
                    primitive_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    primitive_payload_local,
                    primitive_tag_local,
                    function,
                )?;
                self.emit_primitive_to_string_payload(
                    primitive_payload_local,
                    primitive_tag_local,
                    function,
                )?;
                self.release_temp_local(primitive_tag_local);
                self.release_temp_local(primitive_payload_local);
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::ToObject => {
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_spec_operation_to_locals(
                    operation,
                    operands,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(payload_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::ToPropertyKey => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                self.emit_value_to_property_key_locals(payload_local, tag_local, function)?;
                function.instruction(&Instruction::LocalGet(payload_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::ToIntegerOrInfinity => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let number_payload_local = self.reserve_temp_local();
                let integer_payload_local = self.reserve_temp_local();
                self.compile_expr_to_number_payload(operand, function)?;
                function.instruction(&Instruction::LocalSet(number_payload_local));
                self.emit_to_integer_or_infinity_number_payload_from_number_payload(
                    number_payload_local,
                    integer_payload_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(integer_payload_local));
                self.release_temp_local(integer_payload_local);
                self.release_temp_local(number_payload_local);
                Ok(())
            }
            SpecOperationIr::ToLength => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                let length_local = self.reserve_temp_local();
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                self.emit_to_length_i64_from_value_locals(
                    tag_local,
                    payload_local,
                    length_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(length_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                self.release_temp_local(length_local);
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::ToIndex => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                let index_local = self.reserve_temp_local();
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                self.emit_to_index_i64_from_value_locals(
                    tag_local,
                    payload_local,
                    index_local,
                    "ToIndex out of range",
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                self.release_temp_local(index_local);
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::SameValue => {
                let [lhs, rhs] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_same_value_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                Ok(())
            }
            SpecOperationIr::SameValueZero => {
                let [lhs, rhs] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_same_value_zero_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                Ok(())
            }
            SpecOperationIr::StrictEqualityComparison => {
                let [lhs, rhs] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_strict_equality_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                Ok(())
            }
            SpecOperationIr::IsLooselyEqual => {
                let [lhs, rhs] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                if !lhs.possible_kinds.contains(ValueKind::String)
                    && !rhs.possible_kinds.contains(ValueKind::String)
                {
                    self.compile_loose_equality_nonstring_i32(lhs, rhs, function)?;
                } else {
                    self.compile_loose_equality_i32(lhs, rhs, function)?;
                }
                function.instruction(&Instruction::I64ExtendI32U);
                Ok(())
            }
            SpecOperationIr::Get
            | SpecOperationIr::GetV
            | SpecOperationIr::GetMethod
            | SpecOperationIr::Call
            | SpecOperationIr::Construct => {
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_spec_operation_to_locals(
                    operation,
                    operands,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(payload_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::Set
            | SpecOperationIr::HasProperty
            | SpecOperationIr::HasOwnProperty
            | SpecOperationIr::DeletePropertyOrThrow => {
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_spec_operation_to_locals(
                    operation,
                    operands,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(payload_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
            SpecOperationIr::CreateDataPropertyOrThrow | SpecOperationIr::CopyDataProperties => {
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_spec_operation_to_locals(
                    operation,
                    operands,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(payload_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                Ok(())
            }
        }
    }

    pub(crate) fn compile_spec_operation_to_locals(
        &mut self,
        operation: SpecOperationIr,
        operands: &[TypedExpr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match operation {
            SpecOperationIr::IsCallable => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let operand_payload_local = self.reserve_temp_local();
                let operand_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    operand,
                    operand_payload_local,
                    operand_tag_local,
                    function,
                )?;
                self.emit_is_callable_i32(operand_tag_local, operand_payload_local, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(operand_tag_local);
                self.release_temp_local(operand_payload_local);
                Ok(())
            }
            SpecOperationIr::IsConstructor => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let operand_payload_local = self.reserve_temp_local();
                let operand_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    operand,
                    operand_payload_local,
                    operand_tag_local,
                    function,
                )?;
                self.emit_is_constructor_i32(operand_tag_local, operand_payload_local, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(operand_tag_local);
                self.release_temp_local(operand_payload_local);
                Ok(())
            }
            SpecOperationIr::IsPropertyKey => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let operand_payload_local = self.reserve_temp_local();
                let operand_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    operand,
                    operand_payload_local,
                    operand_tag_local,
                    function,
                )?;
                self.emit_is_property_key_i32(operand_tag_local, function);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(operand_tag_local);
                self.release_temp_local(operand_payload_local);
                Ok(())
            }
            SpecOperationIr::ToBoolean => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.emit_to_boolean_payload_from_expr(operand, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                Ok(())
            }
            SpecOperationIr::ToPrimitive(hint) => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_expr_to_primitive_locals(
                    operand,
                    hint,
                    payload_local,
                    tag_local,
                    function,
                )
            }
            SpecOperationIr::ToNumeric => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_expr_to_numeric_locals(operand, payload_local, tag_local, function)
            }
            SpecOperationIr::ToNumber => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_expr_to_number_payload(operand, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                Ok(())
            }
            SpecOperationIr::ToBigInt => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let operand_payload_local = self.reserve_temp_local();
                let operand_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    operand,
                    operand_payload_local,
                    operand_tag_local,
                    function,
                )?;
                self.emit_value_to_bigint_locals(
                    operand_tag_local,
                    operand_payload_local,
                    false,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(operand_tag_local);
                self.release_temp_local(operand_payload_local);
                Ok(())
            }
            SpecOperationIr::ToString => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                // Same reasoning as the `ToString` arm in
                // `compile_spec_operation_payload`: dispatch
                // Object/Array/Arguments ourselves and propagate correctly
                // instead of relying on `emit_value_to_string_payload`'s
                // internal hard-return-on-throw, and don't stamp a bogus
                // String tag over an abrupt completion.
                let primitive_payload_local = self.reserve_temp_local();
                let primitive_tag_local = self.reserve_temp_local();
                self.emit_tagged_to_primitive_locals(
                    ToPrimitiveHint::String,
                    payload_local,
                    tag_local,
                    primitive_payload_local,
                    primitive_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    primitive_payload_local,
                    primitive_tag_local,
                    function,
                )?;
                self.emit_primitive_to_string_payload(
                    primitive_payload_local,
                    primitive_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(payload_local));
                self.release_temp_local(primitive_tag_local);
                self.release_temp_local(primitive_payload_local);
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                Ok(())
            }
            SpecOperationIr::ToObject => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_expr_to_object_locals(operand, payload_local, tag_local, function)
            }
            SpecOperationIr::ToPropertyKey => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_expr_to_locals(operand, payload_local, tag_local, function)?;
                self.emit_value_to_property_key_locals(payload_local, tag_local, function)?;
                Ok(())
            }
            SpecOperationIr::ToIntegerOrInfinity => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let number_payload_local = self.reserve_temp_local();
                self.compile_expr_to_number_payload(operand, function)?;
                function.instruction(&Instruction::LocalSet(number_payload_local));
                self.emit_to_integer_or_infinity_number_payload_from_number_payload(
                    number_payload_local,
                    payload_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(number_payload_local);
                Ok(())
            }
            SpecOperationIr::ToLength => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let operand_payload_local = self.reserve_temp_local();
                let operand_tag_local = self.reserve_temp_local();
                let length_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    operand,
                    operand_payload_local,
                    operand_tag_local,
                    function,
                )?;
                self.emit_to_length_i64_from_value_locals(
                    operand_tag_local,
                    operand_payload_local,
                    length_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(length_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(length_local);
                self.release_temp_local(operand_tag_local);
                self.release_temp_local(operand_payload_local);
                Ok(())
            }
            SpecOperationIr::ToIndex => {
                let [operand] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 1 operand, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let operand_payload_local = self.reserve_temp_local();
                let operand_tag_local = self.reserve_temp_local();
                let index_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    operand,
                    operand_payload_local,
                    operand_tag_local,
                    function,
                )?;
                self.emit_to_index_i64_from_value_locals(
                    operand_tag_local,
                    operand_payload_local,
                    index_local,
                    "ToIndex out of range",
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(index_local));
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(index_local);
                self.release_temp_local(operand_tag_local);
                self.release_temp_local(operand_payload_local);
                Ok(())
            }
            SpecOperationIr::SameValue => {
                let [lhs, rhs] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_same_value_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                Ok(())
            }
            SpecOperationIr::SameValueZero => {
                let [lhs, rhs] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_same_value_zero_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                Ok(())
            }
            SpecOperationIr::StrictEqualityComparison => {
                let [lhs, rhs] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_strict_equality_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                Ok(())
            }
            SpecOperationIr::IsLooselyEqual => {
                let [lhs, rhs] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                if !lhs.possible_kinds.contains(ValueKind::String)
                    && !rhs.possible_kinds.contains(ValueKind::String)
                {
                    self.compile_loose_equality_nonstring_i32(lhs, rhs, function)?;
                } else {
                    self.compile_loose_equality_i32(lhs, rhs, function)?;
                }
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                Ok(())
            }
            SpecOperationIr::Get => {
                let [target, key] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let object_payload_local = self.reserve_temp_local();
                let object_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    target,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(key, key_payload_local, key_tag_local, function)?;
                self.emit_value_to_property_key_payload(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_payload_local));
                self.emit_property_key_tag_from_source_tag(key_tag_local, key_tag_local, function);
                match target.kind {
                    ValueKind::Object
                    | ValueKind::Array
                    | ValueKind::Arguments
                    | ValueKind::Function => {}
                    ValueKind::Dynamic => {
                        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
                        function.instruction(&Instruction::If(BlockType::Empty));
                    }
                    _ => {
                        self.emit_throw_runtime_error(
                            TYPE_ERROR_NAME,
                            "Get target is not an object",
                            self.result_local,
                            self.result_tag_local,
                            function,
                        )?;
                        if let Some(target) = self.active_throw_target() {
                            self.emit_branch_to_target(target, 1, function);
                        } else {
                            self.emit_return_current_completion(function);
                        }
                    }
                }
                self.emit_dynamic_property_read_with_key_locals(
                    object_payload_local,
                    object_tag_local,
                    object_payload_local,
                    object_tag_local,
                    key_payload_local,
                    key_tag_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                if target.kind == ValueKind::Dynamic {
                    function.instruction(&Instruction::Else);
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "Get target is not an object",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if let Some(target) = self.active_throw_target() {
                        self.emit_branch_to_target(target, 1, function);
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
                }
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(object_tag_local);
                self.release_temp_local(object_payload_local);
                Ok(())
            }
            SpecOperationIr::GetV => {
                let [target, key] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let key = spec_operation_property_key_operand(key);
                self.compile_property_read_to_locals(
                    target,
                    &key,
                    payload_local,
                    tag_local,
                    function,
                )
            }
            SpecOperationIr::GetMethod => {
                let [_, _] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                self.compile_spec_operation_to_locals(
                    SpecOperationIr::GetV,
                    operands,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::Else);
                self.emit_is_callable_i32(tag_local, payload_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "GetMethod target is not callable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, 1, function);
                } else {
                    self.emit_return_current_completion(function);
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                Ok(())
            }
            SpecOperationIr::Call => {
                let Some((callee, rest)) = operands.split_first() else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: Call expects at least callee and thisArg",
                    ));
                };
                let Some((this_arg, args)) = rest.split_first() else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: Call expects callee and thisArg operands",
                    ));
                };
                let callee_payload_local = self.reserve_temp_local();
                let callee_tag_local = self.reserve_temp_local();
                let this_payload_local = self.reserve_temp_local();
                let this_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    callee,
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    callee_payload_local,
                    callee_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(
                    this_arg,
                    this_payload_local,
                    this_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    this_payload_local,
                    this_tag_local,
                    function,
                )?;
                let mut arg_locals = Vec::with_capacity(args.len());
                for arg in args {
                    let arg_payload_local = self.reserve_temp_local();
                    let arg_tag_local = self.reserve_temp_local();
                    self.compile_expr_to_locals(arg, arg_payload_local, arg_tag_local, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        arg_payload_local,
                        arg_tag_local,
                        function,
                    )?;
                    arg_locals.push((arg_payload_local, arg_tag_local));
                }
                self.emit_function_handle_call(
                    callee_payload_local,
                    callee_tag_local,
                    Some((this_payload_local, Some(this_tag_local))),
                    &arg_locals,
                    payload_local,
                    tag_local,
                    function,
                )?;
                for (arg_payload_local, arg_tag_local) in arg_locals.into_iter().rev() {
                    self.release_temp_local(arg_tag_local);
                    self.release_temp_local(arg_payload_local);
                }
                self.release_temp_local(this_tag_local);
                self.release_temp_local(this_payload_local);
                self.release_temp_local(callee_tag_local);
                self.release_temp_local(callee_payload_local);
                Ok(())
            }
            SpecOperationIr::Construct => {
                let Some((callee, args)) = operands.split_first() else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: Construct expects a callee operand",
                    ));
                };
                self.emit_construct(callee, args, None, payload_local, tag_local, function)
            }
            SpecOperationIr::HasProperty => {
                let [target, key] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let object_payload_local = self.reserve_temp_local();
                let object_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                let has_property_local = self.reserve_temp_local();
                self.compile_expr_to_locals(key, key_payload_local, key_tag_local, function)?;
                self.compile_expr_to_locals(
                    target,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                match target.kind {
                    ValueKind::Object
                    | ValueKind::Array
                    | ValueKind::Arguments
                    | ValueKind::Function => {}
                    ValueKind::Dynamic => {
                        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
                        function.instruction(&Instruction::If(BlockType::Empty));
                    }
                    _ => {
                        self.emit_throw_runtime_error(
                            "TypeError",
                            "right-hand side of `in` is not an object",
                            self.result_local,
                            self.result_tag_local,
                            function,
                        )?;
                        self.emit_dispatch_current_completion(function)?;
                        self.release_temp_local(has_property_local);
                        self.release_temp_local(key_tag_local);
                        self.release_temp_local(key_payload_local);
                        self.release_temp_local(object_tag_local);
                        self.release_temp_local(object_payload_local);
                        return Ok(());
                    }
                }
                self.emit_value_to_property_key_payload(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_payload_local));
                self.emit_object_has_property_i32(
                    object_payload_local,
                    object_tag_local,
                    key_payload_local,
                    has_property_local,
                    function,
                )?;
                if target.kind == ValueKind::Dynamic {
                    function.instruction(&Instruction::Else);
                    self.emit_throw_runtime_error(
                        "TypeError",
                        "right-hand side of `in` is not an object",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if let Some(target) = self.active_throw_target() {
                        self.emit_branch_to_target(target, 1, function);
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::LocalGet(has_property_local));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(has_property_local);
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(object_tag_local);
                self.release_temp_local(object_payload_local);
                Ok(())
            }
            SpecOperationIr::HasOwnProperty => {
                let [target, key] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let object_payload_local = self.reserve_temp_local();
                let object_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                let descriptor_payload_local = self.reserve_temp_local();
                let descriptor_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    target,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(key, key_payload_local, key_tag_local, function)?;
                self.emit_value_to_property_key_payload(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_payload_local));
                self.emit_property_key_tag_from_source_tag(key_tag_local, key_tag_local, function);
                match target.kind {
                    ValueKind::Object
                    | ValueKind::Array
                    | ValueKind::Arguments
                    | ValueKind::Function => {}
                    ValueKind::Dynamic => {
                        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
                        function.instruction(&Instruction::If(BlockType::Empty));
                    }
                    _ => {
                        self.emit_throw_runtime_error(
                            TYPE_ERROR_NAME,
                            "HasOwnProperty target is not an object",
                            self.result_local,
                            self.result_tag_local,
                            function,
                        )?;
                        if let Some(target) = self.active_throw_target() {
                            self.emit_branch_to_target(target, 1, function);
                        } else {
                            self.emit_return_current_completion(function);
                        }
                    }
                }
                let get_own_meta = self
                    .functions
                    .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.getOwnPropertyDescriptor`",
                        )
                    })?;
                self.emit_function_value_payload(&get_own_meta, function)?;
                function.instruction(&Instruction::LocalSet(descriptor_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(descriptor_tag_local));
                self.emit_function_handle_call(
                    descriptor_payload_local,
                    descriptor_tag_local,
                    None,
                    &[
                        (object_payload_local, object_tag_local),
                        (key_payload_local, key_tag_local),
                    ],
                    descriptor_payload_local,
                    descriptor_tag_local,
                    function,
                )?;
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalGet(descriptor_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                if target.kind == ValueKind::Dynamic {
                    function.instruction(&Instruction::Else);
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "HasOwnProperty target is not an object",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if let Some(target) = self.active_throw_target() {
                        self.emit_branch_to_target(target, 1, function);
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(descriptor_tag_local);
                self.release_temp_local(descriptor_payload_local);
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(object_tag_local);
                self.release_temp_local(object_payload_local);
                Ok(())
            }
            SpecOperationIr::DeletePropertyOrThrow => {
                let [target, key] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let object_payload_local = self.reserve_temp_local();
                let object_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    target,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(key, key_payload_local, key_tag_local, function)?;
                self.emit_value_to_property_key_payload(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_payload_local));
                // Symbol keys are carried in the property-key payload itself via
                // `PROPERTY_KEY_SYMBOL_MARKER`, and `emit_object_delete` re-derives the
                // key tag from that payload, so no String-only gate is needed here.
                self.emit_property_key_tag_from_source_tag(key_tag_local, key_tag_local, function);
                match target.kind {
                    ValueKind::Object
                    | ValueKind::Array
                    | ValueKind::Arguments
                    | ValueKind::Function => {}
                    ValueKind::Dynamic => {
                        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
                        function.instruction(&Instruction::If(BlockType::Empty));
                    }
                    _ => {
                        self.emit_throw_runtime_error(
                            TYPE_ERROR_NAME,
                            "DeletePropertyOrThrow target is not an object",
                            self.result_local,
                            self.result_tag_local,
                            function,
                        )?;
                        if let Some(target) = self.active_throw_target() {
                            self.emit_branch_to_target(target, 1, function);
                        } else {
                            self.emit_return_current_completion(function);
                        }
                    }
                }
                self.emit_object_delete(
                    object_payload_local,
                    object_tag_local,
                    key_payload_local,
                    payload_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot delete property",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, 1, function);
                } else {
                    self.emit_return_current_completion(function);
                }
                function.instruction(&Instruction::End);
                if target.kind == ValueKind::Dynamic {
                    function.instruction(&Instruction::Else);
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "DeletePropertyOrThrow target is not an object",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if let Some(target) = self.active_throw_target() {
                        self.emit_branch_to_target(target, 1, function);
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(object_tag_local);
                self.release_temp_local(object_payload_local);
                Ok(())
            }
            SpecOperationIr::Set => {
                let [target, key, value] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 3 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let object_payload_local = self.reserve_temp_local();
                let object_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                let value_payload_local = self.reserve_temp_local();
                let value_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    target,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(key, key_payload_local, key_tag_local, function)?;
                self.emit_value_to_property_key_payload(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_payload_local));
                // `emit_ordinary_set_result` is symbol-aware: it gates the
                // Array `length` fast path and the typed-array canonical-index
                // path on a String key tag, and re-derives the key tag from the
                // payload before forwarding to a Proxy `set` trap. A Symbol key
                // tag can therefore flow straight through.
                self.emit_property_key_tag_from_source_tag(key_tag_local, key_tag_local, function);
                self.compile_expr_to_locals(value, value_payload_local, value_tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                match target.kind {
                    ValueKind::Object
                    | ValueKind::Array
                    | ValueKind::Arguments
                    | ValueKind::Function => {}
                    ValueKind::Dynamic => {
                        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
                        function.instruction(&Instruction::If(BlockType::Empty));
                    }
                    _ => {
                        self.emit_throw_runtime_error(
                            TYPE_ERROR_NAME,
                            "Set target is not an object",
                            self.result_local,
                            self.result_tag_local,
                            function,
                        )?;
                        if let Some(target) = self.active_throw_target() {
                            self.emit_branch_to_target(target, 1, function);
                        } else {
                            self.emit_return_current_completion(function);
                        }
                    }
                }
                self.emit_ordinary_set_result(
                    object_payload_local,
                    object_tag_local,
                    object_payload_local,
                    object_tag_local,
                    key_payload_local,
                    key_tag_local,
                    value_payload_local,
                    value_tag_local,
                    payload_local,
                    function,
                )?;
                if target.kind == ValueKind::Dynamic {
                    function.instruction(&Instruction::Else);
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "Set target is not an object",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if let Some(target) = self.active_throw_target() {
                        self.emit_branch_to_target(target, 1, function);
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(value_tag_local);
                self.release_temp_local(value_payload_local);
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(object_tag_local);
                self.release_temp_local(object_payload_local);
                Ok(())
            }
            SpecOperationIr::CreateDataPropertyOrThrow => {
                let [target, key, value] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 3 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let object_payload_local = self.reserve_temp_local();
                let object_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                let value_payload_local = self.reserve_temp_local();
                let value_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    target,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(key, key_payload_local, key_tag_local, function)?;
                self.emit_value_to_property_key_payload(
                    key_payload_local,
                    key_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(key_payload_local));
                // `emit_create_data_property_or_throw` matches own keys with
                // `emit_property_key_payload_equality_i32` and stores the raw
                // property-key payload (symbol marker included) into the entry
                // slot, so symbol keys need no String-only gate here.
                self.emit_property_key_tag_from_source_tag(key_tag_local, key_tag_local, function);
                self.compile_expr_to_locals(value, value_payload_local, value_tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                match target.kind {
                    ValueKind::Object
                    | ValueKind::Array
                    | ValueKind::Arguments
                    | ValueKind::Function => {}
                    ValueKind::Dynamic => {
                        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
                        function.instruction(&Instruction::If(BlockType::Empty));
                    }
                    _ => {
                        self.emit_throw_runtime_error(
                            TYPE_ERROR_NAME,
                            "CreateDataPropertyOrThrow target is not an object",
                            self.result_local,
                            self.result_tag_local,
                            function,
                        )?;
                        if let Some(target) = self.active_throw_target() {
                            self.emit_branch_to_target(target, 1, function);
                        } else {
                            self.emit_return_current_completion(function);
                        }
                    }
                }
                self.emit_create_data_property_or_throw(
                    object_payload_local,
                    object_tag_local,
                    key_payload_local,
                    value_payload_local,
                    value_tag_local,
                    "Cannot redefine non-configurable property",
                    "Cannot define property on non-extensible object",
                    None,
                    function,
                )?;
                if target.kind == ValueKind::Dynamic {
                    function.instruction(&Instruction::Else);
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "CreateDataPropertyOrThrow target is not an object",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if let Some(target) = self.active_throw_target() {
                        self.emit_branch_to_target(target, 1, function);
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(value_tag_local);
                self.release_temp_local(value_payload_local);
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(object_tag_local);
                self.release_temp_local(object_payload_local);
                Ok(())
            }
            SpecOperationIr::CopyDataProperties => {
                let [target, source] = operands else {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: {} expects 2 operands, got {}",
                        operation.name(),
                        operands.len()
                    )));
                };
                let target_payload_local = self.reserve_temp_local();
                let target_tag_local = self.reserve_temp_local();
                let source_payload_local = self.reserve_temp_local();
                let source_tag_local = self.reserve_temp_local();
                let source_object_payload_local = self.reserve_temp_local();
                let source_object_tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(
                    target,
                    target_payload_local,
                    target_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    target_payload_local,
                    target_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(
                    source,
                    source_payload_local,
                    source_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    source_payload_local,
                    source_tag_local,
                    function,
                )?;
                self.compile_nullish_tagged_i32(source_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.push_control(ControlFrameKind::If);
                function.instruction(&Instruction::Else);
                self.emit_value_to_object_locals(
                    source_payload_local,
                    source_tag_local,
                    source_object_payload_local,
                    source_object_tag_local,
                    function,
                )?;
                self.emit_copy_data_properties_into(
                    source_object_payload_local,
                    source_object_tag_local,
                    &[],
                    target_payload_local,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.release_temp_local(source_object_tag_local);
                self.release_temp_local(source_object_payload_local);
                self.release_temp_local(source_tag_local);
                self.release_temp_local(source_payload_local);
                self.release_temp_local(target_tag_local);
                self.release_temp_local(target_payload_local);
                Ok(())
            }
        }
    }

    pub(crate) fn emit_to_boolean_payload_from_expr(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_truthy_i32(expr, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        Ok(())
    }

    pub(crate) fn emit_to_boolean_payload_from_tagged_locals(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_truthy_tagged_i32(tag_local, payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        Ok(())
    }

    pub(crate) fn compile_truthy_local_i32(
        &mut self,
        kind: ValueKind,
        local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match kind {
            ValueKind::Undefined | ValueKind::Null => {
                function.instruction(&Instruction::I32Const(0));
            }
            ValueKind::Object | ValueKind::Array | ValueKind::Arguments | ValueKind::Symbol => {
                function.instruction(&Instruction::I32Const(1));
            }
            ValueKind::Function => {
                let tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_is_htmldda_function_i32(tag_local, local, function)?;
                function.instruction(&Instruction::I32Eqz);
                self.release_temp_local(tag_local);
            }
            ValueKind::Boolean => {
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I32WrapI64);
            }
            ValueKind::String => {
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32Eqz);
            }
            ValueKind::Number => {
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Ne);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Eqz);
            }
            ValueKind::BigInt => {
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
            }
            ValueKind::Dynamic => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: dynamic truthiness kind",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn emit_is_constructor_i32(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let flags_local = self.reserve_temp_local();
        let is_htmldda_local = self.reserve_temp_local();
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_function_constructable_flag(current_payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(flags_local));
        self.emit_is_htmldda_function_i32(current_tag_local, current_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(is_htmldda_local));
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(is_htmldda_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
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
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            current_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(result_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        self.release_temp_local(is_htmldda_local);
        self.release_temp_local(flags_local);
        Ok(())
    }

    pub(crate) fn emit_is_htmldda_function_i32(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let flags_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_load_function_flags(payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_IS_HTMLDDA as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);

        self.release_temp_local(flags_local);
        Ok(())
    }

    pub(crate) fn compile_truthy_tagged_i32(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.compile_truthy_local_i32(ValueKind::Number, payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.compile_truthy_local_i32(ValueKind::BigInt, payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        self.emit_is_htmldda_function_i32(tag_local, payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_nullish_tagged_i32(
        &self,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        Ok(())
    }

    pub(crate) fn compile_expr_to_primitive_locals(
        &mut self,
        expr: &TypedExpr,
        hint: ToPrimitiveHint,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_expr_to_primitive_locals_with_extra_depth(
            expr,
            hint,
            payload_local,
            tag_local,
            0,
            function,
        )
    }

    /// Same as `compile_expr_to_primitive_locals`, but for call sites that sit
    /// `extra_depth` untracked wasm blocks deeper than `self.control_stack`
    /// reflects (e.g. already inside a manually-emitted `If`), so the internal
    /// throw propagation's `Br` reaches the correct active handler.
    pub(crate) fn compile_expr_to_primitive_locals_with_extra_depth(
        &mut self,
        expr: &TypedExpr,
        hint: ToPrimitiveHint,
        payload_local: u32,
        tag_local: u32,
        extra_depth: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY) {
            self.compile_expr_to_locals(expr, payload_local, tag_local, function)?;
            return Ok(());
        }

        let raw_payload_local = self.reserve_temp_local();
        let raw_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(expr, raw_payload_local, raw_tag_local, function)?;
        self.emit_tagged_to_primitive_locals(
            hint,
            raw_payload_local,
            raw_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        // `emit_tagged_to_primitive_locals` (via `emit_object_to_primitive_locals`)
        // leaves an abrupt ToPrimitive completion as THROW with the error already
        // in `payload_local`/`tag_local` rather than branching internally (see
        // `emit_object_to_primitive_locals_locals_inner`). Propagate here to
        // reach the active in-function try/catch handler (or return the
        // completion when there is none); `extra_depth` accounts for any
        // untracked wasm blocks the caller already has open at this point.
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            extra_depth,
            function,
        )?;
        self.release_temp_local(raw_tag_local);
        self.release_temp_local(raw_payload_local);
        Ok(())
    }

    /// `emit_tagged_to_primitive_locals` plus the throw propagation that
    /// `compile_expr_to_primitive_locals_with_extra_depth` performs, for
    /// callers that need to split "evaluate the operand" from "coerce it".
    fn emit_to_primitive_from_raw_locals(
        &mut self,
        hint: ToPrimitiveHint,
        raw_payload_local: u32,
        raw_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_tagged_to_primitive_locals(
            hint,
            raw_payload_local,
            raw_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            0,
            function,
        )
    }

    /// Evaluates both operands of a binary operator first, and only then runs
    /// ToPrimitive on each, left to right.
    ///
    /// Calling `compile_expr_to_primitive_locals` once per operand is wrong for
    /// binary operators: it coerces the left operand as soon as it is
    /// evaluated, so an object lhs runs its @@toPrimitive/valueOf/toString hook
    /// before the rhs expression is evaluated at all. The spec evaluates both
    /// operands (EvaluateStringOrNumericBinaryExpression steps 1-4) before
    /// ApplyStringOrNumericBinaryOperator coerces either of them, which is
    /// observable whenever both sides have side effects.
    pub(crate) fn compile_operand_pair_to_primitive_locals(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        hint: ToPrimitiveHint,
        lhs_payload_local: u32,
        lhs_tag_local: u32,
        rhs_payload_local: u32,
        rhs_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY) {
            // Nothing to coerce on the left, so the naive order is already the
            // spec order.
            self.compile_expr_to_locals(lhs, lhs_payload_local, lhs_tag_local, function)?;
            return self.compile_expr_to_primitive_locals(
                rhs,
                hint,
                rhs_payload_local,
                rhs_tag_local,
                function,
            );
        }

        let lhs_raw_payload = self.reserve_temp_local();
        let lhs_raw_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_raw_payload, lhs_raw_tag, function)?;

        if rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY) {
            self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
            self.emit_to_primitive_from_raw_locals(
                hint,
                lhs_raw_payload,
                lhs_raw_tag,
                lhs_payload_local,
                lhs_tag_local,
                function,
            )?;
        } else {
            let rhs_raw_payload = self.reserve_temp_local();
            let rhs_raw_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(rhs, rhs_raw_payload, rhs_raw_tag, function)?;
            self.emit_to_primitive_from_raw_locals(
                hint,
                lhs_raw_payload,
                lhs_raw_tag,
                lhs_payload_local,
                lhs_tag_local,
                function,
            )?;
            self.emit_to_primitive_from_raw_locals(
                hint,
                rhs_raw_payload,
                rhs_raw_tag,
                rhs_payload_local,
                rhs_tag_local,
                function,
            )?;
            self.release_temp_local(rhs_raw_tag);
            self.release_temp_local(rhs_raw_payload);
        }

        self.release_temp_local(lhs_raw_tag);
        self.release_temp_local(lhs_raw_payload);
        Ok(())
    }

    /// Evaluates both operands of a numeric binary operator and only then
    /// converts each to a Number, in the order `op` prescribes.
    ///
    /// This is `compile_operand_pair_to_primitive_locals` for the operators
    /// that need a Number rather than a primitive (`* / - % ** & | ^ << >>
    /// >>>`, and `+` once lowering has proved neither side is a string).
    /// Compiling "coerce the left operand, then evaluate the right one" - the
    /// naive order produced by a single `compile_expr_to_number_payload` per
    /// operand - runs an object lhs's valueOf/toString hook before the rhs
    /// expression is evaluated at all. 13.15.4 evaluates both operands before
    /// ApplyStringOrNumericBinaryOperator coerces either, which is observable
    /// whenever both sides have side effects.
    ///
    /// The order of the *conversions* is taken from `op` rather than from a
    /// caller-supplied flag, because the two orders differ and picking the
    /// wrong one is silently observable - see `NumericBinaryOperator`.
    ///
    /// Leaves the Number payloads in `lhs_number_local` and `rhs_number_local`.
    pub(crate) fn compile_operand_pair_to_number_locals(
        &mut self,
        op: NumericBinaryOperator,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        lhs_number_local: u32,
        rhs_number_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Both operands convert with a plain tag switch: no ToPrimitive hook to
        // order, no string parse, no TypeError. The naive order is then the
        // spec order and the cheap conversion is faithful.
        let direct_to_number = lhs.possible_kinds.is_subset_of(KindSet::DIRECT_TO_NUMBER)
            && rhs.possible_kinds.is_subset_of(KindSet::DIRECT_TO_NUMBER);
        if direct_to_number {
            self.compile_expr_to_number_payload_nonstring(lhs, function)?;
            function.instruction(&Instruction::LocalSet(lhs_number_local));
            self.compile_expr_to_number_payload_nonstring(rhs, function)?;
            function.instruction(&Instruction::LocalSet(rhs_number_local));
            return Ok(());
        }

        // 13.15.4 steps 1-4: both operand expressions are evaluated before any
        // coercion runs.
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;

        // `emit_value_to_number_payload` IS ToNumeric: it runs ToPrimitive on a
        // heap operand itself, so the plain order needs no separate step, and
        // it is outlined into a shared helper - inlining the per-kind composite
        // twice per operator overruns Cranelift's function size limit on
        // arithmetic-heavy scripts.
        if op.applies_to_primitive_before_numeric() {
            // Only `+` takes 13.15.3 step 1: ToPrimitive on both operands
            // before ToNumeric on either.
            let lhs_primitive_payload = self.reserve_temp_local();
            let lhs_primitive_tag = self.reserve_temp_local();
            let rhs_primitive_payload = self.reserve_temp_local();
            let rhs_primitive_tag = self.reserve_temp_local();
            self.emit_to_primitive_from_raw_locals(
                ToPrimitiveHint::Number,
                lhs_payload,
                lhs_tag,
                lhs_primitive_payload,
                lhs_primitive_tag,
                function,
            )?;
            self.emit_to_primitive_from_raw_locals(
                ToPrimitiveHint::Number,
                rhs_payload,
                rhs_tag,
                rhs_primitive_payload,
                rhs_primitive_tag,
                function,
            )?;
            self.emit_operand_to_number_local(
                lhs_primitive_payload,
                lhs_primitive_tag,
                lhs_number_local,
                function,
            )?;
            self.emit_operand_to_number_local(
                rhs_primitive_payload,
                rhs_primitive_tag,
                rhs_number_local,
                function,
            )?;
            self.release_temp_local(rhs_primitive_tag);
            self.release_temp_local(rhs_primitive_payload);
            self.release_temp_local(lhs_primitive_tag);
            self.release_temp_local(lhs_primitive_payload);
        } else {
            self.emit_operand_to_number_local(lhs_payload, lhs_tag, lhs_number_local, function)?;
            self.emit_operand_to_number_local(rhs_payload, rhs_tag, rhs_number_local, function)?;
        }

        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    /// ToNumeric on one already evaluated operand, into `number_local`.
    fn emit_operand_to_number_local(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        number_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::LocalSet(number_local));
        Ok(())
    }

    pub(crate) fn emit_tagged_to_primitive_locals(
        &mut self,
        hint: ToPrimitiveHint,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_to_primitive_locals(
            hint,
            input_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_to_string_locals(input_payload_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // A Function is an ordinary object for ToPrimitive purposes: run the
        // same @@toPrimitive/valueOf/toString hook chain as a plain object
        // (see `emit_function_to_primitive_locals`), rather than passing the
        // function through unchanged as if it were already a primitive.
        self.emit_function_to_primitive_locals(
            hint,
            input_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_tagged_to_primitive_locals_without_throw_propagation(
        &mut self,
        hint: ToPrimitiveHint,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_to_primitive_locals_without_throw_propagation(
            hint,
            input_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_to_string_locals(input_payload_local, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_to_primitive_locals_without_throw_propagation(
            hint,
            input_payload_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_object_to_primitive_locals(
        &mut self,
        hint: ToPrimitiveHint,
        object_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_to_primitive_locals_inner(
            hint,
            object_local,
            ValueKind::Object,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_object_to_primitive_locals_without_throw_propagation(
        &mut self,
        hint: ToPrimitiveHint,
        object_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_to_primitive_locals_inner(
            hint,
            object_local,
            ValueKind::Object,
            payload_local,
            tag_local,
            function,
        )
    }

    /// Same as `emit_object_to_primitive_locals`, but for a value already
    /// known to be tag `Function`. Function objects run the exact same
    /// OrdinaryToPrimitive algorithm as plain objects (they are ordinary
    /// objects that happen to be callable) — the only difference is that a
    /// function's heap record does not reserve the Object record's
    /// boxed-primitive slot at the same offset, so the fast path that reads
    /// it must be skipped (functions can never be `new Number(...)`-style
    /// boxed-primitive wrappers).
    pub(crate) fn emit_function_to_primitive_locals(
        &mut self,
        hint: ToPrimitiveHint,
        object_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_to_primitive_locals_inner(
            hint,
            object_local,
            ValueKind::Function,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_function_to_primitive_locals_without_throw_propagation(
        &mut self,
        hint: ToPrimitiveHint,
        object_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_to_primitive_locals_inner(
            hint,
            object_local,
            ValueKind::Function,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_object_to_primitive_locals_inner(
        &mut self,
        hint: ToPrimitiveHint,
        object_local: u32,
        object_tag_kind: ValueKind,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let hook_names: &[&str] = match hint {
            ToPrimitiveHint::String => &["Symbol.toPrimitive", "toString", "valueOf"],
            ToPrimitiveHint::Default | ToPrimitiveHint::Number => {
                &["Symbol.toPrimitive", "valueOf", "toString"]
            }
        };

        let boxed_kind_local = self.reserve_temp_local();
        if matches!(object_tag_kind, ValueKind::Object) {
            self.load_i64_to_local_from_offset(
                object_local,
                HEAP_OBJECT_BOXED_KIND_OFFSET,
                boxed_kind_local,
                function,
            );
        } else {
            // Non-Object tags (currently only `Function`) never represent a
            // boxed-primitive wrapper, and their heap record does not
            // reserve `HEAP_OBJECT_BOXED_KIND_OFFSET` for that purpose (it
            // overlaps unrelated fields) — force the fast path below to be
            // skipped rather than reading it.
            function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NONE as i64));
            function.instruction(&Instruction::LocalSet(boxed_kind_local));
        }
        // BigInt and Symbol wrappers must consult their prototype hooks
        // dynamically. Neither wrapper has an intrinsic @@toPrimitive, so
        // OrdinaryToPrimitive must observe redefinitions of valueOf/toString.
        // The remaining primitive wrappers retain their existing direct-slot
        // path.
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NONE as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_BIGINT as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        let hook_value_payload = self.reserve_temp_local();
        let hook_value_tag = self.reserve_temp_local();
        let call_result_payload = self.reserve_temp_local();
        let call_result_tag = self.reserve_temp_local();
        let primitive_result_local = self.reserve_temp_local();
        let call_attempted_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(primitive_result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(call_attempted_local));
        function.instruction(&Instruction::I64Const(object_tag_kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));

        for hook_name in hook_names {
            let key_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(primitive_result_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            // `Symbol.toPrimitive` is a symbol-valued PropertyKey, so its
            // lookup payload must carry `PROPERTY_KEY_SYMBOL_MARKER` — a plain
            // string payload with the same bytes is a *different* key and never
            // matches a stored `[Symbol.toPrimitive]` entry (see
            // `emit_property_key_payload_equality_i32`). `toString` and
            // `valueOf` stay plain string keys;
            // `static_builtin_property_key_payload` picks the right encoding
            // per name, exactly as the property-definition side does.
            function.instruction(&Instruction::I64Const(
                self.strings.static_builtin_property_key_payload(hook_name),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read_without_throw_propagation(
                object_local,
                object_tag_local,
                object_local,
                object_tag_local,
                key_local,
                hook_value_payload,
                hook_value_tag,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(self.completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(hook_value_payload));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(hook_value_tag));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(primitive_result_local));
            function.instruction(&Instruction::Else);
            if *hook_name == "Symbol.toPrimitive" {
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(call_attempted_local));
                // @@toPrimitive receives the requested hint as its sole
                // argument. Keep the ordinary valueOf/toString calls below
                // argument-free, as required by OrdinaryToPrimitive.
                let hint_payload_local = self.reserve_temp_local();
                let hint_tag_local = self.reserve_temp_local();
                let hint = match hint {
                    ToPrimitiveHint::String => "string",
                    ToPrimitiveHint::Number => "number",
                    ToPrimitiveHint::Default => "default",
                };
                function.instruction(&Instruction::I64Const(self.strings.payload(hint)));
                function.instruction(&Instruction::LocalSet(hint_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(hint_tag_local));
                self.emit_function_handle_call_without_throw_propagation(
                    hook_value_payload,
                    hook_value_tag,
                    Some((object_local, Some(object_tag_local))),
                    &[(hint_payload_local, hint_tag_local)],
                    call_result_payload,
                    call_result_tag,
                    function,
                )?;
                self.release_temp_local(hint_tag_local);
                self.release_temp_local(hint_payload_local);
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(call_result_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(call_result_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_throw_from_locals(call_result_payload, call_result_tag, function)?;
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::Else);
                self.emit_is_primitive_tag_i32(call_result_tag, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(call_result_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(call_result_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot convert object to primitive value",
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot convert object to primitive value",
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            } else {
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(call_attempted_local));
                self.emit_function_handle_call_without_throw_propagation(
                    hook_value_payload,
                    hook_value_tag,
                    Some((object_local, Some(object_tag_local))),
                    &[],
                    call_result_payload,
                    call_result_tag,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(call_result_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(call_result_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_throw_from_locals(call_result_payload, call_result_tag, function)?;
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::Else);
                self.emit_is_primitive_tag_i32(call_result_tag, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(call_result_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(call_result_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                if *hook_name == "toString" {
                    // OrdinaryToPrimitive: Get(O, "toString") for an ordinary object with
                    // no own (or otherwise resolvable) `toString` still resolves to the
                    // inherited Object.prototype.toString, whose call yields
                    // "[object Object]". Reconstruct that default when the property read
                    // surfaced no callable (undefined) and the receiver is an ordinary
                    // object that inherits Object.prototype, so plain objects coerce to a
                    // primitive instead of throwing.
                    //
                    // This must NOT apply when the object (or its chain) has a real
                    // `toString` property whose value happens to be `undefined` (e.g. an
                    // own `toString: undefined` shadowing the inherited default) — that
                    // case legitimately yields "no callable here", falls through to the
                    // next hook without a default, and can end in a TypeError. Use
                    // HasProperty to distinguish "genuinely absent" (apply the default)
                    // from "present but non-callable" (do not).
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(hook_value_tag));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    self.emit_ordinary_object_default_to_string_applies_i32(
                        object_local,
                        object_tag_local,
                        function,
                    );
                    function.instruction(&Instruction::I32And);
                    let has_tostring_local = self.reserve_temp_local();
                    self.emit_object_has_property_i32(
                        object_local,
                        object_tag_local,
                        key_local,
                        has_tostring_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(has_tostring_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32And);
                    self.release_temp_local(has_tostring_local);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload("[object Object]"),
                    ));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(primitive_result_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            self.release_temp_local(key_local);
        }

        function.instruction(&Instruction::LocalGet(primitive_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert object to primitive value",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(primitive_result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(object_tag_local);
        self.release_temp_local(call_attempted_local);
        self.release_temp_local(primitive_result_local);
        self.release_temp_local(call_result_tag);
        self.release_temp_local(call_result_payload);
        self.release_temp_local(hook_value_tag);
        self.release_temp_local(hook_value_payload);
        function.instruction(&Instruction::End);
        // A throw here (an abrupt hook read/call, a non-primitive hook return,
        // or no primitive result) remains in the completion and output locals.
        // ToPrimitive can run inside raw Wasm blocks not reflected in
        // `self.control_stack`, so callers must propagate only after those
        // blocks have closed.
        self.release_temp_local(boxed_kind_local);
        Ok(())
    }

    /// Pushes an i32 (1/0) indicating whether `object_local` is an ordinary object
    /// (no exotic internal brand) whose prototype chain reaches `Object.prototype`.
    /// When true, `OrdinaryToPrimitive`'s inherited `Object.prototype.toString`
    /// default ("[object Object]") applies; when false (e.g. a null-prototype object
    /// or an exotic such as an Error) the caller must fall back to throwing.
    pub(crate) fn emit_ordinary_object_default_to_string_applies_i32(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        function: &mut Function,
    ) {
        let search_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let next_proto_local = self.reserve_temp_local();
        let next_proto_tag_local = self.reserve_temp_local();
        let inherits_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(inherits_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(search_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(search_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        // Reached the end of the prototype chain (null) without Object.prototype.
        function.instruction(&Instruction::LocalGet(search_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        // Found Object.prototype in the chain.
        function.instruction(&Instruction::LocalGet(search_local));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(inherits_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_ordinary_get_prototype_of(
            search_local,
            search_tag_local,
            next_proto_local,
            next_proto_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(next_proto_local));
        function.instruction(&Instruction::LocalSet(search_local));
        function.instruction(&Instruction::LocalGet(next_proto_tag_local));
        function.instruction(&Instruction::LocalSet(search_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Only ordinary objects (internal brand 0) use the plain "[object Object]"
        // default; exotics such as Error carry their own toString semantics.
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(inherits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);

        self.release_temp_local(brand_local);
        self.release_temp_local(inherits_local);
        self.release_temp_local(next_proto_tag_local);
        self.release_temp_local(next_proto_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_local);
    }

    pub(crate) fn emit_is_primitive_tag_i32(&self, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        for kind in [
            ValueKind::Null,
            ValueKind::Boolean,
            ValueKind::Number,
            ValueKind::BigInt,
            ValueKind::Symbol,
            ValueKind::String,
        ] {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_is_bigint_tag_i32(&self, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_is_heap_object_like_tag_i32(&self, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        for kind in [ValueKind::Array, ValueKind::Function, ValueKind::Arguments] {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn compile_coercive_add_to_locals(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        let lhs_string_local = self.reserve_temp_local();
        let rhs_string_local = self.reserve_temp_local();

        self.compile_operand_pair_to_primitive_locals(
            lhs,
            rhs,
            ToPrimitiveHint::Default,
            lhs_payload,
            lhs_tag,
            rhs_payload,
            rhs_tag,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(lhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(lhs_payload, lhs_tag, function)?;
        function.instruction(&Instruction::LocalSet(lhs_string_local));
        self.emit_value_to_string_payload(rhs_payload, rhs_tag, function)?;
        function.instruction(&Instruction::LocalSet(rhs_string_local));
        self.emit_concat_string_payloads_local(lhs_string_local, rhs_string_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_payload));
        function.instruction(&Instruction::LocalGet(rhs_payload));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(lhs_tag, lhs_payload, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        self.emit_value_to_number_payload(rhs_tag, rhs_payload, function)?;
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(rhs_string_local);
        self.release_temp_local(lhs_string_local);
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    pub(crate) fn compile_expr_to_object_locals(
        &mut self,
        expr: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let input_payload_local = self.reserve_temp_local();
        let input_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(expr, input_payload_local, input_tag_local, function)?;
        self.emit_value_to_object_locals(
            input_payload_local,
            input_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(input_tag_local);
        self.release_temp_local(input_payload_local);
        Ok(())
    }

    pub(crate) fn emit_value_to_object_locals(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::Block(BlockType::Empty));
        for kind in [
            ValueKind::Object,
            ValueKind::Array,
            ValueKind::Function,
            ValueKind::Arguments,
        ] {
            function.instruction(&Instruction::LocalGet(input_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(input_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(input_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        for (kind, prototype_global_index, boxed_kind) in [
            (
                ValueKind::Number,
                NUMBER_PROTOTYPE_GLOBAL_INDEX,
                BOXED_PRIMITIVE_KIND_NUMBER,
            ),
            (
                ValueKind::String,
                STRING_PROTOTYPE_GLOBAL_INDEX,
                BOXED_PRIMITIVE_KIND_STRING,
            ),
            (
                ValueKind::Boolean,
                BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
                BOXED_PRIMITIVE_KIND_BOOLEAN,
            ),
            (
                ValueKind::Symbol,
                SYMBOL_PROTOTYPE_GLOBAL_INDEX,
                BOXED_PRIMITIVE_KIND_SYMBOL,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(input_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_boxed_wrapper_from_locals(
                prototype_global_index,
                boxed_kind,
                input_payload_local,
                input_tag_local,
                payload_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        let bigint_constructor_local = self.reserve_temp_local();
        let bigint_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(bigint_constructor_local));
        self.load_i64_to_local_from_offset(
            bigint_constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            bigint_prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(bigint_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_store_boxed_primitive_metadata(
            payload_local,
            BOXED_PRIMITIVE_KIND_BIGINT,
            input_payload_local,
            input_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(bigint_prototype_local);
        self.release_temp_local(bigint_constructor_local);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert undefined or null to object",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_value_to_current_function_realm_object_locals(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        for kind in [
            ValueKind::Object,
            ValueKind::Array,
            ValueKind::Function,
            ValueKind::Arguments,
        ] {
            function.instruction(&Instruction::LocalGet(input_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(input_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(input_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        for (kind, intrinsic_offset, prototype_global_index, boxed_kind) in [
            (
                ValueKind::Number,
                HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET,
                NUMBER_PROTOTYPE_GLOBAL_INDEX,
                BOXED_PRIMITIVE_KIND_NUMBER,
            ),
            (
                ValueKind::String,
                HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET,
                STRING_PROTOTYPE_GLOBAL_INDEX,
                BOXED_PRIMITIVE_KIND_STRING,
            ),
            (
                ValueKind::Boolean,
                HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET,
                BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
                BOXED_PRIMITIVE_KIND_BOOLEAN,
            ),
            (
                ValueKind::Symbol,
                HEAP_REALM_INTRINSICS_SYMBOL_PROTOTYPE_OFFSET,
                SYMBOL_PROTOTYPE_GLOBAL_INDEX,
                BOXED_PRIMITIVE_KIND_SYMBOL,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(input_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_realm_intrinsic_prototype_or_global(
                realm_local,
                intrinsic_offset,
                prototype_global_index,
                prototype_local,
                function,
            );
            self.emit_alloc_boxed_wrapper_with_prototype_from_locals(
                prototype_local,
                boxed_kind,
                input_payload_local,
                input_tag_local,
                payload_local,
                function,
            )?;
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        let bigint_constructor_local = self.reserve_temp_local();
        let bigint_fallback_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(bigint_constructor_local));
        self.load_i64_to_local_from_offset(
            bigint_constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            bigint_fallback_prototype_local,
            function,
        );
        self.emit_load_realm_intrinsic_prototype_or_local(
            realm_local,
            HEAP_REALM_INTRINSICS_BIGINT_PROTOTYPE_OFFSET,
            bigint_fallback_prototype_local,
            prototype_local,
            function,
        );
        self.emit_alloc_boxed_wrapper_with_prototype_from_locals(
            prototype_local,
            BOXED_PRIMITIVE_KIND_BIGINT,
            input_payload_local,
            input_tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(bigint_fallback_prototype_local);
        self.release_temp_local(bigint_constructor_local);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_throw_current_function_realm_type_error(
            "Object.prototype.valueOf called on null or undefined",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_local);
        self.release_temp_local(realm_local);
        Ok(())
    }

    pub(crate) fn emit_to_integer_or_infinity_number_payload_from_number_payload(
        &self,
        number_payload_local: u32,
        out_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(out_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(out_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::LocalSet(out_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(out_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_expr_to_numeric_locals(
        &mut self,
        expr: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();

        self.compile_expr_to_primitive_locals(
            expr,
            ToPrimitiveHint::Number,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_numeric_locals_without_throw_return(
            primitive_payload_local,
            primitive_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        Ok(())
    }

    pub(crate) fn emit_value_to_numeric_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.outline_value_to_numeric {
            if let Some(helper) = self.value_to_numeric_helper_function_index() {
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalGet(tag_local));
                for _ in 0..4 {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::LocalGet(self.current_env_local));
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(payload_local, tag_local, function);
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
                return Ok(());
            }
        }

        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();

        self.emit_tagged_to_primitive_locals_without_throw_propagation(
            ToPrimitiveHint::Number,
            payload_local,
            tag_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_numeric_locals_without_throw_return(
            primitive_payload_local,
            primitive_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        Ok(())
    }

    pub(crate) fn compile_expr_to_number_payload(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.kind == ValueKind::Number
            && expr.possible_kinds.is_singleton()
            && expr.possible_kinds.contains(ValueKind::Number)
            && !expr_result_tag_is_runtime_dynamic(&expr.expr)
        {
            self.compile_expr_payload(expr, function)?;
            return Ok(());
        }

        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.compile_expr_to_primitive_locals(
            expr,
            ToPrimitiveHint::Number,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn compile_expr_to_number_payload_nonstring(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if expr.kind == ValueKind::Number
            && expr.possible_kinds.is_singleton()
            && expr.possible_kinds.contains(ValueKind::Number)
            && !expr_result_tag_is_runtime_dynamic(&expr.expr)
        {
            self.compile_expr_payload(expr, function)?;
            return Ok(());
        }

        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.compile_expr_to_primitive_locals(
            expr,
            ToPrimitiveHint::Number,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_nonstring_value_to_number_payload(tag_local, payload_local, function)?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn compile_coercive_binary_number_to_locals(
        &mut self,
        op: ArithmeticBinaryOp,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload_local = self.reserve_temp_local();
        let lhs_tag_local = self.reserve_temp_local();
        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(lhs, lhs_payload_local, lhs_tag_local, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;

        self.emit_value_to_numeric_locals(lhs_payload_local, lhs_tag_local, function)?;
        self.emit_value_to_numeric_locals(rhs_payload_local, rhs_tag_local, function)?;

        // Both representations of a BigInt count as "BigInt" for the mixing
        // check, so an inline operand and a heap-backed one are not treated as
        // different types.
        self.emit_is_bigint_tag_i32(lhs_tag_local, function);
        self.emit_is_bigint_tag_i32(rhs_tag_local, function);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot mix BigInt and other types",
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            payload_local,
            tag_local,
            0,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_is_bigint_tag_i32(lhs_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_bigint_binary_op_to_locals(
            BigIntHelperOp::from_arithmetic(op),
            lhs_payload_local,
            lhs_tag_local,
            rhs_payload_local,
            rhs_tag_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::Else);
        if matches!(op, ArithmeticBinaryOp::Exp) {
            self.emit_number_pow_payload(
                lhs_payload_local,
                rhs_payload_local,
                payload_local,
                function,
            )?;
        } else if matches!(op, ArithmeticBinaryOp::Mod) {
            function.instruction(&Instruction::LocalGet(lhs_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(lhs_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(rhs_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Div);
            function.instruction(&Instruction::F64Trunc);
            function.instruction(&Instruction::LocalGet(rhs_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Mul);
            function.instruction(&Instruction::F64Sub);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(payload_local));
        } else {
            function.instruction(&Instruction::LocalGet(lhs_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(rhs_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            match op {
                ArithmeticBinaryOp::Add => function.instruction(&Instruction::F64Add),
                ArithmeticBinaryOp::Sub => function.instruction(&Instruction::F64Sub),
                ArithmeticBinaryOp::Mul => function.instruction(&Instruction::F64Mul),
                ArithmeticBinaryOp::Div => function.instruction(&Instruction::F64Div),
                ArithmeticBinaryOp::Mod | ArithmeticBinaryOp::Exp => unreachable!(),
            };
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(payload_local));
        }
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.release_temp_local(lhs_tag_local);
        self.release_temp_local(lhs_payload_local);
        Ok(())
    }

    pub(crate) fn emit_primitive_to_numeric_locals_without_throw_return(
        &mut self,
        primitive_payload_local: u32,
        primitive_tag_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_bigint_tag_i32(primitive_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(primitive_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload_without_throw_return(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_nan_payload(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
    }

    pub(crate) fn emit_number_pow_payload(
        &mut self,
        base_payload_local: u32,
        exponent_payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if let Some(function_index) = self.functions.number_pow_import_function_index() {
            function.instruction(&Instruction::LocalGet(base_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(exponent_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::Call(function_index));
            function.instruction(&Instruction::I64ReinterpretF64);
        } else {
            function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
            function.instruction(&Instruction::I64ReinterpretF64);
        }
        function.instruction(&Instruction::LocalSet(output_local));
        Ok(())
    }

    pub(crate) fn emit_to_index_i64_from_value_locals(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        index_local: u32,
        error_message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_index_from_number_payload(payload_local, index_local, error_message, function)
    }

    pub(crate) fn emit_to_index_from_number_payload(
        &mut self,
        number_payload_local: u32,
        index_local: u32,
        error_message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            9_007_199_254_740_991.0,
        )));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            -9_007_199_254_740_991.0,
        )));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_value_to_number_payload(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // ToNumber appears at ~130 builtin sites and the full per-kind composite
        // (ToPrimitive on objects, array→string, BigInt/Symbol throws, string
        // parse) is several KB inline; call the shared helper instead (except
        // while compiling the helper itself). The already-Number fast path stays
        // inline so the common case never pays the call. A BigInt/Symbol/
        // ToPrimitive throw inside the helper is surfaced through the completion
        // slots and re-raised here with a completion return — the same discipline
        // the inline composite's own throw sites use (`emit_return_current_
        // completion`), which is valid at any block depth.
        if self.outline_value_to_number {
            if let Some(helper) = self.value_to_number_helper_function_index() {
                let result_payload_local = self.reserve_temp_local();
                let result_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalSet(result_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(result_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalGet(tag_local));
                for _ in 0..5 {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(result_payload_local, result_tag_local, function);
                function.instruction(&Instruction::End);
                self.emit_propagate_throw_from_locals_if_needed(
                    result_payload_local,
                    result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(result_payload_local));
                self.release_temp_local(result_tag_local);
                self.release_temp_local(result_payload_local);
                return Ok(());
            }
        }
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_object_to_primitive_locals(
            ToPrimitiveHint::Number,
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_array_to_string_locals(
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_number_payload(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::LocalSet(primitive_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(primitive_tag_local));
        self.emit_primitive_to_number_payload(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_value_to_number_payload_without_throw_return(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_object_to_primitive_locals_without_throw_propagation(
            ToPrimitiveHint::Number,
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(primitive_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_nan_payload(function);
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload_without_throw_return(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_array_to_string_locals(
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_number_payload_without_throw_return(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::LocalSet(primitive_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(primitive_tag_local));
        self.emit_primitive_to_number_payload_without_throw_return(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload_without_throw_return(
            tag_local,
            payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_primitive_to_number_payload(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_primitive_to_number_payload_inner(tag_local, payload_local, true, function)
    }

    pub(crate) fn emit_primitive_to_number_payload_without_throw_return(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_primitive_to_number_payload_inner(tag_local, payload_local, false, function)
    }

    pub(crate) fn emit_primitive_to_number_payload_inner(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        return_on_throw: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_nan_payload(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(tag_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert BigInt to number",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        self.emit_nan_payload(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert Symbol to number",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if return_on_throw {
            self.emit_return_current_completion(function);
        }
        self.emit_nan_payload(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_string_to_number_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_nan_payload(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_value_to_number_payload_allow_bigint(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_object_to_primitive_locals(
            ToPrimitiveHint::Number,
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        // A ToPrimitive throw here leaves completion=THROW with the error in
        // `primitive_payload_local`/`primitive_tag_local` (Object-tagged) rather
        // than branching (see `emit_object_to_primitive_locals_locals_inner`).
        // Skip the further primitive->number conversion in that case so it
        // doesn't reinterpret the error object as a NaN-producing primitive and
        // silently mask the throw; callers of this function are responsible for
        // checking `self.completion_local` and propagating.
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload_allow_bigint(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_array_to_string_locals(
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_primitive_to_number_payload_allow_bigint(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::LocalSet(primitive_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(primitive_tag_local));
        self.emit_primitive_to_number_payload_allow_bigint(
            primitive_tag_local,
            primitive_payload_local,
            function,
        )?;
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload_allow_bigint(tag_local, payload_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_primitive_to_number_payload_allow_bigint(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_number_payload(tag_local, payload_local, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_value_to_property_key_payload(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let property_key_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(property_key_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(payload_local, tag_local, function)?;
        function.instruction(&Instruction::LocalSet(property_key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(property_key_local));

        self.release_temp_local(property_key_local);
        Ok(())
    }

    /// Converts a tagged value held in `payload_local`/`tag_local` to a
    /// PropertyKey in place. Unlike `emit_value_to_property_key_payload`, this
    /// path keeps abrupt ToPrimitive completions in locals long enough to route
    /// them through the active in-function throw handler rather than returning
    /// from the whole Wasm function.
    pub(crate) fn emit_value_to_property_key_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();

        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::String,
            payload_local,
            tag_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        // `emit_tagged_to_primitive_locals` deliberately leaves abrupt
        // completions in locals. This call site has no manually-emitted Wasm
        // blocks open, so no additional untracked depth is needed here.
        self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
            primitive_payload_local,
            primitive_tag_local,
            0,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(primitive_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_string_payload(
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        Ok(())
    }

    pub(crate) fn emit_is_property_key_i32(&self, tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_value_to_bigint_locals(
        &mut self,
        input_tag_local: u32,
        input_payload_local: u32,
        allow_number: bool,
        output_payload_local: u32,
        output_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();

        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::Number,
            input_payload_local,
            input_tag_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_primitive_to_bigint_locals(
            primitive_tag_local,
            primitive_payload_local,
            allow_number,
            output_payload_local,
            output_tag_local,
            function,
        )?;

        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        Ok(())
    }

    pub(crate) fn emit_primitive_to_bigint_locals(
        &mut self,
        input_tag_local: u32,
        input_payload_local: u32,
        allow_number: bool,
        output_payload_local: u32,
        output_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_bigint_tag_i32(input_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::LocalSet(output_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(input_payload_local));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(output_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if allow_number {
            function.instruction(&Instruction::LocalGet(input_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(input_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                RANGE_ERROR_NAME,
                "cannot convert Number to BigInt",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
                function.instruction(&Instruction::LocalGet(input_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    RANGE_ERROR_NAME,
                    "cannot convert Number to BigInt",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::LocalGet(input_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(input_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Trunc);
            function.instruction(&Instruction::F64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                RANGE_ERROR_NAME,
                "cannot convert non-integer Number to BigInt",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(input_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::LocalSet(output_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
            function.instruction(&Instruction::LocalSet(output_tag_local));
        } else {
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "cannot convert Number to BigInt",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_to_bigint_locals(
            input_payload_local,
            output_payload_local,
            output_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "cannot convert value to BigInt",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_string_to_bigint_locals(
        &mut self,
        string_payload_local: u32,
        output_payload_local: u32,
        output_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let radix_local = self.reserve_temp_local();
        let prefix_seen_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let saw_digit_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        let limb_capacity_local = self.reserve_temp_local();
        let limb_index_local = self.reserve_temp_local();
        let limb_local = self.reserve_temp_local();
        let carry_local = self.reserve_temp_local();
        let low_product_local = self.reserve_temp_local();
        let high_product_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let trimmed_string_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(output_tag_local));

        self.emit_ecmascript_trim_payload_from_locals(string_payload_local, true, true, function)?;
        function.instruction(&Instruction::LocalSet(trimmed_string_payload_local));
        self.emit_unpack_string_payload(
            trimmed_string_payload_local,
            offset_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(prefix_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        for (lower, upper, radix) in [
            (b'x', b'X', 16_i64),
            (b'o', b'O', 8_i64),
            (b'b', b'B', 2_i64),
        ] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(lower as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(upper as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(radix));
            function.instruction(&Instruction::LocalSet(radix_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(prefix_seen_local));
            function.instruction(&Instruction::LocalGet(start_local));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(start_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prefix_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(prefix_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(magnitude_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(limbs_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(limb_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(limb_capacity_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(saw_digit_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64 - 10));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'A' as i64 - 10));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::LocalSet(carry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(limb_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(limb_index_local));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::LocalSet(limb_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::LocalGet(limb_index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(limb_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low_product_local));
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(low_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high_product_local));
        function.instruction(&Instruction::LocalGet(low_product_local));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalGet(high_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(limb_local));
        function.instruction(&Instruction::LocalGet(high_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(carry_local));

        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::LocalSet(magnitude_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::LocalGet(limb_index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(limb_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(limb_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(15));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(limb_capacity_local));
        function.instruction(&Instruction::LocalGet(limb_capacity_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(limb_capacity_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(limb_capacity_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(limb_local));
        self.emit_heap_alloc_from_local(limb_local, function)?;
        function.instruction(&Instruction::LocalSet(limbs_local));
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(limb_count_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(saw_digit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(saw_digit_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "cannot convert value to BigInt",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_heap_alloc_const(HEAP_BIGINT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(carry_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_SIGN_OFFSET,
            carry_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_CAP_OFFSET,
            limb_capacity_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(record_local));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::LocalSet(output_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(output_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(output_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_alloc_one_limb_bigint(-1, magnitude_local, function)?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::LocalSet(output_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(output_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_alloc_one_limb_bigint(1, magnitude_local, function)?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::LocalSet(output_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(trimmed_string_payload_local);
        self.release_temp_local(record_local);
        self.release_temp_local(high_product_local);
        self.release_temp_local(low_product_local);
        self.release_temp_local(carry_local);
        self.release_temp_local(limb_local);
        self.release_temp_local(limb_index_local);
        self.release_temp_local(limb_capacity_local);
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(magnitude_local);
        self.release_temp_local(invalid_local);
        self.release_temp_local(saw_digit_local);
        self.release_temp_local(negative_local);
        self.release_temp_local(prefix_seen_local);
        self.release_temp_local(radix_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(end_local);
        self.release_temp_local(start_local);
        self.release_temp_local(len_local);
        self.release_temp_local(offset_local);

        Ok(())
    }

    pub(crate) fn emit_nonstring_value_to_number_payload(
        &self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_nan_payload(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_canonical_numeric_index_string(
        &mut self,
        string_payload_local: u32,
        number_payload_local: u32,
        is_canonical_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let canonical_string_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(is_canonical_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("-0")));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_payload_equality_i32(string_payload_local, self.scratch_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(-0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_canonical_local));
        function.instruction(&Instruction::Else);
        self.emit_string_to_number_payload(string_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(number_payload_local));
        self.emit_number_to_string_payload(number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(canonical_string_payload_local));
        self.emit_string_payload_equality_i32(
            string_payload_local,
            canonical_string_payload_local,
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(is_canonical_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(canonical_string_payload_local);
        Ok(())
    }

    pub(crate) fn emit_string_to_number_payload(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // ToNumber of a string operand appears throughout the builtins and its
        // inline parse state machine is several KB; call the shared helper
        // instead (except while compiling the helper itself). The helper returns
        // the standard four-i64 tuple with the f64-bits payload first.
        if self.outline_string_to_number {
            if let Some(helper) = self.string_to_number_helper_function_index() {
                function.instruction(&Instruction::LocalGet(string_payload_local));
                for _ in 0..6 {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::Call(helper));
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::Drop);
                return Ok(());
            }
        }
        let offset_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let output_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        let saw_digit_local = self.reserve_temp_local();
        let dot_seen_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();
        let exponent_saw_digit_local = self.reserve_temp_local();
        let infinity_match_local = self.reserve_temp_local();
        let decimal_start_local = self.reserve_temp_local();
        let decimal_len_local = self.reserve_temp_local();
        let decimal_payload_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_payload_local, offset_local, len_local, function);
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        for bytes in [
            &[0xC2, 0xA0][..],       // U+00A0
            &[0xE1, 0x9A, 0x80][..], // U+1680
            &[0xE2, 0x80, 0x80][..], // U+2000
            &[0xE2, 0x80, 0x81][..], // U+2001
            &[0xE2, 0x80, 0x82][..], // U+2002
            &[0xE2, 0x80, 0x83][..], // U+2003
            &[0xE2, 0x80, 0x84][..], // U+2004
            &[0xE2, 0x80, 0x85][..], // U+2005
            &[0xE2, 0x80, 0x86][..], // U+2006
            &[0xE2, 0x80, 0x87][..], // U+2007
            &[0xE2, 0x80, 0x88][..], // U+2008
            &[0xE2, 0x80, 0x89][..], // U+2009
            &[0xE2, 0x80, 0x8A][..], // U+200A
            &[0xE2, 0x80, 0xA8][..], // U+2028
            &[0xE2, 0x80, 0xA9][..], // U+2029
            &[0xE2, 0x80, 0xAF][..], // U+202F
            &[0xE2, 0x81, 0x9F][..], // U+205F
            &[0xE3, 0x80, 0x80][..], // U+3000
            &[0xEF, 0xBB, 0xBF][..], // U+FEFF
        ] {
            Self::emit_skip_utf8_whitespace_forward(
                function,
                end_local,
                start_local,
                byte_local,
                bytes,
            );
        }
        self.emit_is_ascii_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        for bytes in [
            &[0xC2, 0xA0][..],       // U+00A0
            &[0xE1, 0x9A, 0x80][..], // U+1680
            &[0xE2, 0x80, 0x80][..], // U+2000
            &[0xE2, 0x80, 0x81][..], // U+2001
            &[0xE2, 0x80, 0x82][..], // U+2002
            &[0xE2, 0x80, 0x83][..], // U+2003
            &[0xE2, 0x80, 0x84][..], // U+2004
            &[0xE2, 0x80, 0x85][..], // U+2005
            &[0xE2, 0x80, 0x86][..], // U+2006
            &[0xE2, 0x80, 0x87][..], // U+2007
            &[0xE2, 0x80, 0x88][..], // U+2008
            &[0xE2, 0x80, 0x89][..], // U+2009
            &[0xE2, 0x80, 0x8A][..], // U+200A
            &[0xE2, 0x80, 0xA8][..], // U+2028
            &[0xE2, 0x80, 0xA9][..], // U+2029
            &[0xE2, 0x80, 0xAF][..], // U+202F
            &[0xE2, 0x81, 0x9F][..], // U+205F
            &[0xE3, 0x80, 0x80][..], // U+3000
            &[0xEF, 0xBB, 0xBF][..], // U+FEFF
        ] {
            Self::emit_skip_utf8_whitespace_backward(
                function,
                start_local,
                end_local,
                byte_local,
                bytes,
            );
        }
        self.emit_is_ascii_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(end_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(b'0' as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(b'x' as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I32Const(b'X' as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(saw_digit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        self.emit_hex_value_or_minus_one(byte_local, digit_local, function);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(saw_digit_local));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(16.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(saw_digit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        self.emit_nan_payload(function);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalSet(decimal_start_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(infinity_match_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(infinity_match_local));
        for (offset, byte) in b"Infinity".iter().copied().enumerate() {
            function.instruction(&Instruction::LocalGet(start_local));
            function.instruction(&Instruction::I64Const(offset as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(byte_local));
            function.instruction(&Instruction::LocalGet(infinity_match_local));
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(infinity_match_local));
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(infinity_match_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(saw_digit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dot_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exponent_saw_digit_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(saw_digit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(dot_seen_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(dot_seen_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'E' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(saw_digit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(exponent_saw_digit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(exponent_saw_digit_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(saw_digit_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(decimal_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(decimal_len_local));
        self.emit_pack_string_payload(decimal_start_local, decimal_len_local, function);
        function.instruction(&Instruction::LocalSet(decimal_payload_local));
        self.emit_decimal_to_binary64_payload(decimal_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        self.emit_nan_payload(function);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(output_local));

        self.release_temp_local(decimal_payload_local);
        self.release_temp_local(decimal_len_local);
        self.release_temp_local(decimal_start_local);
        self.release_temp_local(infinity_match_local);
        self.release_temp_local(exponent_saw_digit_local);
        self.release_temp_local(invalid_local);
        self.release_temp_local(negative_local);
        self.release_temp_local(dot_seen_local);
        self.release_temp_local(saw_digit_local);
        self.release_temp_local(result_local);
        self.release_temp_local(output_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(end_local);
        self.release_temp_local(start_local);
        self.release_temp_local(len_local);
        self.release_temp_local(offset_local);
        Ok(())
    }

    pub(crate) fn emit_is_ascii_whitespace_i32(&self, byte_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64Eq);
        for byte in [b'\t', b'\n', 0x0B, 0x0C, b'\r'] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    pub(crate) fn emit_skip_utf8_whitespace_forward(
        function: &mut Function,
        end_local: u32,
        index_local: u32,
        byte_local: u32,
        bytes: &[u8],
    ) {
        debug_assert!(bytes.len() >= 2);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(bytes[0] as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const((bytes.len() - 1) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));

        for (offset, byte) in bytes.iter().copied().enumerate().skip(1) {
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(offset as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I32Const(byte as i32));
            function.instruction(&Instruction::I32Eq);
            if offset > 1 {
                function.instruction(&Instruction::I32And);
            }
        }

        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(bytes.len() as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_skip_utf8_whitespace_backward(
        function: &mut Function,
        start_local: u32,
        end_local: u32,
        byte_local: u32,
        bytes: &[u8],
    ) {
        debug_assert!(bytes.len() >= 2);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(bytes[bytes.len() - 1] as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(bytes.len() as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));

        for (offset, byte) in bytes.iter().copied().enumerate() {
            function.instruction(&Instruction::LocalGet(end_local));
            function.instruction(&Instruction::I64Const(bytes.len() as i64));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(offset as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I32Const(byte as i32));
            function.instruction(&Instruction::I32Eq);
            if offset > 0 {
                function.instruction(&Instruction::I32And);
            }
        }

        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64Const(bytes.len() as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(end_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_loose_equality_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
            && rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
        {
            let lhs_payload = self.reserve_temp_local();
            let lhs_tag = self.reserve_temp_local();
            let rhs_payload = self.reserve_temp_local();
            let rhs_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
            self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
            self.emit_loose_tagged_equality_i32(
                lhs_tag,
                lhs_payload,
                rhs_tag,
                rhs_payload,
                function,
            )?;
            self.release_temp_local(rhs_tag);
            self.release_temp_local(rhs_payload);
            self.release_temp_local(lhs_tag);
            self.release_temp_local(lhs_payload);
            return Ok(());
        }

        let lhs_raw_payload = self.reserve_temp_local();
        let lhs_raw_tag = self.reserve_temp_local();
        let rhs_raw_payload = self.reserve_temp_local();
        let rhs_raw_tag = self.reserve_temp_local();
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        let equality_result_local = self.reserve_temp_local();
        let completed_local = self.reserve_temp_local();

        self.compile_expr_to_locals(lhs, lhs_raw_payload, lhs_raw_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_raw_payload, rhs_raw_tag, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(equality_result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(completed_local));
        self.compile_nullish_tagged_i32(lhs_raw_tag, function)?;
        self.compile_nullish_tagged_i32(rhs_raw_tag, function)?;
        function.instruction(&Instruction::I32And);
        self.compile_nullish_tagged_i32(lhs_raw_tag, function)?;
        self.emit_is_htmldda_function_i32(rhs_raw_tag, rhs_raw_payload, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.compile_nullish_tagged_i32(rhs_raw_tag, function)?;
        self.emit_is_htmldda_function_i32(lhs_raw_tag, lhs_raw_payload, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(equality_result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(completed_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(completed_local));
        function.instruction(&Instruction::I64Eqz);
        self.compile_nullish_tagged_i32(lhs_raw_tag, function)?;
        self.compile_nullish_tagged_i32(rhs_raw_tag, function)?;
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(completed_local));
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(lhs_raw_tag, function);
        self.emit_is_heap_object_like_tag_i32(rhs_raw_tag, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_raw_payload));
        function.instruction(&Instruction::LocalGet(rhs_raw_payload));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(equality_result_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(completed_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(completed_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.push_control(ControlFrameKind::If);
        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::Default,
            lhs_raw_payload,
            lhs_raw_tag,
            lhs_payload,
            lhs_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(lhs_payload, lhs_tag, function)?;
        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::Default,
            rhs_raw_payload,
            rhs_raw_tag,
            rhs_payload,
            rhs_tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(rhs_payload, rhs_tag, function)?;
        self.emit_loose_tagged_equality_i32(lhs_tag, lhs_payload, rhs_tag, rhs_payload, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(equality_result_local));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(equality_result_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(completed_local);
        self.release_temp_local(equality_result_local);
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        self.release_temp_local(rhs_raw_tag);
        self.release_temp_local(rhs_raw_payload);
        self.release_temp_local(lhs_raw_tag);
        self.release_temp_local(lhs_raw_payload);
        return Ok(());
    }

    pub(crate) fn compile_loose_equality_nonstring_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
            || !rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
        {
            return self.compile_loose_equality_i32(lhs, rhs, function);
        }
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        let temp_number_local = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        function.instruction(&Instruction::LocalGet(lhs_tag));
        function.instruction(&Instruction::LocalGet(rhs_tag));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_tagged_payload_equality_i32(
            lhs_tag,
            lhs_payload,
            rhs_tag,
            rhs_payload,
            function,
        );
        function.instruction(&Instruction::Else);
        self.compile_nullish_tagged_i32(lhs_tag, function)?;
        self.compile_nullish_tagged_i32(rhs_tag, function)?;
        function.instruction(&Instruction::I32And);
        self.compile_nullish_tagged_i32(lhs_tag, function)?;
        self.emit_is_htmldda_function_i32(rhs_tag, rhs_payload, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.compile_nullish_tagged_i32(rhs_tag, function)?;
        self.emit_is_htmldda_function_i32(lhs_tag, lhs_payload, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_value_to_number_payload(lhs_tag, lhs_payload, function)?;
        function.instruction(&Instruction::LocalSet(temp_number_local));
        self.emit_number_payload_loose_equal_i32(
            temp_number_local,
            rhs_tag,
            rhs_payload,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_value_to_number_payload(rhs_tag, rhs_payload, function)?;
        function.instruction(&Instruction::LocalSet(temp_number_local));
        self.emit_number_payload_loose_equal_i32(
            temp_number_local,
            lhs_tag,
            lhs_payload,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(lhs_tag, function);
        self.emit_is_bigint_tag_i32(rhs_tag, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_mixed_bigint_equality_i32(lhs_tag, lhs_payload, rhs_payload, function);
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(lhs_tag, function);
        function.instruction(&Instruction::LocalGet(rhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_bigint_number_equality_i32(lhs_tag, lhs_payload, rhs_payload, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_bigint_tag_i32(rhs_tag, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_bigint_number_equality_i32(rhs_tag, rhs_payload, lhs_payload, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(temp_number_local);
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    pub(crate) fn emit_nonstring_tagged_payload_equality_i32(
        &mut self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        _rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_heap_bigint_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_loose_tagged_equality_i32(
        &mut self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let temp_number_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_tagged_payload_equality_i32(
            lhs_tag_local,
            lhs_payload_local,
            rhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.compile_nullish_tagged_i32(lhs_tag_local, function)?;
        self.compile_nullish_tagged_i32(rhs_tag_local, function)?;
        function.instruction(&Instruction::I32And);
        self.compile_nullish_tagged_i32(lhs_tag_local, function)?;
        self.emit_is_htmldda_function_i32(rhs_tag_local, rhs_payload_local, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.compile_nullish_tagged_i32(rhs_tag_local, function)?;
        self.emit_is_htmldda_function_i32(lhs_tag_local, lhs_payload_local, function)?;
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(temp_number_local));
        self.emit_number_payload_loose_equal_i32(
            temp_number_local,
            rhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(temp_number_local));
        self.emit_number_payload_loose_equal_i32(
            temp_number_local,
            lhs_tag_local,
            lhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_number_payload_loose_equal_i32(
            lhs_payload_local,
            rhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_number_payload_loose_equal_i32(
            rhs_payload_local,
            lhs_tag_local,
            lhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(lhs_tag_local, function);
        self.emit_is_bigint_tag_i32(rhs_tag_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_mixed_bigint_equality_i32(
            lhs_tag_local,
            lhs_payload_local,
            rhs_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(lhs_tag_local, function);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_bigint_number_equality_i32(
            lhs_tag_local,
            lhs_payload_local,
            rhs_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_bigint_tag_i32(rhs_tag_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_bigint_number_equality_i32(
            rhs_tag_local,
            rhs_payload_local,
            lhs_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(temp_number_local);
        Ok(())
    }

    pub(crate) fn emit_mixed_bigint_equality_i32(
        &mut self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_heap_bigint_inline_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        self.emit_heap_bigint_inline_equality_i32(rhs_payload_local, lhs_payload_local, function);
        function.instruction(&Instruction::End);
    }

    fn emit_bigint_number_equality_i32(
        &mut self,
        bigint_tag_local: u32,
        bigint_payload_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(bigint_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_heap_bigint_number_equality_i32(
            bigint_payload_local,
            number_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_inline_bigint_number_equality_i32(
            bigint_payload_local,
            number_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
    }

    fn emit_inline_bigint_number_equality_i32(
        &mut self,
        bigint_payload_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) {
        let integer_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(i64::MIN as f64)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            9_223_372_036_854_775_808.0,
        )));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(integer_local));
        function.instruction(&Instruction::LocalGet(bigint_payload_local));
        function.instruction(&Instruction::LocalGet(integer_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);

        self.release_temp_local(integer_local);
    }

    fn emit_heap_bigint_inline_equality_i32(
        &mut self,
        heap_payload_local: u32,
        inline_payload_local: u32,
        function: &mut Function,
    ) {
        let heap_sign_local = self.reserve_temp_local();
        let heap_limbs_local = self.reserve_temp_local();
        let heap_limb_count_local = self.reserve_temp_local();
        let inline_sign_local = self.reserve_temp_local();
        let inline_magnitude_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            heap_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            heap_sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            heap_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            heap_limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            heap_payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            heap_limb_count_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(inline_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(inline_sign_local));

        function.instruction(&Instruction::LocalGet(inline_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(inline_payload_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(inline_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(inline_magnitude_local));

        function.instruction(&Instruction::LocalGet(inline_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(heap_sign_local));
        function.instruction(&Instruction::LocalGet(inline_sign_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(heap_limb_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(heap_limbs_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalGet(inline_magnitude_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);

        self.release_temp_local(inline_magnitude_local);
        self.release_temp_local(inline_sign_local);
        self.release_temp_local(heap_limb_count_local);
        self.release_temp_local(heap_limbs_local);
        self.release_temp_local(heap_sign_local);
    }

    fn emit_heap_bigint_number_equality_i32(
        &mut self,
        heap_payload_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) {
        // Reconstruct the integer from its IEEE-754 significand and exponent so
        // comparison against arbitrary-precision limbs cannot round unequal values.
        let number_bits_local = self.reserve_temp_local();
        let exponent_local = self.reserve_temp_local();
        let significand_local = self.reserve_temp_local();
        let shift_local = self.reserve_temp_local();
        let first_limb_local = self.reserve_temp_local();
        let bit_offset_local = self.reserve_temp_local();
        let expected_limb_count_local = self.reserve_temp_local();
        let expected_sign_local = self.reserve_temp_local();
        let expected_limb_local = self.reserve_temp_local();
        let heap_sign_local = self.reserve_temp_local();
        let heap_limbs_local = self.reserve_temp_local();
        let heap_limb_count_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let equal_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));

        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(number_bits_local));

        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(expected_sign_local));

        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(1023));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(exponent_local));

        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Const((1_i64 << 52) - 1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(significand_local));

        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(first_limb_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(bit_offset_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(expected_limb_count_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_limb_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(bit_offset_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(expected_limb_count_local));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            heap_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            heap_sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            heap_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            heap_limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            heap_payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            heap_limb_count_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(heap_sign_local));
        function.instruction(&Instruction::LocalGet(expected_sign_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(heap_limb_count_local));
        function.instruction(&Instruction::LocalGet(expected_limb_count_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(equal_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(heap_limb_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(expected_limb_local));
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(first_limb_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::LocalGet(bit_offset_local));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(expected_limb_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(bit_offset_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(first_limb_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::LocalGet(bit_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(expected_limb_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(expected_limb_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(heap_limbs_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalGet(expected_limb_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(equal_local));
        function.instruction(&Instruction::LocalGet(equal_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(equal_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);

        self.release_temp_local(equal_local);
        self.release_temp_local(index_local);
        self.release_temp_local(heap_limb_count_local);
        self.release_temp_local(heap_limbs_local);
        self.release_temp_local(heap_sign_local);
        self.release_temp_local(expected_limb_local);
        self.release_temp_local(expected_sign_local);
        self.release_temp_local(expected_limb_count_local);
        self.release_temp_local(bit_offset_local);
        self.release_temp_local(first_limb_local);
        self.release_temp_local(shift_local);
        self.release_temp_local(significand_local);
        self.release_temp_local(exponent_local);
        self.release_temp_local(number_bits_local);
    }

    pub(crate) fn emit_number_payload_loose_equal_i32(
        &mut self,
        number_payload_local: u32,
        other_tag_local: u32,
        other_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let other_number_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(other_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(other_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(other_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_to_number_payload(other_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(other_number_local));
        function.instruction(&Instruction::LocalGet(number_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(other_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(other_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_bigint_number_equality_i32(
            other_tag_local,
            other_payload_local,
            number_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(other_number_local);
        Ok(())
    }

    pub(crate) fn compile_compare_value_i32(
        &mut self,
        op: RelationalBinaryOp,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if lhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
            && rhs.possible_kinds.is_subset_of(KindSet::PRIMITIVE_ONLY)
        {
            let lhs_payload = self.reserve_temp_local();
            let lhs_tag = self.reserve_temp_local();
            let rhs_payload = self.reserve_temp_local();
            let rhs_tag = self.reserve_temp_local();
            self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
            self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
            self.emit_compare_tagged_i32(op, lhs_tag, lhs_payload, rhs_tag, rhs_payload, function)?;
            self.release_temp_local(rhs_tag);
            self.release_temp_local(rhs_payload);
            self.release_temp_local(lhs_tag);
            self.release_temp_local(lhs_payload);
            return Ok(());
        }
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        self.compile_operand_pair_to_primitive_locals(
            lhs,
            rhs,
            ToPrimitiveHint::Number,
            lhs_payload,
            lhs_tag,
            rhs_payload,
            rhs_tag,
            function,
        )?;
        self.emit_compare_tagged_i32(op, lhs_tag, lhs_payload, rhs_tag, rhs_payload, function)?;
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    pub(crate) fn emit_compare_tagged_i32(
        &mut self,
        op: RelationalBinaryOp,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_number_local = self.reserve_temp_local();
        let rhs_number_local = self.reserve_temp_local();
        self.emit_is_bigint_tag_i32(lhs_tag_local, function);
        self.emit_is_bigint_tag_i32(rhs_tag_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_bigint_relational_i32(
            op,
            lhs_payload_local,
            lhs_tag_local,
            rhs_payload_local,
            rhs_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(lhs_tag_local, function);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_bigint_number_relational_i32(
            op,
            lhs_payload_local,
            lhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_bigint_tag_i32(rhs_tag_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        let reversed_op = match op {
            RelationalBinaryOp::LessThan => RelationalBinaryOp::GreaterThan,
            RelationalBinaryOp::LessThanOrEqual => RelationalBinaryOp::GreaterThanOrEqual,
            RelationalBinaryOp::GreaterThan => RelationalBinaryOp::LessThan,
            RelationalBinaryOp::GreaterThanOrEqual => RelationalBinaryOp::LessThanOrEqual,
        };
        self.emit_bigint_number_relational_i32(
            reversed_op,
            rhs_payload_local,
            rhs_tag_local,
            lhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_compare_i32(op, lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(lhs_tag_local, lhs_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(lhs_number_local));
        self.emit_value_to_number_payload(rhs_tag_local, rhs_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(rhs_number_local));
        function.instruction(&Instruction::LocalGet(lhs_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rhs_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        match op {
            RelationalBinaryOp::LessThan => function.instruction(&Instruction::F64Lt),
            RelationalBinaryOp::LessThanOrEqual => function.instruction(&Instruction::F64Le),
            RelationalBinaryOp::GreaterThan => function.instruction(&Instruction::F64Gt),
            RelationalBinaryOp::GreaterThanOrEqual => function.instruction(&Instruction::F64Ge),
        };
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(rhs_number_local);
        self.release_temp_local(lhs_number_local);
        Ok(())
    }

    pub(crate) fn emit_string_payload_compare_i32(
        &mut self,
        op: RelationalBinaryOp,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
        self.emit_string_payload_utf16_compare_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::I32Const(0));
        match op {
            RelationalBinaryOp::LessThan => function.instruction(&Instruction::I32LtS),
            RelationalBinaryOp::LessThanOrEqual => function.instruction(&Instruction::I32LeS),
            RelationalBinaryOp::GreaterThan => function.instruction(&Instruction::I32GtS),
            RelationalBinaryOp::GreaterThanOrEqual => function.instruction(&Instruction::I32GeS),
        };
    }

    /// Compare string payloads in ECMAScript's UTF-16 code-unit order.
    ///
    /// The payload stores UTF-8, with WTF-8 encodings for lone surrogates.
    /// Astral scalars therefore yield their high surrogate first and retain the
    /// low surrogate for the next iteration without decoding the scalar again.
    /// The emitted value is -1, 0, or 1 as an i32.
    pub(crate) fn emit_string_payload_utf16_compare_i32(
        &mut self,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
        let lhs_offset = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_offset = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let lhs_index_local = self.reserve_temp_local();
        let rhs_index_local = self.reserve_temp_local();
        let lhs_pending_low_surrogate_local = self.reserve_temp_local();
        let rhs_pending_low_surrogate_local = self.reserve_temp_local();
        let lhs_byte_local = self.reserve_temp_local();
        let rhs_byte_local = self.reserve_temp_local();
        let lhs_codepoint_local = self.reserve_temp_local();
        let rhs_codepoint_local = self.reserve_temp_local();
        let lhs_advance_local = self.reserve_temp_local();
        let rhs_advance_local = self.reserve_temp_local();
        let decode_temp_local = self.reserve_temp_local();
        let lhs_code_unit_local = self.reserve_temp_local();
        let rhs_code_unit_local = self.reserve_temp_local();
        let lhs_exhausted_local = self.reserve_temp_local();
        let rhs_exhausted_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(lhs_payload_local, lhs_offset, lhs_len, function);
        self.emit_unpack_string_payload(rhs_payload_local, rhs_offset, rhs_len, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lhs_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rhs_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lhs_pending_low_surrogate_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rhs_pending_low_surrogate_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(lhs_pending_low_surrogate_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(lhs_exhausted_local));
        function.instruction(&Instruction::LocalGet(rhs_pending_low_surrogate_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(rhs_exhausted_local));
        function.instruction(&Instruction::LocalGet(lhs_exhausted_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(rhs_exhausted_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_exhausted_local));
        function.instruction(&Instruction::LocalGet(rhs_exhausted_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_exhausted_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(lhs_pending_low_surrogate_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(lhs_offset, lhs_index_local, lhs_byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            lhs_offset,
            lhs_index_local,
            lhs_len,
            lhs_byte_local,
            lhs_codepoint_local,
            lhs_advance_local,
            decode_temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(lhs_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_index_local));
        function.instruction(&Instruction::LocalGet(lhs_codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_code_unit_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(lhs_pending_low_surrogate_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_codepoint_local));
        function.instruction(&Instruction::LocalSet(lhs_code_unit_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_code_unit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(lhs_pending_low_surrogate_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(rhs_pending_low_surrogate_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(rhs_offset, rhs_index_local, rhs_byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            rhs_offset,
            rhs_index_local,
            rhs_len,
            rhs_byte_local,
            rhs_codepoint_local,
            rhs_advance_local,
            decode_temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(rhs_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_index_local));
        function.instruction(&Instruction::LocalGet(rhs_codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(rhs_codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_code_unit_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(rhs_pending_low_surrogate_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rhs_codepoint_local));
        function.instruction(&Instruction::LocalSet(rhs_code_unit_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rhs_codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_code_unit_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rhs_pending_low_surrogate_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(lhs_code_unit_local));
        function.instruction(&Instruction::LocalGet(rhs_code_unit_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_code_unit_local));
        function.instruction(&Instruction::LocalGet(rhs_code_unit_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(result_local);
        self.release_temp_local(rhs_exhausted_local);
        self.release_temp_local(lhs_exhausted_local);
        self.release_temp_local(rhs_code_unit_local);
        self.release_temp_local(lhs_code_unit_local);
        self.release_temp_local(decode_temp_local);
        self.release_temp_local(rhs_advance_local);
        self.release_temp_local(lhs_advance_local);
        self.release_temp_local(rhs_codepoint_local);
        self.release_temp_local(lhs_codepoint_local);
        self.release_temp_local(rhs_byte_local);
        self.release_temp_local(lhs_byte_local);
        self.release_temp_local(rhs_pending_low_surrogate_local);
        self.release_temp_local(lhs_pending_low_surrogate_local);
        self.release_temp_local(rhs_index_local);
        self.release_temp_local(lhs_index_local);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_offset);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_offset);
    }

    pub(crate) fn compile_typeof_payload(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // One predicate, spelled once. This used to hand-roll a five-variant
        // `matches!` that meant the same thing as
        // `expr_result_tag_is_runtime_dynamic` but omitted every call form, so a
        // call whose inferred kind was a (wrong) singleton got constant-folded
        // into a literal type name instead of re-reading the runtime tag. That
        // is exactly the distrust `compile_expr_to_locals` already applies
        // (expressions.rs:2560); duplicating it here only created a second,
        // weaker copy. `ExprIr::Arguments` is not part of the planning
        // predicate, so it stays explicit.
        let is_runtime_storage_read = matches!(&expr.expr, ExprIr::Arguments)
            || expr_result_tag_is_runtime_dynamic(&expr.expr);
        if expr.possible_kinds.is_singleton()
            && !is_runtime_storage_read
            && expr.kind != ValueKind::Object
        {
            if expr.kind == ValueKind::Function {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                let tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_is_htmldda_function_i32(tag_local, self.scratch_local, function)?;
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(self.strings.payload("function")));
                function.instruction(&Instruction::End);
                self.release_temp_local(tag_local);
            } else {
                self.emit_typeof_payload_for_kind(expr.kind, function);
            }
            return Ok(());
        }
        self.compile_expr_to_locals(expr, self.scratch_local, self.result_tag_local, function)?;
        self.emit_typeof_payload_from_tag_payload_local(
            self.result_tag_local,
            self.scratch_local,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn emit_typeof_payload_for_kind(&self, kind: ValueKind, function: &mut Function) {
        let value = match kind {
            ValueKind::Undefined => "undefined",
            ValueKind::Null | ValueKind::Object | ValueKind::Array | ValueKind::Arguments => {
                "object"
            }
            ValueKind::Boolean => "boolean",
            ValueKind::Number => "number",
            ValueKind::BigInt => "bigint",
            ValueKind::Symbol => "symbol",
            ValueKind::String => "string",
            ValueKind::Function => "function",
            ValueKind::Dynamic => unreachable!(),
        };
        function.instruction(&Instruction::I64Const(self.strings.payload(value)));
    }

    pub(crate) fn emit_typeof_payload_from_tag_payload_local(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let typeof_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("object")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));

        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("boolean")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("number")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("bigint")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("symbol")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("string")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_htmldda_function_i32(tag_local, payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("function")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_proxy_target_is_callable_for_typeof_i32(tag_local, payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("function")));
        function.instruction(&Instruction::LocalSet(typeof_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typeof_payload_local));
        self.release_temp_local(typeof_payload_local);
        Ok(())
    }

    pub(crate) fn emit_proxy_target_is_callable_for_typeof_i32(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            current_payload_local,
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
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            current_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(result_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    pub(crate) fn emit_is_callable_i32(
        &mut self,
        tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_proxy_target_is_callable_for_typeof_i32(tag_local, payload_local, function)
    }

    pub(crate) fn compile_string_concat_payload(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        let lhs_string = self.reserve_temp_local();
        let rhs_string = self.reserve_temp_local();
        let lhs_offset = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_offset = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let total_len = self.reserve_temp_local();
        let dst_offset = self.reserve_temp_local();
        let rhs_dst_offset = self.reserve_temp_local();

        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.emit_value_to_string_payload(lhs_payload, lhs_tag, function)?;
        function.instruction(&Instruction::LocalSet(lhs_string));
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        self.emit_value_to_string_payload(rhs_payload, rhs_tag, function)?;
        function.instruction(&Instruction::LocalSet(rhs_string));

        self.emit_unpack_string_payload(lhs_string, lhs_offset, lhs_len, function);
        self.emit_unpack_string_payload(rhs_string, rhs_offset, rhs_len, function);

        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len));
        self.emit_heap_alloc_from_local(total_len, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset));

        self.emit_copy_bytes(lhs_offset, dst_offset, lhs_len, function);
        function.instruction(&Instruction::LocalGet(dst_offset));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_dst_offset));
        self.emit_copy_bytes(rhs_offset, rhs_dst_offset, rhs_len, function);
        self.emit_pack_string_payload(dst_offset, total_len, function);

        self.release_temp_local(rhs_dst_offset);
        self.release_temp_local(dst_offset);
        self.release_temp_local(total_len);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_offset);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_offset);
        self.release_temp_local(rhs_string);
        self.release_temp_local(lhs_string);
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    pub(crate) fn emit_function_to_string_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_payload_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            payload_local,
            HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
            source_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(source_payload_local));
        self.release_temp_local(source_payload_local);
        Ok(())
    }

    pub(crate) fn emit_value_to_string_payload(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Dynamic string concatenation and ToString sites hit this constantly
        // and the full per-kind composite is tens of KB inline; call the shared
        // helper instead (except while compiling the helper itself). The string
        // fast path stays inline so the common case never pays the call. A
        // ToPrimitive/Symbol throw inside the helper is surfaced through the
        // completion slots and re-raised here through the active catch target.
        if self.outline_value_to_string {
            if let Some(helper) = self.value_to_string_helper_function_index() {
                let result_payload_local = self.reserve_temp_local();
                let result_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.push_control(ControlFrameKind::If);
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalSet(result_payload_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalGet(tag_local));
                for _ in 0..5 {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::Call(helper));
                self.store_call_results(result_payload_local, result_tag_local, function);
                self.emit_propagate_throw_from_locals_if_needed(
                    result_payload_local,
                    result_tag_local,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(result_payload_local));
                self.release_temp_local(result_tag_local);
                self.release_temp_local(result_payload_local);
                return Ok(());
            }
        }
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(tag_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_bigint_value_to_string_payload(payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_function_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert a Symbol value to a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();
        self.emit_object_to_primitive_locals(
            ToPrimitiveHint::String,
            payload_local,
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        // A ToPrimitive throw here leaves completion=THROW with the error
        // (Object-tagged) in `primitive_payload_local`/`primitive_tag_local`
        // rather than branching (see
        // `emit_object_to_primitive_locals_locals_inner`). Feeding an
        // Object-tagged value into `emit_primitive_to_string_payload` hits its
        // unrecognized-tag `unreachable` trap, so skip that call on throw;
        // callers of this function are responsible for checking
        // `self.completion_local` and propagating.
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::Else);
        self.emit_primitive_to_string_payload(
            primitive_payload_local,
            primitive_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(primitive_tag_local);
        self.release_temp_local(primitive_payload_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        let array_string_local = self.reserve_temp_local();
        let array_tag_local = self.reserve_temp_local();
        self.emit_array_to_string_locals(
            payload_local,
            array_string_local,
            array_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(array_string_local));
        self.release_temp_local(array_tag_local);
        self.release_temp_local(array_string_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Arguments]"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_primitive_to_string_payload(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(tag_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_bigint_value_to_string_payload(payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert a Symbol value to a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_primitive_to_string_payload_to_local_without_throw_return(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_local));

        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        self.emit_is_bigint_tag_i32(tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_value_to_string_payload(payload_local, tag_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert a Symbol value to a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_bigint_value_to_string_payload(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let radix_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_heap_bigint_to_radix_string_payload(payload_local, radix_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_bigint_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(radix_local);
        Ok(())
    }

    pub(crate) fn emit_bigint_to_string_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let abs_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let digit_start_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(abs_local));
        self.emit_count_decimal_digits_u64(abs_local, digits_local, function);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));
        self.emit_heap_alloc_from_local(total_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(digit_start_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.store_ascii_byte_i64(dst_offset_local, b'-', function);
        self.emit_increment_local(digit_start_local, 1, function);
        function.instruction(&Instruction::End);
        self.emit_write_decimal_u64(abs_local, digit_start_local, digits_local, function);
        self.emit_pack_string_payload(dst_offset_local, total_len_local, function);

        self.release_temp_local(digit_start_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(digits_local);
        self.release_temp_local(abs_local);
        self.release_temp_local(sign_local);
        Ok(())
    }

    pub(crate) fn emit_bigint_to_string_with_radix_result(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let radix_payload_local = self.reserve_temp_local();
        let radix_tag_local = self.reserve_temp_local();
        let radix_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, radix_payload_local, radix_tag_local, function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::LocalGet(radix_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(radix_tag_local, radix_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(radix_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(36));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "BigInt.prototype.toString radix out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_bigint_to_radix_string_payload(payload_local, radix_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(radix_local);
        self.release_temp_local(radix_tag_local);
        self.release_temp_local(radix_payload_local);
        Ok(())
    }

    pub(crate) fn emit_bigint_to_radix_string_payload(
        &mut self,
        payload_local: u32,
        radix_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let abs_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let digit_start_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(abs_local));
        self.emit_count_radix_digits_u64(abs_local, radix_local, digits_local, function);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));
        self.emit_heap_alloc_from_local(total_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(digit_start_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.store_ascii_byte_i64(dst_offset_local, b'-', function);
        self.emit_increment_local(digit_start_local, 1, function);
        function.instruction(&Instruction::End);
        self.emit_write_radix_u64(
            abs_local,
            radix_local,
            digit_start_local,
            digits_local,
            function,
        );
        self.emit_pack_string_payload(dst_offset_local, total_len_local, function);

        self.release_temp_local(digit_start_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(digits_local);
        self.release_temp_local(abs_local);
        self.release_temp_local(sign_local);
        Ok(())
    }

    pub(crate) fn emit_heap_bigint_to_radix_string_payload(
        &mut self,
        payload_local: u32,
        radix_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        let working_limbs_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let limb_local = self.reserve_temp_local();
        let current_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let quotient_high_local = self.reserve_temp_local();
        let quotient_low_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let any_quotient_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();
        let output_start_local = self.reserve_temp_local();
        let output_end_local = self.reserve_temp_local();
        let write_local = self.reserve_temp_local();
        let output_len_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(capacity_local));
        self.emit_heap_alloc_from_local(capacity_local, function)?;
        function.instruction(&Instruction::LocalSet(working_limbs_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(limb_local));
        function.instruction(&Instruction::LocalGet(working_limbs_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(capacity_local));
        self.emit_heap_alloc_from_local(capacity_local, function)?;
        function.instruction(&Instruction::LocalSet(output_start_local));
        function.instruction(&Instruction::LocalGet(output_start_local));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_end_local));
        function.instruction(&Instruction::LocalGet(output_end_local));
        function.instruction(&Instruction::LocalSet(write_local));

        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(any_quotient_local));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(working_limbs_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(limb_local));

        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(quotient_high_local));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remainder_local));

        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(current_local));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(quotient_low_local));
        function.instruction(&Instruction::LocalGet(current_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remainder_local));

        function.instruction(&Instruction::LocalGet(quotient_high_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(quotient_low_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(working_limbs_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::LocalGet(any_quotient_local));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(any_quotient_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(write_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(write_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(any_quotient_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(write_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(b'-' as i32));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_end_local));
        function.instruction(&Instruction::LocalGet(write_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(output_len_local));
        self.emit_pack_string_payload(write_local, output_len_local, function);

        self.release_temp_local(output_len_local);
        self.release_temp_local(write_local);
        self.release_temp_local(output_end_local);
        self.release_temp_local(output_start_local);
        self.release_temp_local(capacity_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(any_quotient_local);
        self.release_temp_local(quotient_local);
        self.release_temp_local(quotient_low_local);
        self.release_temp_local(quotient_high_local);
        self.release_temp_local(remainder_local);
        self.release_temp_local(current_local);
        self.release_temp_local(limb_local);
        self.release_temp_local(index_local);
        self.release_temp_local(working_limbs_local);
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(sign_local);
        Ok(())
    }

    pub(crate) fn emit_heap_bigint_to_string_with_radix_result(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let radix_payload_local = self.reserve_temp_local();
        let radix_tag_local = self.reserve_temp_local();
        let radix_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, radix_payload_local, radix_tag_local, function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::LocalGet(radix_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(radix_tag_local, radix_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(radix_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(36));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "BigInt.prototype.toString radix out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_heap_bigint_to_radix_string_payload(payload_local, radix_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(radix_local);
        self.release_temp_local(radix_tag_local);
        self.release_temp_local(radix_payload_local);
        Ok(())
    }

    pub(crate) fn emit_number_to_string_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Number formatting appears in nearly every builtin body and its inline
        // expansion is several KB; call the shared helper instead (except while
        // compiling the helper itself). The helper returns the standard four-i64
        // tuple with the string payload in the first slot.
        if self.outline_number_to_string {
            if let Some(helper) = self.number_to_string_helper_function_index() {
                function.instruction(&Instruction::LocalGet(payload_local));
                for _ in 0..6 {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::Call(helper));
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::Drop);
                return Ok(());
            }
        }
        let output_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let abs_local = self.reserve_temp_local();
        let int_f_local = self.reserve_temp_local();
        let int_u_local = self.reserve_temp_local();
        let frac_scaled_local = self.reserve_temp_local();
        let frac_width_local = self.reserve_temp_local();
        let int_digits_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let int_start_local = self.reserve_temp_local();
        let frac_start_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("NaN")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(abs_local));
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-Infinity")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("Infinity")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1e-7)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-1e-7")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("1e-7")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1e-8)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-1e-8")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("1e-8")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1e20)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("-100000000000000000000"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            self.strings.payload("100000000000000000000"),
        ));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1e22)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-1e+22")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("1e+22")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(10203040506070809000.0)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("-10203040506070809000"),
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            self.strings.payload("10203040506070809000"),
        ));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1e19)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-1e+21")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("1e+21")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(int_f_local));
        function.instruction(&Instruction::LocalGet(int_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(int_u_local));
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(int_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Const(Ieee64::from(1_000_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Nearest);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(frac_scaled_local));
        function.instruction(&Instruction::LocalGet(frac_scaled_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(int_u_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(int_u_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(frac_scaled_local));
        function.instruction(&Instruction::End);
        self.emit_count_decimal_digits_u64(int_u_local, int_digits_local, function);
        self.emit_fraction_width_local(frac_scaled_local, frac_width_local, function);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(int_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(frac_width_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(frac_width_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));
        self.emit_heap_alloc_from_local(total_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(int_start_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(dst_offset_local, b'-', function);
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(int_start_local));
        function.instruction(&Instruction::End);
        self.emit_write_decimal_u64(int_u_local, int_start_local, int_digits_local, function);
        function.instruction(&Instruction::LocalGet(int_start_local));
        function.instruction(&Instruction::LocalGet(int_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frac_start_local));
        function.instruction(&Instruction::LocalGet(frac_width_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.store_ascii_byte_i64(frac_start_local, b'.', function);
        function.instruction(&Instruction::LocalGet(frac_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frac_start_local));
        self.emit_write_decimal_u64(
            frac_scaled_local,
            frac_start_local,
            frac_width_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(dst_offset_local, total_len_local, function);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_local));

        self.release_temp_local(frac_start_local);
        self.release_temp_local(int_start_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(int_digits_local);
        self.release_temp_local(frac_width_local);
        self.release_temp_local(frac_scaled_local);
        self.release_temp_local(int_u_local);
        self.release_temp_local(int_f_local);
        self.release_temp_local(abs_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(output_local);
        Ok(())
    }

    pub(crate) fn emit_number_to_string_with_radix_result(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let radix_payload_local = self.reserve_temp_local();
        let radix_tag_local = self.reserve_temp_local();
        let radix_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, radix_payload_local, radix_tag_local, function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::LocalGet(radix_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(radix_tag_local, radix_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(radix_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(radix_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(radix_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(36));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Number.prototype.toString radix out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        self.emit_number_to_radix_string_payload(payload_local, radix_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(radix_local);
        self.release_temp_local(radix_tag_local);
        self.release_temp_local(radix_payload_local);
        Ok(())
    }

    pub(crate) fn emit_number_to_radix_string_payload(
        &mut self,
        payload_local: u32,
        radix_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let output_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let abs_local = self.reserve_temp_local();
        let int_f_local = self.reserve_temp_local();
        let frac_f_local = self.reserve_temp_local();
        let int_u_local = self.reserve_temp_local();
        let fits_u64_local = self.reserve_temp_local();
        let int_digits_local = self.reserve_temp_local();
        let frac_digits_local = self.reserve_temp_local();
        let total_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let digit_start_local = self.reserve_temp_local();
        let frac_start_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("NaN")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(abs_local));
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-Infinity")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("Infinity")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(sign_local));

        // int_f = trunc(abs); frac_f = abs - int_f (in [0, 1)).
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(int_f_local));
        function.instruction(&Instruction::LocalGet(abs_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(int_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(frac_f_local));

        // `i64.trunc_f64_u` traps outside [0, 2^64); values at or beyond that
        // (e.g. 1e21) must not reach it, so branch to a bounded
        // floating-point digit walk instead of trapping.
        function.instruction(&Instruction::LocalGet(int_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(18446744073709551616.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(fits_u64_local));

        function.instruction(&Instruction::LocalGet(fits_u64_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(int_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(int_u_local));
        self.emit_count_radix_digits_u64(int_u_local, radix_local, int_digits_local, function);
        function.instruction(&Instruction::Else);
        self.emit_count_radix_digits_f64_bounded(
            int_f_local,
            radix_local,
            int_digits_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.emit_count_radix_fraction_digits(
            frac_f_local,
            radix_local,
            frac_digits_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(int_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(frac_digits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(frac_digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_len_local));
        self.emit_heap_alloc_from_local(total_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(digit_start_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(dst_offset_local, b'-', function);
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_start_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(fits_u64_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_write_radix_u64(
            int_u_local,
            radix_local,
            digit_start_local,
            int_digits_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_write_radix_f64_bounded(
            int_f_local,
            radix_local,
            digit_start_local,
            int_digits_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::LocalGet(int_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frac_start_local));
        function.instruction(&Instruction::LocalGet(frac_digits_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(frac_start_local, b'.', function);
        function.instruction(&Instruction::LocalGet(frac_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frac_start_local));
        self.emit_write_radix_fraction_digits(
            frac_f_local,
            radix_local,
            frac_start_local,
            frac_digits_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.emit_pack_string_payload(dst_offset_local, total_len_local, function);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_local));

        self.release_temp_local(frac_start_local);
        self.release_temp_local(digit_start_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(total_len_local);
        self.release_temp_local(frac_digits_local);
        self.release_temp_local(int_digits_local);
        self.release_temp_local(fits_u64_local);
        self.release_temp_local(int_u_local);
        self.release_temp_local(frac_f_local);
        self.release_temp_local(int_f_local);
        self.release_temp_local(abs_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(output_local);
        Ok(())
    }

    pub(crate) fn emit_number_to_fixed_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let output_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_payload_local));
        self.emit_return_current_completion_if_throw(function);
        for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
            function.instruction(&Instruction::LocalGet(arg_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                RANGE_ERROR_NAME,
                "Number.prototype.toFixed fraction digits out of range",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Number.prototype.toFixed fraction digits out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(1e21)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("0.0")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("0")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1000000000000000128.0)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("1000000000000000128"),
        ));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(output_local);
        self.release_temp_local(digits_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(crate) fn emit_number_to_exponential_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert a Symbol value to a number",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(crate) fn emit_number_to_precision_payload(
        &mut self,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let precision_local = self.reserve_temp_local();
        let output_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(payload_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_payload_local));
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("NaN")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("Infinity")));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("-Infinity")));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Number.prototype.toPrecision precision out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        for infinite in [f64::INFINITY, f64::NEG_INFINITY] {
            function.instruction(&Instruction::LocalGet(arg_payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(infinite)));
            function.instruction(&Instruction::F64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                RANGE_ERROR_NAME,
                "Number.prototype.toPrecision precision out of range",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(precision_local));
        function.instruction(&Instruction::LocalGet(precision_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(precision_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Number.prototype.toPrecision precision out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for (value, precision, text) in NUMBER_TO_PRECISION_CASES {
            self.emit_number_precision_case(
                payload_local,
                precision_local,
                *value,
                *precision,
                text,
                output_local,
                function,
            );
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(output_local);
        self.release_temp_local(precision_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(crate) fn emit_number_precision_case(
        &mut self,
        payload_local: u32,
        precision_local: u32,
        value: f64,
        precision: i64,
        text: &str,
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(value)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(precision_local));
        function.instruction(&Instruction::I64Const(precision));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(text)));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_fraction_width_local(
        &mut self,
        frac_scaled_local: u32,
        width_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let zeros_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(frac_scaled_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zeros_local));
        function.instruction(&Instruction::LocalGet(frac_scaled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(width_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(zeros_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(zeros_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::LocalSet(frac_scaled_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::LocalGet(zeros_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(width_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(zeros_local);
        self.release_temp_local(temp_local);
    }

    pub(crate) fn emit_count_decimal_digits_u64(
        &mut self,
        value_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(temp_local);
    }

    pub(crate) fn emit_count_radix_digits_u64(
        &mut self,
        value_local: u32,
        radix_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(temp_local);
    }

    pub(crate) fn emit_write_decimal_u64(
        &mut self,
        value_local: u32,
        start_offset_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let pos_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(start_offset_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(digit_local);
        self.release_temp_local(pos_local);
        self.release_temp_local(index_local);
        self.release_temp_local(temp_local);
    }

    pub(crate) fn emit_write_radix_u64(
        &mut self,
        value_local: u32,
        radix_local: u32,
        start_offset_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let pos_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let char_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(start_offset_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(char_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(pos_local);
        self.release_temp_local(index_local);
        self.release_temp_local(temp_local);
    }

    // Bounded, floating-point-based counterparts of `emit_count_radix_digits_u64`
    // / `emit_write_radix_u64` for integer magnitudes that do not fit in a u64
    // (so `i64.trunc_f64_u` would trap). Digit extraction stays in the f64
    // domain the whole time (never truncates the magnitude itself into an
    // integer type), so it can never trap; only the final small remainder
    // (always in `[0, radix)`) is ever truncated to i64. For power-of-two
    // radixes (2, 8, 16, 32) every division/multiplication by the radix is
    // exact in IEEE 754, so the produced digits are exact. For other radixes
    // at magnitudes this large, digits beyond the double's ~53 bits of
    // precision are not fully significant regardless of algorithm (the
    // double itself does not carry that information); the loop is bounded so
    // it always terminates instead of hanging or trapping.
    const MAX_RADIX_BIG_INTEGER_DIGITS: i64 = 1100;
    const MAX_RADIX_FRACTION_DIGITS: i64 = 1100;

    pub(crate) fn emit_count_radix_digits_f64_bounded(
        &mut self,
        value_local: u32,
        radix_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let radix_f_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(radix_f_local));

        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(radix_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(radix_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(Self::MAX_RADIX_BIG_INTEGER_DIGITS));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(radix_f_local);
        self.release_temp_local(temp_local);
    }

    pub(crate) fn emit_write_radix_f64_bounded(
        &mut self,
        value_local: u32,
        radix_local: u32,
        start_offset_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let radix_f_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let pos_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let char_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(radix_f_local));

        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(start_offset_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(pos_local));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(radix_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(quotient_local));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(radix_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Sub);
        // For non-power-of-two radixes at magnitudes far beyond exact double
        // precision (e.g. Number.MAX_VALUE), `quotient * radix_f` is itself
        // only accurate to ~1e-16 relative error, but `quotient` there can be
        // ~1e300+, so the *absolute* error can be enormous — nowhere near
        // "infinitesimally outside [0, radix_f)". `temp - quotient * radix_f`
        // can land far negative or far positive. `i64.trunc_f64_u` traps on
        // any input outside `[0, 2^64)`, so clamp both bounds defensively
        // before truncating: never let floating-point noise crash the VM
        // (the digit itself is already best-effort/inexact at this
        // magnitude for non-power-of-two radixes; clamping just guarantees
        // it stays a valid digit character instead of trapping).
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Max);
        function.instruction(&Instruction::LocalGet(radix_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Min);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(digit_local));

        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(char_local));
        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));

        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(char_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(quotient_local);
        self.release_temp_local(pos_local);
        self.release_temp_local(index_local);
        self.release_temp_local(radix_f_local);
        self.release_temp_local(temp_local);
    }

    // Counts/writes the fractional digits of the Number::toString(radix)
    // representation. `frac_local` holds the f64 bit pattern of a value in
    // `[0, 1)` (`abs - trunc(abs)`). Bounded the same way as the integer
    // helpers above so a fraction that never exactly terminates in the
    // requested radix (e.g. any non-power-of-two radix applied to a value
    // whose exact binary fraction doesn't share the radix's prime factors)
    // still produces a finite string instead of looping forever.
    pub(crate) fn emit_count_radix_fraction_digits(
        &mut self,
        frac_local: u32,
        radix_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let radix_f_local = self.reserve_temp_local();
        let digit_f_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(radix_f_local));

        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(digits_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Le);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(radix_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(temp_local));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(digit_f_local));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(digit_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(temp_local));

        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digits_local));

        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64Const(Self::MAX_RADIX_FRACTION_DIGITS));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(digit_f_local);
        self.release_temp_local(radix_f_local);
        self.release_temp_local(temp_local);
    }

    pub(crate) fn emit_write_radix_fraction_digits(
        &mut self,
        frac_local: u32,
        radix_local: u32,
        start_offset_local: u32,
        digits_local: u32,
        function: &mut Function,
    ) {
        let temp_local = self.reserve_temp_local();
        let radix_f_local = self.reserve_temp_local();
        let digit_f_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let char_local = self.reserve_temp_local();
        let pos_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(radix_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(radix_f_local));

        function.instruction(&Instruction::LocalGet(frac_local));
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(start_offset_local));
        function.instruction(&Instruction::LocalSet(pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(radix_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(temp_local));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(digit_f_local));

        function.instruction(&Instruction::LocalGet(digit_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(digit_local));

        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(char_local));

        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(char_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));

        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(digit_f_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(temp_local));

        function.instruction(&Instruction::LocalGet(pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pos_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
        self.release_temp_local(pos_local);
        self.release_temp_local(char_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(digit_f_local);
        self.release_temp_local(radix_f_local);
        self.release_temp_local(temp_local);
    }

    pub(crate) fn store_ascii_byte_i64(
        &self,
        offset_local: u32,
        byte: u8,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(i32::from(byte)));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    pub(crate) fn emit_string_search_argument_is_regexp_to_local(
        &mut self,
        search_payload_local: u32,
        search_tag_local: u32,
        search_is_regexp_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let search_match_payload_local = self.reserve_temp_local();
        let search_match_tag_local = self.reserve_temp_local();
        let search_match_key_local = self.reserve_temp_local();
        let search_prototype_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(search_is_regexp_local));
        self.emit_is_heap_object_like_tag_i32(search_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.match"),
        ));
        function.instruction(&Instruction::LocalSet(search_match_key_local));
        self.emit_object_read(
            search_payload_local,
            search_tag_local,
            search_payload_local,
            search_tag_local,
            search_match_key_local,
            search_match_payload_local,
            search_match_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(search_match_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.compile_truthy_tagged_i32(
            search_match_tag_local,
            search_match_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(search_is_regexp_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(search_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            search_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            search_prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(search_prototype_local));
        function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(search_is_regexp_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(search_prototype_local);
        self.release_temp_local(search_match_key_local);
        self.release_temp_local(search_match_tag_local);
        self.release_temp_local(search_match_payload_local);
        Ok(())
    }

    pub(crate) fn emit_unpack_string_payload(
        &self,
        payload_local: u32,
        offset_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(offset_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(len_local));
    }

    pub(crate) fn emit_pack_string_payload(
        &self,
        offset_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Or);
    }

    pub(crate) fn emit_string_slice_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        start_local: u32,
        len_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let src_start_offset_local = self.reserve_temp_local();
        let alloc_len_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_start_offset_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_heap_alloc_from_local(alloc_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        self.emit_copy_bytes(
            src_start_offset_local,
            dst_offset_local,
            len_local,
            function,
        );
        self.emit_pack_string_payload(dst_offset_local, len_local, function);

        self.release_temp_local(dst_offset_local);
        self.release_temp_local(alloc_len_local);
        self.release_temp_local(src_start_offset_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    /// Materialize a UTF-16 code-unit range from our UTF-8/WTF-8 string payload.
    ///
    /// An astral scalar occupies two ECMAScript code units.  Keeping both units
    /// preserves its canonical four-byte UTF-8 representation; selecting either
    /// half instead emits that surrogate as three-byte WTF-8.
    pub(crate) fn emit_utf16_code_unit_range_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        start_local: u32,
        len_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let unit_index_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let actual_len_local = self.reserve_temp_local();
        let alloc_len_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::Else);
        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        // A selected UTF-16 unit materializes to at most a three-byte WTF-8
        // sequence.  The allocator receives an aligned upper bound; the packed
        // payload below records the exact number of bytes emitted.
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(alloc_len_local));
        self.emit_heap_alloc_from_local(alloc_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(unit_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_load_string_byte(src_offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            byte_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_advance_local));

        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Astral scalar: independently select its high and low code units.
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(actual_len_local));
        self.emit_pack_string_payload(dst_offset_local, actual_len_local, function);
        function.instruction(&Instruction::End);

        self.release_temp_local(temp_local);
        self.release_temp_local(unit_advance_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(alloc_len_local);
        self.release_temp_local(actual_len_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(unit_index_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(end_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_utf16_code_unit_len_from_utf8_locals(
        &mut self,
        src_offset_local: u32,
        src_len_local: u32,
        dst_len_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dst_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(src_offset_local, index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(dst_len_local));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_len_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(temp_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
    }

    pub(crate) fn emit_utf16_code_unit_index_to_utf8_byte_offset_from_string_payload(
        &mut self,
        string_payload_local: u32,
        target_index_local: u32,
        dst_byte_offset_local: u32,
        function: &mut Function,
    ) {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let unit_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(dst_byte_offset_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::LocalSet(dst_byte_offset_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalSet(dst_byte_offset_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.emit_load_string_byte(src_offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            byte_index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_advance_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(target_index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_byte_offset_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(temp_local);
        self.release_temp_local(unit_advance_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(unit_index_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
    }

    pub(crate) fn emit_ecmascript_trim_payload_from_locals(
        &mut self,
        string_payload_local: u32,
        trim_start: bool,
        trim_end: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let end_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalSet(start_local));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end_local));

        if trim_start {
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(start_local));
            function.instruction(&Instruction::LocalGet(end_local));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(start_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(byte_local));
            for bytes in ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8 {
                Self::emit_skip_utf8_whitespace_forward(
                    function,
                    end_local,
                    start_local,
                    byte_local,
                    bytes,
                );
            }
            self.emit_is_ascii_whitespace_i32(byte_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(start_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(start_local));
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        if trim_end {
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(end_local));
            function.instruction(&Instruction::LocalGet(start_local));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(end_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(byte_local));
            for bytes in ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8 {
                Self::emit_skip_utf8_whitespace_backward(
                    function,
                    start_local,
                    end_local,
                    byte_local,
                    bytes,
                );
            }
            self.emit_is_ascii_whitespace_i32(byte_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::LocalSet(end_local));
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(start_local));
        self.emit_string_slice_payload_from_locals(
            string_payload_local,
            start_local,
            len_local,
            function,
        )?;

        self.release_temp_local(len_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(end_local);
        self.release_temp_local(start_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_copy_bytes(
        &mut self,
        src_offset_local: u32,
        dst_offset_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let src_addr_local = self.reserve_temp_local();
        let dst_addr_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(src_addr_local));
        function.instruction(&Instruction::LocalGet(src_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_addr_local));
        function.instruction(&Instruction::LocalGet(dst_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(byte_local);
        self.release_temp_local(dst_addr_local);
        self.release_temp_local(src_addr_local);
        self.release_temp_local(index_local);
    }

    pub(crate) fn emit_string_payload_equality_i32(
        &mut self,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            lhs_payload_local,
            rhs_payload_local,
            None,
            function,
        );
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_string_payload_equality_i32_with_ascii_case_folding(
        &mut self,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        ascii_case_folding_local: Option<u32>,
        function: &mut Function,
    ) {
        // Property-name matching and key switches hit this at thousands of
        // sites per builtin body; call the shared helper instead of inlining
        // the ~65-instruction byte-compare loop (except inside the helper
        // itself). The helper returns the standard four-i64 tuple with the
        // 0/1 result in the first slot.
        if self.outline_string_equality {
            if let Some(helper) = self.string_equality_helper_function_index() {
                function.instruction(&Instruction::LocalGet(lhs_payload_local));
                function.instruction(&Instruction::LocalGet(rhs_payload_local));
                if let Some(ascii_case_folding_local) = ascii_case_folding_local {
                    function.instruction(&Instruction::LocalGet(ascii_case_folding_local));
                } else {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::Call(helper));
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::I32WrapI64);
                return;
            }
        }
        let lhs_offset = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_offset = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let lhs_addr_local = self.reserve_temp_local();
        let rhs_addr_local = self.reserve_temp_local();
        let lhs_byte_local = self.reserve_temp_local();
        let rhs_byte_local = self.reserve_temp_local();
        let lhs_lower_local = ascii_case_folding_local.map(|_| self.reserve_temp_local());
        let rhs_lower_local = ascii_case_folding_local.map(|_| self.reserve_temp_local());
        let result_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(lhs_payload_local, lhs_offset, lhs_len, function);
        self.emit_unpack_string_payload(rhs_payload_local, rhs_offset, rhs_len, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(lhs_offset));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(lhs_addr_local));
        function.instruction(&Instruction::LocalGet(rhs_offset));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(rhs_addr_local));
        function.instruction(&Instruction::LocalGet(lhs_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(lhs_byte_local));
        function.instruction(&Instruction::LocalGet(rhs_addr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(rhs_byte_local));
        if let Some(ascii_case_folding_local) = ascii_case_folding_local {
            let lhs_lower_local = lhs_lower_local.expect("ASCII folding reserves lhs local");
            let rhs_lower_local = rhs_lower_local.expect("ASCII folding reserves rhs local");
            function.instruction(&Instruction::LocalGet(lhs_byte_local));
            function.instruction(&Instruction::LocalGet(rhs_byte_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(ascii_case_folding_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            self.emit_ascii_lowercase_byte_to_local(lhs_byte_local, lhs_lower_local, function);
            self.emit_ascii_lowercase_byte_to_local(rhs_byte_local, rhs_lower_local, function);
            function.instruction(&Instruction::LocalGet(lhs_lower_local));
            function.instruction(&Instruction::LocalGet(rhs_lower_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(ascii_case_folding_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I32Eqz);
        } else {
            function.instruction(&Instruction::LocalGet(lhs_byte_local));
            function.instruction(&Instruction::LocalGet(rhs_byte_local));
            function.instruction(&Instruction::I64Ne);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(result_local);
        if let Some(rhs_lower_local) = rhs_lower_local {
            self.release_temp_local(rhs_lower_local);
        }
        if let Some(lhs_lower_local) = lhs_lower_local {
            self.release_temp_local(lhs_lower_local);
        }
        self.release_temp_local(rhs_byte_local);
        self.release_temp_local(lhs_byte_local);
        self.release_temp_local(rhs_addr_local);
        self.release_temp_local(lhs_addr_local);
        self.release_temp_local(index_local);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_offset);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_offset);
    }

    pub(crate) fn compile_strict_equality_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_tag_dynamic = expr_result_tag_is_runtime_dynamic(&lhs.expr);
        let rhs_tag_dynamic = expr_result_tag_is_runtime_dynamic(&rhs.expr);
        if !lhs_tag_dynamic
            && !rhs_tag_dynamic
            && lhs.possible_kinds.is_singleton()
            && rhs.possible_kinds.is_singleton()
            && lhs.kind != rhs.kind
        {
            function.instruction(&Instruction::I32Const(0));
            return Ok(());
        }

        if !lhs_tag_dynamic
            && !rhs_tag_dynamic
            && lhs.possible_kinds.is_singleton()
            && rhs.possible_kinds.is_singleton()
        {
            match lhs.kind {
                ValueKind::Number => {
                    self.compile_expr_payload(lhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Eq);
                }
                ValueKind::String => {
                    // Keep the left payload on the Wasm stack across the right
                    // operand's emission, exactly like the `Number` arm above.
                    // Spilling it to `scratch_local` first is unsound: right
                    // operands that themselves lower through `scratch_local`
                    // (`typeof x`, property reads, ...) overwrite the spill, so
                    // `"object" === typeof x` used to compare a value against
                    // itself-shifted garbage and report `false`.
                    self.compile_expr_payload(lhs, function)?;
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.emit_string_payload_equality_i32(
                        self.scratch_local,
                        self.result_local,
                        function,
                    );
                }
                ValueKind::Function | ValueKind::BigInt => {
                    let lhs_payload = self.reserve_temp_local();
                    let lhs_tag = self.reserve_temp_local();
                    let rhs_payload = self.reserve_temp_local();
                    let rhs_tag = self.reserve_temp_local();
                    self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
                    self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
                    self.emit_tagged_payload_equality_i32(
                        lhs_tag,
                        lhs_payload,
                        rhs_tag,
                        rhs_payload,
                        function,
                    )?;
                    self.release_temp_local(rhs_tag);
                    self.release_temp_local(rhs_payload);
                    self.release_temp_local(lhs_tag);
                    self.release_temp_local(lhs_payload);
                }
                _ => {
                    self.compile_expr_payload(lhs, function)?;
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::I64Eq);
                }
            }
            self.set_completion_kind(CompletionKind::Normal, function);
            return Ok(());
        }

        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        self.emit_tagged_payload_equality_i32(
            lhs_tag,
            lhs_payload,
            rhs_tag,
            rhs_payload,
            function,
        )?;
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        self.set_completion_kind(CompletionKind::Normal, function);
        Ok(())
    }

    pub(crate) fn emit_assert_same_value(
        &mut self,
        actual: &TypedExpr,
        expected: &TypedExpr,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_same_value_i32(actual, expected, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            ERROR_NAME,
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn compile_same_value_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_tag_dynamic = expr_result_tag_is_runtime_dynamic(&lhs.expr);
        let rhs_tag_dynamic = expr_result_tag_is_runtime_dynamic(&rhs.expr);
        if !lhs_tag_dynamic
            && !rhs_tag_dynamic
            && lhs.possible_kinds.is_singleton()
            && rhs.possible_kinds.is_singleton()
            && lhs.kind != rhs.kind
        {
            function.instruction(&Instruction::I32Const(0));
            return Ok(());
        }

        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        self.emit_tagged_payload_same_value_i32(
            lhs_tag,
            lhs_payload,
            rhs_tag,
            rhs_payload,
            function,
        )?;
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    pub(crate) fn compile_same_value_zero_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_tag_dynamic = expr_result_tag_is_runtime_dynamic(&lhs.expr);
        let rhs_tag_dynamic = expr_result_tag_is_runtime_dynamic(&rhs.expr);
        if !lhs_tag_dynamic
            && !rhs_tag_dynamic
            && lhs.possible_kinds.is_singleton()
            && rhs.possible_kinds.is_singleton()
            && lhs.kind != rhs.kind
        {
            function.instruction(&Instruction::I32Const(0));
            return Ok(());
        }

        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        self.emit_tagged_payload_same_value_zero_i32(
            lhs_tag,
            lhs_payload,
            rhs_tag,
            rhs_payload,
            function,
        )?;
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    pub(crate) fn emit_tagged_payload_same_value_i32(
        &mut self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_heap_bigint_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_tagged_payload_same_value_zero_i32(
        &mut self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_heap_bigint_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_tagged_payload_equality_i32(
        &mut self,
        lhs_tag_local: u32,
        lhs_payload_local: u32,
        rhs_tag_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_heap_bigint_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_equality_i32(lhs_payload_local, rhs_payload_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_heap_bigint_equality_i32(
        &mut self,
        lhs_payload_local: u32,
        rhs_payload_local: u32,
        function: &mut Function,
    ) {
        let lhs_sign_local = self.reserve_temp_local();
        let rhs_sign_local = self.reserve_temp_local();
        let lhs_limbs_local = self.reserve_temp_local();
        let rhs_limbs_local = self.reserve_temp_local();
        let lhs_len_local = self.reserve_temp_local();
        let rhs_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let equal_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            lhs_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            lhs_sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            rhs_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            rhs_sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            lhs_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            lhs_limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            rhs_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            rhs_limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            lhs_payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            lhs_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            rhs_payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            rhs_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(lhs_sign_local));
        function.instruction(&Instruction::LocalGet(rhs_sign_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(lhs_len_local));
        function.instruction(&Instruction::LocalGet(rhs_len_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(equal_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(lhs_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(lhs_limbs_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalGet(rhs_limbs_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(equal_local));
        function.instruction(&Instruction::LocalGet(equal_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(equal_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);

        self.release_temp_local(equal_local);
        self.release_temp_local(index_local);
        self.release_temp_local(rhs_len_local);
        self.release_temp_local(lhs_len_local);
        self.release_temp_local(rhs_limbs_local);
        self.release_temp_local(lhs_limbs_local);
        self.release_temp_local(rhs_sign_local);
        self.release_temp_local(lhs_sign_local);
    }
}
