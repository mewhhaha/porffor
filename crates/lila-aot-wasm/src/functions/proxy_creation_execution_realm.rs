use super::*;

enum ProxyCreationRealmIntrinsic {
    ObjectPrototype,
    FunctionPrototype,
    TypeErrorPrototype,
}

impl ProxyCreationRealmIntrinsic {
    const fn offset(self) -> u64 {
        match self {
            Self::ObjectPrototype => HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            Self::FunctionPrototype => HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
            Self::TypeErrorPrototype => HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
        }
    }
}

/// The Realm-owned identities created or thrown by the Proxy constructor
/// algorithms.
///
/// Its fields stay private to the `functions` module. Proxy creation can only
/// consume the complete set, so an allocation cannot pair one Realm's record
/// with another Realm's Object or Function prototype.
#[must_use = "Proxy creation execution Realm must be explicitly released"]
pub(crate) struct ProxyCreationExecutionRealm {
    pub(super) realm_local: u32,
    pub(super) object_prototype_local: u32,
    pub(super) function_prototype_local: u32,
    type_error_prototype_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_proxy_creation_execution_realm(
        &mut self,
        function: &mut Function,
    ) -> ProxyCreationExecutionRealm {
        let realm_local = self.reserve_temp_local();
        let object_prototype_local = self.reserve_temp_local();
        let function_prototype_local = self.reserve_temp_local();
        let type_error_prototype_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();
        let active_function_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::GlobalGet(PROXY_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(active_function_local));
        self.load_i64_to_local_from_offset(
            active_function_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(intrinsics_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        for (intrinsic, prototype_local) in [
            (
                ProxyCreationRealmIntrinsic::ObjectPrototype,
                object_prototype_local,
            ),
            (
                ProxyCreationRealmIntrinsic::FunctionPrototype,
                function_prototype_local,
            ),
            (
                ProxyCreationRealmIntrinsic::TypeErrorPrototype,
                type_error_prototype_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(
                intrinsics_local,
                intrinsic.offset(),
                prototype_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(prototype_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(active_function_local);
        self.release_temp_local(intrinsics_local);
        ProxyCreationExecutionRealm {
            realm_local,
            object_prototype_local,
            function_prototype_local,
            type_error_prototype_local,
        }
    }

    pub(crate) fn emit_throw_proxy_creation_type_error(
        &mut self,
        realm: &ProxyCreationExecutionRealm,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            message,
            realm.type_error_prototype_local,
            self.result_local,
            self.result_tag_local,
            function,
        )
    }

    pub(crate) fn emit_alloc_proxy_revocable_result_object(
        &mut self,
        realm: &ProxyCreationExecutionRealm,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_alloc_plain_object_with_prototype(
            Some(realm.object_prototype_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(result_payload_local));
        Ok(())
    }

    pub(crate) fn emit_proxy_revoke_target_function(
        &mut self,
        realm: &ProxyCreationExecutionRealm,
        target_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&StandardBuiltinId::ProxyRevoke.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `[[ProxyRevoke]]`",
                )
            })?;
        self.emit_function_value_payload(&meta, function)?;
        function.instruction(&Instruction::LocalSet(target_payload_local));
        self.emit_store_function_defining_realm(target_payload_local, realm.realm_local, function);
        self.store_i64_local_at_offset(
            target_payload_local,
            HEAP_PROTOTYPE_OFFSET,
            realm.function_prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            target_payload_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Function.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            target_payload_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            realm.type_error_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            target_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            target_payload_local,
            function,
        );
        Ok(())
    }

    pub(crate) fn release_proxy_creation_execution_realm(
        &mut self,
        realm: ProxyCreationExecutionRealm,
    ) {
        self.release_temp_local(realm.type_error_prototype_local);
        self.release_temp_local(realm.function_prototype_local);
        self.release_temp_local(realm.object_prototype_local);
        self.release_temp_local(realm.realm_local);
    }
}
