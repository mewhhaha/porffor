use super::*;
use crate::functions::OrdinaryDefaultPrototype;

/// This entry borrows the actual callable role rather than selecting an
/// identity from a global or an environment-shaped integer.
impl FunctionBuilder<'_> {
    pub(super) fn emit_iterator_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let new_target = schema.reserve_value_local(function);
        let active = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let output = schema.reserve_completion(function);
        self.compile_new_target_to_locals(&new_target, function)?;
        active.set_reference(
            self.body_entry_locals()
                .and_then(|entry| entry.function_object())
                .expect("Iterator constructor owns its actual callable entry"),
            schema,
            function,
        );
        output.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        new_target.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Eq);
        self.emit_tagged_payload_same_value_i32(&new_target, &active, function)?;
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ITERATOR_CONSTRUCTOR_CANNOT_BE_CALLED,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_get_prototype_from_constructor(
            &new_target,
            OrdinaryDefaultPrototype::Iterator,
            &pending,
            function,
        )?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(&pending, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let object = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(pending.value()), function)?,
            function,
        );
        output.value().set_reference(&object, schema, function);
        output.set_kind(CompletionKind::Normal, function);
        object.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        output.clear(function);
        pending.clear(function);
        active.clear(function);
        new_target.clear(function);
        Ok(())
    }
}
