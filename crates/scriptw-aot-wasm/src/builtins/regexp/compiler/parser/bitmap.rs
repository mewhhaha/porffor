use super::*;

pub(super) fn clear_bitmap(
    function: &mut Function,
    bitmap: I64Local,
    character_mode: &CompilerCharacterMode,
) {
    bitmap.load(function);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I32Const(0));
    character_mode.emit_bitmap_bytes(function);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::MemoryFill(0));
}
pub(super) fn copy_bitmap(
    function: &mut Function,
    destination: I64Local,
    source: I64Local,
    character_mode: &CompilerCharacterMode,
) {
    destination.load(function);
    function.instruction(&Instruction::I32WrapI64);
    source.load(function);
    function.instruction(&Instruction::I32WrapI64);
    character_mode.emit_bitmap_bytes(function);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::MemoryCopy {
        src_mem: 0,
        dst_mem: 0,
    });
}

#[derive(Clone, Copy)]
pub(super) enum ClassSetBitmapOperation {
    Union,
    Intersection,
    Subtraction,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_bitmap_member(
        &self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        bitmap: I64Local,
        character: I64Local,
        member: I64Local,
        function: &mut Function,
    ) {
        character_mode.emit_require_character(self, compiler, character, function);
        bitmap.load(function);
        character.load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        character.load(function);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        member.store(function);
    }
    pub(super) fn emit_regexp_parser_bitmap_set(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        bitmap: I64Local,
        character: I64Local,
        function: &mut Function,
    ) {
        let address = self.runtime_schema().reserve_i64_local(function);
        let word = self.runtime_schema().reserve_i64_local(function);
        character_mode.emit_require_character(self, compiler, character, function);
        bitmap.load(function);
        character.load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        address.store(function);
        load(function, address, 0, word);
        word.load(function);
        function.instruction(&Instruction::I64Const(1));
        character.load(function);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        word.store(function);
        store(function, address, 0, word);
        self.runtime_schema().release_i64_local(word, function);
        self.runtime_schema().release_i64_local(address, function);
    }
    pub(super) fn emit_regexp_parser_bitmap_range(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        bitmap: I64Local,
        first: I64Local,
        last: I64Local,
        function: &mut Function,
    ) {
        let character = self.runtime_schema().reserve_i64_local(function);
        copy(function, character, first);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        character.load(function);
        last.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_parser_bitmap_set(compiler, character_mode, bitmap, character, function);
        self.emit_regexp_scratch_increment(character, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(character, function);
    }
    fn emit_regexp_parser_fold_bitmap(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        function: &mut Function,
    ) {
        self.emit_regexp_parser_fold_bitmap_operand(
            compiler,
            mode,
            modifiers,
            compiler.class_bitmap,
            function,
        );
    }

    pub(super) fn emit_regexp_parser_fold_bitmap_operand(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        operand_bitmap: I64Local,
        function: &mut Function,
    ) {
        let index = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        let source = self.runtime_schema().reserve_i64_local(function);
        let canonical = self.runtime_schema().reserve_i64_local(function);
        let member = self.runtime_schema().reserve_i64_local(function);
        modifiers.ignore_case.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy_bitmap(
            function,
            compiler.folded_bitmap,
            operand_bitmap,
            character_mode,
        );
        // First collect canonical keys, then expand all members sharing them.
        // Reading the original set in pass one avoids order-dependent closure.
        for (bitmap, tested, added) in [
            (operand_bitmap, source, canonical),
            (compiler.folded_bitmap, canonical, source),
        ] {
            set(function, index, 0);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            index.load(function);
            character_mode.fold_count().load(function);
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            character_mode.fold_table().load(function);
            index.load(function);
            function.instruction(&Instruction::I64Const(8));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            address.store(function);
            for (offset, target) in [(0, source), (4, canonical)] {
                address.load(function);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load32U(MemArg {
                    offset,
                    align: 2,
                    memory_index: 0,
                }));
                target.store(function);
            }
            self.emit_regexp_parser_bitmap_member(
                compiler,
                character_mode,
                bitmap,
                tested,
                member,
                function,
            );
            eq(function, member, 1);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_regexp_parser_bitmap_set(
                compiler,
                character_mode,
                compiler.folded_bitmap,
                added,
                function,
            );
            function.instruction(&Instruction::End);
            self.emit_regexp_scratch_increment(index, 1, function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        copy_bitmap(
            function,
            operand_bitmap,
            compiler.folded_bitmap,
            character_mode,
        );
        function.instruction(&Instruction::End);
        for local in [member, canonical, source, address, index] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_regexp_parser_complement_bitmap(
        &mut self,
        mode: &CompilerCharacterMode,
        bitmap: I64Local,
        function: &mut Function,
    ) {
        let offset = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        let word = self.runtime_schema().reserve_i64_local(function);
        set(function, offset, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        offset.load(function);
        mode.emit_bitmap_bytes(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        bitmap.load(function);
        offset.load(function);
        function.instruction(&Instruction::I64Add);
        address.store(function);
        load(function, address, 0, word);
        word.load(function);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Xor);
        word.store(function);
        store(function, address, 0, word);
        self.emit_regexp_scratch_increment(offset, 8, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [word, address, offset] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
    pub(super) fn emit_regexp_parser_union_bitmap(
        &mut self,
        mode: &CompilerCharacterMode,
        source: I64Local,
        destination: I64Local,
        function: &mut Function,
    ) {
        self.emit_regexp_parser_combine_bitmap(
            mode,
            source,
            destination,
            ClassSetBitmapOperation::Union,
            function,
        );
    }

    pub(super) fn emit_regexp_parser_combine_bitmap(
        &mut self,
        mode: &CompilerCharacterMode,
        source: I64Local,
        destination: I64Local,
        operation: ClassSetBitmapOperation,
        function: &mut Function,
    ) {
        let offset = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        let source_word = self.runtime_schema().reserve_i64_local(function);
        let destination_word = self.runtime_schema().reserve_i64_local(function);
        set(function, offset, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        offset.load(function);
        mode.emit_bitmap_bytes(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        source.load(function);
        offset.load(function);
        function.instruction(&Instruction::I64Add);
        address.store(function);
        load(function, address, 0, source_word);
        destination.load(function);
        offset.load(function);
        function.instruction(&Instruction::I64Add);
        address.store(function);
        load(function, address, 0, destination_word);
        destination_word.load(function);
        source_word.load(function);
        match operation {
            ClassSetBitmapOperation::Union => {
                function.instruction(&Instruction::I64Or);
            }
            ClassSetBitmapOperation::Intersection => {
                function.instruction(&Instruction::I64And);
            }
            ClassSetBitmapOperation::Subtraction => {
                // The destination is the left operand; only the source is
                // complemented. Both operands already own case closure.
                function.instruction(&Instruction::I64Const(-1));
                function.instruction(&Instruction::I64Xor);
                function.instruction(&Instruction::I64And);
            }
        }
        destination_word.store(function);
        store(function, address, 0, destination_word);
        self.emit_regexp_scratch_increment(offset, 8, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [destination_word, source_word, address, offset] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_regexp_parser_append_range(
        &mut self,
        compiler: &CompilerLocals,
        first: I64Local,
        last: I64Local,
        function: &mut Function,
    ) {
        compiler.range_count.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_MAX_RANGE_ENTRIES as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::Ranges),
            function,
        );
        function.instruction(&Instruction::End);
        compiler.ranges.load(function);
        compiler.range_count.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        last.load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        first.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        self.emit_regexp_scratch_increment(compiler.range_count, 1, function);
    }

    pub(super) fn emit_regexp_parser_bitmap_ranges(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        address: I64Local,
        negated: I64Local,
        modifiers: &ModifierLocals,
        function: &mut Function,
    ) {
        let first_entry = self.runtime_schema().reserve_i64_local(function);
        let character = self.runtime_schema().reserve_i64_local(function);
        let run_start = self.runtime_schema().reserve_i64_local(function);
        let run_end = self.runtime_schema().reserve_i64_local(function);
        let member = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_parser_fold_bitmap(compiler, character_mode, modifiers, function);
        copy(function, first_entry, compiler.range_count);
        set(function, character, 0);
        set(function, run_start, u64::MAX);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        character.load(function);
        character_mode.emit_cardinality(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        set(function, member, 0);
        character.load(function);
        character_mode.emit_cardinality(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_bitmap_member(
            compiler,
            character_mode,
            compiler.class_bitmap,
            character,
            member,
            function,
        );
        function.instruction(&Instruction::End);
        eq(function, member, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, run_start, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, run_start, character);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        eq(function, run_start, u64::MAX);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, run_end, character);
        self.emit_regexp_scratch_increment(run_end, -1, function);
        self.emit_regexp_parser_append_range(compiler, run_start, run_end, function);
        set(function, run_start, u64::MAX);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(character, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        store_const(
            function,
            address,
            NodeWord::Opcode as u64,
            REGEXP_OPCODE_UNICODE_PROPERTY,
        );
        store(function, address, NodeWord::Operand0 as u64, first_entry);
        compiler.range_count.load(function);
        first_entry.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        negated.load(function);
        function.instruction(&Instruction::I64Or);
        member.store(function);
        store(function, address, NodeWord::Operand1 as u64, member);
        for local in [member, run_end, run_start, character, first_entry] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
