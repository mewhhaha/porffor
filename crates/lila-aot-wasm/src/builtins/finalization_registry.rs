use super::super::*;
use crate::functions::NewTargetPrototypeFallback;

enum FinalizationRegistryTypeError {
    ConstructorRequiresNew,
    CleanupCallbackNotCallable,
    TargetCannotBeHeldWeakly,
    TargetMatchesHoldings,
    UnregisterTokenCannotBeHeldWeakly,
    ReceiverMissingCells,
}

impl FinalizationRegistryTypeError {
    fn message(self) -> &'static str {
        match self {
            Self::ConstructorRequiresNew => "FinalizationRegistry constructor requires new",
            Self::CleanupCallbackNotCallable => {
                "FinalizationRegistry cleanup callback is not callable"
            }
            Self::TargetCannotBeHeldWeakly => "FinalizationRegistry target cannot be held weakly",
            Self::TargetMatchesHoldings => {
                "FinalizationRegistry target and holdings must not be the same value"
            }
            Self::UnregisterTokenCannotBeHeldWeakly => {
                "FinalizationRegistry unregister token cannot be held weakly"
            }
            Self::ReceiverMissingCells => {
                "FinalizationRegistry method receiver does not have [[Cells]]"
            }
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_finalization_registry_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let cleanup_callback_payload_local = self.reserve_temp_local();
        let cleanup_callback_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let registry_payload_local = self.reserve_temp_local();
        let registry_record_local = self.reserve_temp_local();
        let cells_ptr_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalization_registry_type_error(
            FinalizationRegistryTypeError::ConstructorRequiresNew,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(
            0,
            cleanup_callback_payload_local,
            cleanup_callback_tag_local,
            function,
        );
        self.emit_is_callable_i32(
            cleanup_callback_tag_local,
            cleanup_callback_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalization_registry_type_error(
            FinalizationRegistryTypeError::CleanupCallbackNotCallable,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_new_target_prototype_to_locals(
            FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
            NewTargetPrototypeFallback::RealmIntrinsic(
                HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET,
            ),
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(prototype_payload_local),
            Some(prototype_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(registry_payload_local));
        self.emit_heap_alloc_const(HEAP_FINALIZATION_REGISTRY_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(registry_record_local));
        self.emit_heap_alloc_const(
            MIN_HEAP_CAPACITY * HEAP_FINALIZATION_REGISTRY_CELL_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(cells_ptr_local));
        self.store_i64_local_at_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_TAG_OFFSET,
            cleanup_callback_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CLEANUP_CALLBACK_PAYLOAD_OFFSET,
            cleanup_callback_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,
            cells_ptr_local,
            function,
        );
        self.store_i64_const_at_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET,
            MIN_HEAP_CAPACITY,
            function,
        );
        self.store_i64_const_at_offset(
            registry_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_FINALIZATION_REGISTRY,
            function,
        );
        self.store_i64_local_at_offset(
            registry_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            registry_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(registry_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(cells_ptr_local);
        self.release_temp_local(registry_record_local);
        self.release_temp_local(registry_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(cleanup_callback_tag_local);
        self.release_temp_local(cleanup_callback_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_finalization_registry_register(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let registry_record_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let holdings_payload_local = self.reserve_temp_local();
        let holdings_tag_local = self.reserve_temp_local();
        let token_payload_local = self.reserve_temp_local();
        let token_tag_local = self.reserve_temp_local();
        let cells_ptr_local = self.reserve_temp_local();
        let cells_len_local = self.reserve_temp_local();
        let cells_cap_local = self.reserve_temp_local();
        let new_cap_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let new_cells_ptr_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_cell_local = self.reserve_temp_local();
        let new_cell_local = self.reserve_temp_local();
        let copied_value_local = self.reserve_temp_local();
        let cell_local = self.reserve_temp_local();

        self.emit_finalization_registry_record_from_receiver(registry_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_can_be_held_weakly_i32(target_payload_local, target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalization_registry_type_error(
            FinalizationRegistryTypeError::TargetCannotBeHeldWeakly,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(1, holdings_payload_local, holdings_tag_local, function);
        self.emit_tagged_payload_same_value_i32(
            target_tag_local,
            target_payload_local,
            holdings_tag_local,
            holdings_payload_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalization_registry_type_error(
            FinalizationRegistryTypeError::TargetMatchesHoldings,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(2, token_payload_local, token_tag_local, function);
        function.instruction(&Instruction::LocalGet(token_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_can_be_held_weakly_i32(token_payload_local, token_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalization_registry_type_error(
            FinalizationRegistryTypeError::UnregisterTokenCannotBeHeldWeakly,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,
            cells_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET,
            cells_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET,
            cells_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cells_len_local));
        function.instruction(&Instruction::LocalGet(cells_cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cells_cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(
            HEAP_FINALIZATION_REGISTRY_CELL_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_cells_ptr_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cells_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        for (cells_base_local, indexed_cell_local) in [
            (cells_ptr_local, old_cell_local),
            (new_cells_ptr_local, new_cell_local),
        ] {
            function.instruction(&Instruction::LocalGet(cells_base_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(
                HEAP_FINALIZATION_REGISTRY_CELL_SIZE as i64,
            ));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(indexed_cell_local));
        }
        self.load_i64_to_local_from_offset(
            old_cell_local,
            HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,
            copied_value_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        for cell_state in FinalizationRegistryCellState::ALL {
            function.instruction(&Instruction::LocalGet(copied_value_local));
            function.instruction(&Instruction::I64Const(cell_state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            match cell_state {
                FinalizationRegistryCellState::Vacant | FinalizationRegistryCellState::Occupied => {
                    self.store_finalization_registry_cell_state(
                        new_cell_local,
                        cell_state,
                        function,
                    );
                }
            }
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        for offset in [
            HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET,
            HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET,
            HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET,
            HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET,
            HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,
            HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(
                old_cell_local,
                offset,
                copied_value_local,
                function,
            );
            self.store_i64_local_at_offset(new_cell_local, offset, copied_value_local, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,
            new_cells_ptr_local,
            function,
        );
        self.store_i64_local_at_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_CAP_OFFSET,
            new_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(new_cells_ptr_local));
        function.instruction(&Instruction::LocalSet(cells_ptr_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cells_ptr_local));
        function.instruction(&Instruction::LocalGet(cells_len_local));
        function.instruction(&Instruction::I64Const(
            HEAP_FINALIZATION_REGISTRY_CELL_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cell_local));
        self.store_finalization_registry_cell_state(
            cell_local,
            &FinalizationRegistryCellState::Occupied,
            function,
        );
        for (offset, value_local) in [
            (
                HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET,
                target_tag_local,
            ),
            (
                HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET,
                target_payload_local,
            ),
            (
                HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET,
                holdings_tag_local,
            ),
            (
                HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET,
                holdings_payload_local,
            ),
            (
                HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,
                token_tag_local,
            ),
            (
                HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,
                token_payload_local,
            ),
        ] {
            self.store_i64_local_at_offset(cell_local, offset, value_local, function);
        }
        function.instruction(&Instruction::LocalGet(cells_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cells_len_local));
        self.store_i64_local_at_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET,
            cells_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(cell_local);
        self.release_temp_local(copied_value_local);
        self.release_temp_local(new_cell_local);
        self.release_temp_local(old_cell_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_cells_ptr_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(new_cap_local);
        self.release_temp_local(cells_cap_local);
        self.release_temp_local(cells_len_local);
        self.release_temp_local(cells_ptr_local);
        self.release_temp_local(token_tag_local);
        self.release_temp_local(token_payload_local);
        self.release_temp_local(holdings_tag_local);
        self.release_temp_local(holdings_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(registry_record_local);
        Ok(())
    }

    pub(crate) fn emit_finalization_registry_unregister(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let registry_record_local = self.reserve_temp_local();
        let token_payload_local = self.reserve_temp_local();
        let token_tag_local = self.reserve_temp_local();
        let cells_ptr_local = self.reserve_temp_local();
        let cells_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let cell_local = self.reserve_temp_local();
        let cell_state_local = self.reserve_temp_local();
        let stored_token_payload_local = self.reserve_temp_local();
        let stored_token_tag_local = self.reserve_temp_local();

        self.emit_finalization_registry_record_from_receiver(registry_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, token_payload_local, token_tag_local, function);
        self.emit_can_be_held_weakly_i32(token_payload_local, token_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalization_registry_type_error(
            FinalizationRegistryTypeError::UnregisterTokenCannotBeHeldWeakly,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.load_i64_to_local_from_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_PTR_OFFSET,
            cells_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            registry_record_local,
            HEAP_FINALIZATION_REGISTRY_CELLS_LEN_OFFSET,
            cells_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(cells_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(cells_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(
            HEAP_FINALIZATION_REGISTRY_CELL_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cell_local));
        self.load_i64_to_local_from_offset(
            cell_local,
            HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,
            cell_state_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        for cell_state in FinalizationRegistryCellState::ALL {
            function.instruction(&Instruction::LocalGet(cell_state_local));
            function.instruction(&Instruction::I64Const(cell_state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            match cell_state {
                FinalizationRegistryCellState::Vacant => {}
                FinalizationRegistryCellState::Occupied => {
                    self.load_i64_to_local_from_offset(
                        cell_local,
                        HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,
                        stored_token_tag_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        cell_local,
                        HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,
                        stored_token_payload_local,
                        function,
                    );
                    self.emit_tagged_payload_same_value_i32(
                        stored_token_tag_local,
                        stored_token_payload_local,
                        token_tag_local,
                        token_payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.store_finalization_registry_cell_state(
                        cell_local,
                        &FinalizationRegistryCellState::Vacant,
                        function,
                    );
                    for offset in [
                        HEAP_FINALIZATION_REGISTRY_CELL_TARGET_TAG_OFFSET,
                        HEAP_FINALIZATION_REGISTRY_CELL_TARGET_PAYLOAD_OFFSET,
                        HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_TAG_OFFSET,
                        HEAP_FINALIZATION_REGISTRY_CELL_HOLDINGS_PAYLOAD_OFFSET,
                        HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_TAG_OFFSET,
                        HEAP_FINALIZATION_REGISTRY_CELL_TOKEN_PAYLOAD_OFFSET,
                    ] {
                        self.store_i64_const_at_offset(cell_local, offset, 0, function);
                    }
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::End);
                }
            }
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(stored_token_tag_local);
        self.release_temp_local(stored_token_payload_local);
        self.release_temp_local(cell_state_local);
        self.release_temp_local(cell_local);
        self.release_temp_local(index_local);
        self.release_temp_local(cells_len_local);
        self.release_temp_local(cells_ptr_local);
        self.release_temp_local(token_tag_local);
        self.release_temp_local(token_payload_local);
        self.release_temp_local(registry_record_local);
        Ok(())
    }

    fn store_finalization_registry_cell_state(
        &mut self,
        cell_local: u32,
        state: &FinalizationRegistryCellState,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            cell_local,
            HEAP_FINALIZATION_REGISTRY_CELL_STATE_OFFSET,
            state.word(),
            function,
        );
    }

    fn emit_finalization_registry_record_from_receiver(
        &mut self,
        registry_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalization_registry_type_error(
            FinalizationRegistryTypeError::ReceiverMissingCells,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_FINALIZATION_REGISTRY as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalization_registry_type_error(
            FinalizationRegistryTypeError::ReceiverMissingCells,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            registry_record_local,
            function,
        );

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    fn emit_finalization_registry_type_error(
        &mut self,
        error: FinalizationRegistryTypeError,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            error.message(),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}

macro_rules! finalization_registry_cell_state_domain {
    ($($variant:ident = $word:literal),+ $(,)?) => {
        enum FinalizationRegistryCellState {
            $($variant),+
        }

        impl FinalizationRegistryCellState {
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            const fn word(&self) -> u64 {
                match self {
                    $(Self::$variant => $word),+
                }
            }
        }
    };
}

finalization_registry_cell_state_domain! {
    Vacant = 0,
    Occupied = 1,
}
