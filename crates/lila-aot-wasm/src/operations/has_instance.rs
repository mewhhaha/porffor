use super::*;

/// One already-evaluated ECMAScript value admitted to the has-instance
/// dispatcher. The raw local pair stays private so the two abstract-operation
/// signatures below cannot transpose `object` and `constructor` accidentally.
#[must_use]
struct HasInstanceValueLocals {
    payload: u32,
    tag: u32,
}

impl HasInstanceValueLocals {
    fn new(payload: u32, tag: u32) -> Self {
        Self { payload, tag }
    }
}

/// The two specification entry points that share has-instance execution.
///
/// This type is intentionally non-`Copy`. `InstanceofOperator(O, C)` must
/// perform observable `@@hasInstance` dispatch, while
/// `OrdinaryHasInstance(C, O)` must not redispatch to the inherited intrinsic.
/// A bound target is the one ordinary transition back to the operator entry.
#[must_use]
enum HasInstanceRequestLocals {
    InstanceofOperator {
        object: HasInstanceValueLocals,
        constructor: HasInstanceValueLocals,
    },
    OrdinaryHasInstance {
        constructor: HasInstanceValueLocals,
        object: HasInstanceValueLocals,
    },
}

enum HasInstanceRuntimeState {
    InstanceofOperator,
    OrdinaryHasInstance,
}

