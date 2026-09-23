use super::*;
use lila_ir::{
    RegExpModifierOverride, RegExpScopedModifier, REGEXP_BACKREFERENCE_IGNORE_CASE,
    REGEXP_CHARACTER_ESCAPES, REGEXP_DIGIT_RANGES, REGEXP_HEX_DIGIT_RANGES,
    REGEXP_LEGACY_THREE_DIGIT_OCTAL_LAST, REGEXP_OPCODE_ASSERT_END, REGEXP_OPCODE_ASSERT_START,
    REGEXP_OPCODE_DOT, REGEXP_OPCODE_LITERAL_ASCII, REGEXP_OPCODE_LITERAL_CODE_POINT,
    REGEXP_OPCODE_NUMBERED_BACKREFERENCE, REGEXP_OPCODE_UNICODE_PROPERTY,
    REGEXP_OPCODE_WORD_BOUNDARY, REGEXP_WHITESPACE_RANGES, REGEXP_WORD_RANGES,
};

mod atoms;
mod bitmap;
mod classes;
mod escapes;
mod groups;
mod modifiers;
mod primitives;
mod quantifiers;
use modifiers::ModifierLocals;
use primitives::*;

struct ParserLocals {
    group: u32,
    sequence: u32,
    term: u32,
    address: u32,
    unit: u32,
    total_capture_count: u32,
    has_named_capture: u32,
    modifiers: ModifierLocals,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_runtime_parse(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let parser = ParserLocals {
            group: self.reserve_temp_local(),
            sequence: self.reserve_temp_local(),
            term: self.reserve_temp_local(),
            address: self.reserve_temp_local(),
            unit: self.reserve_temp_local(),
            total_capture_count: self.reserve_temp_local(),
            has_named_capture: self.reserve_temp_local(),
            modifiers: ModifierLocals {
                ignore_case: self.reserve_temp_local(),
                multiline: self.reserve_temp_local(),
                dot_all: self.reserve_temp_local(),
            },
        };
        self.emit_regexp_parser_capture_census(compiler, &parser, function);
        set(function, compiler.cursor, 0);
        set(function, parser.sequence, 0);
        self.emit_regexp_parser_new_node(
            compiler,
            NodeKind::Root,
            parser.sequence,
            parser.group,
            parser.address,
            function,
        );
        function.instruction(&Instruction::LocalGet(compiler.flags));
        function.instruction(&Instruction::I64Const(FLAG_IGNORE_CASE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(parser.modifiers.ignore_case));
        for local in [parser.modifiers.multiline, parser.modifiers.dot_all] {
            set(
                function,
                local,
                RegExpModifierOverride::Inherit.operand_code(),
            );
        }
        for (word, local) in parser.modifiers.words() {
            store(function, parser.address, word as u64, local);
        }
        self.emit_regexp_parser_new_sequence(compiler, &parser, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        // Closing groups and starting alternatives reload the owning lexical flags.
        self.emit_regexp_node_address(compiler, parser.group, parser.address, function);
        for (word, local) in parser.modifiers.words() {
            load(function, parser.address, word as u64, local);
        }
        function.instruction(&Instruction::LocalGet(compiler.cursor));
        function.instruction(&Instruction::LocalGet(compiler.unit_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, parser.group, 1);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::UnclosedGroup),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_parser_finish_group(compiler, &parser, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        peek(compiler, function, compiler.cursor, 0, parser.unit);
        eq(function, parser.unit, b'|' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_finish_sequence(compiler, &parser, function);
        self.emit_increment_local(compiler.cursor, 1, function);
        self.emit_regexp_parser_new_sequence(compiler, &parser, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        eq(function, parser.unit, b')' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, parser.group, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::UnexpectedToken),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_increment_local(compiler.cursor, 1, function);
        self.emit_regexp_parser_finish_group(compiler, &parser, function);
        copy(function, parser.term, parser.group);
        self.emit_regexp_node_address(compiler, parser.term, parser.address, function);
        load(
            function,
            parser.address,
            NodeWord::Parent as u64,
            parser.sequence,
        );
        self.emit_regexp_node_address(compiler, parser.sequence, parser.address, function);
        load(
            function,
            parser.address,
            NodeWord::Parent as u64,
            parser.group,
        );
        self.emit_regexp_parser_quantifier(compiler, parser.term, function);
        self.emit_regexp_parser_finish_term(compiler, &parser, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        eq(function, parser.unit, b'(' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_open_group(compiler, &parser, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_regexp_parser_new_node(
            compiler,
            NodeKind::Atom,
            parser.sequence,
            parser.term,
            parser.address,
            function,
        );
        self.emit_regexp_parser_append(compiler, parser.sequence, parser.term, function);
        self.emit_regexp_parser_atom(compiler, &parser, function);
        self.emit_regexp_parser_quantifier(compiler, parser.term, function);
        self.emit_regexp_parser_finish_term(compiler, &parser, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            parser.modifiers.dot_all,
            parser.modifiers.multiline,
            parser.modifiers.ignore_case,
            parser.has_named_capture,
            parser.total_capture_count,
            parser.unit,
            parser.address,
            parser.term,
            parser.sequence,
            parser.group,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
