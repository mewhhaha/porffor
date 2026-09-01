use super::*;

/// The exact receiver, its temporary GetV lookup object and the result slot.
///
/// This value is deliberately private and non-`Copy`. GetV borrows it, then
/// callability validation consumes it so the lookup object cannot escape into
/// the eventual Call receiver.
#[must_use = "Object.prototype.toLocaleString receiver roles must reach validation"]
struct ObjectToLocaleStringGetVLocals {
    original_receiver: TaggedLocals,
    boxed_lookup: TaggedLocals,
    method: TaggedLocals,
}

/// A callable `toString` method paired with its exact Invoke receiver.
///
/// This token is deliberately private and non-`Copy`. Its sole consumer takes
/// ownership before emitting Proxy-aware Call with no arguments.
#[must_use = "a validated Object.prototype.toLocaleString invocation must be called"]
struct ValidatedObjectToLocaleStringInvocationLocals {
    method: TaggedLocals,
    receiver: TaggedLocals,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_object_to_locale_string_get_v(
        &mut self,
        get_v: &ObjectToLocaleStringGetVLocals,
        key_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_read(
            get_v.boxed_lookup.payload,
            get_v.boxed_lookup.tag,
            get_v.original_receiver.payload,
            get_v.original_receiver.tag,
            key_local,
            get_v.method.payload,
            get_v.method.tag,
            function,
        )
    }

    fn emit_validate_object_to_locale_string_invocation(
        &mut self,
        get_v: ObjectToLocaleStringGetVLocals,
        function: &mut Function,
    ) -> Result<ValidatedObjectToLocaleStringInvocationLocals, EmitError> {
        let ObjectToLocaleStringGetVLocals {
            original_receiver,
            method,
            ..
        } = get_v;
        self.emit_is_callable_i32(method.tag, method.payload, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.prototype.toLocaleString target is not callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        Ok(ValidatedObjectToLocaleStringInvocationLocals {
            method,
            receiver: original_receiver,
        })
    }

    fn emit_call_validated_object_to_locale_string_invocation(
        &mut self,
        invocation: ValidatedObjectToLocaleStringInvocationLocals,
        result: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let ValidatedObjectToLocaleStringInvocationLocals { method, receiver } = invocation;

        self.emit_function_or_proxy_call_leave_throw_completion(
            method.payload,
            method.tag,
            receiver.payload,
            receiver.tag,
            &[],
            result.payload,
            result.tag,
            function,
        )
    }

    pub(in crate::builtins) fn compile_object_prototype_to_locale_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.toLocaleString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.toLocaleString receiver",
            )
        })?;
        let lookup_payload_local = self.reserve_temp_local();
        let lookup_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.prototype.toLocaleString called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(receiver_payload_local));
        function.instruction(&Instruction::LocalSet(lookup_payload_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(lookup_tag_local));
        self.emit_value_to_current_function_realm_object_locals(
            lookup_payload_local,
            lookup_tag_local,
            lookup_payload_local,
            lookup_tag_local,
            function,
        )?;

        let get_v = ObjectToLocaleStringGetVLocals {
            original_receiver: TaggedLocals::new(receiver_payload_local, receiver_tag_local),
            boxed_lookup: TaggedLocals::new(lookup_payload_local, lookup_tag_local),
            method: TaggedLocals::new(method_payload_local, method_tag_local),
        };
        function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_to_locale_string_get_v(&get_v, key_local, function)?;
        self.emit_return_current_completion_if_throw(function);
        let invocation = self.emit_validate_object_to_locale_string_invocation(get_v, function)?;
        self.emit_call_validated_object_to_locale_string_invocation(
            invocation,
            TaggedLocals::new(self.result_local, self.result_tag_local),
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(lookup_tag_local);
        self.release_temp_local(lookup_payload_local);
        Ok(())
    }
}
