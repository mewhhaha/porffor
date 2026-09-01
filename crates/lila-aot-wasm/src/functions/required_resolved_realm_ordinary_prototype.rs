use super::*;

/// The ordinary-object intrinsic prototypes selected by
/// `GetPrototypeFromConstructor` in constructor fallback paths.
///
/// `%Array.prototype%` is deliberately absent because it has an Array layout
/// and a distinct representation tag. Keeping this domain closed prevents a
/// caller from pairing an arbitrary realm-intrinsic offset with an entry-realm
/// fallback.
#[derive(Clone, Copy)]
pub(crate) enum OrdinaryDefaultPrototype {
    Object,
    MessageError(ErrorMessageConstructorKind),
    String,
    Number,
    Boolean,
    Date,
    Iterator,
    RegExp,
    Promise,
}

impl OrdinaryDefaultPrototype {
    const fn offset(self) -> u64 {
        match self {
            Self::Object => HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            Self::MessageError(kind) => kind.prototype_slot().offset(),
            Self::String => HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET,
            Self::Number => HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET,
            Self::Boolean => HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET,
            Self::Date => HEAP_REALM_INTRINSICS_DATE_PROTOTYPE_OFFSET,
            Self::Iterator => HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET,
            Self::RegExp => HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET,
            Self::Promise => HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET,
        }
    }
}

/// A populated ordinary-object prototype loaded from a realm already proven
/// by `GetFunctionRealm`.
///
/// The local is non-`Copy` and private so construction must consume it through
/// the operation that installs both its payload and Object representation tag.
#[must_use = "the resolved-realm prototype must be installed with its representation tag"]
pub(super) struct ResolvedRealmOrdinaryPrototypeLocal(u32);

impl<'a> FunctionBuilder<'a> {
    /// Load a required ordinary-object intrinsic from a realm proven by
    /// `GetFunctionRealm`.
    ///
    /// A resolved ECMAScript realm always has an intrinsic record and every
    /// intrinsic in [`OrdinaryDefaultPrototype`]. Missing backend bootstrap
    /// state is therefore an internal invariant failure, never permission to
    /// substitute an entry-realm global.
    pub(super) fn emit_load_required_resolved_realm_ordinary_prototype(
        &mut self,
        realm: ResolvedFunctionRealmLocal,
        intrinsic: OrdinaryDefaultPrototype,
        function: &mut Function,
    ) -> ResolvedRealmOrdinaryPrototypeLocal {
        let prototype_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(realm.index()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            realm.index(),
            HEAP_REALM_INTRINSICS_OFFSET,
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
            intrinsic.offset(),
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.release_temp_local(intrinsics_local);
        ResolvedRealmOrdinaryPrototypeLocal(prototype_local)
    }

    /// Resolve and install one required ordinary default prototype from the
    /// original new target's function realm. Keeping the opaque realm result,
    /// required slot load and tagged witness consumption together prevents a
    /// fallback policy from exposing a realm before revoked/invalid outcomes
    /// are routed or from substituting an entry-realm global.
    pub(crate) fn emit_required_new_target_realm_ordinary_prototype(
        &mut self,
        new_target_payload_local: u32,
        new_target_tag_local: u32,
        intrinsic: OrdinaryDefaultPrototype,
        prototype_payload_local: u32,
        prototype_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_result =
            self.emit_get_function_realm(new_target_payload_local, new_target_tag_local, function);
        let realm = self.emit_route_function_realm_result(
            realm_result,
            FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn {
                payload_local: self.result_local,
                tag_local: self.result_tag_local,
            },
            function,
        )?;
        let prototype =
            self.emit_load_required_resolved_realm_ordinary_prototype(realm, intrinsic, function);
        self.emit_install_resolved_realm_ordinary_prototype(
            prototype,
            prototype_payload_local,
            prototype_tag_local,
            function,
        );
        self.release_resolved_function_realm_local(realm);
        Ok(())
    }

    /// Consume a required ordinary-object prototype and install its payload
    /// and representation tag as one transition.
    pub(super) fn emit_install_resolved_realm_ordinary_prototype(
        &mut self,
        prototype: ResolvedRealmOrdinaryPrototypeLocal,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(prototype.0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(prototype.0);
    }
}
