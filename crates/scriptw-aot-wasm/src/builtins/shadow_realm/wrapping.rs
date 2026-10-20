//! Callable boundaries use ordinary native closures with a strong target
//! capture. Only allocation uses the destination Realm; observations and
//! abrupt-completion conversion stay in the active method or wrapper Realm.

use crate::functions::FunctionRealmRevokedRoute;
use crate::gc_types::{
    BuiltinClosureCapture, BuiltinClosureCaptureKind, BuiltinClosureCaptureSchema,
    BuiltinClosurePayload, CompletionLocals, FunctionContext, FunctionContextSchema, GcI32Constant,
    GcLocal, GcOperand, Nullable, RealmRecord, ShadowRealmWrappedFunctionContext,
    ShadowRealmWrappedFunctionContextSchema, StoredValue, ValueArray, ValueLocals,
};
use crate::{
    BlockType, CompletionKind, ControlFrameKind, ControlTarget, EmitError, Function,
    FunctionBuilder, Instruction, RuntimeErrorMessage, StandardBuiltinId,
};

impl FunctionBuilder<'_> {
    /// GetWrappedValue never exposes an original object across the boundary.
    /// This also accepts a value borrowed from `output`: capture it before
    /// initializing the completion, so forwarding a returned value is safe.
    pub(crate) fn emit_shadow_realm_wrapped_value(
        &mut self,
        value: &ValueLocals,
        destination: &GcLocal<RealmRecord>,
        output: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let source = schema.reserve_value_local(function);
        let wrapped_value = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let zero = schema.reserve_i32_local(function);
        source.copy_from(value, function);
        wrapped_value.set_undefined(function);
        pending.initialize(function);
        output.set_normal(&source, function);
        zero.set_constant(0, function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        self.emit_is_heap_object_like_tag_i32(source.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_is_callable_i32(&source, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::SHADOW_REALM_CALLABLE_REQUIRED,
            output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let target = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&source, function),
            function,
        );
        let context = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ShadowRealmWrappedFunctionContext>()
                .construct((GcOperand::reference(&target, schema),), function),
            function,
        );
        let capture = schema
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<BuiltinClosureCapture>()
                    .publish(
                        BuiltinClosurePayload::ShadowRealmWrappedFunction(&context),
                        schema,
                        function,
                    )
                    .nullable(),
                function,
            );
        let meta = self
            .functions
            .get(&StandardBuiltinId::ShadowRealmWrappedFunctionCall.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing ShadowRealm wrapped-call dependency"))?;
        let materialization =
            self.emit_realm_function_materialization_context_from_realm(destination, function);
        let wrapped = schema.reserve_gc_local(function).initialize(
            self.emit_function_value_payload_in_realm_with_capture(
                &meta,
                &materialization,
                &capture,
                function,
            )?,
            function,
        );
        self.release_realm_function_materialization_context(materialization, function);
        capture.clear(function);
        context.clear(function);
        target.clear(function);
        wrapped_value.set_reference(&wrapped, schema, function);
        wrapped.clear(function);

        // CopyNameAndLength performs HasOwnProperty(length), Get(length),
        // then Get(name). Its descriptor definitions replace the factory's
        // initial metadata rather than appending duplicate own keys.
        self.emit_copy_function_name_and_length(
            &source,
            &wrapped_value,
            None,
            zero,
            &pending,
            function,
        )?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::SHADOW_REALM_ABRUPT,
            output,
            function,
        )?;
        function.instruction(&Instruction::Else);
        output.set_normal(&wrapped_value, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i32_local(zero, function);
        pending.clear(function);
        wrapped_value.clear(function);
        source.clear(function);
        Ok(())
    }

    fn emit_shadow_realm_wrapped_target(&self, target: &ValueLocals, function: &mut Function) {
        let schema = self.runtime_schema();
        let entry = self
            .body_entry_locals()
            .expect("ShadowRealm wrapped callable has its native entry");
        let capture = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::BUILTIN_CAPTURE)
                .read(
                    entry.function_context().expect("wrapped callable context"),
                    schema,
                    function,
                )
                .reference()
                .require_non_null(function),
            function,
        );
        let kind = schema.reserve_i32_local(function);
        schema
            .struct_type::<BuiltinClosureCapture>()
            .field(BuiltinClosureCaptureSchema::KIND)
            .read(&capture, schema, function)
            .store(kind, function);
        kind.load(function);
        function.instruction(&Instruction::I32Const(GcI32Constant::encode(
            BuiltinClosureCaptureKind::ShadowRealmWrappedFunction,
        )));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        schema.release_i32_local(kind, function);
        let context = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<BuiltinClosureCapture>()
                .field(BuiltinClosureCaptureSchema::SHADOW_REALM_WRAPPED_FUNCTION)
                .read(&capture, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ShadowRealmWrappedFunctionContext>()
                .field(ShadowRealmWrappedFunctionContextSchema::TARGET)
                .read(&context, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, target, schema, function);
        stored.clear(function);
        context.clear(function);
        capture.clear(function);
    }

    /// Boundary errors already belong to this active wrapper's Realm. Retain
    /// their completion; only an abrupt target Call needs a fresh TypeError.
    fn emit_shadow_realm_wrap_abrupt_exit(
        &mut self,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) {
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(pending, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn compile_shadow_realm_wrapped_function_call_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        let original_this = schema.reserve_value_local(function);
        let wrapped_this = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let output = schema.reserve_completion(function);
        let index = schema.reserve_i32_local(function);
        let count = schema.reserve_i32_local(function);
        pending.initialize(function);
        output.initialize(function);
        wrapped_this.set_undefined(function);
        argument.set_undefined(function);
        self.emit_shadow_realm_wrapped_target(&target, function);
        let entry = self
            .body_entry_locals()
            .expect("ShadowRealm wrapped callable has its native entry");
        original_this.copy_from(entry.this_value(), function);
        let arguments = schema
            .reserve_gc_local(function)
            .initialize(entry.arguments().load(schema, function), function);
        let caller_realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let exit = self.open_frame(ControlFrameKind::Block, function);

        // GetFunctionRealm follows strong bound/proxy targets without Get.
        // A revoked target throws before any argument metadata is observed.
        let target_realm = self.emit_get_function_realm(&target, function);
        let target_realm = self.emit_route_function_realm_result(
            target_realm,
            FunctionRealmRevokedRoute::ThrowTypeErrorAndBranch {
                result: &output,
                target: exit,
            },
            function,
        )?;
        let array = schema.array_type::<ValueArray>();
        array.length(&arguments, schema, function);
        count.store(function);
        let empty = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&argument, function),
            function,
        );
        let wrapped_arguments = schema.reserve_gc_local(function).initialize(
            array.filled(GcOperand::reference(&empty, schema), count, function),
            function,
        );
        empty.clear(function);
        index.set_constant(0, function);
        let arguments_done = self.open_frame(ControlFrameKind::Block, function);
        let again = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(arguments_done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_argument_vector_entry_to_value(&arguments, index, &argument, function);
        self.emit_shadow_realm_wrapped_value(&argument, target_realm.realm(), &pending, function)?;
        self.emit_shadow_realm_wrap_abrupt_exit(&pending, &output, exit, function);
        let element = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(pending.value(), function),
            function,
        );
        array.write(
            &wrapped_arguments,
            index,
            GcOperand::reference(&element, schema),
            schema,
            function,
        );
        element.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(again, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        // `this` crosses after every argument, including each callable's
        // length/name observations, and before the actual target invocation.
        self.emit_shadow_realm_wrapped_value(
            &original_this,
            target_realm.realm(),
            &pending,
            function,
        )?;
        self.emit_shadow_realm_wrap_abrupt_exit(&pending, &output, exit, function);
        wrapped_this.copy_from(pending.value(), function);
        self.emit_function_or_proxy_call_with_argv(
            &target,
            &wrapped_this,
            &wrapped_arguments,
            &pending,
            function,
        )?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_shadow_realm_wrapped_value(pending.value(), &caller_realm, &output, function)?;
        function.instruction(&Instruction::Else);
        // CreateTypeErrorCopy must not inspect thrown values, even their
        // message/name properties or conversion hooks.
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::SHADOW_REALM_ABRUPT,
            &output,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        wrapped_arguments.clear(function);
        self.release_resolved_function_realm_local(target_realm, function);
        caller_realm.clear(function);
        arguments.clear(function);
        schema.release_i32_local(count, function);
        schema.release_i32_local(index, function);
        output.clear(function);
        pending.clear(function);
        argument.clear(function);
        wrapped_this.clear(function);
        original_this.clear(function);
        target.clear(function);
        Ok(())
    }
}
