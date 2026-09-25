use super::*;

impl<'a> FunctionBuilder<'a> {
    /// Install `Symbol.prototype.description` on a created Realm's
    /// `%Symbol.prototype%`: the same getter-only, non-enumerable,
    /// configurable accessor the entry Realm installs, and in the same
    /// position (after `toString`/`valueOf`, before `@@toPrimitive`). The
    /// getter is materialized in the created Realm, so a Symbol primitive read
    /// there reaches that Realm's own function object.
    pub(super) fn emit_define_created_realm_symbol_description_getter(
        &mut self,
        symbol_prototype_local: u32,
        realm_functions: &RealmFunctionMaterializationContext,
        type_error_prototype_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(meta) = self
            .functions
            .get(&StandardBuiltinId::SymbolPrototypeDescriptionGetter.function_id())
            .cloned()
        else {
            return Ok(());
        };
        let getter = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let key_local = self.reserve_temp_local();
        self.emit_function_value_payload_in_realm(
            &meta,
            realm_functions,
            getter.payload,
            function,
        )?;
        self.store_i64_local_at_offset(
            getter.payload,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            getter.payload,
            function,
        );
        self.store_i64_local_at_offset(
            getter.payload,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            type_error_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter.tag));
        function.instruction(&Instruction::I64Const(self.strings.payload("description")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_accessor(
            symbol_prototype_local,
            key_local,
            AccessorDescriptorLocals::Getter(AccessorGetterLocals::new(getter)),
            function,
        )?;
        self.release_temp_local(key_local);
        self.release_temp_local(getter.tag);
        self.release_temp_local(getter.payload);
        Ok(())
    }
}
