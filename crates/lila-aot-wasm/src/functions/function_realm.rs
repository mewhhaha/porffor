use super::*;

enum FunctionRealmOutcome {
    Resolved,
    Revoked,
    Invalid,
}

impl FunctionRealmOutcome {
    const fn runtime_code(&self) -> i64 {
        match self {
            Self::Resolved => 0,
            Self::Revoked => 1,
            Self::Invalid => 2,
        }
    }
}

/// The raw run-time result of `GetFunctionRealm` before its non-resolved
/// outcomes have been routed.
///
/// Its fields are intentionally private. A caller can only obtain the realm
/// local by consuming this value through
/// [`FunctionBuilder::emit_route_function_realm_result`], which handles both
/// `Revoked` and `Invalid` before returning a resolved witness.
#[must_use]
pub(crate) struct FunctionRealmResultLocals {
    realm_local: u32,
    outcome_local: u32,
}

/// A Wasm local whose `GetFunctionRealm` non-resolved outcomes have both been
/// handled according to an explicit route.
#[derive(Clone, Copy)]
#[must_use]
pub(crate) struct ResolvedFunctionRealmLocal(u32);

impl ResolvedFunctionRealmLocal {
    pub(crate) const fn index(self) -> u32 {
        self.0
    }
}

/// What a consumer does when `GetFunctionRealm` encounters a revoked Proxy.
///
/// `Invalid` is deliberately absent: every route traps for that internal
/// invariant failure. Promise job creation uses the current realm for a
/// revoked callback, while constructor/default-prototype consumers surface
/// the required TypeError and leave their enclosing control-flow region.
pub(crate) enum FunctionRealmRevokedRoute {
    UseCurrentRealm,
    ThrowTypeErrorAndReturn {
        payload_local: u32,
        tag_local: u32,
    },
    ThrowTypeErrorAndBranch {
        payload_local: u32,
        tag_local: u32,
        relative_depth: u32,
    },
}

impl<'a> FunctionBuilder<'a> {
    /// Implements GetFunctionRealm's recursive bound/proxy traversal without
    /// performing any user-visible property access.
    ///
    /// Constructor callers invoke this only after their observable
    /// `Get(newTarget, "prototype")`; Promise jobs invoke it on the already
    /// captured callback. The returned locals are opaque until a caller
    /// consumes them through [`Self::emit_route_function_realm_result`].
    pub(crate) fn emit_get_function_realm(
        &mut self,
        source_payload_local: u32,
        source_tag_local: u32,
        function: &mut Function,
    ) -> FunctionRealmResultLocals {
        let realm_local = self.reserve_temp_local();
        let outcome_local = self.reserve_temp_local();
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let proxy_handler_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(source_payload_local));
        function.instruction(&Instruction::LocalSet(current_payload_local));
        function.instruction(&Instruction::LocalGet(source_tag_local));
        function.instruction(&Instruction::LocalSet(current_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(realm_local));
        function.instruction(&Instruction::I64Const(
            FunctionRealmOutcome::Invalid.runtime_code(),
        ));
        function.instruction(&Instruction::LocalSet(outcome_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_function_flags(current_payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            current_tag_local,
            function,
        );
        // inner if, outer function-tag if, loop
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            FunctionRealmOutcome::Resolved.runtime_code(),
        ));
        function.instruction(&Instruction::LocalSet(outcome_local));
        function.instruction(&Instruction::End);
        // function-tag if, loop, exit block
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_handler_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            FunctionRealmOutcome::Revoked.runtime_code(),
        ));
        function.instruction(&Instruction::LocalSet(outcome_local));
        // revoked if, proxy if, object-tag if, loop, exit block
        function.instruction(&Instruction::Br(4));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            current_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            current_payload_local,
            function,
        );
        // proxy if, object-tag if, loop
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Unknown/non-callable representation: retain the explicit Invalid
        // outcome. Validated newTarget values must reach an ordinary function.
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_handler_local);
        self.release_temp_local(record_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        FunctionRealmResultLocals {
            realm_local,
            outcome_local,
        }
    }

    /// Consume a raw GetFunctionRealm result, route a revoked Proxy according
    /// to the caller's closed policy, and trap an invalid callable
    /// representation before exposing the realm local.
    pub(crate) fn emit_route_function_realm_result(
        &mut self,
        result: FunctionRealmResultLocals,
        revoked_route: FunctionRealmRevokedRoute,
        function: &mut Function,
    ) -> Result<ResolvedFunctionRealmLocal, EmitError> {
        function.instruction(&Instruction::LocalGet(result.outcome_local));
        function.instruction(&Instruction::I64Const(
            FunctionRealmOutcome::Revoked.runtime_code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        match revoked_route {
            FunctionRealmRevokedRoute::UseCurrentRealm => {
                function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(result.realm_local));
            }
            FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn {
                payload_local,
                tag_local,
            } => {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "cannot get function realm from a revoked Proxy",
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
            }
            FunctionRealmRevokedRoute::ThrowTypeErrorAndBranch {
                payload_local,
                tag_local,
                relative_depth,
            } => {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "cannot get function realm from a revoked Proxy",
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Br(relative_depth));
            }
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result.outcome_local));
        function.instruction(&Instruction::I64Const(
            FunctionRealmOutcome::Invalid.runtime_code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        self.release_temp_local(result.outcome_local);
        Ok(ResolvedFunctionRealmLocal(result.realm_local))
    }

    pub(crate) fn release_resolved_function_realm_local(
        &mut self,
        realm: ResolvedFunctionRealmLocal,
    ) {
        self.release_temp_local(realm.index());
    }
}
