use super::*;

/// Only successful name Get/default/ToString can mint this owner. Reading
/// message consumes it, so that observable phase cannot run before name.
#[must_use]
struct PreparedErrorNameLocal(GcLocal<StringValue>);

impl FunctionBuilder<'_> {
    pub(super) fn emit_error_prototype_to_string(
        &mut self,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        receiver.copy_from(
            self.body_entry_locals()
                .expect("Error.prototype.toString owns its native entry")
                .this_value(),
            function,
        );
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        let name = self.emit_error_to_string_prepare_name(&receiver, function)?;
        self.emit_error_to_string_message_and_result(&receiver, name, result, function)?;
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ERROR_PROTOTYPE_TOSTRING_RECEIVER_IS_NOT_OBJECT,
            result,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        receiver.clear(function);
        Ok(())
    }

    fn emit_error_to_string_prepare_name(
        &mut self,
        receiver: &ValueLocals,
        function: &mut Function,
    ) -> Result<PreparedErrorNameLocal, EmitError> {
        let name = self.emit_error_to_string_get_string(receiver, "name", "Error", function)?;
        Ok(PreparedErrorNameLocal(name))
    }

    fn emit_error_to_string_get_string(
        &mut self,
        receiver: &ValueLocals,
        key_name: &str,
        default: &str,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let string = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(default, function)?,
            function,
        );
        let key = self.emit_error_property_key(key_name, function)?;
        let pending = schema.reserve_completion(function);
        self.emit_object_read_with_throw_routing(
            receiver,
            receiver,
            &key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        pending.value().tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        let input = schema.reserve_value_local(function);
        input.copy_from(pending.value(), function);
        self.emit_value_to_string_payload(&input, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        string.replace(
            pending
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        input.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.clear(function);
        key.clear(function);
        Ok(string)
    }

    fn emit_error_to_string_message_and_result(
        &mut self,
        receiver: &ValueLocals,
        prepared: PreparedErrorNameLocal,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let PreparedErrorNameLocal(name) = prepared;
        let message = self.emit_error_to_string_get_string(receiver, "message", "", function)?;
        let empty = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        self.emit_string_payload_equality_i32(&name, &empty, function);
        self.open_frame(ControlFrameKind::If, function);
        result.value().set_reference(&message, schema, function);
        function.instruction(&Instruction::Else);
        self.emit_string_payload_equality_i32(&message, &empty, function);
        self.open_frame(ControlFrameKind::If, function);
        result.value().set_reference(&name, schema, function);
        function.instruction(&Instruction::Else);
        let separator = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(": ", function)?,
            function,
        );
        let prefix = schema.reserve_gc_local(function).initialize(
            self.emit_concat_gc_strings(&name, &separator, function),
            function,
        );
        let joined = schema.reserve_gc_local(function).initialize(
            self.emit_concat_gc_strings(&prefix, &message, function),
            function,
        );
        result.value().set_reference(&joined, schema, function);
        joined.clear(function);
        prefix.clear(function);
        separator.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.set_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::I32Const(0));
        result.target().store(function);
        empty.clear(function);
        message.clear(function);
        name.clear(function);
        Ok(())
    }
}
