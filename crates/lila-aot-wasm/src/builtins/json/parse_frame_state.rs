use super::*;

#[must_use = "validated JSON parse frame state must drive typed dispatch"]
pub(super) struct ValidatedJsonParseFrameStateLocal(u32);

impl ValidatedJsonParseFrameStateLocal {
    const fn local(&self) -> u32 {
        self.0
    }

    const fn into_local(self) -> u32 {
        self.0
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_validate_json_parse_frame_state_local(
        &self,
        state_local: u32,
        function: &mut Function,
    ) -> ValidatedJsonParseFrameStateLocal {
        function.instruction(&Instruction::I32Const(0));
        for state in JsonParseFrameState::ALL.iter() {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        ValidatedJsonParseFrameStateLocal(state_local)
    }

    pub(super) fn emit_json_parse_frame_state_is_i32(
        &self,
        state: &ValidatedJsonParseFrameStateLocal,
        expected: JsonParseFrameState,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(state.local()));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
    }

    pub(super) fn emit_push_json_parse_frame(
        &mut self,
        frame_buffer_local: u32,
        frame_capacity_local: u32,
        frame_len_local: u32,
        payload_local: u32,
        tag_local: u32,
        state: ValidatedJsonParseFrameStateLocal,
        key_or_index_local: u32,
        metadata_payload_local: u32,
        metadata_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_capacity_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let copy_index_local = self.reserve_temp_local();
        let old_frame_local = self.reserve_temp_local();
        let new_frame_local = self.reserve_temp_local();
        let frame_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::LocalGet(frame_capacity_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frame_capacity_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_capacity_local));
        function.instruction(&Instruction::LocalGet(new_capacity_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_frame_local));
        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_frame_local));
        for offset in [
            JSON_PARSE_FRAME_PAYLOAD_OFFSET,
            JSON_PARSE_FRAME_TAG_OFFSET,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_FRAME_KEY_OR_INDEX_OFFSET,
            JSON_PARSE_FRAME_METADATA_PAYLOAD_OFFSET,
            JSON_PARSE_FRAME_METADATA_TAG_OFFSET,
        ] {
            self.load_i64_from_offset(old_frame_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_frame_local, offset, self.scratch_local, function);
        }
        self.emit_increment_local(copy_index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(new_capacity_local));
        function.instruction(&Instruction::LocalSet(frame_capacity_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frame_local));
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            state.local(),
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_KEY_OR_INDEX_OFFSET,
            key_or_index_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_METADATA_PAYLOAD_OFFSET,
            metadata_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_METADATA_TAG_OFFSET,
            metadata_tag_local,
            function,
        );
        self.emit_increment_local(frame_len_local, 1, function);

        self.release_temp_local(frame_local);
        self.release_temp_local(new_frame_local);
        self.release_temp_local(old_frame_local);
        self.release_temp_local(copy_index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(new_capacity_local);
        Ok(())
    }

    pub(super) fn release_validated_json_parse_frame_state_local(
        &mut self,
        state: ValidatedJsonParseFrameStateLocal,
    ) {
        self.release_temp_local(state.into_local());
    }
}
