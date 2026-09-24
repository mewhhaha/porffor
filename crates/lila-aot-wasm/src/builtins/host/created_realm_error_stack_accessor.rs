use super::*;

impl<'a> FunctionBuilder<'a> {
    /// Install `Error.prototype.stack` (proposal-error-stack-accessor) on a
    /// created Realm's `%Error.prototype%`: an accessor with both functions,
    /// `{ [[Enumerable]]: false, [[Configurable]]: true }`.
    ///
    /// Each accessor function holds itself as its environment, exactly like the
    /// realm's other builtins, so its TypeErrors and the setter's
    /// `%Error.prototype%` home object resolve to this Realm rather than to the
    /// caller's.
    pub(super) fn emit_install_created_realm_error_stack_accessor(
        &mut self,
        realm_functions: &RealmFunctionMaterializationContext,
        error_prototype_local: u32,
        type_error_prototype_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [getter_meta, setter_meta] = [
            StandardBuiltinId::ErrorPrototypeStackGetter,
            StandardBuiltinId::ErrorPrototypeStackSetter,
        ]
        .map(|builtin| {
            self.functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })
        });
        let (getter_meta, setter_meta) = (getter_meta?, setter_meta?);
        let key_local = self.reserve_temp_local();
        let getter_local = self.reserve_temp_local();
        let setter_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        for (meta, accessor_local) in [(&getter_meta, getter_local), (&setter_meta, setter_local)] {
            self.emit_function_value_payload_in_realm(
                meta,
                realm_functions,
                accessor_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                accessor_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                accessor_local,
                function,
            );
            self.store_i64_local_at_offset(
                accessor_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("stack")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_accessor(
            error_prototype_local,
            key_local,
            AccessorDescriptorLocals::GetterAndSetter {
                getter: AccessorGetterLocals::new(TaggedLocals::new(getter_local, tag_local)),
                setter: AccessorSetterLocals::new(TaggedLocals::new(setter_local, tag_local)),
            },
            function,
        )?;

        for local in [tag_local, setter_local, getter_local, key_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
