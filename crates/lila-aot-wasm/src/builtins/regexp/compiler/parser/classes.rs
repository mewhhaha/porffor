use super::escapes::CharacterContext;
use super::*;
use lila_ir::RegExpUnicodeMode;

#[derive(Clone, Copy)]
pub(super) enum ClassEscape {
    Digit,
    NonDigit,
    Word,
    NonWord,
    Whitespace,
    NonWhitespace,
}

impl ClassEscape {
    pub(super) const ALL: [Self; 6] = [
        Self::Digit,
        Self::NonDigit,
        Self::Word,
        Self::NonWord,
        Self::Whitespace,
        Self::NonWhitespace,
    ];
    pub(super) const fn marker(self) -> u8 {
        match self {
            Self::Digit => b'd',
            Self::NonDigit => b'D',
            Self::Word => b'w',
            Self::NonWord => b'W',
            Self::Whitespace => b's',
            Self::NonWhitespace => b'S',
        }
    }
    const fn is_complement(self) -> bool {
        match self {
            Self::Digit | Self::Word | Self::Whitespace => false,
            Self::NonDigit | Self::NonWord | Self::NonWhitespace => true,
        }
    }
}

/// Compute the WordCharacters closure from the same retained simple-fold
/// mappings that the static parser and matcher use; no separate Unicode table.
fn unicode_word_ranges() -> Vec<(u32, u32)> {
    let mut ranges = REGEXP_WORD_RANGES.to_vec();
    for &(source, canonical) in RegExpCaseFolding::Unicode.mappings() {
        if REGEXP_WORD_RANGES
            .iter()
            .any(|&(first, last)| first <= canonical && canonical <= last)
        {
            ranges.push((source, source));
        }
    }
    ranges.sort_unstable();
    let mut normalized: Vec<(u32, u32)> = Vec::new();
    for (first, last) in ranges {
        if let Some(previous) = normalized
            .last_mut()
            .filter(|previous| first <= previous.1 + 1)
        {
            previous.1 = previous.1.max(last);
        } else {
            normalized.push((first, last));
        }
    }
    normalized
}

