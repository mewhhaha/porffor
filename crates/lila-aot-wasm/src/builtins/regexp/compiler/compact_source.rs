//! Compact only checkpoint-owned Pattern bytes; the original GC source is immutable.
use super::*;

/// Forward cursors over the already checked transient byte region. `write <= read`
/// starts at zero and survives every copy or read-only skip. Callers guard every
/// read by the original length, so neither a write nor a lookahead escapes it.
struct CompactSourceCursor {
    read: I64Local,
    write: I64Local,
    byte: I64Local,
    next: I64Local,
}

impl CompactSourceCursor {
    /// The caller has checked `read < source_length`; copying preserves the
    /// cursor invariant even after an arbitrarily long skipped zero run.
    fn copy(&self, compiler: &CompilerLocals, function: &mut Function) {
        compiler.source_pointer.load(function);
        self.write.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        self.byte.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(FunctionBuilder::memarg8(0)));
        for cursor in [self.read, self.write] {
            cursor.load(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            cursor.store(function);
        }
    }
}

impl FunctionBuilder<'_> {
    /// Unicode escapes admit unbounded leading zeros but denote one character.
    /// Keep one zero in each such run before sizing parser/node/task arenas.
    /// This is lexical compaction, not escape decoding: the existing Unicode
    /// prescan and complete parser retain every syntax and code-point check.
    /// Escaped backslashes are consumed as pairs; Legacy bytes are untouched.
    pub(super) fn emit_regexp_compact_unicode_source(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        function: &mut Function,
    ) {
        let cursor = CompactSourceCursor {
            read: self.runtime_schema().reserve_i64_local(function),
            write: self.runtime_schema().reserve_i64_local(function),
            byte: self.runtime_schema().reserve_i64_local(function),
            next: self.runtime_schema().reserve_i64_local(function),
        };
        character_mode.emit_is_unicode(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        for local in [cursor.read, cursor.write] {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.read.load(function);
        compiler.source_length.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_scratch_byte(compiler.source_pointer, cursor.read, cursor.byte, function);
        cursor.copy(compiler, function);
        cursor.byte.load(function);
        function.instruction(&Instruction::I64Const(i64::from(b'\\')));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.read.load(function);
        compiler.source_length.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Copy the complete escape pair. A second backslash cannot open \u{.
        self.emit_regexp_scratch_byte(compiler.source_pointer, cursor.read, cursor.byte, function);
        cursor.copy(compiler, function);
        cursor.byte.load(function);
        function.instruction(&Instruction::I64Const(i64::from(b'u')));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.read.load(function);
        compiler.source_length.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_byte(compiler.source_pointer, cursor.read, cursor.byte, function);
        cursor.byte.load(function);
        function.instruction(&Instruction::I64Const(i64::from(b'{')));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.copy(compiler, function);
        self.emit_regexp_compact_leading_zeros(compiler, &cursor, function);
        // Close prefix, remaining-byte, u-escape, escape-byte and backslash arms.
        for _ in 0..5 {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        cursor.write.load(function);
        compiler.source_length.store(function);
        function.instruction(&Instruction::End);
        for local in [cursor.next, cursor.byte, cursor.write, cursor.read] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_compact_leading_zeros(
        &self,
        compiler: &CompilerLocals,
        cursor: &CompactSourceCursor,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.read.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        compiler.source_length.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_scratch_byte(compiler.source_pointer, cursor.read, cursor.byte, function);
        cursor.byte.load(function);
        function.instruction(&Instruction::I64Const(i64::from(b'0')));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_scratch_byte_at_delta(
            compiler.source_pointer,
            cursor.read,
            1,
            cursor.next,
            function,
        );
        cursor.next.load(function);
        function.instruction(&Instruction::I64Const(i64::from(b'0')));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_scratch_increment(cursor.read, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }
}
