use super::*;

/// The complete run-time ParameterMap fact for one Arguments index.
///
/// Mapping presence and the environment slot are captured from the same
/// pre-mutation descriptor word. This value is deliberately non-`Copy`: a
/// descriptor update may borrow it for the post-define ParameterMap update and
/// mapping restore, then must explicitly release its two temporary locals.
#[must_use = "an Arguments index mapping must survive until the indexed operation finishes"]
pub(crate) struct ArgumentsIndexMappingLocals {
    mapped: u32,
    slot: u32,
}

impl<'a> FunctionBuilder<'a> {
    /// Captures the complete ParameterMap fact before an indexed operation may
    /// replace the descriptor word that carries it.
    pub(crate) fn emit_arguments_index_mapping_from_descriptor_word(
        &mut self,
        descriptor_word_local: u32,
        function: &mut Function,
    ) -> ArgumentsIndexMappingLocals {
        let mapped = self.reserve_temp_local();
        let slot = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(descriptor_word_local));
        function.instruction(&Instruction::I64Const(ARGUMENTS_DESCRIPTOR_MAPPED as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(mapped));
        function.instruction(&Instruction::LocalGet(descriptor_word_local));
        function.instruction(&Instruction::I64Const(MappedSlot::SHIFT as i64));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(slot));
        ArgumentsIndexMappingLocals { mapped, slot }
    }

    /// Reads the mapped environment value without rediscovering its slot from
    /// an indexed descriptor word that may already have changed.
    pub(crate) fn emit_arguments_parameter_map_read(
        &mut self,
        arguments_local: u32,
        mapping: &ArgumentsIndexMappingLocals,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let env_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mapping.mapped));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(mapping.slot));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(mapping.slot));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
        )));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(env_local);
    }

    pub(crate) fn emit_arguments_parameter_map_write(
        &mut self,
        arguments_local: u32,
        mapping: &ArgumentsIndexMappingLocals,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let env_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            arguments_local,
            HEAP_ARGUMENTS_ENV_HANDLE_OFFSET,
            env_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mapping.mapped));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(mapping.slot));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_TAG_OFFSET,
        )));
        function.instruction(&Instruction::LocalGet(env_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(mapping.slot));
        function.instruction(&Instruction::I64Const(ENV_SLOT_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
        )));
        function.instruction(&Instruction::End);
        self.release_temp_local(env_local);
    }

    /// Restores a retained mapping on a newly assembled data descriptor word.
    /// Bit 5 and its bits-32..63 slot payload are emitted together here.
    pub(crate) fn emit_arguments_mapping_restore_on_data_descriptor(
        &mut self,
        mapping: &ArgumentsIndexMappingLocals,
        descriptor_word_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(mapping.mapped));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(descriptor_word_local));
        function.instruction(&Instruction::I64Const(ARGUMENTS_DESCRIPTOR_MAPPED as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(mapping.slot));
        function.instruction(&Instruction::I64Const(MappedSlot::SHIFT as i64));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(descriptor_word_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn release_arguments_index_mapping(&mut self, mapping: ArgumentsIndexMappingLocals) {
        self.release_temp_local(mapping.slot);
        self.release_temp_local(mapping.mapped);
    }
}
