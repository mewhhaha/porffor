//! Annex B accessor definition retains ToObject, callable, then key order.
use super::*;

#[derive(Clone, Copy)]
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
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let object = schema.reserve_value_local(function);
        let key_value = schema.reserve_value_local(function);
        let callable = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        let pending = schema.reserve_completion(function);
        output.initialize(function);
        pending.initialize(function);
        receiver.copy_from(
            self.body_entry_locals()
                .expect("accessor definition owns a native entry")
                .this_value(),
            function,
        );
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_value_to_current_function_realm_object_locals(&receiver, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        object.copy_from(pending.value(), function);
        self.emit_builtin_arg_to_value(1, &callable, function);
        self.emit_is_callable_i32(&callable, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ACCESSOR_MUST_BE_CALLABLE,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_builtin_arg_to_value(0, &key_value, function);
        let key = self.emit_value_to_property_key_locals(&key_value, function)?;
        // The opposite accessor remains absent, preserving an existing value.
        // Callable validation above authenticates the sole present accessor.
        let (get, set) = match accessor {
            AccessorDefinition::Getter => (Presence::Present(&callable), Presence::Absent),
            AccessorDefinition::Setter => (Presence::Absent, Presence::Present(&callable)),
        };
        let descriptor = DescriptorObjectFields {
            get,
            set,
            enumerable: Presence::Present(DescriptorFlag::Known(true)),
            configurable: Presence::Present(DescriptorFlag::Known(true)),
            ..DescriptorObjectFields::empty()
        }
        .from_runtime_checked();
        self.emit_object_define_entry_validated(&object, &key, &descriptor, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        self.compile_truthy_tagged_i32(pending.value(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::TYPEERROR,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        key.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        pending.clear(function);
        output.clear(function);
        callable.clear(function);
        key_value.clear(function);
        object.clear(function);
        receiver.clear(function);
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
