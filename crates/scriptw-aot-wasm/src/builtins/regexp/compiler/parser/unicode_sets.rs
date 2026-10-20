use super::escapes::CharacterContext;
use super::*;

mod workspace;
use workspace::{ClassSetFrameWord, UnicodeSetFrames};

#[derive(Clone, Copy)]
#[repr(u64)]
pub(super) enum ClassSetMode {
    Undecided,
    Union,
    Intersection,
    Subtraction,
}

impl ClassSetMode {
    pub(super) const ALL: [Self; 4] = [
        Self::Undecided,
        Self::Union,
        Self::Intersection,
        Self::Subtraction,
    ];
}

#[derive(Clone, Copy)]
#[repr(u64)]
pub(super) enum ClassSetPhase {
    FirstOperand,
    UnionOperand,
    OperationOperand,
    RangeEnd,
    Tail,
}

impl ClassSetPhase {
    pub(super) const ALL: [Self; 5] = [
        Self::FirstOperand,
        Self::UnionOperand,
        Self::OperationOperand,
        Self::RangeEnd,
        Self::Tail,
    ];
}

/// Only the grammar's completed operand route can mint this owner. Its actual
/// bitmap has been case-closed before a workspace operation consumes it. The
/// finite keys remain independent of static MayContainStrings.
#[must_use]
pub(super) struct ClosedClassSetOperand<'keys> {
    may_contain_strings: I64Local,
    strings: &'keys finite::FiniteStringSet,
}

impl<'keys> ClosedClassSetOperand<'keys> {
    fn close(
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        may_contain_strings: I64Local,
        strings: &'keys finite::FiniteStringSet,
        function: &mut Function,
    ) -> Self {
        emitter.emit_regexp_parser_fold_bitmap_operand(
            compiler,
            mode,
            modifiers,
            compiler.class_bitmap,
            function,
        );
        Self {
            may_contain_strings,
            strings,
        }
    }

    pub(super) fn mcs_local(&self) -> I64Local {
        self.may_contain_strings
    }

    pub(super) fn strings(&self) -> &finite::FiniteStringSet {
        self.strings
    }
}

/// A character kind belongs to the decoder's actual result, rather than its
/// bitmap cardinality. A singleton nested/property/q operand is still a Set.
#[derive(Clone, Copy)]
#[repr(u64)]
enum ClassSetOperandKind {
    Set,
    Character,
}

struct ClassSetOperandLocals {
    kind: I64Local,
    character: I64Local,
    may_contain_strings: I64Local,
    strings: finite::FiniteStringSet,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_unicode_sets_class(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        address: I64Local,
        modifiers: &ModifierLocals,
        named_captures: I64Local,
        function: &mut Function,
    ) {
        let frames = self.reserve_unicode_set_frames(compiler, function);
        let operand = ClassSetOperandLocals {
            kind: self.runtime_schema().reserve_i64_local(function),
            character: self.runtime_schema().reserve_i64_local(function),
            may_contain_strings: self.runtime_schema().reserve_i64_local(function),
            strings: finite::FiniteStringSet::reserve(self, function),
        };
        let phase = self.runtime_schema().reserve_i64_local(function);
        let unit = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        let ready = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_parser_open_class_set(compiler, mode, &frames, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        set(function, ready, 0);
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::UnclosedClass),
            function,
        );
        function.instruction(&Instruction::End);

