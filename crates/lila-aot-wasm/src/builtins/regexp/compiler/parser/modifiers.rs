use super::*;

pub(super) struct ModifierLocals {
    pub(super) ignore_case: u32,
    pub(super) multiline: u32,
    pub(super) dot_all: u32,
}
impl ModifierLocals {
    pub(super) fn words(&self) -> [(NodeWord, u32); 3] {
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
        let unit = self.reserve_temp_local();
        let added = self.reserve_temp_local();
        let removed = self.reserve_temp_local();
        let seen_dash = self.reserve_temp_local();
        let bit = self.reserve_temp_local();
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
        self.emit_increment_local(compiler.cursor, 1, function);
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
        function.instruction(&Instruction::LocalGet(added));
        function.instruction(&Instruction::LocalGet(removed));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(bit));
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
        function.instruction(&Instruction::LocalGet(removed));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(removed));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(added));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(added));
        function.instruction(&Instruction::End);
        self.emit_increment_local(compiler.cursor, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(added));
        function.instruction(&Instruction::LocalGet(removed));
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
                function.instruction(&Instruction::LocalGet(mask));
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
            self.release_temp_local(local);
        }
    }
}
