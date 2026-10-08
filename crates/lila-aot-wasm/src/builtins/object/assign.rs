//! Object.assign observes each source snapshot and publishes writes in order.
use super::*;

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn compile_object_assign_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let target = schema.reserve_value_local(function);
        let source = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        let pending = schema.reserve_completion(function);
        let source_index = schema.reserve_i32_local(function);
        let source_count = schema.reserve_i32_local(function);
        let key_index = schema.reserve_i32_local(function);
        let key_count = schema.reserve_i32_local(function);
        output.initialize(function);
        pending.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);

        self.emit_builtin_arg_to_value(0, &argument, function);
        self.compile_nullish_tagged_i32(argument.tag(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::OBJECT_ASSIGN_CALLED_ON_NULL_OR_UNDEFINED,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(&argument, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        target.copy_from(pending.value(), function);

        schema.array_type::<crate::gc_types::ValueArray>().length(
            self.body_entry_locals()
                .expect("Object.assign owns a native entry")
                .arguments(),
            schema,
            function,
        );
        source_count.store(function);
        function.instruction(&Instruction::I32Const(1));
        source_index.store(function);
        let sources_done = self.open_frame(ControlFrameKind::Block, function);
        let next_source = self.open_frame(ControlFrameKind::Loop, function);
        source_index.load(function);
        source_count.load(function);
        function.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(sources_done, function);
        self.emit_argument_vector_entry_to_value(
            self.body_entry_locals()
                .expect("Object.assign owns a native entry")
                .arguments(),
            source_index,
            &argument,
            function,
        );
        self.compile_nullish_tagged_i32(argument.tag(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_value_to_current_function_realm_object_locals(&argument, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        source.copy_from(pending.value(), function);
        let keys = self.emit_object_own_property_keys(&source, function)?;
        keys.length(key_count, schema, function);
        function.instruction(&Instruction::I32Const(0));
        key_index.store(function);
        let keys_done = self.open_frame(ControlFrameKind::Block, function);
        let next_key = self.open_frame(ControlFrameKind::Loop, function);
        key_index.load(function);
        key_count.load(function);
        function.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(keys_done, function);
        let key = keys.read_key(key_index, self, function)?;
        let descriptor = self.emit_proxy_target_own_descriptor(&source, &key, function)?;
        descriptor.emit_enumerable_i32(schema, function);
        self.open_frame(ControlFrameKind::If, function);
        // Both String and Symbol keys participate. Get uses the boxed source
        // as receiver; Set uses the boxed target and Throw=true.
        self.emit_object_read(&source, &source, &key, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        value.copy_from(pending.value(), function);
        self.emit_ordinary_set_result(&target, &target, &key, &value, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        self.compile_truthy_tagged_i32(pending.value(), function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::CANNOT_ASSIGN_TO_READ_ONLY_PROPERTY,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        descriptor.clear(function);
        key.clear(function);
        key_index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        key_index.store(function);
        self.emit_branch_to_target(next_key, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        keys.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        source_index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        source_index.store(function);
        self.emit_branch_to_target(next_source, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        output.set_normal(&target, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        for local in [key_count, key_index, source_count, source_index] {
            schema.release_i32_local(local, function);
        }
        pending.clear(function);
        output.clear(function);
        value.clear(function);
        source.clear(function);
        target.clear(function);
        argument.clear(function);
        Ok(())
    }
}