fn character_class_ranges(
    escape: ClassEscape,
    mode: RegExpUnicodeMode,
    ignore_case: bool,
) -> Vec<(u32, u32)> {
    let ranges = match escape {
        ClassEscape::Digit | ClassEscape::NonDigit => REGEXP_DIGIT_RANGES.to_vec(),
        ClassEscape::Word | ClassEscape::NonWord if mode.is_unicode_mode() && ignore_case => {
            unicode_word_ranges()
        }
        ClassEscape::Word | ClassEscape::NonWord => REGEXP_WORD_RANGES.to_vec(),
        ClassEscape::Whitespace | ClassEscape::NonWhitespace => REGEXP_WHITESPACE_RANGES.to_vec(),
    };
    if !escape.is_complement() {
        return ranges;
    }
    let maximum = match mode {
        RegExpUnicodeMode::Legacy => 0xffff,
        RegExpUnicodeMode::Unicode | RegExpUnicodeMode::UnicodeSets => 0x10ffff,
    };
    let mut complement = Vec::new();
    let mut next = 0;
    for (first, last) in ranges {
        if next < first {
            complement.push((next, first - 1));
        }
        next = last + 1;
    }
    if next <= maximum {
        complement.push((next, maximum));
    }
    complement
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_add_character_set(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        destination: I64Local,
        character: I64Local,
        class: I64Local,
        function: &mut Function,
    ) {
        let first = self.runtime_schema().reserve_i64_local(function);
        let last = self.runtime_schema().reserve_i64_local(function);
        eq(function, class, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_bitmap_set(
            compiler,
            character_mode,
            destination,
            character,
            function,
        );
        function.instruction(&Instruction::Else);
        for escape in ClassEscape::ALL {
            eq(function, class, escape.marker() as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            character_mode.emit_is_unicode(function);
            function.instruction(&Instruction::If(BlockType::Empty));
            modifiers.ignore_case.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            for (index, (mode, ignore_case)) in [
                (RegExpUnicodeMode::Unicode, true),
                (RegExpUnicodeMode::Unicode, false),
                (RegExpUnicodeMode::Legacy, false),
            ]
            .into_iter()
            .enumerate()
            {
                if index != 0 {
                    function.instruction(&Instruction::Else);
                }
                for (start, end) in character_class_ranges(escape, mode, ignore_case) {
                    set(function, first, start as u64);
                    set(function, last, end as u64);
                    self.emit_regexp_parser_bitmap_range(
                        compiler,
                        character_mode,
                        destination,
                        first,
                        last,
                        function,
                    );
                }
                if index == 1 {
                    function.instruction(&Instruction::End);
                }
            }
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(last, function);
        self.runtime_schema().release_i64_local(first, function);
    }

    pub(super) fn emit_regexp_parser_word_ranges(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        first: I64Local,
        last: I64Local,
        function: &mut Function,
    ) {
        character_mode.emit_is_unicode(function);
        modifiers.ignore_case.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (index, ranges) in [unicode_word_ranges(), REGEXP_WORD_RANGES.to_vec()]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                function.instruction(&Instruction::Else);
            }
            for (start, end) in ranges {
                set(function, first, start as u64);
                set(function, last, end as u64);
                self.emit_regexp_parser_append_range(compiler, first, last, function);
            }
        }
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_regexp_parser_class(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        address: I64Local,
        modifiers: &ModifierLocals,
        named_captures: I64Local,
        function: &mut Function,
    ) {
        character_mode.emit_is_unicode_sets(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_unicode_sets_class(
            compiler,
            character_mode,
            address,
            modifiers,
            named_captures,
            function,
        );
        function.instruction(&Instruction::Else);
        let negated = self.runtime_schema().reserve_i64_local(function);
        let unit = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        let may_contain_strings = self.runtime_schema().reserve_i64_local(function);
        let strings = finite::FiniteStringSet::reserve(self, function);
        set(function, may_contain_strings, 0);
        bitmap::clear_bitmap(function, compiler.class_bitmap, character_mode);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        peek(compiler, function, compiler.cursor, 0, unit);
        set(function, negated, 0);
        eq(function, unit, b'^' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, negated, 1);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
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
        let first = self.emit_regexp_parser_character_set_operand(
            compiler,
            character_mode,
            CharacterContext::Class { named_captures },
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
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        let last = self.emit_regexp_parser_character_set_operand(
            compiler,
            character_mode,
            CharacterContext::Class { named_captures },
            function,
        );
        first.emit_is_character(function);
        last.emit_is_character(function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        first.emit_range(
            &last,
            self,
            compiler,
            character_mode,
            compiler.class_bitmap,
            function,
        );
        function.instruction(&Instruction::Else);
        character_mode.emit_is_unicode(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidRange),
            function,
        );
        function.instruction(&Instruction::End);
        first.emit_add(
            self,
            compiler,
            character_mode,
            modifiers,
            compiler.class_bitmap,
            &strings,
            may_contain_strings,
            function,
        );
        last.emit_add(
            self,
            compiler,
            character_mode,
            modifiers,
            compiler.class_bitmap,
            &strings,
            may_contain_strings,
            function,
        );
        set(function, unit, b'-' as u64);
        self.emit_regexp_parser_bitmap_set(
            compiler,
            character_mode,
            compiler.class_bitmap,
            unit,
            function,
        );
        function.instruction(&Instruction::End);
        last.release(self, function);
        function.instruction(&Instruction::Else);
        first.emit_add(
            self,
            compiler,
            character_mode,
            modifiers,
            compiler.class_bitmap,
            &strings,
            may_contain_strings,
            function,
        );
        function.instruction(&Instruction::End);
        first.release(self, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        // The non-v decoder already rejects every string-property operand.
        // A completed legacy/u class therefore owns only this bitmap.
        self.emit_regexp_parser_bitmap_ranges(
            compiler,
            character_mode,
            address,
            negated,
            modifiers,
            function,
        );
        strings.release(self, function);
        for local in [may_contain_strings, next, unit, negated] {
            self.runtime_schema().release_i64_local(local, function);
        }
        function.instruction(&Instruction::End);
    }
}
