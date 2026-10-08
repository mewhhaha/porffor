//! Temporary parser/backtracking bytes; no JavaScript value is represented here.
use super::compiler::CompilerLocals;
use super::*;
use crate::gc_types::{
    CodeUnitArray, CompletionLocals, GcLocal, I64Local, NonNullable, StringValue, StringValueSchema,
};
use crate::runtime_helpers::TransientByteAllocArguments;

pub(super) enum ScratchFailure<'a> {
    Matcher {
        checkpoint: I64Local,
        start: I64Local,
    },
    Compiler(&'a CompilerLocals),
    CaptureOutput {
        completion: &'a CompletionLocals,
        exit: ControlTarget,
    },
}
impl ScratchFailure<'_> {
    pub(super) fn emit(
        &self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self {
            Self::Matcher { checkpoint, start } => {
                builder.emit_regexp_match_result(
                    *checkpoint,
                    *start,
                    *start,
                    RegExpMatcherResult::Failed(RegExpMatcherFailure::ResourceExhausted),
                    function,
                );
                function.instruction(&Instruction::Return);
            }
            Self::Compiler(compiler) => {
                builder.emit_regexp_compile_scratch_failure(compiler, function)
            }
            Self::CaptureOutput { completion, exit } => {
                builder.emit_throw_current_function_realm_error(
                    NativeErrorKind::RangeError,
                    RuntimeErrorMessage::REGEXP_MATCHER_SCRATCH_ARENA_EXCEEDS_THE_ENGINE_ADDRESSABLE_RESOURCE_LIMIT,
                    completion,
                    function,
                )?;
                builder.emit_branch_to_target(*exit, function);
            }
        }
        Ok(())
    }
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_scratch_increment(
        &self,
        local: I64Local,
        delta: i64,
        function: &mut Function,
    ) {
        local.load(function);
        function.instruction(&Instruction::I64Const(delta));
        function.instruction(&Instruction::I64Add);
        local.store(function);
    }

    pub(super) fn emit_regexp_scratch_rewind(&self, checkpoint: I64Local, function: &mut Function) {
        checkpoint.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        checkpoint.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        checkpoint.load(function);
        function.instruction(&Instruction::GlobalSet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
    }

    pub(super) fn emit_regexp_scratch_byte(
        &self,
        src_offset_local: I64Local,
        index_local: I64Local,
        byte_local: I64Local,
        function: &mut Function,
    ) {
        src_offset_local.load(function);
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        byte_local.store(function);
    }

    pub(super) fn emit_regexp_scratch_byte_at_delta(
        &self,
        src_offset_local: I64Local,
        index_local: I64Local,
        delta: i64,
        byte_local: I64Local,
        function: &mut Function,
    ) {
        src_offset_local.load(function);
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(delta));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        byte_local.store(function);
    }

    pub(super) fn emit_regexp_scratch_decode_scalar(
        &self,
        src_offset_local: I64Local,
        index_local: I64Local,
        src_len_local: I64Local,
        first_byte_local: I64Local,
        codepoint_local: I64Local,
        advance_local: I64Local,
        temp_local: I64Local,
        function: &mut Function,
    ) {
        first_byte_local.load(function);
        codepoint_local.store(function);
        function.instruction(&Instruction::I64Const(1));
        advance_local.store(function);

        first_byte_local.load(function);
        function.instruction(&Instruction::I64Const(0xC0));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        first_byte_local.load(function);
        function.instruction(&Instruction::I64Const(0xE0));
        function.instruction(&Instruction::I64LtU);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        src_len_local.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_byte_at_delta(
            src_offset_local,
            index_local,
            1,
            temp_local,
            function,
        );
        first_byte_local.load(function);
        function.instruction(&Instruction::I64Const(0x1F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Shl);
        temp_local.load(function);
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        codepoint_local.store(function);
        function.instruction(&Instruction::I64Const(2));
        advance_local.store(function);
        function.instruction(&Instruction::Else);
        first_byte_local.load(function);
        function.instruction(&Instruction::I64Const(0xF0));
        function.instruction(&Instruction::I64LtU);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        src_len_local.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        first_byte_local.load(function);
        function.instruction(&Instruction::I64Const(0x0F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Shl);
        codepoint_local.store(function);
        self.emit_regexp_scratch_byte_at_delta(
            src_offset_local,
            index_local,
            1,
            temp_local,
            function,
        );
        codepoint_local.load(function);
        temp_local.load(function);
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        codepoint_local.store(function);
        self.emit_regexp_scratch_byte_at_delta(
            src_offset_local,
            index_local,
            2,
            temp_local,
            function,
        );
        codepoint_local.load(function);
        temp_local.load(function);
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        codepoint_local.store(function);
        function.instruction(&Instruction::I64Const(3));
        advance_local.store(function);
        function.instruction(&Instruction::Else);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        src_len_local.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        first_byte_local.load(function);
        function.instruction(&Instruction::I64Const(0x07));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(18));
        function.instruction(&Instruction::I64Shl);
        codepoint_local.store(function);
        for (delta, shift) in [(1, 12), (2, 6), (3, 0)] {
            self.emit_regexp_scratch_byte_at_delta(
                src_offset_local,
                index_local,
                delta,
                temp_local,
                function,
            );
            codepoint_local.load(function);
            temp_local.load(function);
            function.instruction(&Instruction::I64Const(0x3F));
            function.instruction(&Instruction::I64And);
            if shift != 0 {
                function.instruction(&Instruction::I64Const(shift));
                function.instruction(&Instruction::I64Shl);
            }
            function.instruction(&Instruction::I64Or);
            codepoint_local.store(function);
        }
        function.instruction(&Instruction::I64Const(4));
        advance_local.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_regexp_scratch_store_byte(
        &self,
        offset_local: I64Local,
        byte_local: I64Local,
        function: &mut Function,
    ) {
        offset_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        byte_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    pub(super) fn emit_regexp_scratch_store_codepoint(
        &self,
        dst_pos_local: I64Local,
        codepoint_local: I64Local,
        temp_local: I64Local,
        function: &mut Function,
    ) {
        codepoint_local.load(function);
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_store_byte(dst_pos_local, codepoint_local, function);
        self.emit_regexp_scratch_increment(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        codepoint_local.load(function);
        function.instruction(&Instruction::I64Const(0x800));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_store_head(
            dst_pos_local,
            codepoint_local,
            temp_local,
            6,
            0xC0,
            function,
        );
        self.emit_regexp_scratch_store_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            0,
            function,
        );
        function.instruction(&Instruction::Else);
        codepoint_local.load(function);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_store_head(
            dst_pos_local,
            codepoint_local,
            temp_local,
            12,
            0xE0,
            function,
        );
        self.emit_regexp_scratch_store_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            6,
            function,
        );
        self.emit_regexp_scratch_store_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            0,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_regexp_scratch_store_head(
            dst_pos_local,
            codepoint_local,
            temp_local,
            18,
            0xF0,
            function,
        );
        self.emit_regexp_scratch_store_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            12,
            function,
        );
        self.emit_regexp_scratch_store_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            6,
            function,
        );
        self.emit_regexp_scratch_store_tail(
            dst_pos_local,
            codepoint_local,
            temp_local,
            0,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_regexp_scratch_store_head(
        &self,
        dst_pos_local: I64Local,
        codepoint_local: I64Local,
        temp_local: I64Local,
        shift: i64,
        mask: i64,
        function: &mut Function,
    ) {
        codepoint_local.load(function);
        function.instruction(&Instruction::I64Const(shift));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(mask));
        function.instruction(&Instruction::I64Or);
        temp_local.store(function);
        self.emit_regexp_scratch_store_byte(dst_pos_local, temp_local, function);
        self.emit_regexp_scratch_increment(dst_pos_local, 1, function);
    }

    pub(super) fn emit_regexp_scratch_store_tail(
        &self,
        dst_pos_local: I64Local,
        codepoint_local: I64Local,
        temp_local: I64Local,
        shift: i64,
        function: &mut Function,
    ) {
        codepoint_local.load(function);
        if shift != 0 {
            function.instruction(&Instruction::I64Const(shift));
            function.instruction(&Instruction::I64ShrU);
        }
        function.instruction(&Instruction::I64Const(0x3F));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64Or);
        temp_local.store(function);
        self.emit_regexp_scratch_store_byte(dst_pos_local, temp_local, function);
        self.emit_regexp_scratch_increment(dst_pos_local, 1, function);
    }

    pub(super) fn emit_regexp_scratch_utf16_length(
        &mut self,
        src_offset_local: I64Local,
        src_len_local: I64Local,
        dst_len_local: I64Local,
        function: &mut Function,
    ) {
        let index_local = self.runtime_schema().reserve_i64_local(function);
        let byte_local = self.runtime_schema().reserve_i64_local(function);
        let codepoint_local = self.runtime_schema().reserve_i64_local(function);
        let advance_local = self.runtime_schema().reserve_i64_local(function);
        let temp_local = self.runtime_schema().reserve_i64_local(function);

        function.instruction(&Instruction::I64Const(0));
        dst_len_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        index_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        src_len_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_scratch_byte(src_offset_local, index_local, byte_local, function);
        self.emit_regexp_scratch_decode_scalar(
            src_offset_local,
            index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        dst_len_local.load(function);
        codepoint_local.load(function);
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Add);
        dst_len_local.store(function);
        index_local.load(function);
        advance_local.load(function);
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.runtime_schema()
            .release_i64_local(temp_local, function);
        self.runtime_schema()
            .release_i64_local(advance_local, function);
        self.runtime_schema()
            .release_i64_local(codepoint_local, function);
        self.runtime_schema()
            .release_i64_local(byte_local, function);
        self.runtime_schema()
            .release_i64_local(index_local, function);
    }

    pub(super) fn emit_regexp_scratch_load_word(
        &self,
        base_local: I64Local,
        offset: u64,
        dest_local: I64Local,
        function: &mut Function,
    ) {
        self.emit_regexp_scratch_load_word_stack(base_local, offset, function);
        dest_local.store(function);
    }

    pub(super) fn emit_regexp_scratch_load_word_stack(
        &self,
        base_local: I64Local,
        offset: u64,
        function: &mut Function,
    ) {
        base_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(offset)));
    }

    pub(super) fn emit_regexp_scratch_store_word_const(
        &self,
        base_local: I64Local,
        offset: u64,
        value: u64,
        function: &mut Function,
    ) {
        base_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Const(value as i64));
        function.instruction(&Instruction::I64Store(Self::memarg64(offset)));
    }

    pub(super) fn emit_regexp_scratch_store_word(
        &self,
        base_local: I64Local,
        offset: u64,
        value_local: I64Local,
        function: &mut Function,
    ) {
        base_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        value_local.load(function);
        function.instruction(&Instruction::I64Store(Self::memarg64(offset)));
    }

    pub(super) fn emit_regexp_scratch_or_byte_flag(
        &self,
        byte_local: I64Local,
        expected: u8,
        flag_local: I64Local,
        function: &mut Function,
    ) {
        flag_local.load(function);
        byte_local.load(function);
        function.instruction(&Instruction::I64Const(expected as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Or);
        flag_local.store(function);
    }

    /// Copy an immutable UTF-16 root into a bounded private WTF-8 work span.
    /// The span is valid only above the caller's saved private-byte cursor.
    pub(super) fn emit_regexp_transient_text(
        &mut self,
        input: &GcLocal<StringValue>,
        pointer: I64Local,
        byte_length: I64Local,
        failure: ScratchFailure<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let units = schema
            .reserve_gc_local::<CodeUnitArray, NonNullable>(function)
            .initialize(
                schema
                    .field(StringValueSchema::CODE_UNITS)
                    .read(input, schema, function)
                    .reference(),
                function,
            );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let next = schema.reserve_i32_local(function);
        let next_index = schema.reserve_i32_local(function);
        let size = schema.reserve_i64_local(function);
        let allocation = schema.reserve_i64_local(function);
        let position = self.runtime_schema().reserve_i64_local(function);
        let point = self.runtime_schema().reserve_i64_local(function);
        let temp = self.runtime_schema().reserve_i64_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        length.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        size.store(function);
        self.emit_regexp_transient_allocation(size, allocation, &failure, function)?;
        allocation.load(function);
        pointer.store(function);
        allocation.load(function);
        position.store(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        let finished = self.open_frame(ControlFrameKind::Block, function);
        let copy = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(finished, function);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        point.store(function);
        // Combine only a real high/low pair. Lone surrogates retain three WTF-8 bytes.
        unit.load(function);
        function.instruction(&Instruction::I32Const(0xd800));
        function.instruction(&Instruction::I32GeU);
        unit.load(function);
        function.instruction(&Instruction::I32Const(0xdbff));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        next_index.store(function);
        next_index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32LtU);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, next_index, schema, function)
            .store(next, function);
        next.load(function);
        function.instruction(&Instruction::I32Const(0xdc00));
        function.instruction(&Instruction::I32GeU);
        next.load(function);
        function.instruction(&Instruction::I32Const(0xdfff));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        unit.load(function);
        function.instruction(&Instruction::I32Const(0xd800));
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Const(10));
        function.instruction(&Instruction::I32Shl);
        next.load(function);
        function.instruction(&Instruction::I32Const(0xdc00));
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I32Const(0x10000));
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64ExtendI32U);
        point.store(function);
        next_index.load(function);
        index.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_store_codepoint(position, point, temp, function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(copy, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        position.load(function);
        pointer.load(function);
        function.instruction(&Instruction::I64Sub);
        byte_length.store(function);
        self.runtime_schema().release_i64_local(temp, function);
        self.runtime_schema().release_i64_local(point, function);
        self.runtime_schema().release_i64_local(position, function);
        schema.release_i64_local(allocation, function);
        schema.release_i64_local(size, function);
        schema.release_i32_local(next_index, function);
        schema.release_i32_local(next, function);
        schema.release_i32_local(unit, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(length, function);
        units.clear(function);
        Ok(())
    }

    pub(super) fn emit_regexp_transient_allocation(
        &mut self,
        size: I64Local,
        output: I64Local,
        failure: &ScratchFailure<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let end = schema.reserve_i64_local(function);
        let available = schema.reserve_i64_local(function);
        function.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        size.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(-8));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        end.store(function);
        end.load(function);
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, function);
        failure.emit(self, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::MemorySize(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(WASM_PAGE_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        available.store(function);
        end.load(function);
        available.load(function);
        function.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, function);
        end.load(function);
        available.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(65535));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryGrow(0));
        function.instruction(&Instruction::I32Const(-1));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        failure.emit(self, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema
            .call_helper(
                TransientByteAllocArguments::new(size),
                self.runtime_helper_base()?,
                function,
            )
            .store(output, function);
        schema.release_i64_local(available, function);
        schema.release_i64_local(end, function);
        Ok(())
    }

    pub(in crate::builtins) fn emit_regexp_capture_output_allocation(
        &mut self,
        size: I64Local,
        output: I64Local,
        completion: &CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_regexp_transient_allocation(
            size,
            output,
            &ScratchFailure::CaptureOutput { completion, exit },
            function,
        )
    }
}
