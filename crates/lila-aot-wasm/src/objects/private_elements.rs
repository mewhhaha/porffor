use super::*;

/// The five legal private-element heap rows, with their required locals.
///
/// Keeping receiver and value presence inside the variant prevents the raw
/// kind/`Option` Cartesian product described by
/// `docs/rust-rewrite/contracts/private-element-entry-protocol.md`.
enum PrivateElementEntryLocals {
    Brand {
        receiver: (u32, u32),
    },
    Field {
        receiver: (u32, u32),
        value: (u32, u32),
    },
    SetterDefinition {
        value: (u32, u32),
    },
    MethodDefinition {
        value: (u32, u32),
    },
    GetterDefinition {
        value: (u32, u32),
    },
}

impl PrivateElementEntryLocals {
    const fn kind(&self) -> PrivateElementHeapKind {
        match self {
            Self::Brand { .. } => PrivateElementHeapKind::Brand,
            Self::Field { .. } => PrivateElementHeapKind::Field,
            Self::SetterDefinition { .. } => PrivateElementHeapKind::SetterDefinition,
            Self::MethodDefinition { .. } => PrivateElementHeapKind::MethodDefinition,
            Self::GetterDefinition { .. } => PrivateElementHeapKind::GetterDefinition,
        }
    }

    const fn receiver(&self) -> Option<(u32, u32)> {
        match self {
            Self::Brand { receiver } | Self::Field { receiver, .. } => Some(*receiver),
            Self::SetterDefinition { .. }
            | Self::MethodDefinition { .. }
            | Self::GetterDefinition { .. } => None,
        }
    }

    const fn value(&self) -> Option<(u32, u32)> {
        match self {
            Self::Brand { .. } => None,
            Self::Field { value, .. }
            | Self::SetterDefinition { value }
            | Self::MethodDefinition { value }
            | Self::GetterDefinition { value } => Some(*value),
        }
    }
}

#[cfg(test)]
mod private_element_entry_protocol_tests {
    use super::*;

