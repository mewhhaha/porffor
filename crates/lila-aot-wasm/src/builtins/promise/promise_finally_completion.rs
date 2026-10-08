use super::*;

enum PromiseFinallyCompletion {
    Fulfill,
    Reject,
}
impl FunctionBuilder<'_> {
    pub(crate) fn emit_promise_then_finally(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_finally_continuation(PromiseFinallyCompletion::Fulfill, function)
    }
    pub(crate) fn emit_promise_catch_finally(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_finally_continuation(PromiseFinallyCompletion::Reject, function)
    }

    fn emit_promise_finally_continuation(
        &mut self,
        completion: PromiseFinallyCompletion,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let context = self.emit_promise_finally_context(function);
        let original = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &original, function);
        let callback = schema.reserve_value_local(function);
        let constructor = schema.reserve_value_local(function);
        for (field, value) in [
            (PromiseFinallyContextSchema::ON_FINALLY, &callback),
            (PromiseFinallyContextSchema::CONSTRUCTOR, &constructor),
        ] {
            let stored = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<PromiseFinallyContext>()
                    .field(field)
                    .read(&context, schema, function)
                    .reference(),
                function,
            );
            schema
                .struct_type::<StoredValue>()
                .read_into(&stored, value, schema, function);
            stored.clear(function);
        }
        let undefined = schema.reserve_value_local(function);
        undefined.set_undefined(function);
        let arguments = self.emit_pre_evaluated_arg_vector(&[], function);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_function_or_proxy_call_with_argv(
            &callback, &undefined, &arguments, &pending, function,
        )?;
        self.emit_promise_abrupt_exit(&pending, exit, function);
        let resolve_context = self.emit_promise_resolve_operation_realm_context(
            PromiseResolveRealmAuthority::CurrentFunction,
            function,
        )?;
        let cleanup = schema.reserve_value_local(function);
        cleanup.copy_from(pending.value(), function);
        self.emit_call_promise_resolve_operation(
            &resolve_context,
            &constructor,
            &cleanup,
            &pending,
            function,
        )?;
        self.emit_promise_abrupt_exit(&pending, exit, function);
        let promise = schema.reserve_value_local(function);
        promise.copy_from(pending.value(), function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&original, function),
            function,
        );
        let value_context = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PromiseFinallyValueContext>()
                .construct((GcOperand::reference(&stored, schema),), function),
            function,
        );
        let materialization =
            self.emit_current_function_promise_internal_function_materialization_context(function);
        let entry = match completion {
            PromiseFinallyCompletion::Fulfill => {
                PromiseInternalFunction::ValueThunk(&value_context)
            }
            PromiseFinallyCompletion::Reject => PromiseInternalFunction::Thrower(&value_context),
        };
        let callable =
            self.emit_promise_internal_function_value(entry, &materialization, function)?;
        let continuation = schema.reserve_value_local(function);
        continuation.set_reference(&callable, schema, function);
        let name = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("then", function)?,
            function,
        );
        let key = PropertyKeyLocals::from_string(schema, &name, function);
        self.emit_object_read_with_throw_routing(
            &promise,
            &promise,
            &key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        self.emit_promise_abrupt_exit(&pending, exit, function);
        let method = schema.reserve_value_local(function);
        method.copy_from(pending.value(), function);
        let args = self.emit_pre_evaluated_arg_vector(&[&continuation], function);
        self.emit_function_or_proxy_call_with_argv(&method, &promise, &args, &pending, function)?;
        self.completion().copy_from(&pending, function);
        args.clear(function);
        method.clear(function);
        key.clear(function);
        name.clear(function);
        continuation.clear(function);
        callable.clear(function);
        self.release_promise_internal_function_materialization_context(materialization, function);
        value_context.clear(function);
        stored.clear(function);
        promise.clear(function);
        cleanup.clear(function);
        self.release_promise_resolve_operation_realm_context(resolve_context, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        pending.clear(function);
        arguments.clear(function);
        undefined.clear(function);
        constructor.clear(function);
        callback.clear(function);
        original.clear(function);
        context.clear(function);
        Ok(())
    }

    pub(crate) fn emit_promise_value_thunk(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_finally_value_thunk(PromiseFinallyCompletion::Fulfill, function)
    }
    pub(crate) fn emit_promise_thrower(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_promise_finally_value_thunk(PromiseFinallyCompletion::Reject, function)
    }
    fn emit_promise_finally_value_thunk(
        &mut self,
        completion: PromiseFinallyCompletion,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let context = self.emit_promise_finally_value_context(function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PromiseFinallyValueContext>()
                .field(PromiseFinallyValueContextSchema::VALUE)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        let value = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &value, schema, function);
        match completion {
            PromiseFinallyCompletion::Fulfill => self.completion().set_normal(&value, function),
            PromiseFinallyCompletion::Reject => self.completion().set_throw(&value, function),
        }
        value.clear(function);
        stored.clear(function);
        context.clear(function);
        Ok(())
    }
}
