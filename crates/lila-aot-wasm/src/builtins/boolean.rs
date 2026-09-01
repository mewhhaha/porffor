use super::super::*;

enum BooleanBuiltin {
    Constructor,
    PrototypeToString,
    PrototypeValueOf,
}

enum BooleanPrototypeOperation {
    ToString,
    ValueOf,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_boolean_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_boolean_builtin(BooleanBuiltin::Constructor, function)
    }

    pub(super) fn emit_boolean_prototype_to_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_boolean_builtin(BooleanBuiltin::PrototypeToString, function)
    }

    pub(super) fn emit_boolean_prototype_value_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_boolean_builtin(BooleanBuiltin::PrototypeValueOf, function)
    }

    fn emit_boolean_builtin(
        &mut self,
        builtin: BooleanBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            BooleanBuiltin::Constructor => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let primitive_payload_local = self.reserve_temp_local();
                let primitive_tag_local = self.reserve_temp_local();
                let has_arg_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::LocalGet(self.argc_param_local()));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(has_arg_local));
                function.instruction(&Instruction::LocalGet(has_arg_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(primitive_payload_local));
                function.instruction(&Instruction::Else);
                self.emit_to_boolean_payload_from_tagged_locals(
                    arg_tag_local,
                    arg_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(primitive_payload_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(primitive_tag_local));
                function.instruction(&Instruction::LocalGet(primitive_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(primitive_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(has_arg_local);
                self.release_temp_local(primitive_tag_local);
                self.release_temp_local(primitive_payload_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            BooleanBuiltin::PrototypeToString => {
                self.emit_boolean_prototype_builtin(BooleanPrototypeOperation::ToString, function)?
            }
            BooleanBuiltin::PrototypeValueOf => {
                self.emit_boolean_prototype_builtin(BooleanPrototypeOperation::ValueOf, function)?
            }
        }
        Ok(())
    }

    fn emit_boolean_prototype_builtin(
        &mut self,
        operation: BooleanPrototypeOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Boolean prototype receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Boolean prototype receiver",
            )
        })?;
        let boxed_kind_local = self.reserve_temp_local();
        let boolean_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(boolean_payload_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_BOOLEAN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boolean_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Boolean.prototype method requires a Boolean receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Boolean.prototype method requires a Boolean receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        match operation {
            BooleanPrototypeOperation::ValueOf => {
                function.instruction(&Instruction::LocalGet(boolean_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            BooleanPrototypeOperation::ToString => {
                function.instruction(&Instruction::LocalGet(boolean_payload_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(self.strings.payload("false")));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(self.strings.payload("true")));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }

        self.release_temp_local(boolean_payload_local);
        self.release_temp_local(boxed_kind_local);
        Ok(())
    }
}
