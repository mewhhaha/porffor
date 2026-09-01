use super::*;

enum OwnDescriptorPredicateBuiltin {
    ObjectHasOwn,
    PrototypeHasOwnProperty,
    PrototypePropertyIsEnumerable,
}

impl<'a> FunctionBuilder<'a> {
    fn compile_object_own_descriptor_predicate_builtin(
        &mut self,
        builtin: OwnDescriptorPredicateBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();

        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Object.getOwnPropertyDescriptor builtin")
            })?;

        match &builtin {
            OwnDescriptorPredicateBuiltin::ObjectHasOwn => {
                self.emit_builtin_arg_to_locals(
                    0,
                    receiver_payload_local,
                    receiver_tag_local,
                    function,
                );
                self.emit_builtin_arg_to_locals(1, key_payload_local, key_tag_local, function);
            }
            OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty
            | OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable => {
                let this_payload_local = self.this_payload_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "missing Object.prototype own-descriptor predicate receiver",
                    )
                })?;
                let this_tag_local = self.this_tag_local.ok_or_else(|| {
                    EmitError::unsupported(
                        "missing Object.prototype own-descriptor predicate receiver",
                    )
                })?;
                function.instruction(&Instruction::LocalGet(this_payload_local));
                function.instruction(&Instruction::LocalSet(receiver_payload_local));
                function.instruction(&Instruction::LocalGet(this_tag_local));
                function.instruction(&Instruction::LocalSet(receiver_tag_local));
                self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);
            }
        }

        match &builtin {
            OwnDescriptorPredicateBuiltin::ObjectHasOwn => {
                self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Object.hasOwn called on null or undefined",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_value_to_current_function_realm_object_locals(
                    receiver_payload_local,
                    receiver_tag_local,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
                self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
            }
            OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty => {
                self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
                self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Object.prototype.hasOwnProperty called on null or undefined",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_value_to_current_function_realm_object_locals(
                    receiver_payload_local,
                    receiver_tag_local,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
            }
            OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable => {
                self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;
                self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Object.prototype.propertyIsEnumerable called on null or undefined",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_value_to_current_function_realm_object_locals(
                    receiver_payload_local,
                    receiver_tag_local,
                    object_payload_local,
                    object_tag_local,
                    function,
                )?;
            }
        }

        self.emit_direct_js_call(
            &get_own_descriptor_meta,
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
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        match &builtin {
            OwnDescriptorPredicateBuiltin::ObjectHasOwn
            | OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty => {
                function.instruction(&Instruction::LocalGet(descriptor_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
            }
            OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable => {
                let enumerable_key_local = self.reserve_temp_local();
                let enumerable_present_local = self.reserve_temp_local();
                let enumerable_payload_local = self.reserve_temp_local();
                let enumerable_tag_local = self.reserve_temp_local();

                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(descriptor_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
                function.instruction(&Instruction::LocalSet(enumerable_key_local));
                self.emit_object_own_data_field_read(
                    descriptor_payload_local,
                    descriptor_tag_local,
                    enumerable_key_local,
                    enumerable_present_local,
                    enumerable_payload_local,
                    enumerable_tag_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(enumerable_present_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(enumerable_payload_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);

                self.release_temp_local(enumerable_tag_local);
                self.release_temp_local(enumerable_payload_local);
                self.release_temp_local(enumerable_present_local);
                self.release_temp_local(enumerable_key_local);
            }
        }

        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_has_own_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_own_descriptor_predicate_builtin(
            OwnDescriptorPredicateBuiltin::ObjectHasOwn,
            function,
        )
    }

    pub(in crate::builtins) fn compile_object_prototype_has_own_property_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_own_descriptor_predicate_builtin(
            OwnDescriptorPredicateBuiltin::PrototypeHasOwnProperty,
            function,
        )
    }

    pub(in crate::builtins) fn compile_object_prototype_property_is_enumerable_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_own_descriptor_predicate_builtin(
            OwnDescriptorPredicateBuiltin::PrototypePropertyIsEnumerable,
            function,
        )
    }
}
