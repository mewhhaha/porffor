use super::*;

/// Storage reserved for a created realm's `%Array.prototype%`, before an
/// Array-layout object has been emitted into it.
///
/// This type is deliberately neither `Copy` nor constructible outside this
/// module. Initialization consumes it, so bootstrap cannot publish the local
/// while it still contains an arbitrary payload.
#[must_use]
pub(crate) struct ReservedRealmArrayPrototypeLocal(u32);

/// A Wasm local proven to contain an initialized created-realm
/// `%Array.prototype%` Array exotic object.
///
/// The raw local is private. Created-realm publication and property/link
/// installation accept only this state, and final release consumes it.
#[must_use]
pub(crate) struct RealmArrayPrototypeLocal(u32);

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn reserve_realm_array_prototype_local(
        &mut self,
    ) -> ReservedRealmArrayPrototypeLocal {
        ReservedRealmArrayPrototypeLocal(self.reserve_temp_local())
    }

    /// Consume reserved storage and initialize it with the Array exotic
    /// layout required by a created realm's `%Array.prototype%`.
    pub(crate) fn emit_initialize_realm_array_prototype(
        &mut self,
        reserved: ReservedRealmArrayPrototypeLocal,
        object_prototype_local: u32,
        function: &mut Function,
    ) -> Result<RealmArrayPrototypeLocal, EmitError> {
        let length_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_local));
        self.emit_alloc_array_payload_with_length(length_local, reserved.0, function)?;
        self.store_i64_local_at_offset(
            reserved.0,
            HEAP_PROTOTYPE_OFFSET,
            object_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            reserved.0,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.release_temp_local(length_local);
        Ok(RealmArrayPrototypeLocal(reserved.0))
    }

    pub(crate) fn emit_store_realm_array_prototype(
        &mut self,
        realm: RealmRecordLocal,
        prototype: &RealmArrayPrototypeLocal,
        function: &mut Function,
    ) {
        let intrinsics_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            realm.index(),
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        self.store_i64_local_at_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            prototype.0,
            function,
        );
        self.release_temp_local(intrinsics_local);
    }

    pub(crate) fn emit_define_realm_array_prototype_data_with_flags(
        &mut self,
        prototype: &RealmArrayPrototypeLocal,
        key: &str,
        payload_local: u32,
        tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let writable_local = self.reserve_temp_local();
        let enumerable_local = self.reserve_temp_local();
        let configurable_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(key),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(i64::from(writable)));
        function.instruction(&Instruction::LocalSet(writable_local));
        function.instruction(&Instruction::I64Const(i64::from(enumerable)));
        function.instruction(&Instruction::LocalSet(enumerable_local));
        function.instruction(&Instruction::I64Const(i64::from(configurable)));
        function.instruction(&Instruction::LocalSet(configurable_local));
        self.emit_array_define_named_data_descriptor(
            prototype.0,
            key_local,
            payload_local,
            tag_local,
            writable_local,
            enumerable_local,
            configurable_local,
            None,
            None,
            None,
            None,
            None,
            function,
        )?;
        self.release_temp_local(configurable_local);
        self.release_temp_local(enumerable_local);
        self.release_temp_local(writable_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    /// Install the two `%Array%` / `%Array.prototype%` links using the
    /// representation and attributes required by the intrinsic registry.
    pub(crate) fn emit_bind_realm_array_constructor_prototype(
        &mut self,
        constructor_local: u32,
        prototype: &RealmArrayPrototypeLocal,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype.0,
            function,
        );
        function.instruction(&Instruction::I64Const(
            self.strings
                .static_builtin_property_key_payload("prototype"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data_with_configurable(
            constructor_local,
            key_local,
            prototype.0,
            tag_local,
            false,
            false,
            false,
            function,
        )?;

        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_define_realm_array_prototype_data_with_flags(
            prototype,
            "constructor",
            constructor_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn release_realm_array_prototype_local(
        &mut self,
        prototype: RealmArrayPrototypeLocal,
    ) {
        self.release_temp_local(prototype.0);
    }
}
