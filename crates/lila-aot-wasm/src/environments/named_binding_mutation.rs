use super::global_environment::GlobalBindingFailure;
use super::named_environment::*;
use super::*;

impl FunctionBuilder<'_> {
    /// CreateMutableBinding(name, true) followed by InitializeBinding. Existing
    /// bindings keep both their cell identity and deletion attributes.
    pub(crate) fn emit_create_eval_variable_binding(
        &mut self,
        environment_local: u32,
        key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entry_local = self.reserve_temp_local();
        self.emit_find_own_named_binding(environment_local, key_local, entry_local, function);
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let old_entries_local = self.reserve_temp_local();
        let old_count_local = self.reserve_temp_local();
        let new_entries_local = self.reserve_temp_local();
        let byte_count_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let source_local = self.reserve_temp_local();
        let word_local = self.reserve_temp_local();
        let cell_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_NAMED_ENTRIES_OFFSET,
            old_entries_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_NAMED_COUNT_OFFSET,
            old_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(old_count_local));
        function.instruction(&Instruction::I64Const(NAMED_BINDING_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(byte_count_local));
        function.instruction(&Instruction::LocalGet(byte_count_local));
        function.instruction(&Instruction::I64Const(NAMED_BINDING_SIZE as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(word_local));
        self.emit_heap_alloc_from_local(word_local, function)?;
        function.instruction(&Instruction::LocalSet(new_entries_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(byte_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(old_entries_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_local));
        self.load_i64_to_local_from_offset(source_local, 0, word_local, function);
        function.instruction(&Instruction::LocalGet(new_entries_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, 0, word_local, function);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_entries_local));
        function.instruction(&Instruction::LocalGet(byte_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.emit_heap_alloc_const(ENV_SLOT_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(cell_local));
        self.store_i64_local_at_offset(
            cell_local,
            ENV_SLOT_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(cell_local, ENV_SLOT_TAG_OFFSET, value_tag_local, function);
        self.store_i64_local_at_offset(entry_local, NAMED_BINDING_KEY_OFFSET, key_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            NAMED_BINDING_CELL_OFFSET,
            cell_local,
            function,
        );
        for (offset, value) in [
            (NAMED_BINDING_MUTABLE_OFFSET, 1),
            (NAMED_BINDING_DELETABLE_OFFSET, 1),
            (NAMED_BINDING_PRESENT_OFFSET, 1),
            (NAMED_BINDING_LEXICAL_CONFLICT_OFFSET, 0),
            (NAMED_BINDING_IMMUTABLE_STRICT_OFFSET, 0),
        ] {
            self.store_i64_const_at_offset(entry_local, offset, value, function);
        }
        self.store_i64_local_at_offset(
            environment_local,
            ENV_NAMED_ENTRIES_OFFSET,
            new_entries_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(old_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_count_local));
        self.store_i64_local_at_offset(
            environment_local,
            ENV_NAMED_COUNT_OFFSET,
            old_count_local,
            function,
        );
        self.release_temp_local(cell_local);
        self.release_temp_local(word_local);
        self.release_temp_local(source_local);
        self.release_temp_local(cursor_local);
        self.release_temp_local(byte_count_local);
        self.release_temp_local(new_entries_local);
        self.release_temp_local(old_count_local);
        self.release_temp_local(old_entries_local);
        function.instruction(&Instruction::End);
        self.release_temp_local(entry_local);
        Ok(())
    }

    pub(crate) fn emit_set_named_environment_binding(
        &mut self,
        environment_local: u32,
        key_local: u32,
        strictness: Strictness,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entry_local = self.reserve_temp_local();
        let cell_local = self.reserve_temp_local();
        let previous_payload_local = self.reserve_temp_local();
        let previous_tag_local = self.reserve_temp_local();
        let mutable_local = self.reserve_temp_local();
        self.emit_find_own_named_binding(environment_local, key_local, entry_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        match strictness {
            Strictness::Strict => {
                self.emit_throw_global_binding_error(
                    GlobalBindingFailure::UnresolvableAssignment,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_current_completion_if_throw(function);
            }
            Strictness::Sloppy => {
                self.emit_create_eval_variable_binding(
                    environment_local,
                    key_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
            }
        }
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_global_lexical_read(
            entry_local,
            previous_payload_local,
            previous_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            entry_local,
            NAMED_BINDING_MUTABLE_OFFSET,
            mutable_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mutable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if strictness == Strictness::Sloppy {
            self.load_i64_to_local_from_offset(
                entry_local,
                NAMED_BINDING_IMMUTABLE_STRICT_OFFSET,
                mutable_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(mutable_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::BrIf(1));
        }
        self.emit_throw_global_binding_error(
            GlobalBindingFailure::Immutable,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            entry_local,
            NAMED_BINDING_CELL_OFFSET,
            cell_local,
            function,
        );
        self.store_i64_local_at_offset(
            cell_local,
            ENV_SLOT_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(cell_local, ENV_SLOT_TAG_OFFSET, value_tag_local, function);
        function.instruction(&Instruction::End);
        self.release_temp_local(mutable_local);
        self.release_temp_local(previous_tag_local);
        self.release_temp_local(previous_payload_local);
        self.release_temp_local(cell_local);
        self.release_temp_local(entry_local);
        Ok(())
    }
}
