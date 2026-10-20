//! TestIntegrityLevel observes extensibility before any property snapshot.
use super::*;

#[derive(Clone, Copy)]
enum IntegrityTest {
    Sealed,
    Frozen,
}

impl FunctionBuilder<'_> {
    fn compile_object_integrity_test_builtin(
        &mut self,
        mode: IntegrityTest,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        let pending = schema.reserve_completion(function);
        let accepted = schema.reserve_i32_local(function);
        let count = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let reject = schema.reserve_i32_local(function);
        output.initialize(function);
        pending.initialize(function);
        function.instruction(&Instruction::I32Const(1));
        accepted.store(function);
        self.emit_builtin_arg_to_value(0, &target, function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_is_heap_object_like_tag_i32(target.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_object_is_extensible(&target, &pending, function)?;
        self.emit_native_object_abrupt_exit(&pending, &output, exit, function);
        self.compile_truthy_tagged_i32(pending.value(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0));
        accepted.store(function);
        function.instruction(&Instruction::Else);
        let keys = self.emit_object_own_property_keys(&target, function)?;
        keys.length(count, schema, function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(done, function);
        let key = keys.read_key(index, self, function)?;
        let descriptor = self.emit_proxy_target_own_descriptor(&target, &key, function)?;
        descriptor.emit_configurable_i32(schema, function);
        match mode {
            IntegrityTest::Sealed => {}
            IntegrityTest::Frozen => {
                descriptor.emit_accessor_i32(schema, function);
                function.instruction(&Instruction::I32Eqz);
                descriptor.emit_writable_i32(schema, function);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I32Or);
            }
        }
        reject.store(function);
        descriptor.clear(function);
        key.clear(function);
        reject.load(function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0));
        accepted.store(function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        keys.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        output.value().set_boolean(accepted, function);
        output.set_kind(CompletionKind::Normal, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        for local in [reject, index, count, accepted] {
            schema.release_i32_local(local, function);
        }
        pending.clear(function);
        output.clear(function);
        target.clear(function);
        Ok(())
    }

    pub(in crate::builtins) fn compile_object_is_sealed_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_integrity_test_builtin(IntegrityTest::Sealed, function)
    }

    pub(in crate::builtins) fn compile_object_is_frozen_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_object_integrity_test_builtin(IntegrityTest::Frozen, function)
    }
}
