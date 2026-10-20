//! Annex B accessor lookup stops at the first present own descriptor.
use super::*;

#[derive(Clone, Copy)]
enum PrototypeLookup {
    Getter,
    Setter,
}

impl FunctionBuilder<'_> {
    fn compile_object_prototype_lookup_builtin(
        &mut self,
        mode: PrototypeLookup,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let object = schema.reserve_value_local(function);
        let key_value = schema.reserve_value_local(function);
        let selected = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        let pending = schema.reserve_completion(function);
        let found = schema.reserve_i32_local(function);
        output.initialize(function);
        pending.initialize(function);
        receiver.copy_from(
            self.body_entry_locals()
                .expect("accessor lookup owns a native entry")
                .this_value(),
            function,
        );
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_value_to_current_function_realm_object_locals(&receiver, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        object.copy_from(pending.value(), function);
        self.emit_builtin_arg_to_value(0, &key_value, function);
        let key = self.emit_value_to_property_key_locals(&key_value, function)?;
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        let descriptor = self.emit_proxy_target_own_descriptor(&object, &key, function)?;
        descriptor.emit_found_i32(schema, function);
        found.store(function);
        found.load(function);
        self.open_frame(ControlFrameKind::If, function);
        selected.set_undefined(function);
        descriptor.emit_accessor_i32(schema, function);
        self.open_frame(ControlFrameKind::If, function);
        match mode {
            PrototypeLookup::Getter => descriptor.read_getter(&selected, schema, function),
            PrototypeLookup::Setter => descriptor.read_setter(&selected, schema, function),
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        output.set_normal(&selected, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        descriptor.clear(function);
        found.load(function);
        self.emit_branch_if_to_target(done, function);
        self.emit_object_get_prototype_of(&object, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        pending.value().tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null as i32));
        function.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(done, function);
        object.copy_from(pending.value(), function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        key.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        schema.release_i32_local(found, function);
        pending.clear(function);
        output.clear(function);
        selected.clear(function);
        key_value.clear(function);
        object.clear(function);
        receiver.clear(function);
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_prototype_lookup_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_prototype_lookup_builtin(PrototypeLookup::Getter, function)
    }
    pub(in crate::builtins) fn compile_object_prototype_lookup_setter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_prototype_lookup_builtin(PrototypeLookup::Setter, function)
    }
}