    #[test]
    fn private_element_rows_fix_wire_and_storage_projections() {
        let receiver = (11, 12);
        let value = (21, 22);
        let rows = [
            (
                PrivateElementEntryLocals::Brand { receiver },
                PrivateElementHeapKind::Brand,
                Some(receiver),
                None,
            ),
            (
                PrivateElementEntryLocals::Field { receiver, value },
                PrivateElementHeapKind::Field,
                Some(receiver),
                Some(value),
            ),
            (
                PrivateElementEntryLocals::SetterDefinition { value },
                PrivateElementHeapKind::SetterDefinition,
                None,
                Some(value),
            ),
            (
                PrivateElementEntryLocals::MethodDefinition { value },
                PrivateElementHeapKind::MethodDefinition,
                None,
                Some(value),
            ),
            (
                PrivateElementEntryLocals::GetterDefinition { value },
                PrivateElementHeapKind::GetterDefinition,
                None,
                Some(value),
            ),
        ];

        for (entry, kind, expected_receiver, expected_value) in &rows {
            assert_eq!(entry.kind(), *kind);
            assert_eq!(entry.receiver(), *expected_receiver);
            assert_eq!(entry.value(), *expected_value);
            assert_eq!(kind.has_receiver(), expected_receiver.is_some());
            assert_eq!(kind.has_value(), expected_value.is_some());
        }

        assert_eq!(
            rows.map(|(_, kind, _, _)| kind.wire_word()),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(
            [
                PrivateElementDefinitionKind::Setter,
                PrivateElementDefinitionKind::Method,
                PrivateElementDefinitionKind::Getter,
            ]
            .map(|kind| kind.heap_kind().wire_word()),
            [2, 3, 4]
        );
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_current_private_environment_to_local(
        &mut self,
        private_environment_local: u32,
        function: &mut Function,
    ) {
        if let Some(active_private_environment_local) =
            self.active_private_environment_locals.last().copied()
        {
            function.instruction(&Instruction::LocalGet(active_private_environment_local));
            function.instruction(&Instruction::LocalSet(private_environment_local));
            return;
        }
        if self
            .current_function_meta()
            .is_some_and(WasmFunctionMeta::has_function_context)
        {
            self.load_i64_to_local_from_offset(
                self.class_function_context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                private_environment_local,
                function,
            );
            return;
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(private_environment_local));
    }

    pub(crate) fn emit_private_name_token_to_local(
        &mut self,
        private_name_id: PrivateNameId,
        token_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.active_private_environment_locals.is_empty()
            && !self
                .current_function_meta()
                .is_some_and(WasmFunctionMeta::has_function_context)
        {
            return Err(EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: private name outside class execution context",
            ));
        }
        self.emit_current_private_environment_to_local(token_local, function);

        let stored_class_scope_local = self.reserve_temp_local();
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(token_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            "TypeError",
            "private environment is missing its declared name",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            token_local,
            HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET,
            stored_class_scope_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(stored_class_scope_local));
        function.instruction(&Instruction::I64Const(private_name_id.class_scope() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            token_local,
            HEAP_PRIVATE_ENV_PARENT_OFFSET,
            token_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(token_local));
        function.instruction(&Instruction::I64Const(
            (HEAP_PRIVATE_ENV_SLOT_BASE_OFFSET
                + private_name_id.name_ordinal() as u64 * HEAP_PRIVATE_ENV_SLOT_SIZE)
                as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(token_local));
        self.release_temp_local(stored_class_scope_local);
        Ok(())
    }

    pub(crate) fn emit_private_brand_add(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        token_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            token_local,
            PrivateElementEntryLocals::Brand {
                receiver: (receiver_payload_local, receiver_tag_local),
            },
            function,
        )
    }

    pub(crate) fn emit_private_field_add(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        token_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            token_local,
            PrivateElementEntryLocals::Field {
                receiver: (receiver_payload_local, receiver_tag_local),
                value: (value_payload_local, value_tag_local),
            },
            function,
        )
    }

    pub(crate) fn emit_private_setter_definition_add(
        &mut self,
        token_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            token_local,
            PrivateElementEntryLocals::SetterDefinition {
                value: (setter_payload_local, setter_tag_local),
            },
            function,
        )
    }

