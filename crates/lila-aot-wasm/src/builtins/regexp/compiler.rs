use super::*;
use crate::runtime_helpers::RegExpCompilerStatus;
use lila_ir::{
    RegExpProgramWord, REGEXP_MAX_INSTRUCTIONS, REGEXP_MAX_RANGE_ENTRIES,
    REGEXP_PROGRAM_HEADER_SIZE, REGEXP_PROGRAM_MAGIC_VERSION,
};

mod contracts;
mod lowerer;
mod parser;

use contracts::*;

impl FunctionBuilder<'_> {
    /// Compiles runtime legacy Pattern grammar directly to the same immutable
    /// instruction descriptor used by statically compiled RegExps. The helper
    /// performs no JavaScript calls and owns everything above its entry heap
    /// checkpoint until publication or rollback.
    pub(crate) fn compile_regexp_compiler_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::RegExpCompiler);
        let compiler = self.reserve_regexp_compiler_locals();
        function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(compiler.heap_checkpoint));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(compiler.cursor));
        self.emit_regexp_compile_flags(&compiler, &mut function);
        self.emit_regexp_compile_workspace(&compiler, &mut function);
        self.emit_regexp_compile_units(&compiler, &mut function);
        let result = self
            .emit_regexp_runtime_parse(&compiler, &mut function)
            .and_then(|()| self.emit_regexp_runtime_lower(&compiler, &mut function));
        self.emit_regexp_compile_publish(&compiler, &mut function);
        self.release_regexp_compiler_locals(compiler);
        function.instruction(&Instruction::End);
        result?;
        Ok(self.finish_function(function))
    }

    fn emit_regexp_compile_flags(&mut self, compiler: &CompilerLocals, function: &mut Function) {
        let pointer = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let bit = self.reserve_temp_local();
        let seen = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(pointer));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(length));
        for local in [index, seen, compiler.flags] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte_at_delta(pointer, index, 0, byte, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(bit));
        for (position, flag) in b"dgimsuvy".iter().copied().enumerate() {
            function.instruction(&Instruction::LocalGet(byte));
            function.instruction(&Instruction::I64Const(flag as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1 << position));
            function.instruction(&Instruction::LocalSet(bit));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::LocalGet(seen));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidFlags),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::LocalGet(seen));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(seen));
        self.emit_increment_local(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(seen));
        function.instruction(&Instruction::I64Const(0x60));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x60));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidFlags),
            function,
        );
        function.instruction(&Instruction::End);
        for (mask, capability) in [
            (0x20, CompileCapability::Unicode),
            (0x40, CompileCapability::UnicodeSets),
        ] {
            function.instruction(&Instruction::LocalGet(seen));
            function.instruction(&Instruction::I64Const(mask));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_regexp_compile_failure(
                compiler,
                CompileFailure::Unsupported(capability),
                function,
            );
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(seen));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(
            (FLAG_IGNORE_CASE | FLAG_MULTILINE | FLAG_DOT_ALL) as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(compiler.flags));
        for local in [seen, bit, byte, index, length, pointer] {
            self.release_temp_local(local);
        }
    }

    fn emit_regexp_compile_workspace(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) {
        let end = self.reserve_temp_local();
        let available = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(compiler.source_pointer));
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalTee(compiler.source_length));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compiler.unit_capacity));
        function.instruction(&Instruction::LocalGet(compiler.unit_capacity));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(compiler.node_capacity));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(
            (8 * REGEXP_MAX_INSTRUCTIONS + 32) as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(compiler.task_capacity));
        function.instruction(&Instruction::LocalGet(compiler.heap_checkpoint));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(compiler.heap_checkpoint));
        function.instruction(&Instruction::LocalSet(end));
        // All products fit u64: each capacity is linear in a u32 byte length.
        // Addressability is checked before memory.grow or any workspace write.
        for (pointer, count, stride) in [
            (compiler.units, compiler.unit_capacity, 8),
            (compiler.nodes, compiler.node_capacity, NODE_BYTES),
            (compiler.tasks, compiler.task_capacity, TASK_BYTES),
        ] {
            function.instruction(&Instruction::LocalGet(end));
            function.instruction(&Instruction::LocalTee(pointer));
            function.instruction(&Instruction::LocalGet(count));
            function.instruction(&Instruction::I64Const(stride as i64));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(end));
        }
        for (pointer, length) in [
            (compiler.class_bitmap, CLASS_BITMAP_BYTES),
            (compiler.folded_bitmap, CLASS_BITMAP_BYTES),
            (compiler.graph_visited, REGEXP_MAX_INSTRUCTIONS as u64 * 8),
            (compiler.graph_work, REGEXP_MAX_INSTRUCTIONS as u64 * 8),
            (compiler.graph_colors, REGEXP_MAX_INSTRUCTIONS as u64 * 8),
            (compiler.graph_edges, REGEXP_MAX_INSTRUCTIONS as u64 * 8),
            (compiler.descriptor, REGEXP_PROGRAM_HEADER_SIZE as u64),
            (
                compiler.instructions,
                REGEXP_MAX_INSTRUCTIONS as u64 * REGEXP_INSTRUCTION_WIDTH as u64,
            ),
            (
                compiler.ranges,
                REGEXP_MAX_RANGE_ENTRIES as u64 * REGEXP_RANGE_ENTRY_WIDTH as u64,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(end));
            function.instruction(&Instruction::LocalTee(pointer));
            function.instruction(&Instruction::I64Const(length as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(end));
        }
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::AddressSpace),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::MemorySize(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(65_536));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(available));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(available));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(available));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(65_535));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryGrow(0));
        function.instruction(&Instruction::I32Const(-1));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::MemoryGrowth),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(compiler.heap_checkpoint));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(compiler.heap_checkpoint));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        let table = self
            .strings
            .regexp_case_folding_table(RegExpCaseFolding::Legacy)
            .expect("runtime RegExp compiler requires its legacy folding table");
        function.instruction(&Instruction::I64Const(table.ptr as i64));
        function.instruction(&Instruction::LocalSet(compiler.fold_table));
        function.instruction(&Instruction::I64Const(table.count as i64));
        function.instruction(&Instruction::LocalSet(compiler.fold_count));
        for local in [available, end] {
            self.release_temp_local(local);
        }
    }

    fn emit_regexp_compile_units(&mut self, compiler: &CompilerLocals, function: &mut Function) {
        let index = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let codepoint = self.reserve_temp_local();
        let advance = self.reserve_temp_local();
        let temp = self.reserve_temp_local();
        for local in [index, compiler.unit_count] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(compiler.source_length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte_at_delta(compiler.source_pointer, index, 0, byte, function);
        self.emit_decode_utf8_scalar_at_index(
            compiler.source_pointer,
            index,
            compiler.source_length,
            byte,
            codepoint,
            advance,
            temp,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(temp));
        self.emit_regexp_compile_store_unit(compiler, temp, function);
        function.instruction(&Instruction::LocalGet(codepoint));
        function.instruction(&Instruction::I64Const(0x3ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint));
        function.instruction(&Instruction::End);
        self.emit_regexp_compile_store_unit(compiler, codepoint, function);
        self.emit_increment_by_local(index, advance, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [temp, advance, codepoint, byte, index] {
            self.release_temp_local(local);
        }
    }

    fn emit_regexp_compile_store_unit(
        &self,
        compiler: &CompilerLocals,
        unit: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(compiler.units));
        function.instruction(&Instruction::LocalGet(compiler.unit_count));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(unit));
        function.instruction(&Instruction::I64Store(Self::memarg8(0)));
        self.emit_increment_local(compiler.unit_count, 1, function);
    }

    fn emit_regexp_compile_publish(&mut self, compiler: &CompilerLocals, function: &mut Function) {
        let length = self.reserve_temp_local();
        let ranges = self.reserve_temp_local();
        let handle = self.reserve_temp_local();
        let valid = self.reserve_temp_local();
        let retained_end = self.reserve_temp_local();
        let layout = self.reserve_regexp_program_layout();
        function.instruction(&Instruction::LocalGet(compiler.instructions));
        function.instruction(&Instruction::LocalGet(compiler.instruction_count));
        function.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(ranges));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(compiler.ranges));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::I64Const(REGEXP_RANGE_ENTRY_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: 0,
        });
        function.instruction(&Instruction::LocalGet(ranges));
        function.instruction(&Instruction::LocalGet(compiler.descriptor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::I64Const(REGEXP_RANGE_ENTRY_WIDTH as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(length));
        for word in RegExpProgramWord::ALL {
            match word {
                RegExpProgramWord::MagicVersion => self.store_i64_const_at_offset(
                    compiler.descriptor,
                    word.offset(),
                    REGEXP_PROGRAM_MAGIC_VERSION,
                    function,
                ),
                RegExpProgramWord::NamedGroupTableOffset => {
                    self.store_i64_const_at_offset(compiler.descriptor, word.offset(), 0, function)
                }
                RegExpProgramWord::ByteLength => self.store_i64_local_at_offset(
                    compiler.descriptor,
                    word.offset(),
                    length,
                    function,
                ),
                RegExpProgramWord::InstructionCount => self.store_i64_local_at_offset(
                    compiler.descriptor,
                    word.offset(),
                    compiler.instruction_count,
                    function,
                ),
                RegExpProgramWord::CaptureCount => self.store_i64_local_at_offset(
                    compiler.descriptor,
                    word.offset(),
                    compiler.capture_count,
                    function,
                ),
                RegExpProgramWord::RangeCount => self.store_i64_local_at_offset(
                    compiler.descriptor,
                    word.offset(),
                    compiler.range_count,
                    function,
                ),
                RegExpProgramWord::SplitCount => self.store_i64_local_at_offset(
                    compiler.descriptor,
                    word.offset(),
                    compiler.split_count,
                    function,
                ),
                RegExpProgramWord::RepeatableSplitCount => self.store_i64_local_at_offset(
                    compiler.descriptor,
                    word.offset(),
                    compiler.repeatable_split_count,
                    function,
                ),
            }
        }
        function.instruction(&Instruction::LocalGet(compiler.descriptor));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(handle));
        self.emit_regexp_program_layout(handle, &layout, valid, function);
        function.instruction(&Instruction::LocalGet(valid));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(compiler.heap_checkpoint));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(compiler.descriptor));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: 0,
        });
        function.instruction(&Instruction::LocalGet(compiler.heap_checkpoint));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(-8));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(retained_end));
        self.emit_regexp_compile_release_tail(retained_end, function);
        function.instruction(&Instruction::LocalGet(compiler.heap_checkpoint));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(
            RegExpCompilerStatus::Compiled.abi_word(),
        ));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        self.release_regexp_program_layout(layout);
        for local in [retained_end, valid, handle, ranges, length] {
            self.release_temp_local(local);
        }
    }
}
