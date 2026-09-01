use super::*;

/// The complete set of compiler-owned sources for a bound function's exact
/// `[[BoundThis]]` value.
///
/// This domain is private so sibling emitters cannot pass an already-adapted
/// payload/tag pair to bound-function storage. The dispatcher below owns both
/// local reservation and materialization for each legal source.
enum ExactBoundThisSource<'realm> {
    BindArgumentZero,
    ProxyRevocationObject {
        proxy_payload_local: u32,
        realm: &'realm ProxyCreationExecutionRealm,
    },
}

impl<'a> FunctionBuilder<'a> {
    /// Create the bound function required by `Function.prototype.bind`.
    ///
    /// Argument zero is captured here so the builtin emitter cannot adapt or
    /// otherwise replace it before it becomes `[[BoundThis]]`.
    pub(crate) fn emit_alloc_bound_function_for_bind(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        bound_args_payload_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_alloc_bound_function_from_exact_source(
            target_payload_local,
            target_tag_local,
            ExactBoundThisSource::BindArgumentZero,
            bound_args_payload_local,
            payload_local,
            function,
        )
    }

    /// Create the hidden revocation closure used by `Proxy.revocable`.
    ///
    /// The captured Proxy is already an Object; this entry point installs its
    /// exact identity without exposing the raw bound-this tag to the Proxy
    /// emitter.
    pub(crate) fn emit_alloc_proxy_revocation_bound_function(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        proxy_payload_local: u32,
        realm: &ProxyCreationExecutionRealm,
        bound_args_payload_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_alloc_bound_function_from_exact_source(
            target_payload_local,
            target_tag_local,
            ExactBoundThisSource::ProxyRevocationObject {
                proxy_payload_local,
                realm,
            },
            bound_args_payload_local,
            payload_local,
            function,
        )
    }

    fn emit_alloc_bound_function_from_exact_source(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        source: ExactBoundThisSource<'_>,
        bound_args_payload_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let bound_this_payload_local = self.reserve_temp_local();
        let bound_this_tag_local = self.reserve_temp_local();
        let internal_prototype_local = self.reserve_temp_local();

        match source {
            ExactBoundThisSource::BindArgumentZero => {
                self.emit_builtin_arg_to_locals(
                    0,
                    bound_this_payload_local,
                    bound_this_tag_local,
                    function,
                );
                function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(internal_prototype_local));
            }
            ExactBoundThisSource::ProxyRevocationObject {
                proxy_payload_local,
                realm,
            } => {
                function.instruction(&Instruction::LocalGet(proxy_payload_local));
                function.instruction(&Instruction::LocalSet(bound_this_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(bound_this_tag_local));
                function.instruction(&Instruction::LocalGet(realm.function_prototype_local));
                function.instruction(&Instruction::LocalSet(internal_prototype_local));
            }
        }

        self.emit_alloc_bound_function_value(
            target_payload_local,
            target_tag_local,
            bound_this_payload_local,
            bound_this_tag_local,
            bound_args_payload_local,
            internal_prototype_local,
            payload_local,
            function,
        )?;

        self.release_temp_local(internal_prototype_local);
        self.release_temp_local(bound_this_tag_local);
        self.release_temp_local(bound_this_payload_local);
        Ok(())
    }

    fn emit_alloc_bound_function_value(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        bound_this_payload_local: u32,
        bound_this_tag_local: u32,
        bound_args_payload_local: u32,
        internal_prototype_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Bound function objects dispatch through `[[BoundFunctionInvoke]]`'s
        // funcref-table slot, so its real body must be emitted.
        self.functions
            .record_standard_builtin(StandardBuiltinId::BoundFunctionInvoker);
        let meta = self
            .functions
            .get(&StandardBuiltinId::BoundFunctionInvoker.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `[[BoundFunctionInvoke]]`",
                )
            })?;
        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_BOUND_FUNCTION_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_TAG_OFFSET,
            bound_this_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_THIS_PAYLOAD_OFFSET,
            bound_this_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_ARGS_PAYLOAD_OFFSET,
            bound_args_payload_local,
            function,
        );

        self.emit_load_function_constructable_flag(target_payload_local, flags_local, function);
        self.emit_heap_alloc_const(HEAP_FUNCTION_OBJECT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BOUND_FUNCTION_SELF_PAYLOAD_OFFSET,
            object_local,
            function,
        );
        self.emit_heap_alloc_const(MIN_HEAP_CAPACITY * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, MIN_HEAP_CAPACITY, function);
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            internal_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Function.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TABLE_INDEX_OFFSET,
            meta.table_index as u64,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(flags_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            flags_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            ValueKind::Undefined.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_TO_STRING_PAYLOAD_OFFSET,
            self.strings.payload(meta.to_string_value.as_str()) as u64,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_DATA_VIEW_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_NUMBER_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_BOOLEAN_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        for (_, _, offset) in error_realm_prototype_entries() {
            self.load_i64_to_local_from_offset(
                target_payload_local,
                offset,
                self.scratch_local,
                function,
            );
            self.store_i64_local_at_offset(object_local, offset, self.scratch_local, function);
        }
        self.copy_function_realm_typed_array_prototypes(
            target_payload_local,
            object_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
            self.scratch_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,
            0,
            function,
        );

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));

        self.release_temp_local(flags_local);
        self.release_temp_local(record_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }
}
