use super::super::bitmap::{self, ClassSetBitmapOperation};
use super::super::*;
use super::{ClassSetMode, ClassSetPhase, ClosedClassSetOperand};

/// These words belong only to the checked source-bounded class-frame region.
/// Strings retains the actual finite keys independently of the exact grammar
/// MayContainStrings result.
#[derive(Clone, Copy)]
#[repr(u64)]
pub(super) enum ClassSetFrameWord {
    Bitmap = 0,
    SourceOffset = 8,
    Negated = 16,
    Mode = 24,
    Phase = 32,
    MayContainStrings = 40,
    PendingCharacter = 48,
    Strings = 56,
}

/// One durable accumulator per active nesting depth. Frame addresses stay
/// private, so an unrelated AST-node address cannot be used as class state.
pub(super) struct UnicodeSetFrames {
    depth: I64Local,
    address: I64Local,
    bitmap: I64Local,
}

impl UnicodeSetFrames {
    pub(super) fn emit_load(
        &self,
        word: ClassSetFrameWord,
        out: I64Local,
        function: &mut Function,
    ) {
        load(function, self.address, word as u64, out);
    }

    pub(super) fn emit_store(
        &self,
        word: ClassSetFrameWord,
        source: I64Local,
        function: &mut Function,
    ) {
        store(function, self.address, word as u64, source);
    }

    pub(super) fn emit_store_const(
        &self,
        word: ClassSetFrameWord,
        value: u64,
        function: &mut Function,
    ) {
        store_const(function, self.address, word as u64, value);
    }

