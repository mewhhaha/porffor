use super::*;
use crate::environments::environment_reference::EnvironmentIdentifierRead;
use lila_ir::{
    EnvironmentCompoundOperationIr, EnvironmentIdentifierIr, EnvironmentIdentifierOperationIr,
};

impl FunctionBuilder<'_> {
    fn environment_operand(&mut self, name: &str, payload: u32, tag: u32) -> TypedExpr {
        self.binding_scopes
            .last_mut()
            .expect("operand scope exists")
            .insert(
                name.to_string(),
                BindingStorage::Dynamic {
                    payload_local: payload,
                    tag_local: tag,
                },
            );
        TypedExpr {
            kind: ValueKind::Dynamic,
            possible_kinds: KindSet::all_runtime_tags(),
            heap_shape: None,
            function_targets: lila_ir::FunctionTargetKnowledge::unknown(),
            expr: ExprIr::Identifier(name.to_string()),
        }
    }

    pub(crate) fn compile_environment_identifier_to_locals(
        &mut self,
        identifier: &EnvironmentIdentifierIr,
        output: u32,
        output_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let key_expression = TypedExpr {
            kind: ValueKind::String,
            possible_kinds: KindSet::from_kind(ValueKind::String),
            heap_shape: None,
            function_targets: lila_ir::FunctionTargetKnowledge::none(),
            expr: ExprIr::String(identifier.name.clone()),
        };
        self.compile_expr_to_locals(&key_expression, key, tag, function)?;
        let reference =
            self.emit_resolve_environment_identifier(key, identifier.strictness, function)?;
        use EnvironmentIdentifierOperationIr as Operation;
        match &identifier.operation {
            Operation::Assign { value: rhs } => {
                self.compile_expr_to_locals(rhs, value, tag, function)?;
                self.emit_environment_identifier_put(&reference, value, tag, function)?;
            }
            Operation::Delete => {
                self.emit_environment_identifier_delete(&reference, value, function)?;
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag));
            }
            operation => {
                let read = if matches!(operation, Operation::Typeof) {
                    EnvironmentIdentifierRead::Typeof
                } else {
                    EnvironmentIdentifierRead::Value
                };
                self.emit_environment_identifier_get(&reference, read, value, tag, function)?;
                match operation {
                    Operation::Read => {}
                    Operation::Typeof => {
                        self.emit_typeof_payload_from_tag_payload_local(tag, value, function)?;
                        function.instruction(&Instruction::LocalSet(value));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag));
                    }
                    Operation::Update {
                        operation,
                        return_mode,
                    } => {
                        self.emit_value_to_numeric_locals(value, tag, function)?;
                        let old = self.reserve_temp_local();
                        function.instruction(&Instruction::LocalGet(value));
                        function.instruction(&Instruction::LocalSet(old));
                        self.emit_update_delta_from_locals(
                            *operation,
                            NumericUpdateValueKind::Dynamic,
                            value,
                            tag,
                            function,
                        );
                        function.instruction(&Instruction::LocalSet(value));
                        self.emit_environment_identifier_put(&reference, value, tag, function)?;
                        if *return_mode == UpdateReturnMode::Postfix {
                            function.instruction(&Instruction::LocalGet(old));
                            function.instruction(&Instruction::LocalSet(value));
                        }
                        self.release_temp_local(old);
                    }
                    Operation::EagerCompound { operation, rhs } => {
                        self.push_scope();
                        let lhs = self.environment_operand("\0environment.old", value, tag);
                        let result = match operation {
                            EnvironmentCompoundOperationIr::Add => {
                                self.compile_coercive_add_to_locals(&lhs, rhs, value, tag, function)
                            }
                            EnvironmentCompoundOperationIr::Arithmetic(op) => self
                                .compile_coercive_binary_number_to_locals(
                                    *op, &lhs, rhs, value, tag, function,
                                ),
                            EnvironmentCompoundOperationIr::Bitwise(op) => self
                                .compile_bitwise_numeric_to_locals(
                                    *op, &lhs, rhs, value, tag, function,
                                ),
                        };
                        self.pop_scope();
                        result?;
                        self.emit_environment_identifier_put(&reference, value, tag, function)?;
                    }
                    Operation::LogicalCompound { operation, rhs } => {
                        match operation {
                            LogicalBinaryOp::And | LogicalBinaryOp::Or => {
                                self.compile_truthy_tagged_i32(tag, value, function)?;
                                if *operation == LogicalBinaryOp::Or {
                                    function.instruction(&Instruction::I32Eqz);
                                }
                            }
                            LogicalBinaryOp::Coalesce => {
                                function.instruction(&Instruction::LocalGet(tag));
                                function.instruction(&Instruction::I64Const(
                                    ValueKind::Null.tag() as i64
                                ));
                                function.instruction(&Instruction::I64Eq);
                                function.instruction(&Instruction::LocalGet(tag));
                                function.instruction(&Instruction::I64Const(
                                    ValueKind::Undefined.tag() as i64,
                                ));
                                function.instruction(&Instruction::I64Eq);
                                function.instruction(&Instruction::I32Or);
                            }
                        }
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.compile_expr_to_locals(rhs, value, tag, function)?;
                        self.emit_environment_identifier_put(&reference, value, tag, function)?;
                        function.instruction(&Instruction::End);
                    }
                    Operation::Call { args, direct_eval } => {
                        let receiver = self.reserve_temp_local();
                        let receiver_tag = self.reserve_temp_local();
                        self.emit_environment_identifier_call_base(
                            &reference,
                            receiver,
                            receiver_tag,
                            function,
                        );
                        self.push_scope();
                        let callee = self.environment_operand("\0environment.callee", value, tag);
                        let this = self.environment_operand(
                            "\0environment.receiver",
                            receiver,
                            receiver_tag,
                        );
                        let result = self.emit_indirect_call(
                            &callee,
                            Some(&this),
                            args,
                            None,
                            direct_eval.as_ref(),
                            value,
                            tag,
                            function,
                        );
                        self.pop_scope();
                        result?;
                        self.release_temp_local(receiver_tag);
                        self.release_temp_local(receiver);
                    }
                    Operation::Assign { .. } | Operation::Delete => {
                        unreachable!("write-only operations handled before GetValue")
                    }
                }
            }
        }
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::LocalSet(output));
        function.instruction(&Instruction::LocalGet(tag));
        function.instruction(&Instruction::LocalSet(output_tag));
        self.release_environment_identifier_reference(reference);
        self.release_temp_local(tag);
        self.release_temp_local(value);
        self.release_temp_local(key);
        Ok(())
    }
}
