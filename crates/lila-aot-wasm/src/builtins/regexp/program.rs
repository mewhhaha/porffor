use super::*;
use lila_ir::{
    RegExpProgramWord, REGEXP_MAX_INSTRUCTIONS, REGEXP_MAX_RANGE_ENTRIES,
    REGEXP_PROGRAM_HEADER_SIZE, REGEXP_PROGRAM_MAGIC_VERSION,
};

/// Decoded sections retain the allocation extent which authorized their reads.
/// Counts never come from independently mutable RegExp object fields.
pub(crate) struct RegExpProgramLayoutLocals {
    pub(crate) allocation: u32,
    pub(crate) byte_length: u32,
    pub(crate) end: u32,
    pub(crate) instructions: u32,
    pub(crate) instruction_count: u32,
    pub(crate) capture_count: u32,
    pub(crate) ranges: u32,
    pub(crate) range_count: u32,
    pub(crate) range_end: u32,
    pub(crate) split_count: u32,
    pub(crate) repeatable_split_count: u32,
    pub(crate) named_groups: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn reserve_regexp_program_layout(&mut self) -> RegExpProgramLayoutLocals {
        RegExpProgramLayoutLocals {
            allocation: self.reserve_temp_local(),
            byte_length: self.reserve_temp_local(),
            end: self.reserve_temp_local(),
            instructions: self.reserve_temp_local(),
            instruction_count: self.reserve_temp_local(),
            capture_count: self.reserve_temp_local(),
            ranges: self.reserve_temp_local(),
            range_count: self.reserve_temp_local(),
            range_end: self.reserve_temp_local(),
            split_count: self.reserve_temp_local(),
            repeatable_split_count: self.reserve_temp_local(),
            named_groups: self.reserve_temp_local(),
        }
    }

    pub(crate) fn release_regexp_program_layout(&mut self, layout: RegExpProgramLayoutLocals) {
        for local in [
            layout.named_groups,
            layout.repeatable_split_count,
            layout.split_count,
            layout.range_end,
            layout.range_count,
            layout.ranges,
            layout.capture_count,
            layout.instruction_count,
            layout.instructions,
            layout.end,
            layout.byte_length,
            layout.allocation,
        ] {
            self.release_temp_local(local);
        }
    }

    /// Produces valid=0 without dereferencing an invalid allocation or section.
    /// The caller routes corrupt-artifact failure before consuming the layout.
    pub(crate) fn emit_regexp_program_layout(
        &mut self,
        handle: u32,
        layout: &RegExpProgramLayoutLocals,
        valid: u32,
        function: &mut Function,
    ) {
        let word = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(handle));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(layout.allocation));
        function.instruction(&Instruction::LocalGet(handle));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(layout.byte_length));
        function.instruction(&Instruction::LocalGet(layout.allocation));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(layout.allocation));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(layout.byte_length));
        function.instruction(&Instruction::I64Const(REGEXP_PROGRAM_HEADER_SIZE as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(layout.allocation));
        function.instruction(&Instruction::LocalGet(layout.byte_length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(layout.end));
        function.instruction(&Instruction::MemorySize(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(0));
        self.load_i64_to_local_from_offset(
            layout.allocation,
            RegExpProgramWord::MagicVersion.offset(),
            word,
            function,
        );
        function.instruction(&Instruction::LocalGet(word));
        function.instruction(&Instruction::I64Const(REGEXP_PROGRAM_MAGIC_VERSION as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));
        self.load_i64_to_local_from_offset(
            layout.allocation,
            RegExpProgramWord::ByteLength.offset(),
            word,
            function,
        );
        function.instruction(&Instruction::LocalGet(word));
        function.instruction(&Instruction::LocalGet(layout.byte_length));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));
        for (field, local) in [
            (
                RegExpProgramWord::InstructionCount,
                layout.instruction_count,
            ),
            (RegExpProgramWord::CaptureCount, layout.capture_count),
            (RegExpProgramWord::RangeCount, layout.range_count),
            (RegExpProgramWord::SplitCount, layout.split_count),
            (
                RegExpProgramWord::RepeatableSplitCount,
                layout.repeatable_split_count,
            ),
            (
                RegExpProgramWord::NamedGroupTableOffset,
                layout.named_groups,
            ),
        ] {
            self.load_i64_to_local_from_offset(layout.allocation, field.offset(), local, function);
        }
        function.instruction(&Instruction::LocalGet(layout.instruction_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(layout.instruction_count));
        function.instruction(&Instruction::I64Const(REGEXP_MAX_INSTRUCTIONS as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(layout.capture_count));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(layout.range_count));
        function.instruction(&Instruction::I64Const(REGEXP_MAX_RANGE_ENTRIES as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(layout.split_count));
        function.instruction(&Instruction::LocalGet(layout.instruction_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(layout.repeatable_split_count));
        function.instruction(&Instruction::LocalGet(layout.split_count));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(layout.allocation));
        function.instruction(&Instruction::I64Const(REGEXP_PROGRAM_HEADER_SIZE as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(layout.instructions));
        function.instruction(&Instruction::LocalGet(layout.instruction_count));
        function.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(layout.ranges));
        function.instruction(&Instruction::LocalGet(layout.range_count));
        function.instruction(&Instruction::I64Const(REGEXP_RANGE_ENTRY_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(layout.range_end));
        function.instruction(&Instruction::LocalGet(layout.end));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(layout.named_groups));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(layout.range_end));
        function.instruction(&Instruction::LocalGet(layout.end));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(layout.named_groups));
        function.instruction(&Instruction::LocalGet(layout.range_end));
        function.instruction(&Instruction::LocalGet(layout.allocation));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(layout.end));
        function.instruction(&Instruction::LocalGet(layout.range_end));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(layout.named_groups));
        function.instruction(&Instruction::LocalGet(layout.allocation));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(layout.named_groups));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(valid));
        function.instruction(&Instruction::End);
        self.release_temp_local(word);
    }
}
