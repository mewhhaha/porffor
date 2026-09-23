use super::*;

pub(super) fn clear_bitmap(function: &mut Function, bitmap: u32) {
    function.instruction(&Instruction::LocalGet(bitmap));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I32Const(0));
    function.instruction(&Instruction::I32Const(CLASS_BITMAP_BYTES as i32));
    function.instruction(&Instruction::MemoryFill(0));
}
fn copy_bitmap(function: &mut Function, destination: u32, source: u32) {
    function.instruction(&Instruction::LocalGet(destination));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(source));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I32Const(CLASS_BITMAP_BYTES as i32));
    function.instruction(&Instruction::MemoryCopy {
        src_mem: 0,
        dst_mem: 0,
    });
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_bitmap_member(
        &self,
        bitmap: u32,
        character: u32,
        member: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(bitmap));
        function.instruction(&Instruction::LocalGet(character));
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
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(member));
    }
    pub(super) fn emit_regexp_parser_bitmap_set(
        &mut self,
        bitmap: u32,
        character: u32,
        function: &mut Function,
    ) {
        let address = self.reserve_temp_local();
        let word = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(bitmap));
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address));
        load(function, address, 0, word);
        function.instruction(&Instruction::LocalGet(word));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(word));
        store(function, address, 0, word);
        self.release_temp_local(word);
        self.release_temp_local(address);
    }
    pub(super) fn emit_regexp_parser_bitmap_range(
        &mut self,
        bitmap: u32,
        first: u32,
        last: u32,
        function: &mut Function,
    ) {
        let character = self.reserve_temp_local();
        copy(function, character, first);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::LocalGet(last));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_parser_bitmap_set(bitmap, character, function);
        self.emit_increment_local(character, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(character);
    }
    pub(super) fn emit_regexp_parser_fold_bitmap(
        &mut self,
        compiler: &CompilerLocals,
        modifiers: &ModifierLocals,
        function: &mut Function,
    ) {
        let index = self.reserve_temp_local();
        let address = self.reserve_temp_local();
        let source = self.reserve_temp_local();
        let canonical = self.reserve_temp_local();
        let member = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(modifiers.ignore_case));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy_bitmap(function, compiler.folded_bitmap, compiler.class_bitmap);
        // First collect canonical keys, then expand all members sharing them.
        // Reading the original set in pass one avoids order-dependent closure.
        for (bitmap, tested, added) in [
            (compiler.class_bitmap, source, canonical),
            (compiler.folded_bitmap, canonical, source),
        ] {
            set(function, index, 0);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::LocalGet(compiler.fold_count));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(compiler.fold_table));
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::I64Const(8));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(address));
            for (offset, target) in [(0, source), (4, canonical)] {
                function.instruction(&Instruction::LocalGet(address));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load32U(MemArg {
                    offset,
                    align: 2,
                    memory_index: 0,
                }));
                function.instruction(&Instruction::LocalSet(target));
            }
            self.emit_regexp_parser_bitmap_member(bitmap, tested, member, function);
            eq(function, member, 1);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_regexp_parser_bitmap_set(compiler.folded_bitmap, added, function);
            function.instruction(&Instruction::End);
            self.emit_increment_local(index, 1, function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        copy_bitmap(function, compiler.class_bitmap, compiler.folded_bitmap);
        function.instruction(&Instruction::End);
        for local in [member, canonical, source, address, index] {
            self.release_temp_local(local);
        }
    }

    pub(super) fn emit_regexp_parser_append_range(
        &mut self,
        compiler: &CompilerLocals,
        first: u32,
        last: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::I64Const(REGEXP_MAX_RANGE_ENTRIES as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::Ranges),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(compiler.ranges));
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(last));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(first));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        self.emit_increment_local(compiler.range_count, 1, function);
    }

    pub(super) fn emit_regexp_parser_bitmap_ranges(
        &mut self,
        compiler: &CompilerLocals,
        address: u32,
        negated: u32,
        modifiers: &ModifierLocals,
        function: &mut Function,
    ) {
        let first_entry = self.reserve_temp_local();
        let character = self.reserve_temp_local();
        let run_start = self.reserve_temp_local();
        let run_end = self.reserve_temp_local();
        let member = self.reserve_temp_local();
        self.emit_regexp_parser_fold_bitmap(compiler, modifiers, function);
        copy(function, first_entry, compiler.range_count);
        set(function, character, 0);
        set(function, run_start, u64::MAX);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        set(function, member, 0);
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_bitmap_member(compiler.class_bitmap, character, member, function);
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
        self.emit_increment_local(run_end, -1, function);
        self.emit_regexp_parser_append_range(compiler, run_start, run_end, function);
        set(function, run_start, u64::MAX);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_local(character, 1, function);
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
        function.instruction(&Instruction::LocalGet(compiler.range_count));
        function.instruction(&Instruction::LocalGet(first_entry));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(negated));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(member));
        store(function, address, NodeWord::Operand1 as u64, member);
        for local in [member, run_end, run_start, character, first_entry] {
            self.release_temp_local(local);
        }
    }
}