        // A closed child returns one durable Set operand to the waiting parent.
        // Root publication occurs only after the workspace validates its final
        // phase and any negated-class MayContainStrings early error.
        eq(function, unit, b']' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        frames.emit_finish(
            self,
            compiler,
            mode,
            operand.may_contain_strings,
            &operand.strings,
            function,
        );
        frames.emit_is_empty(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_finish_finite_class_set(
            compiler,
            mode,
            address,
            modifiers,
            &operand.strings,
            function,
        );
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        set(function, operand.kind, ClassSetOperandKind::Set as u64);
        set(function, operand.character, 0);
        set(function, ready, 1);
        function.instruction(&Instruction::End);

        eq(function, ready, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        frames.emit_load(ClassSetFrameWord::Phase, phase, function);
        eq(function, phase, ClassSetPhase::Tail as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_class_set_tail(compiler, &frames, function);
        function.instruction(&Instruction::End);
        peek(compiler, function, compiler.cursor, 0, unit);
        peek(compiler, function, compiler.cursor, 1, next);
        eq(function, unit, b'[' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_open_class_set(compiler, mode, &frames, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_regexp_parser_class_set_operand(
            compiler,
            mode,
            modifiers,
            named_captures,
            &operand,
            unit,
            next,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_parser_consume_class_set_operand(
            compiler, mode, modifiers, &frames, &operand, function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [ready, next, unit, phase] {
            self.runtime_schema().release_i64_local(local, function);
        }
        operand.strings.release(self, function);
        for local in [operand.may_contain_strings, operand.character, operand.kind] {
            self.runtime_schema().release_i64_local(local, function);
        }
        frames.release(self, function);
    }

    fn emit_regexp_parser_open_class_set(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        frames: &UnicodeSetFrames,
        function: &mut Function,
    ) {
        let source_offset = self.runtime_schema().reserve_i64_local(function);
        let negated = self.runtime_schema().reserve_i64_local(function);
        let unit = self.runtime_schema().reserve_i64_local(function);
        copy(function, source_offset, compiler.cursor);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, b'^' as u64);
        function.instruction(&Instruction::I64ExtendI32U);
        negated.store(function);
        eq(function, negated, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::End);
        frames.emit_push(self, compiler, mode, source_offset, negated, function);
        for local in [unit, negated, source_offset] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// A union can accept another operand. An operation chain can accept only
    /// the same operator followed by one operand; ranges and bare adjacency
    /// cannot become operands of an intersection/subtraction chain.
    fn emit_regexp_parser_class_set_tail(
        &mut self,
        compiler: &CompilerLocals,
        frames: &UnicodeSetFrames,
        function: &mut Function,
    ) {
        let expression_mode = self.runtime_schema().reserve_i64_local(function);
        let unit = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        let after = self.runtime_schema().reserve_i64_local(function);
        frames.emit_load(ClassSetFrameWord::Mode, expression_mode, function);
        peek(compiler, function, compiler.cursor, 0, unit);
        peek(compiler, function, compiler.cursor, 1, next);
        eq(function, expression_mode, ClassSetMode::Union as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_class_set_operator(unit, next, function);
        function.instruction(&Instruction::I32Eqz);
        self.emit_regexp_require_class_set(compiler, function);
        frames.emit_store_const(
            ClassSetFrameWord::Phase,
            ClassSetPhase::UnionOperand as u64,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        for (operation, marker) in [
            (ClassSetMode::Intersection, b'&'),
            (ClassSetMode::Subtraction, b'-'),
        ] {
            eq(function, expression_mode, operation as u64);
            eq(function, unit, marker as u64);
            eq(function, next, marker as u64);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
        }
        self.emit_regexp_require_class_set(compiler, function);
        eq(function, expression_mode, ClassSetMode::Intersection as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 2, after);
        eq(function, after, b'&' as u64);
        function.instruction(&Instruction::I32Eqz);
        self.emit_regexp_require_class_set(compiler, function);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(compiler.cursor, 2, function);
        frames.emit_store_const(
            ClassSetFrameWord::Phase,
            ClassSetPhase::OperationOperand as u64,
            function,
        );
        function.instruction(&Instruction::End);
        for local in [after, next, unit, expression_mode] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_regexp_parser_class_set_operand(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        named_captures: I64Local,
        operand: &ClassSetOperandLocals,
        unit: I64Local,
        next: I64Local,
        function: &mut Function,
    ) {
        bitmap::clear_bitmap(function, compiler.class_bitmap, mode);
        set(function, operand.may_contain_strings, 0);
        operand.strings.emit_clear(function);
        eq(function, unit, b'\\' as u64);
        eq(function, next, b'q' as u64);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_parser_class_string(
            compiler,
            mode,
            modifiers,
            named_captures,
            operand.may_contain_strings,
            &operand.strings,
            function,
        );
        // Cardinality never changes the grammar's Set classification.
        set(function, operand.kind, ClassSetOperandKind::Set as u64);
        set(function, operand.character, 0);
        function.instruction(&Instruction::Else);
        let decoded = self.emit_regexp_parser_character_set_operand(
            compiler,
            mode,
            CharacterContext::ClassSet { named_captures },
            function,
        );
        decoded.emit_is_character(function);
        function.instruction(&Instruction::I64ExtendI32U);
        operand.kind.store(function);
        decoded.emit_character_to(operand.character, function);
        decoded.emit_add(
            self,
            compiler,
            mode,
            modifiers,
            compiler.class_bitmap,
            &operand.strings,
            operand.may_contain_strings,
            function,
        );
        decoded.release(self, function);
        function.instruction(&Instruction::End);
    }

    fn emit_regexp_parser_consume_class_set_operand(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        frames: &UnicodeSetFrames,
        operand: &ClassSetOperandLocals,
        function: &mut Function,
    ) {
        let phase = self.runtime_schema().reserve_i64_local(function);
        let expression_mode = self.runtime_schema().reserve_i64_local(function);
        let pending_character = self.runtime_schema().reserve_i64_local(function);
        let unit = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        frames.emit_load(ClassSetFrameWord::Phase, phase, function);
        frames.emit_load(ClassSetFrameWord::Mode, expression_mode, function);
        eq(function, phase, ClassSetPhase::RangeEnd as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(
            function,
            operand.kind,
            ClassSetOperandKind::Character as u64,
        );
        self.emit_regexp_require_class_set(compiler, function);
        frames.emit_load(
            ClassSetFrameWord::PendingCharacter,
            pending_character,
            function,
        );
        pending_character.load(function);
        operand.character.load(function);
        function.instruction(&Instruction::I64LeU);
        self.emit_regexp_require_class_set(compiler, function);
        bitmap::clear_bitmap(function, compiler.class_bitmap, mode);
        self.emit_regexp_parser_bitmap_range(
            compiler,
            mode,
            compiler.class_bitmap,
            pending_character,
            operand.character,
            function,
        );
        set(function, operand.may_contain_strings, 0);
        operand.strings.emit_clear(function);
        let closed = ClosedClassSetOperand::close(
            self,
            compiler,
            mode,
            modifiers,
            operand.may_contain_strings,
            &operand.strings,
            function,
        );
        frames.emit_apply_operand(self, compiler, mode, &closed, function);
        frames.emit_store_const(
            ClassSetFrameWord::Mode,
            ClassSetMode::Union as u64,
            function,
        );
        frames.emit_store_const(
            ClassSetFrameWord::Phase,
            ClassSetPhase::Tail as u64,
            function,
        );
        function.instruction(&Instruction::Else);
        eq(function, phase, ClassSetPhase::OperationOperand as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        let closed = ClosedClassSetOperand::close(
            self,
            compiler,
            mode,
            modifiers,
            operand.may_contain_strings,
            &operand.strings,
            function,
        );
        frames.emit_apply_operand(self, compiler, mode, &closed, function);
        frames.emit_store_const(
            ClassSetFrameWord::Phase,
            ClassSetPhase::Tail as u64,
            function,
        );
        function.instruction(&Instruction::Else);
        eq(function, phase, ClassSetPhase::FirstOperand as u64);
        eq(function, phase, ClassSetPhase::UnionOperand as u64);
        function.instruction(&Instruction::I32Or);
        self.emit_regexp_require_class_set(compiler, function);
        peek(compiler, function, compiler.cursor, 0, unit);
        peek(compiler, function, compiler.cursor, 1, next);
        eq(function, unit, b'-' as u64);
        eq(function, next, b'-' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        eq(function, next, b']' as u64);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(
            function,
            operand.kind,
            ClassSetOperandKind::Character as u64,
        );
        self.emit_regexp_require_class_set(compiler, function);
        frames.emit_store(
            ClassSetFrameWord::PendingCharacter,
            operand.character,
            function,
        );
        frames.emit_store_const(
            ClassSetFrameWord::Phase,
            ClassSetPhase::RangeEnd as u64,
            function,
        );
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Else);
        let closed = ClosedClassSetOperand::close(
            self,
            compiler,
            mode,
            modifiers,
            operand.may_contain_strings,
            &operand.strings,
            function,
        );
        // The first operand is committed while Mode is Undecided. Choosing an
        // intersection after that commit cannot AND it into an empty set.
        frames.emit_apply_operand(self, compiler, mode, &closed, function);
        self.emit_regexp_parser_class_set_operator(unit, next, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, expression_mode, ClassSetMode::Undecided as u64);
        self.emit_regexp_require_class_set(compiler, function);
        eq(function, unit, b'&' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        frames.emit_store_const(
            ClassSetFrameWord::Mode,
            ClassSetMode::Intersection as u64,
            function,
        );
        function.instruction(&Instruction::Else);
        frames.emit_store_const(
            ClassSetFrameWord::Mode,
            ClassSetMode::Subtraction as u64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        frames.emit_store_const(
            ClassSetFrameWord::Mode,
            ClassSetMode::Union as u64,
            function,
        );
        function.instruction(&Instruction::End);
        frames.emit_store_const(
            ClassSetFrameWord::Phase,
            ClassSetPhase::Tail as u64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [next, unit, pending_character, expression_mode, phase] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Every decoded alternative contributes an exact canonical key. Static
    /// MayContainStrings is based on source code-point cardinality, before
    /// any operand union/intersection/subtraction removes members.
    fn emit_regexp_parser_class_string(
        &mut self,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        named_captures: I64Local,
        may_contain_strings: I64Local,
        strings: &finite::FiniteStringSet,
        function: &mut Function,
    ) {
        let unit = self.runtime_schema().reserve_i64_local(function);
        let decoded = finite::DecodedQString::reserve(self, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 2, function);
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, b'{' as u64);
        self.emit_regexp_require_class_set(compiler, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, unit);
        eq(function, unit, u64::MAX);
        eq(function, unit, b']' as u64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        eq(function, unit, b'|' as u64);
        eq(function, unit, b'}' as u64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, decoded.count_local(), 1);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, may_contain_strings, 1);
        function.instruction(&Instruction::End);
        strings.emit_insert_decoded(self, compiler, mode, &decoded, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        eq(function, unit, b'}' as u64);
        function.instruction(&Instruction::BrIf(2));
        decoded.emit_clear(function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        let character =
            self.emit_regexp_parser_class_set_character(compiler, mode, named_captures, function);
        decoded.emit_append(self, compiler, mode, modifiers, character.local(), function);
        character.release(self, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        decoded.release(self, function);
        self.runtime_schema().release_i64_local(unit, function);
    }

    fn emit_regexp_parser_class_set_operator(
        &self,
        unit: I64Local,
        next: I64Local,
        function: &mut Function,
    ) {
        eq(function, unit, b'&' as u64);
        eq(function, next, b'&' as u64);
        function.instruction(&Instruction::I32And);
        eq(function, unit, b'-' as u64);
        eq(function, next, b'-' as u64);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_regexp_require_class_set(&self, compiler: &CompilerLocals, function: &mut Function) {
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidRange),
            function,
        );
        function.instruction(&Instruction::End);
    }
}
