use super::super::*;
use crate::gc_types::{
    BuiltinClosureCapture, BuiltinClosureCaptureSchema, FunctionContextSchema, GcOperand,
    ProxyObject, ProxyObjectSchema, ProxyRevocationContext, ProxyRevocationContextSchema,
    ScalarValue, StoredValue, StringValue,
};
use crate::operations::PropertyKeyLocals;

impl<'a> FunctionBuilder<'a> {
    pub(super) fn compile_proxy_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let execution_realm = self.emit_proxy_creation_execution_realm(function);
        let new_target = self
            .body_entry_locals()
            .ok_or_else(|| EmitError::unsupported("missing checked Proxy builtin entry"))?
            .new_target();
        new_target.tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_proxy_creation_type_error(
            &execution_realm,
            RuntimeErrorMessage::CONSTRUCTOR_PROXY_REQUIRES_NEW,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        let handler = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &target, function);
        self.emit_builtin_arg_to_value(1, &handler, function);
        let proxy_slot = schema.reserve_gc_local(function);
        let proxy = proxy_slot.initialize(
            self.emit_alloc_proxy_with_slots(&target, &handler, &execution_realm, function)?,
            function,
        );
        let value = schema.reserve_value_local(function);
        value.set_reference(&proxy, schema, function);
        self.completion().set_normal(&value, function);
        value.clear(function);
        proxy.clear(function);
        handler.clear(function);
        target.clear(function);
        self.release_proxy_creation_execution_realm(execution_realm, function);
        Ok(())
    }

    pub(super) fn compile_proxy_revocable_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let execution_realm = self.emit_proxy_creation_execution_realm(function);
        let schema = self.runtime_schema();
        let target = schema.reserve_value_local(function);
        let handler = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &target, function);
        self.emit_builtin_arg_to_value(1, &handler, function);
        let proxy_slot = schema.reserve_gc_local(function);
        let proxy = proxy_slot.initialize(
            self.emit_alloc_proxy_with_slots(&target, &handler, &execution_realm, function)?,
            function,
        );
        let revoke_slot = schema.reserve_gc_local(function);
        let revoke = revoke_slot.initialize(
            self.emit_proxy_revoke_target_function(&execution_realm, &proxy, function)?,
            function,
        );
        let result_slot = schema.reserve_gc_local(function);
        let result = result_slot.initialize(
            self.emit_alloc_plain_object_with_prototype(
                Some(execution_realm.object_prototype()),
                function,
            )?,
            function,
        );
        let proxy_value = schema.reserve_value_local(function);
        proxy_value.set_reference(&proxy, schema, function);
        let revoke_value = schema.reserve_value_local(function);
        revoke_value.set_reference(&revoke, schema, function);
        for (name, value) in [("proxy", &proxy_value), ("revoke", &revoke_value)] {
            let name_slot = schema.reserve_gc_local::<StringValue, _>(function);
            let name = name_slot.initialize(
                self.emit_interned_string_reference(name, function)?,
                function,
            );
            let key = PropertyKeyLocals::from_string(schema, &name, function);
            self.emit_object_append_data_property_with_flags(
                &result, &key, value, true, true, true, function,
            )?;
            key.clear(function);
            name.clear(function);
        }
        let result_value = schema.reserve_value_local(function);
        result_value.set_reference(&result, schema, function);
        self.completion().set_normal(&result_value, function);
        result_value.clear(function);
        revoke_value.clear(function);
        proxy_value.clear(function);
        result.clear(function);
        revoke.clear(function);
        proxy.clear(function);
        handler.clear(function);
        target.clear(function);
        self.release_proxy_creation_execution_realm(execution_realm, function);
        Ok(())
    }

    /// The builtin revoker owns its captured proxy, independently of Call this.
    /// Clearing the capture first makes repeated or reentrant calls inert.
    pub(super) fn compile_proxy_revoke_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let context = self
            .body_entry_locals()
            .and_then(|entry| entry.function_context())
            .ok_or_else(|| EmitError::unsupported("missing checked Proxy revoker context"))?;
        let schema = self.runtime_schema();
        let capture_slot = schema.reserve_gc_local::<BuiltinClosureCapture, _>(function);
        let capture = capture_slot.initialize(
            schema
                .struct_type::<crate::gc_types::FunctionContext>()
                .field(FunctionContextSchema::BUILTIN_CAPTURE)
                .read(context, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let revocation_slot = schema.reserve_gc_local::<ProxyRevocationContext, _>(function);
        let revocation = revocation_slot.initialize(
            schema
                .struct_type::<BuiltinClosureCapture>()
                .field(BuiltinClosureCaptureSchema::PROXY_REVOCATION)
                .read(&capture, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let proxy_slot = schema.reserve_gc_local(function);
        let captured_proxy = proxy_slot.initialize(
            schema
                .struct_type::<ProxyRevocationContext>()
                .field(ProxyRevocationContextSchema::PROXY)
                .read(&revocation, schema, function)
                .reference(),
            function,
        );
        captured_proxy.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        schema
            .struct_type::<ProxyRevocationContext>()
            .field(ProxyRevocationContextSchema::PROXY)
            .write(&revocation, GcOperand::null(schema), schema, function);
        let live_slot = schema.reserve_gc_local::<ProxyObject, _>(function);
        let live = live_slot.initialize(
            captured_proxy
                .load(schema, function)
                .require_non_null(function),
            function,
        );
        let null_value = schema.reserve_value_local(function);
        null_value.set_scalar(ScalarValue::Null, function);
        let null_slot = schema.reserve_gc_local(function);
        let null_record = null_slot.initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&null_value, function),
            function,
        );
        for field in [ProxyObjectSchema::TARGET, ProxyObjectSchema::HANDLER] {
            schema.struct_type::<ProxyObject>().field(field).write(
                &live,
                GcOperand::reference(&null_record, schema),
                schema,
                function,
            );
        }
        null_record.clear(function);
        null_value.clear(function);
        live.clear(function);
        function.instruction(&Instruction::End);
        self.completion().initialize(function);
        captured_proxy.clear(function);
        revocation.clear(function);
        capture.clear(function);
        Ok(())
    }
}
