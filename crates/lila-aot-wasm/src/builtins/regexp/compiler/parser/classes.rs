use super::escapes::CharacterContext;
use super::*;

fn legacy_class_ranges(marker: u8) -> Vec<(u32, u32)> {
    let ranges = match marker {
        b'd' | b'D' => REGEXP_DIGIT_RANGES,
        b'w' | b'W' => REGEXP_WORD_RANGES,
        b's' | b'S' => REGEXP_WHITESPACE_RANGES,
        _ => unreachable!("closed legacy class escape"),
    };
    if marker.is_ascii_lowercase() {
        return ranges.to_vec();
    }
    let mut complement = Vec::new();
    let mut next = 0;
    for &(first, last) in ranges {
        if next < first {
            complement.push((next, first - 1));
        }
        next = last + 1;
    }
    if next <= 0xffff {
        complement.push((next, 0xffff));
    }
    complement
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_add_character_set(
        &mut self,
        compiler: &CompilerLocals,
        character: u32,
        class: u32,
        function: &mut Function,
    ) {
        let first = self.reserve_temp_local();
        let last = self.reserve_temp_local();
        eq(function, class, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_bitmap_set(compiler.class_bitmap, character, function);
        function.instruction(&Instruction::Else);
        for marker in b"dDwWsS" {
            eq(function, class, *marker as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            for (start, end) in legacy_class_ranges(*marker) {
                set(function, first, start as u64);
                set(function, last, end as u64);
                self.emit_regexp_parser_bitmap_range(compiler.class_bitmap, first, last, function);
            }
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.release_temp_local(last);
        self.release_temp_local(first);
    }

    pub(super) fn emit_regexp_parser_class(
        &mut self,
        compiler: &CompilerLocals,
        address: u32,
        modifiers: &ModifierLocals,
        function: &mut Function,
    ) {
        let negated = self.reserve_temp_local();
        let unit = self.reserve_temp_local();
        let next = self.reserve_temp_local();
        let first = self.reserve_temp_local();
        let first_class = self.reserve_temp_local();
        let last = self.reserve_temp_local();
        let last_class = self.reserve_temp_local();
        bitmap::clear_bitmap(function, compiler.class_bitmap);
        self.emit_increment_local(compiler.cursor, 1, function);
        peek(compiler, function, compiler.cursor, 0, unit);
        set(function, negated, 0);
        eq(function, unit, b'^' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, negated, 1);
        self.emit_increment_local(compiler.cursor, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::UnclosedClass),
            function,
        );
        function.instruction(&Instruction::End);
        eq(function, unit, b']' as u64);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_parser_character(
            compiler,
            CharacterContext::Class,
            first,
            first_class,
            function,
        );
        peek(compiler, function, compiler.cursor, 0, unit);
        peek(compiler, function, compiler.cursor, 1, next);
        eq(function, unit, b'-' as u64);
        eq(function, next, b']' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, next, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::UnclosedClass),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_increment_local(compiler.cursor, 1, function);
        self.emit_regexp_parser_character(
            compiler,
            CharacterContext::Class,
            last,
            last_class,
            function,
        );
        eq(function, first_class, 0);
        eq(function, last_class, 0);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(first));
        function.instruction(&Instruction::LocalGet(last));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidRange),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_parser_bitmap_range(compiler.class_bitmap, first, last, function);
        function.instruction(&Instruction::Else);
        self.emit_regexp_parser_add_character_set(compiler, first, first_class, function);
        self.emit_regexp_parser_add_character_set(compiler, last, last_class, function);
        set(function, unit, b'-' as u64);
        self.emit_regexp_parser_bitmap_set(compiler.class_bitmap, unit, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_regexp_parser_add_character_set(compiler, first, first_class, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_local(compiler.cursor, 1, function);
        self.emit_regexp_parser_bitmap_ranges(compiler, address, negated, modifiers, function);
        for local in [last_class, last, first_class, first, next, unit, negated] {
            self.release_temp_local(local);
        }
    }
}
