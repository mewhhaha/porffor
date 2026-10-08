use super::*;

/// Only the complete result of GetPrototypeFromConstructor can allocate an
/// Error header. The original semantic prototype retains its actual GC kind.
#[must_use]
pub(super) struct ErrorConstructorPrototypeLocals(ValueLocals);

/// An unpublished Error header. Publication installs the NativeErrorObject
/// brand once, after constructor-specific own properties have been prepared.
#[must_use]
pub(super) struct PreparedNativeErrorInstance(GcLocal<OrdinaryObject>);

impl PreparedNativeErrorInstance {
    pub(super) fn header(&self) -> &GcLocal<OrdinaryObject> {
        &self.0
    }

    pub(super) fn publish(
        self,
        builder: &FunctionBuilder<'_>,
        result: &CompletionLocals,
        function: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let error = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<NativeErrorObject>()
                .construct((GcOperand::reference(&self.0, schema),), function),
            function,
        );
        result.value().set_reference(&error, schema, function);
        result.set_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::I32Const(0));
        result.target().store(function);
        error.clear(function);
        self.0.clear(function);
    }
}

impl ErrorConstructorPrototypeLocals {
    pub(super) fn value(&self) -> &ValueLocals {
        &self.0
    }
    pub(super) fn clear(self, function: &mut Function) {
        self.0.clear(function);
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_error_message_constructor(
        &mut self,
        kind: ErrorMessageConstructorKind,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        let prototype = self.emit_error_constructor_prototype(
            OrdinaryDefaultPrototype::MessageError(kind),
            function,
        )?;
        let instance = self.emit_prepare_native_error_instance(prototype.value(), function)?;
        let message =
            self.emit_install_optional_error_message(instance.header(), &argument, function)?;
        self.emit_install_error_cause_from_arg(
            instance.header(),
            ErrorCauseOptionsArgument::MessageError,
            function,
        )?;
        self.emit_install_error_stack_from_message(kind, instance.header(), &message, function)?;
        message.clear(function);
        instance.publish(self, result, function);
        prototype.clear(function);
        argument.clear(function);
        Ok(())
    }

    pub(super) fn emit_error_constructor_prototype(
        &mut self,
        intrinsic: OrdinaryDefaultPrototype,
        function: &mut Function,
    ) -> Result<ErrorConstructorPrototypeLocals, EmitError> {
        let schema = self.runtime_schema();
        let constructor = schema.reserve_value_local(function);
        let entry = self
            .body_entry_locals()
            .expect("native Error constructor owns its callable entry");
        constructor.copy_from(entry.new_target(), function);
        constructor.tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        constructor.set_reference(
            self.body_entry_locals()
                .and_then(|entry| entry.function_object())
                .expect("native Error Call owns the active constructor"),
            schema,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let pending = schema.reserve_completion(function);
        self.emit_get_prototype_from_constructor(&constructor, intrinsic, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        let prototype = schema.reserve_value_local(function);
        prototype.copy_from(pending.value(), function);
        pending.clear(function);
        constructor.clear(function);
        Ok(ErrorConstructorPrototypeLocals(prototype))
    }

    pub(super) fn emit_prepare_native_error_instance(
        &mut self,
        prototype: &ValueLocals,
        function: &mut Function,
    ) -> Result<PreparedNativeErrorInstance, EmitError> {
        let schema = self.runtime_schema();
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(prototype), function)?,
            function,
        );
        Ok(PreparedNativeErrorInstance(header))
    }

    /// Undefined omits the own message. All other values are converted once,
    /// before cause access and before constructor-specific trailing work.
    pub(super) fn emit_install_optional_error_message(
        &mut self,
        header: &GcLocal<OrdinaryObject>,
        argument: &ValueLocals,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let message = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        let pending = schema.reserve_completion(function);
        self.emit_value_to_string_payload(argument, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        message.replace(
            pending
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_append_error_data(header, "message", pending.value(), function)?;
        pending.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(message)
    }

    /// The implementation-defined stack is the native name plus the already
    /// converted message. It performs no observable name/message Gets.
    fn emit_install_error_stack_from_message(
        &mut self,
        kind: ErrorMessageConstructorKind,
        header: &GcLocal<OrdinaryObject>,
        message: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let name = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference(kind.native_error_kind().as_str(), function)?,
            function,
        );
        let empty = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        let stack = schema.reserve_value_local(function);
        self.emit_string_payload_equality_i32(message, &empty, function);
        self.open_frame(ControlFrameKind::If, function);
        stack.set_reference(&name, schema, function);
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
            self.emit_concat_gc_strings(&prefix, message, function),
            function,
        );
        stack.set_reference(&joined, schema, function);
        joined.clear(function);
        prefix.clear(function);
        separator.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_append_error_data(header, "stack", &stack, function)?;
        stack.clear(function);
        empty.clear(function);
        name.clear(function);
        Ok(())
    }
}
