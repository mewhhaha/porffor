use super::*;

pub(super) struct ModifierLocals {
    pub(super) ignore_case: I64Local,
    pub(super) multiline: I64Local,
    pub(super) dot_all: I64Local,
}
impl ModifierLocals {
    pub(super) fn words(&self) -> [(NodeWord, I64Local); 3] {
        [
            (NodeWord::IgnoreCase, self.ignore_case),
            (NodeWord::Multiline, self.multiline),
            (NodeWord::DotAll, self.dot_all),
        ]
    }
}

impl FunctionBuilder<'_> {
    /// Leaves the cursor on `:` for the common group opener to consume.
    pub(super) fn emit_regexp_parser_modifier_prefix(
        &mut self,
        compiler: &CompilerLocals,
        modifiers: &ModifierLocals,
        function: &mut Function,
    ) {
        let unit = self.runtime_schema().reserve_i64_local(function);
        let added = self.runtime_schema().reserve_i64_local(function);
        let removed = self.runtime_schema().reserve_i64_local(function);
        let seen_dash = self.runtime_schema().reserve_i64_local(function);
        let bit = self.runtime_schema().reserve_i64_local(function);
        for local in [added, removed, seen_dash] {
            set(function, local, 0);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, b':' as u64);
        function.instruction(&Instruction::BrIf(1));
        eq(function, unit, b'-' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, seen_dash, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidModifiers),
            function,
        );
        function.instruction(&Instruction::End);
        set(function, seen_dash, 1);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        set(function, bit, 0);
        for modifier in RegExpScopedModifier::ALL {
            eq(function, unit, modifier.marker() as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, bit, modifier.bit() as u64);
            function.instruction(&Instruction::End);
        }
        eq(function, bit, 0);
        added.load(function);
        removed.load(function);
        function.instruction(&Instruction::I64Or);
        bit.load(function);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidModifiers),
            function,
        );
        function.instruction(&Instruction::End);
        eq(function, seen_dash, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        removed.load(function);
        bit.load(function);
        function.instruction(&Instruction::I64Or);
        removed.store(function);
        function.instruction(&Instruction::Else);
        added.load(function);
        bit.load(function);
        function.instruction(&Instruction::I64Or);
        added.store(function);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        added.load(function);
        removed.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidModifiers),
            function,
        );
        function.instruction(&Instruction::End);
        for modifier in RegExpScopedModifier::ALL {
            let (local, enabled, disabled) = match modifier {
                RegExpScopedModifier::IgnoreCase => (modifiers.ignore_case, 1, 0),
                RegExpScopedModifier::Multiline => (
                    modifiers.multiline,
                    RegExpModifierOverride::ForceOn.operand_code(),
                    RegExpModifierOverride::ForceOff.operand_code(),
                ),
                RegExpScopedModifier::DotAll => (
                    modifiers.dot_all,
                    RegExpModifierOverride::ForceOn.operand_code(),
                    RegExpModifierOverride::ForceOff.operand_code(),
                ),
            };
            for (mask, operand) in [(added, enabled), (removed, disabled)] {
                mask.load(function);
                function.instruction(&Instruction::I64Const(modifier.bit() as i64));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                set(function, local, operand);
                function.instruction(&Instruction::End);
            }
        }
        for local in [bit, seen_dash, removed, added, unit] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
