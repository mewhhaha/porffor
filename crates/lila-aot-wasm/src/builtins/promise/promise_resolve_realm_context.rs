use super::*;

#[must_use = "PromiseResolve operation Realm context must be explicitly released"]
pub(super) struct PromiseResolveOperationRealmContext {
    resolve_function_payload_local: u32,
}

#[must_use = "intrinsic PromiseResolve Realm context must be explicitly released"]
pub(super) struct IntrinsicPromiseResolveRealmContext {
    operation: PromiseResolveOperationRealmContext,
    constructor_payload_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_promise_resolve_internal_function_materialization_context(
        &mut self,
        authority: PromiseResolveRealmAuthority<'_>,
        function: &mut Function,
    ) -> PromiseInternalFunctionMaterializationContext {
        match authority {
            PromiseResolveRealmAuthority::CurrentFunction => self
                .emit_current_function_promise_internal_function_materialization_context(function),
            PromiseResolveRealmAuthority::AsyncExecution(realm) => {
                let realm_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(realm.realm_local));
                function.instruction(&Instruction::LocalSet(realm_local));
                self.emit_promise_internal_function_materialization_context_from_realm(
                    realm_local,
                    function,
                )
            }
        }
    }

    pub(super) fn emit_promise_resolve_operation_realm_context(
        &mut self,
        authority: PromiseResolveRealmAuthority<'_>,
        function: &mut Function,
    ) -> Result<PromiseResolveOperationRealmContext, EmitError> {
        let resolve_meta = self
            .functions
            .get(&StandardBuiltinId::PromiseResolve.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Promise.resolve builtin"))?;
        let resolve_function_payload_local = self.reserve_temp_local();
        let materialization_context = self
            .emit_promise_resolve_internal_function_materialization_context(authority, function);
        let result = self.emit_promise_internal_function_value(
            &resolve_meta,
            &materialization_context,
            0,
            resolve_function_payload_local,
            function,
        );
        self.release_promise_internal_function_materialization_context(materialization_context);
        result?;
        Ok(PromiseResolveOperationRealmContext {
            resolve_function_payload_local,
        })
    }

    pub(super) fn emit_intrinsic_promise_resolve_realm_context(
        &mut self,
        authority: PromiseResolveRealmAuthority<'_>,
        function: &mut Function,
    ) -> Result<IntrinsicPromiseResolveRealmContext, EmitError> {
        let resolve_meta = self
            .functions
            .get(&StandardBuiltinId::PromiseResolve.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing intrinsic Promise.resolve builtin"))?;
        let resolve_function_payload_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();
        let materialization_context = self
            .emit_promise_resolve_internal_function_materialization_context(authority, function);

        self.emit_load_promise_internal_function_realm_intrinsics(
            &materialization_context,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET,
            constructor_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let result = self.emit_promise_internal_function_value(
            &resolve_meta,
            &materialization_context,
            0,
            resolve_function_payload_local,
            function,
        );

        self.release_promise_internal_function_materialization_context(materialization_context);
        self.release_temp_local(intrinsics_local);
        result?;
        Ok(IntrinsicPromiseResolveRealmContext {
            operation: PromiseResolveOperationRealmContext {
                resolve_function_payload_local,
            },
            constructor_payload_local,
        })
    }

    pub(super) fn emit_call_promise_resolve_operation(
        &mut self,
        context: &PromiseResolveOperationRealmContext,
        constructor_payload_local: u32,
        constructor_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let resolve_function_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(resolve_function_tag_local));
        let result = self.emit_function_handle_call_without_throw_propagation(
            context.resolve_function_payload_local,
            resolve_function_tag_local,
            Some((constructor_payload_local, Some(constructor_tag_local))),
            &[(value_payload_local, value_tag_local)],
            promise_payload_local,
            promise_tag_local,
            function,
        );
        self.release_temp_local(resolve_function_tag_local);
        result
    }

    pub(super) fn release_promise_resolve_operation_realm_context(
        &mut self,
        context: PromiseResolveOperationRealmContext,
    ) {
        self.release_temp_local(context.resolve_function_payload_local);
    }

    pub(super) fn release_intrinsic_promise_resolve_realm_context(
        &mut self,
        context: IntrinsicPromiseResolveRealmContext,
    ) {
        self.release_temp_local(context.constructor_payload_local);
        self.release_promise_resolve_operation_realm_context(context.operation);
    }

    pub(super) fn emit_intrinsic_promise_resolve_to_locals(
        &mut self,
        context: &IntrinsicPromiseResolveRealmContext,
        value_payload_local: u32,
        value_tag_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        let result = self.emit_call_promise_resolve_operation(
            &context.operation,
            context.constructor_payload_local,
            constructor_tag_local,
            value_payload_local,
            value_tag_local,
            promise_payload_local,
            promise_tag_local,
            function,
        );

        self.release_temp_local(constructor_tag_local);
        result
    }

    pub(super) fn emit_new_intrinsic_promise_resolve_rejection_capability(
        &mut self,
        resolve_context: &IntrinsicPromiseResolveRealmContext,
        rejected_promise_constructor_tag_local: u32,
        rejected_promise_capability_local: u32,
        awaited_promise_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_new_promise_capability(
            resolve_context.constructor_payload_local,
            rejected_promise_constructor_tag_local,
            rejected_promise_capability_local,
            awaited_promise_payload_local,
            self.result_tag_local,
            function,
        )?;
        Ok(())
    }
}