    fn emit_address(&self, compiler: &CompilerLocals, function: &mut Function) {
        compiler.class_set_frames.load(function);
        self.depth.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(CLASS_SET_FRAME_BYTES as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        self.address.store(function);
    }

    pub(super) fn emit_push(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        source_offset: I64Local,
        negated: I64Local,
        function: &mut Function,
    ) {
        self.depth.load(function);
        compiler.unit_capacity.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        emitter.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        emitter.emit_regexp_scratch_increment(self.depth, 1, function);
        self.emit_address(compiler, function);
        self.emit_load(ClassSetFrameWord::Bitmap, self.bitmap, function);
        eq(function, self.bitmap, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        let end = emitter.runtime_schema().reserve_i64_local(function);
        let mut workspace = emitter.extend_regexp_workspace(compiler, end, function);
        workspace.reserve_character_bitmap(self.bitmap, mode, function);
        let region = workspace.finish_region(emitter, compiler, function);
        emitter.emit_regexp_commit_workspace(compiler, region, function);
        self.emit_store(ClassSetFrameWord::Bitmap, self.bitmap, function);
        emitter.runtime_schema().release_i64_local(end, function);
        function.instruction(&Instruction::End);
        bitmap::clear_bitmap(function, self.bitmap, mode);
        self.emit_store(ClassSetFrameWord::SourceOffset, source_offset, function);
        self.emit_store(ClassSetFrameWord::Negated, negated, function);
        self.emit_store_const(
            ClassSetFrameWord::Mode,
            ClassSetMode::Undecided as u64,
            function,
        );
        self.emit_store_const(
            ClassSetFrameWord::Phase,
            ClassSetPhase::FirstOperand as u64,
            function,
        );
        for word in [
            ClassSetFrameWord::MayContainStrings,
            ClassSetFrameWord::PendingCharacter,
            ClassSetFrameWord::Strings,
        ] {
            self.emit_store_const(word, 0, function);
        }
    }

    /// Algebra accepts only an operand whose actual bitmap has completed its
    /// operand-local closure. A new source operation must define both its word
    /// operation and its independent MayContainStrings projection.
    pub(super) fn emit_apply_operand(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        operand: &ClosedClassSetOperand,
        function: &mut Function,
    ) {
        let operation = emitter.runtime_schema().reserve_i64_local(function);
        let strings = emitter.runtime_schema().reserve_i64_local(function);
        let finite_strings = finite::FiniteStringSet::reserve(emitter, function);
        let matched = emitter.runtime_schema().reserve_i64_local(function);
        self.emit_load(ClassSetFrameWord::Mode, operation, function);
        self.emit_load(ClassSetFrameWord::Bitmap, self.bitmap, function);
        self.emit_load(ClassSetFrameWord::MayContainStrings, strings, function);
        self.emit_load(
            ClassSetFrameWord::Strings,
            finite_strings.head_local(),
            function,
        );
        set(function, matched, 0);
        for operation_kind in ClassSetMode::ALL {
            eq(function, operation, operation_kind as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            let bitmap_operation = match operation_kind {
                ClassSetMode::Undecided | ClassSetMode::Union => ClassSetBitmapOperation::Union,
                ClassSetMode::Intersection => ClassSetBitmapOperation::Intersection,
                ClassSetMode::Subtraction => ClassSetBitmapOperation::Subtraction,
            };
            emitter.emit_regexp_parser_combine_bitmap(
                mode,
                compiler.class_bitmap,
                self.bitmap,
                bitmap_operation,
                function,
            );
            finite_strings.emit_combine(
                emitter,
                compiler,
                operand.strings(),
                operation_kind,
                function,
            );
            match operation_kind {
                ClassSetMode::Undecided | ClassSetMode::Union => {
                    strings.load(function);
                    operand.mcs_local().load(function);
                    function.instruction(&Instruction::I64Or);
                    strings.store(function);
                }
                ClassSetMode::Intersection => {
                    strings.load(function);
                    operand.mcs_local().load(function);
                    function.instruction(&Instruction::I64And);
                    strings.store(function);
                }
                ClassSetMode::Subtraction => {}
            }
            set(function, matched, 1);
            function.instruction(&Instruction::End);
        }
        eq(function, matched, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        emitter.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        self.emit_store(ClassSetFrameWord::MayContainStrings, strings, function);
        self.emit_store(
            ClassSetFrameWord::Strings,
            finite_strings.head_local(),
            function,
        );
        emitter
            .runtime_schema()
            .release_i64_local(matched, function);
        finite_strings.release(emitter, function);
        for local in [strings, operation] {
            emitter.runtime_schema().release_i64_local(local, function);
        }
    }

    /// A completed frame has no pending operand or range endpoint. Negation
    /// validates grammar MayContainStrings before complementing the closed set.
    pub(super) fn emit_finish(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        finished_mcs: I64Local,
        finished_strings: &finite::FiniteStringSet,
        function: &mut Function,
    ) {
        let phase = emitter.runtime_schema().reserve_i64_local(function);
        let complete = emitter.runtime_schema().reserve_i64_local(function);
        let negated = emitter.runtime_schema().reserve_i64_local(function);
        self.emit_load(ClassSetFrameWord::Phase, phase, function);
        set(function, complete, 0);
        for phase_kind in ClassSetPhase::ALL {
            eq(function, phase, phase_kind as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            match phase_kind {
                ClassSetPhase::FirstOperand | ClassSetPhase::Tail => {
                    set(function, complete, 1);
                }
                ClassSetPhase::UnionOperand
                | ClassSetPhase::OperationOperand
                | ClassSetPhase::RangeEnd => {
                    emitter.emit_regexp_compile_failure(
                        compiler,
                        CompileFailure::Syntax(CompileSyntax::InvalidRange),
                        function,
                    );
                }
            }
            function.instruction(&Instruction::End);
        }
        eq(function, complete, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        emitter.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        self.emit_load(ClassSetFrameWord::MayContainStrings, finished_mcs, function);
        self.emit_load(
            ClassSetFrameWord::Strings,
            finished_strings.head_local(),
            function,
        );
        self.emit_load(ClassSetFrameWord::Negated, negated, function);
        eq(function, negated, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, finished_mcs, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load(ClassSetFrameWord::SourceOffset, compiler.cursor, function);
        emitter.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidEscape),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load(ClassSetFrameWord::Bitmap, self.bitmap, function);
        emitter.emit_regexp_parser_complement_bitmap(mode, self.bitmap, function);
        function.instruction(&Instruction::End);
        self.emit_load(ClassSetFrameWord::Bitmap, self.bitmap, function);
        bitmap::copy_bitmap(function, compiler.class_bitmap, self.bitmap, mode);
        emitter.emit_regexp_scratch_increment(self.depth, -1, function);
        eq(function, self.depth, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_address(compiler, function);
        function.instruction(&Instruction::End);
        for local in [negated, complete, phase] {
            emitter.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_is_empty(&self, function: &mut Function) {
        eq(function, self.depth, 0);
    }

    pub(super) fn release(self, emitter: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in [self.bitmap, self.address, self.depth] {
            emitter.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn reserve_unicode_set_frames(
        &mut self,
        _compiler: &CompilerLocals,
        function: &mut Function,
    ) -> UnicodeSetFrames {
        let frames = UnicodeSetFrames {
            depth: self.runtime_schema().reserve_i64_local(function),
            address: self.runtime_schema().reserve_i64_local(function),
            bitmap: self.runtime_schema().reserve_i64_local(function),
        };
        set(function, frames.depth, 0);
        frames
    }
}