    pub(crate) fn emit_private_method_definition_add(
        &mut self,
        token_local: u32,
        method_payload_local: u32,
        method_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            token_local,
            PrivateElementEntryLocals::MethodDefinition {
                value: (method_payload_local, method_tag_local),
            },
            function,
        )
    }

    pub(crate) fn emit_private_getter_definition_add(
        &mut self,
        token_local: u32,
        getter_payload_local: u32,
        getter_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_private_element_entry_add(
            token_local,
            PrivateElementEntryLocals::GetterDefinition {
                value: (getter_payload_local, getter_tag_local),
            },
            function,
        )
    }

    fn emit_private_element_entry_add(
        &mut self,
        token_local: u32,
        entry: PrivateElementEntryLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_local = self.reserve_temp_local();
        let previous_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let kind = entry.kind();
        let receiver_locals = entry.receiver();
        let value_locals = entry.value();

        debug_assert_eq!(kind.has_receiver(), receiver_locals.is_some());
        debug_assert_eq!(kind.has_value(), value_locals.is_some());

        if let Some((receiver_payload_local, receiver_tag_local)) = receiver_locals {
            let extensible_local = self.reserve_temp_local();
            self.emit_object_is_extensible_i32(
                receiver_payload_local,
                receiver_tag_local,
                extensible_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(extensible_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error_to_active_handler(
                TYPE_ERROR_NAME,
                "private element cannot be installed on non-extensible object",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            self.release_temp_local(extensible_local);

            let existing_entry_local = self.reserve_temp_local();
            self.emit_private_element_find(
                receiver_payload_local,
                token_local,
                existing_entry_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(existing_entry_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error_to_active_handler(
                TYPE_ERROR_NAME,
                "private element already installed on object",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            self.release_temp_local(existing_entry_local);
        }

        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            previous_local,
            function,
        );
        self.emit_heap_alloc_const(HEAP_PRIVATE_ELEMENT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
            previous_local,
            function,
        );
        if let Some((receiver_payload_local, _)) = receiver_locals {
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
                receiver_payload_local,
                function,
            );
        } else {
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
                0,
                function,
            );
        }
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
            token_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            kind.wire_word(),
            function,
        );
        if let Some((value_payload_local, value_tag_local)) = value_locals {
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
                value_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
                value_payload_local,
                function,
            );
        } else {
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
                function,
            );
            self.store_i64_const_at_offset(
                entry_local,
                HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
                0,
                function,
            );
        }
        self.store_i64_local_at_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            entry_local,
            function,
        );

        self.release_temp_local(entry_local);
        self.release_temp_local(previous_local);
        self.release_temp_local(realm_local);
        Ok(())
    }

    fn emit_private_receiver_kind_guard(kind_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::Brand.wire_word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::Field.wire_word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    fn emit_private_definition_kind_guard(kind_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::SetterDefinition.wire_word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::MethodDefinition.wire_word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::GetterDefinition.wire_word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_private_element_find(
        &mut self,
        receiver_local: u32,
        token_local: u32,
        entry_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let stored_receiver_local = self.reserve_temp_local();
        let stored_token_local = self.reserve_temp_local();
        let stored_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            entry_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
            stored_receiver_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
            stored_token_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(stored_receiver_local));
        function.instruction(&Instruction::LocalGet(receiver_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(stored_token_local));
        function.instruction(&Instruction::LocalGet(token_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            stored_kind_local,
            function,
        );
        Self::emit_private_receiver_kind_guard(stored_kind_local, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
            entry_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_kind_local);
        self.release_temp_local(stored_token_local);
        self.release_temp_local(stored_receiver_local);
        self.release_temp_local(realm_local);
    }

    fn emit_private_element_definition_find(
        &mut self,
        token_local: u32,
        kind: PrivateElementDefinitionKind,
        entry_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let stored_receiver_local = self.reserve_temp_local();
        let stored_token_local = self.reserve_temp_local();
        let stored_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_PRIVATE_ELEMENTS_OFFSET,
            entry_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_RECEIVER_OFFSET,
            stored_receiver_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_TOKEN_OFFSET,
            stored_token_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            stored_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(stored_token_local));
        function.instruction(&Instruction::LocalGet(token_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(stored_receiver_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_private_definition_kind_guard(stored_kind_local, function);
        function.instruction(&Instruction::LocalGet(stored_kind_local));
        function.instruction(&Instruction::I64Const(kind.heap_kind().wire_word() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_NEXT_OFFSET,
            entry_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_kind_local);
        self.release_temp_local(stored_token_local);
        self.release_temp_local(stored_receiver_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_private_brand_has_i32(
        &mut self,
        receiver_local: u32,
        token_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let entry_local = self.reserve_temp_local();
        self.emit_private_element_find(receiver_local, token_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));
        self.release_temp_local(entry_local);
    }

    pub(crate) fn compile_private_read_to_locals(
        &mut self,
        target: &TypedExpr,
        private_name_id: PrivateNameId,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.emit_private_read_from_locals(
            target_payload_local,
            target_tag_local,
            private_name_id,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    fn emit_private_read_from_locals(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        private_name_id: PrivateNameId,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_token_local = self.reserve_temp_local();
        let has_brand_local = self.reserve_temp_local();
        let private_entry_local = self.reserve_temp_local();
        let private_kind_local = self.reserve_temp_local();
        let definition_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();

        self.emit_private_brand_guard(
            target_payload_local,
            target_tag_local,
            private_name_id,
            brand_token_local,
            has_brand_local,
            function,
        )?;
        self.emit_private_element_find(
            target_payload_local,
            brand_token_local,
            private_entry_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            private_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(private_kind_local));
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::Field.wire_word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_private_element_definition_find(
            brand_token_local,
            PrivateElementDefinitionKind::Method,
            definition_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(definition_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_private_element_definition_find(
            brand_token_local,
            PrivateElementDefinitionKind::Getter,
            definition_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(definition_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "private accessor has no getter",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::GetterDefinition.wire_word() as i64,
        ));
        function.instruction(&Instruction::LocalSet(private_kind_local));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::MethodDefinition.wire_word() as i64,
        ));
        function.instruction(&Instruction::LocalSet(private_kind_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(private_kind_local));
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::GetterDefinition.wire_word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call_with_throw_propagation(
            getter_payload_local,
            getter_tag_local,
            Some((target_payload_local, Some(target_tag_local))),
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(definition_local);
        self.release_temp_local(private_kind_local);
        self.release_temp_local(private_entry_local);
        self.release_temp_local(has_brand_local);
        self.release_temp_local(brand_token_local);
        Ok(())
    }

    pub(crate) fn compile_private_write_to_locals(
        &mut self,
        target: &TypedExpr,
        private_name_id: PrivateNameId,
        value: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(target, target_payload_local, target_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            target_payload_local,
            target_tag_local,
            function,
        )?;
        self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        self.emit_private_write_from_locals(
            target_payload_local,
            target_tag_local,
            private_name_id,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_private_write_from_locals(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        private_name_id: PrivateNameId,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_token_local = self.reserve_temp_local();
        let has_brand_local = self.reserve_temp_local();
        let private_entry_local = self.reserve_temp_local();
        let private_kind_local = self.reserve_temp_local();
        let setter_definition_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let setter_result_payload_local = self.reserve_temp_local();
        let setter_result_tag_local = self.reserve_temp_local();

        self.emit_private_brand_guard(
            target_payload_local,
            target_tag_local,
            private_name_id,
            brand_token_local,
            has_brand_local,
            function,
        )?;
        self.emit_private_element_find(
            target_payload_local,
            brand_token_local,
            private_entry_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_KIND_OFFSET,
            private_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(private_kind_local));
        function.instruction(&Instruction::I64Const(
            PrivateElementHeapKind::Field.wire_word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_i64_local_at_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            private_entry_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_private_element_definition_find(
            brand_token_local,
            PrivateElementDefinitionKind::Setter,
            setter_definition_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(setter_definition_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "private element has no setter",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            setter_definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_PAYLOAD_OFFSET,
            setter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            setter_definition_local,
            HEAP_PRIVATE_ELEMENT_ENTRY_VALUE_TAG_OFFSET,
            setter_tag_local,
            function,
        );
        self.emit_function_handle_call_with_throw_propagation(
            setter_payload_local,
            setter_tag_local,
            Some((target_payload_local, Some(target_tag_local))),
            &[(value_payload_local, value_tag_local)],
            setter_result_payload_local,
            setter_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(setter_result_tag_local);
        self.release_temp_local(setter_result_payload_local);
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(setter_definition_local);
        self.release_temp_local(private_kind_local);
        self.release_temp_local(private_entry_local);
        self.release_temp_local(has_brand_local);
        self.release_temp_local(brand_token_local);
        Ok(())
    }

    pub(crate) fn emit_private_brand_guard(
        &mut self,
        target_payload_local: u32,
        target_tag_local: u32,
        private_name_id: PrivateNameId,
        brand_token_local: u32,
        has_brand_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_private_name_token_to_local(private_name_id, brand_token_local, function)?;
        self.emit_private_brand_has_i32(
            target_payload_local,
            brand_token_local,
            has_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(has_brand_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "private field access on wrong object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            "TypeError",
            "private field access on wrong object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        function.instruction(&Instruction::End);
        Ok(())
    }
}
