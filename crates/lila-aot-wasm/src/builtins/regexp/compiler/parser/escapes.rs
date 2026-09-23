use super::*;

#[derive(Clone, Copy)]
pub(super) enum CharacterContext {
    Pattern,
    Class,
}

impl FunctionBuilder<'_> {
    /// Decodes one legacy PatternCharacter/ClassAtom. A nonzero class marker
    /// denotes d/D/s/S/w/W; those are sets, never singleton range endpoints.
    pub(super) fn emit_regexp_parser_character(
        &mut self,
        compiler: &CompilerLocals,
        context: CharacterContext,
        character: u32,
        class: u32,
        function: &mut Function,
    ) {
        let escaped = self.reserve_temp_local();
        let next = self.reserve_temp_local();
        set(function, class, 0);
        peek(compiler, function, compiler.cursor, 0, character);
        eq(function, character, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_increment_local(compiler.cursor, 1, function);
        eq(function, character, b'\\' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, escaped);
        eq(function, escaped, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        copy(function, character, escaped);
        self.emit_increment_local(compiler.cursor, 1, function);
        for marker in b"dDsSwW" {
            eq(function, escaped, *marker as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            copy(function, class, escaped);
            function.instruction(&Instruction::End);
        }
        for &(marker, unit) in REGEXP_CHARACTER_ESCAPES {
            eq(function, escaped, marker as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, character, unit as u64);
            function.instruction(&Instruction::End);
        }
        if matches!(context, CharacterContext::Class) {
            eq(function, escaped, b'b' as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, character, 8);
            function.instruction(&Instruction::End);
        }
        eq(function, escaped, b'c' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, next);
        between(function, next, b'a' as u64, b'z' as u64);
        between(function, next, b'A' as u64, b'Z' as u64);
        function.instruction(&Instruction::I32Or);
        if matches!(context, CharacterContext::Class) {
            between(function, next, b'0' as u64, b'9' as u64);
            function.instruction(&Instruction::I32Or);
            eq(function, next, b'_' as u64);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(next));
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(character));
        self.emit_increment_local(compiler.cursor, 1, function);
        function.instruction(&Instruction::Else);
        // Annex B's standalone backslash consumes no following `c`.
        set(function, character, b'\\' as u64);
        self.emit_increment_local(compiler.cursor, -1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for (marker, width) in [(b'x', 2), (b'u', 4)] {
            eq(function, escaped, marker as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_regexp_parser_hex_escape(compiler, character, width, function);
            function.instruction(&Instruction::End);
        }
        between(function, escaped, b'0' as u64, b'7' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_octal_escape(compiler, escaped, character, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(next);
        self.release_temp_local(escaped);
    }

    fn emit_regexp_parser_hex_escape(
        &mut self,
        compiler: &CompilerLocals,
        character: u32,
        width: u64,
        function: &mut Function,
    ) {
        let unit = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let valid = self.reserve_temp_local();
        set(function, value, 0);
        set(function, valid, 1);
        for delta in 0..width {
            peek(compiler, function, compiler.cursor, delta, unit);
            set(function, digit, u64::MAX);
            for &(first, last, initial) in REGEXP_HEX_DIGIT_RANGES {
                between(function, unit, first as u64, last as u64);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(unit));
                function.instruction(&Instruction::I64Const((first - initial) as i64));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(digit));
                function.instruction(&Instruction::End);
            }
            eq(function, digit, u64::MAX);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, valid, 0);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(value));
            function.instruction(&Instruction::I64Const(4));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::LocalGet(digit));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(value));
        }
        eq(function, valid, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, character, value);
        self.emit_increment_local(compiler.cursor, width as i64, function);
        function.instruction(&Instruction::End);
        for local in [valid, value, digit, unit] {
            self.release_temp_local(local);
        }
    }

    fn emit_regexp_parser_octal_escape(
        &mut self,
        compiler: &CompilerLocals,
        first: u32,
        character: u32,
        function: &mut Function,
    ) {
        let remaining = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(first));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(character));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(first));
        function.instruction(&Instruction::I64Const(
            REGEXP_LEGACY_THREE_DIGIT_OCTAL_LAST as i64,
        ));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::Select);
        function.instruction(&Instruction::LocalSet(remaining));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        eq(function, remaining, 0);
        function.instruction(&Instruction::BrIf(1));
        peek(compiler, function, compiler.cursor, 0, digit);
        between(function, digit, b'0' as u64, b'7' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(character));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(character));
        self.emit_increment_local(compiler.cursor, 1, function);
        self.emit_increment_local(remaining, -1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(digit);
        self.release_temp_local(remaining);
    }
}
