use super::*;

impl FunctionBuilder<'_> {
    /// Publish the created Realm's one thrower before source can observe its
    /// Function prototype or create an unmapped Arguments object.
    pub(crate) fn emit_initialize_realm_throw_type_error(
        &mut self,
        context: &RealmFunctionMaterializationContext,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thrower_meta = self
            .functions
            .get(&StandardBuiltinId::ThrowTypeError.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant violated: missing builtin meta `%ThrowTypeError%`",
                )
            })?;
        let thrower_payload_local = self.reserve_temp_local();
        let thrower_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.emit_function_value_payload_in_realm(
            &thrower_meta,
            context,
            thrower_payload_local,
            function,
        )?;
        self.emit_store_non_array_realm_intrinsic(
            context.realm.index(),
            NonArrayRealmIntrinsicSlot::ThrowTypeError,
            thrower_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(thrower_tag_local));
        for name in ["caller", "arguments"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_append_accessor_property_with_flags(
                context.function_prototype_local,
                key_local,
                Some((thrower_payload_local, thrower_tag_local)),
                Some((thrower_payload_local, thrower_tag_local)),
                false,
                true,
                function,
            )?;
        }

        self.release_temp_local(key_local);
        self.release_temp_local(thrower_tag_local);
        self.release_temp_local(thrower_payload_local);
        Ok(())
    }

    /// The active ECMAScript function and its Realm have already been
    /// initialized. Missing bootstrap state cannot be published as a callable
    /// null pointer or replaced by another Realm's intrinsic.
    pub(crate) fn emit_load_required_function_realm_throw_type_error(
        &mut self,
        function_object_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();
        for (base_local, offset, loaded_local) in [
            (
                function_object_local,
                HEAP_FUNCTION_DEFINING_REALM_OFFSET,
                realm_local,
            ),
            (realm_local, HEAP_REALM_INTRINSICS_OFFSET, intrinsics_local),
            (
                intrinsics_local,
                HEAP_REALM_INTRINSICS_THROW_TYPE_ERROR_OFFSET,
                result_local,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(base_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            self.load_i64_to_local_from_offset(base_local, offset, loaded_local, function);
        }
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }
}
