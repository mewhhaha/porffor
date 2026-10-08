use super::*;

#[derive(Clone, Copy)]
pub(super) enum CharacterContext {
    Pattern { named_captures: I64Local },
    Class { named_captures: I64Local },
    ClassSet { named_captures: I64Local },
}

impl CharacterContext {
    const fn named_captures(self) -> I64Local {
        match self {
            Self::Pattern { named_captures }
            | Self::Class { named_captures }
            | Self::ClassSet { named_captures } => named_captures,
        }
    }

    const fn is_class(self) -> bool {
        matches!(self, Self::Class { .. } | Self::ClassSet { .. })
    }
}

/// A decoded ClassSetCharacter owns both strict Unicode escape validation and
/// the exclusion of character-class escapes. Only this owner counts one
/// character of a ClassString, including paired astral atoms.
#[must_use]
pub(super) struct ParsedClassSetCharacter {
    character: I64Local,
}

impl ParsedClassSetCharacter {
    pub(super) fn local(&self) -> I64Local {
        self.character
    }

    pub(super) fn release(self, emitter: &mut FunctionBuilder<'_>, function: &mut Function) {
        emitter
            .runtime_schema()
            .release_i64_local(self.character, function);
    }
}

impl FunctionBuilder<'_> {
    /// Decodes one admitted PatternCharacter/ClassAtom in its validated domain.
    /// A class marker denotes a closed class escape, never a range endpoint.
    pub(super) fn emit_regexp_parser_character(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        context: CharacterContext,
        character: I64Local,
        class: I64Local,
        function: &mut Function,
    ) {
        let escaped = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        if matches!(context, CharacterContext::ClassSet { .. }) {
            self.emit_regexp_parser_require_class_set_escape(compiler, function);
        }
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
        if context.is_class() {
            character_mode.emit_is_unicode_sets(function);
            function.instruction(&Instruction::If(BlockType::Empty));
            for unit in b"()[]{}/-|" {
                eq(function, character, *unit as u64);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_regexp_compile_failure(
                    compiler,
                    CompileFailure::Syntax(CompileSyntax::InvalidEscape),
                    function,
                );
                function.instruction(&Instruction::End);
            }
            peek(compiler, function, compiler.cursor, 1, next);
            for unit in b"&!#$%*+,.:;<=>?@^`~" {
                eq(function, character, *unit as u64);
                next.load(function);
                character.load(function);
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_regexp_compile_failure(
                    compiler,
                    CompileFailure::Syntax(CompileSyntax::InvalidEscape),
                    function,
                );
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
        }
        character_mode.emit_is_unicode(function);
        between(function, character, 0xd800, 0xdbff);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 1, next);
        between(function, next, 0xdc00, 0xdfff);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_surrogate_pair(character, next, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
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
        if context.is_class() {
            character_mode.emit_is_unicode_sets(function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I32Const(0));
            for unit in b"bfnrtvc0xudDsSwW^$\\.*+?()[]{}/|-&!#%,:;<=>@`~" {
                eq(function, escaped, *unit as u64);
                function.instruction(&Instruction::I32Or);
            }
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_regexp_compile_failure(
                compiler,
                CompileFailure::Syntax(CompileSyntax::InvalidEscape),
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        // Named-capture grammar excludes k from IdentityEscape in Pattern and
        // Class contexts: every Unicode pattern, or Annex B's complete census.
        eq(function, escaped, b'k' as u64);
        character_mode.emit_uses_named_capture_grammar(context.named_captures(), function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        copy(function, character, escaped);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        for escape in classes::ClassEscape::ALL {
            eq(function, escaped, escape.marker() as u64);
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
        if context.is_class() {
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
        if matches!(context, CharacterContext::Class { .. }) {
            between(function, next, b'0' as u64, b'9' as u64);
            function.instruction(&Instruction::I32Or);
            eq(function, next, b'_' as u64);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        next.load(function);
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64And);
        character.store(function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Else);
        // Annex B's standalone backslash consumes no following `c`.
        set(function, character, b'\\' as u64);
        self.emit_regexp_scratch_increment(compiler.cursor, -1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, escaped, b'x' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_hex_escape(compiler, character, 2, function);
        function.instruction(&Instruction::End);
        eq(function, escaped, b'u' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_unicode_escape(compiler, character_mode, character, next, function);
        function.instruction(&Instruction::End);
        between(function, escaped, b'0' as u64, b'7' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_octal_escape(compiler, escaped, character, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(next, function);
        self.runtime_schema().release_i64_local(escaped, function);
    }

    pub(super) fn emit_regexp_parser_class_set_character(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        named_captures: I64Local,
        function: &mut Function,
    ) -> ParsedClassSetCharacter {
        let character = self.runtime_schema().reserve_i64_local(function);
        let class = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_parser_character(
            compiler,
            mode,
            CharacterContext::ClassSet { named_captures },
            character,
            class,
            function,
        );
        eq(function, class, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(class, function);
        ParsedClassSetCharacter { character }
    }

    /// The ordinary decoder retains Annex B fallback paths. A ClassSet source
    /// must validate the strict escape before that decoder may produce a value;
    /// its class parser no longer depends on the byte prescan for this proof.
    fn emit_regexp_parser_require_class_set_escape(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) {
        let marker = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        let value = self.runtime_schema().reserve_i64_local(function);
        let valid = self.runtime_schema().reserve_i64_local(function);
        peek(compiler, function, compiler.cursor, 0, marker);
        eq(function, marker, b'\\' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 1, marker);
        peek(compiler, function, compiler.cursor, 2, next);
        eq(function, marker, b'c' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        between(function, next, b'a' as u64, b'z' as u64);
        between(function, next, b'A' as u64, b'Z' as u64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, marker, b'0' as u64);
        between(function, next, b'0' as u64, b'9' as u64);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        for (marker_value, width) in [(b'x', 2), (b'u', 4)] {
            eq(function, marker, marker_value as u64);
            if marker_value == b'u' {
                eq(function, next, b'{' as u64);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32And);
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_regexp_parser_hex_digits(compiler, 2, width, value, valid, function);
            eq(function, valid, 1);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_regexp_compile_failure(
                compiler,
                CompileFailure::Syntax(CompileSyntax::InvalidEscape),
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        for local in [valid, value, next, marker] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Decode one complete Unicode escape in the validated character mode.
    /// Only the fixed arm owns fixed/fixed surrogate pairing. Braced values
    /// remain one atom even when the decoded code point is a surrogate.
    fn emit_regexp_parser_unicode_escape(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        character: I64Local,
        low: I64Local,
        function: &mut Function,
    ) {
        let unit = self.runtime_schema().reserve_i64_local(function);
        character_mode.emit_is_unicode(function);
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, b'{' as u64);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_braced_unicode_escape(compiler, character, function);
        function.instruction(&Instruction::Else);
        self.emit_regexp_parser_hex_escape(compiler, character, 4, function);
        self.emit_regexp_parser_fixed_escape_pair(
            compiler,
            character_mode,
            character,
            low,
            function,
        );
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(unit, function);
    }

    /// Cursor starts at `{` and advances past `}` only after validating the
    /// complete nonempty hexadecimal body and the inclusive code-point bound.
    fn emit_regexp_parser_braced_unicode_escape(
        &mut self,
        compiler: &CompilerLocals,
        character: I64Local,
        function: &mut Function,
    ) {
        let unit = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        let digits = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        set(function, character, 0);
        set(function, digits, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, b'}' as u64);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_parser_hex_digit(unit, digit, function);
        eq(function, digit, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        // Every preceding prefix is at most 0x10ffff, so the next product
        // and addition fit u64 even with arbitrarily many leading zeros.
        character.load(function);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        digit.load(function);
        function.instruction(&Instruction::I64Add);
        character.store(function);
        character.load(function);
        function.instruction(&Instruction::I64Const(0x10ffff));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(digits, 1, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, digits, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        for local in [digits, digit, unit] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_parser_hex_digit(
        &self,
        unit: I64Local,
        digit: I64Local,
        function: &mut Function,
    ) {
        set(function, digit, u64::MAX);
        for &(first, last, initial) in REGEXP_HEX_DIGIT_RANGES {
            between(function, unit, first as u64, last as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            unit.load(function);
            function.instruction(&Instruction::I64Const((first - initial) as i64));
            function.instruction(&Instruction::I64Sub);
            digit.store(function);
            function.instruction(&Instruction::End);
        }
    }

    fn emit_regexp_parser_hex_escape(
        &mut self,
        compiler: &CompilerLocals,
        character: I64Local,
        width: u64,
        function: &mut Function,
    ) {
        let value = self.runtime_schema().reserve_i64_local(function);
        let valid = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_parser_hex_digits(compiler, 0, width, value, valid, function);
        eq(function, valid, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, character, value);
        self.emit_regexp_scratch_increment(compiler.cursor, width as i64, function);
        function.instruction(&Instruction::End);
        for local in [valid, value] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_parser_hex_digits(
        &mut self,
        compiler: &CompilerLocals,
        start_delta: u64,
        width: u64,
        value: I64Local,
        valid: I64Local,
        function: &mut Function,
    ) {
        let unit = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        set(function, value, 0);
        set(function, valid, 1);
        for delta in 0..width {
            peek(
                compiler,
                function,
                compiler.cursor,
                start_delta + delta,
                unit,
            );
            self.emit_regexp_parser_hex_digit(unit, digit, function);
            eq(function, digit, u64::MAX);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, valid, 0);
            function.instruction(&Instruction::End);
            value.load(function);
            function.instruction(&Instruction::I64Const(4));
            function.instruction(&Instruction::I64Shl);
            digit.load(function);
            function.instruction(&Instruction::I64Or);
            value.store(function);
        }
        self.runtime_schema().release_i64_local(digit, function);
        self.runtime_schema().release_i64_local(unit, function);
    }

    fn emit_regexp_parser_surrogate_pair(
        &self,
        high: I64Local,
        low: I64Local,
        function: &mut Function,
    ) {
        high.load(function);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Shl);
        low.load(function);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Add);
        high.store(function);
    }

    fn emit_regexp_parser_fixed_escape_pair(
        &mut self,
        compiler: &CompilerLocals,
        character_mode: &CompilerCharacterMode,
        character: I64Local,
        low: I64Local,
        function: &mut Function,
    ) {
        let marker = self.runtime_schema().reserve_i64_local(function);
        let valid = self.runtime_schema().reserve_i64_local(function);
        character_mode.emit_is_unicode(function);
        between(function, character, 0xd800, 0xdbff);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, marker);
        eq(function, marker, b'\\' as u64);
        peek(compiler, function, compiler.cursor, 1, marker);
        eq(function, marker, b'u' as u64);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_hex_digits(compiler, 2, 4, low, valid, function);
        eq(function, valid, 1);
        between(function, low, 0xdc00, 0xdfff);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_surrogate_pair(character, low, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 6, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(valid, function);
        self.runtime_schema().release_i64_local(marker, function);
    }

    fn emit_regexp_parser_octal_escape(
        &mut self,
        compiler: &CompilerLocals,
        first: I64Local,
        character: I64Local,
        function: &mut Function,
    ) {
        let remaining = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        first.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        character.store(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Const(1));
        first.load(function);
        function.instruction(&Instruction::I64Const(
            REGEXP_LEGACY_THREE_DIGIT_OCTAL_LAST as i64,
        ));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::Select);
        remaining.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        eq(function, remaining, 0);
        function.instruction(&Instruction::BrIf(1));
        peek(compiler, function, compiler.cursor, 0, digit);
        between(function, digit, b'0' as u64, b'7' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        character.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        digit.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        character.store(function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        self.emit_regexp_scratch_increment(remaining, -1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(digit, function);
        self.runtime_schema().release_i64_local(remaining, function);
    }
}
