use super::*;

enum AccessorDefinition {
    Getter,
    Setter,
}

impl FunctionBuilder<'_> {
    fn compile_object_prototype_define_accessor_builtin(
        &mut self,
        accessor: AccessorDefinition,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported("missing Object.prototype accessor definition receiver")
        })?;
        let receiver_tag = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("missing Object.prototype accessor definition receiver tag")
        })?;
        let object = self.reserve_temp_local();
        let object_tag = self.reserve_temp_local();
        let key = self.reserve_temp_local();
        let key_tag = self.reserve_temp_local();
        let callable = self.reserve_temp_local();
        let callable_tag = self.reserve_temp_local();
        let descriptor = self.reserve_temp_local();
        let descriptor_tag = self.reserve_temp_local();
        let attribute_key = self.reserve_temp_local();
        let attribute_value = self.reserve_temp_local();
        let attribute_tag = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, key, key_tag, function);
        self.emit_builtin_arg_to_locals(1, callable, callable_tag, function);
        self.emit_value_to_current_function_realm_object_locals(
            receiver,
            receiver_tag,
            object,
            object_tag,
            function,
        )?;
        self.emit_is_callable_i32(callable_tag, callable, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Accessor must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_property_key_locals(key, key_tag, function)?;
        // The descriptor builtin accepts a JS String or Symbol, not the
        // encoded PropertyKey that the conversion seam returns.
        self.emit_property_key_value_payload_to_local(key, key, function);

        // A partial descriptor must omit the opposite accessor. Its private
        // null prototype also keeps Object.prototype changes out of the
        // descriptor conversion performed by the canonical definition path.
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(descriptor));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag));
        let accessor_name = match accessor {
            AccessorDefinition::Getter => "get",
            AccessorDefinition::Setter => "set",
        };
        function.instruction(&Instruction::I64Const(self.strings.payload(accessor_name)));
        function.instruction(&Instruction::LocalSet(attribute_key));
        self.emit_object_define_enumerable_data(
            descriptor,
            attribute_key,
            callable,
            callable_tag,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(attribute_value));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(attribute_tag));
        for name in ["enumerable", "configurable"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(attribute_key));
            self.emit_object_define_enumerable_data(
                descriptor,
                attribute_key,
                attribute_value,
                attribute_tag,
                function,
            )?;
        }
        let define_property = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .expect("accessor definers root the canonical property-definition builtin");
        self.emit_direct_js_call(
            &define_property,
            None,
            &[
                (object, object_tag),
                (key, key_tag),
                (descriptor, descriptor_tag),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            attribute_tag,
            attribute_value,
            attribute_key,
            descriptor_tag,
            descriptor,
            callable_tag,
            callable,
            key_tag,
            key,
            object_tag,
            object,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_prototype_define_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_prototype_define_accessor_builtin(AccessorDefinition::Getter, function)
    }

    pub(in crate::builtins) fn compile_object_prototype_define_setter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_prototype_define_accessor_builtin(AccessorDefinition::Setter, function)
    }
}