impl HasInstanceRuntimeState {
    const fn runtime_code(&self) -> i64 {
        match self {
            Self::InstanceofOperator => 0,
            Self::OrdinaryHasInstance => 1,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_instanceof_i32(
        &mut self,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload_local = self.reserve_temp_local();
        let lhs_tag_local = self.reserve_temp_local();
        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        // InstanceofOperator evaluates O before C. The request owns these
        // prepared pairs from this point; neither abstract operation recompiles
        // an expression or swaps their specification roles.
        self.compile_expr_to_locals(lhs, lhs_payload_local, lhs_tag_local, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
        self.emit_instanceof_operator_from_locals(
            lhs_payload_local,
            lhs_tag_local,
            rhs_payload_local,
            rhs_tag_local,
            result_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I32WrapI64);

        self.release_temp_local(result_local);
        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.release_temp_local(lhs_tag_local);
        self.release_temp_local(lhs_payload_local);
        Ok(())
    }

    pub(crate) fn emit_instanceof_operator_from_locals(
        &mut self,
        object_payload_local: u32,
        object_tag_local: u32,
        constructor_payload_local: u32,
        constructor_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_has_instance_request(
            HasInstanceRequestLocals::InstanceofOperator {
                object: HasInstanceValueLocals::new(object_payload_local, object_tag_local),
                constructor: HasInstanceValueLocals::new(
                    constructor_payload_local,
                    constructor_tag_local,
                ),
            },
            result_local,
            function,
        )
    }

    pub(crate) fn emit_ordinary_has_instance_from_locals(
        &mut self,
        constructor_payload_local: u32,
        constructor_tag_local: u32,
        object_payload_local: u32,
        object_tag_local: u32,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_has_instance_request(
            HasInstanceRequestLocals::OrdinaryHasInstance {
                constructor: HasInstanceValueLocals::new(
                    constructor_payload_local,
                    constructor_tag_local,
                ),
                object: HasInstanceValueLocals::new(object_payload_local, object_tag_local),
            },
            result_local,
            function,
        )
    }

    fn emit_has_instance_request(
        &mut self,
        request: HasInstanceRequestLocals,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let handler_payload_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let call_result_payload_local = self.reserve_temp_local();
        let call_result_tag_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let bound_record_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let next_prototype_payload_local = self.reserve_temp_local();
        let next_prototype_tag_local = self.reserve_temp_local();

        let (state, constructor, object) = match request {
            HasInstanceRequestLocals::InstanceofOperator {
                object,
                constructor,
            } => (
                HasInstanceRuntimeState::InstanceofOperator,
                constructor,
                object,
            ),
            HasInstanceRequestLocals::OrdinaryHasInstance {
                constructor,
                object,
            } => (
                HasInstanceRuntimeState::OrdinaryHasInstance,
                constructor,
                object,
            ),
        };
        function.instruction(&Instruction::I64Const(state.runtime_code()));
        function.instruction(&Instruction::LocalSet(state_local));
        function.instruction(&Instruction::LocalGet(constructor.payload));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        function.instruction(&Instruction::LocalGet(constructor.tag));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        function.instruction(&Instruction::LocalGet(object.payload));
        function.instruction(&Instruction::LocalSet(object_payload_local));
        function.instruction(&Instruction::LocalGet(object.tag));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));

        let exit = self.open_frame(ControlFrameKind::Block, function);
        let dispatch = self.open_frame(ControlFrameKind::Loop, function);

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            HasInstanceRuntimeState::InstanceofOperator.runtime_code(),
        ));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);

        // InstanceofOperator step 1 rejects a primitive constructor before
        // attempting the observable well-known-symbol property read.
        self.emit_is_heap_object_like_tag_i32(constructor_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Right-hand side of 'instanceof' is not callable",
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // GetMethod(C, @@hasInstance). The encoded symbol payload is distinct
        // from the ordinary string property "Symbol.hasInstance".
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.hasInstance"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            handler_payload_local,
            handler_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            handler_payload_local,
            handler_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(handler_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(handler_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_is_callable_i32(constructor_tag_local, constructor_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Right-hand side of 'instanceof' is not callable",
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(
            HasInstanceRuntimeState::OrdinaryHasInstance.runtime_code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        self.emit_branch_to_target(dispatch, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_is_callable_i32(handler_tag_local, handler_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Right-hand side of 'instanceof' is not callable",
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_indirect_call_from_locals(
            handler_payload_local,
            handler_tag_local,
            Some((constructor_payload_local, constructor_tag_local)),
            &[(object_payload_local, object_tag_local)],
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.emit_to_boolean_payload_from_tagged_locals(
            call_result_tag_local,
            call_result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_branch_to_target(exit, function);

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // OrdinaryHasInstance step 1 returns false for a non-callable C. This
        // deliberately differs from the operator entry's TypeError fallback.
        self.emit_is_callable_i32(constructor_tag_local, constructor_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // A bound function does not read its own `prototype`. Its target is a
        // fresh InstanceofOperator request so an own @@hasInstance remains
        // observable on the target.
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_load_function_flags(constructor_payload_local, flags_local, function);
        function.instruction(&Instruction::LocalGet(flags_local));
        function.instruction(&Instruction::I64Const(FUNCTION_FLAG_BOUND as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.load_i64_to_local_from_offset(
            constructor_payload_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            bound_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            bound_record_local,
            HEAP_BOUND_FUNCTION_TARGET_PAYLOAD_OFFSET,
            constructor_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            bound_record_local,
            HEAP_BOUND_FUNCTION_TARGET_TAG_OFFSET,
            constructor_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(
            HasInstanceRuntimeState::InstanceofOperator.runtime_code(),
        ));
        function.instruction(&Instruction::LocalSet(state_local));
        self.emit_branch_to_target(dispatch, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // A primitive O cannot have C.prototype in its prototype chain.
        self.emit_is_heap_object_like_tag_i32(object_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // OrdinaryHasInstance requires an observable Get(C, "prototype").
        // The ordinary property path consults materialized accessor entries
        // before its function-slot fallback, and the Proxy path invokes traps.
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            constructor_payload_local,
            constructor_tag_local,
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(prototype_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Function has non-object prototype in instanceof check",
            call_result_payload_local,
            call_result_tag_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalSet(search_payload_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(search_tag_local));
        let walk_exit = self.open_frame(ControlFrameKind::Block, function);
        let walk = self.open_frame(ControlFrameKind::Loop, function);
        function.instruction(&Instruction::LocalGet(search_payload_local));
        function.instruction(&Instruction::I64Eqz);
        self.emit_branch_if_to_target(walk_exit, function);
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_object_get_prototype_of(
                search_payload_local,
                search_tag_local,
                next_prototype_payload_local,
                next_prototype_tag_local,
                function,
            )?;
        } else {
            self.emit_ordinary_get_prototype_of(
                search_payload_local,
                search_tag_local,
                next_prototype_payload_local,
                next_prototype_tag_local,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(next_prototype_payload_local));
        function.instruction(&Instruction::LocalGet(prototype_payload_local));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_prototype_payload_local));
        function.instruction(&Instruction::LocalSet(search_payload_local));
        function.instruction(&Instruction::LocalGet(next_prototype_tag_local));
        function.instruction(&Instruction::LocalSet(search_tag_local));
        self.emit_branch_to_target(walk, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_branch_to_target(exit, function);

        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.release_temp_local(next_prototype_tag_local);
        self.release_temp_local(next_prototype_payload_local);
        self.release_temp_local(search_tag_local);
        self.release_temp_local(search_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(bound_record_local);
        self.release_temp_local(flags_local);
        self.release_temp_local(call_result_tag_local);
        self.release_temp_local(call_result_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(handler_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(state_local);
        Ok(())
    }
}
