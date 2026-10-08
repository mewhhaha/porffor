//! A required empty body may replay only from its actual successful completion.
use super::*;
use crate::gc_types::I32Local;
use lila_ir::RegExpOpcode;

mod greedy;
mod nested;
mod references;

struct ProofScope;

/// The proof cannot escape its emitted If or select another pair/workspace.
pub(super) struct ProvenRequiredEmptyReplay<'body, 'scope> {
    pair: &'body CountedLoopLocals,
    workspace: &'body MatcherWorkspace,
    _scope: &'scope ProofScope,
}

impl ProvenRequiredEmptyReplay<'_, '_> {
    pub(super) fn finish_required_batch(self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        self.pair.maximum_kind.load(f);
        f.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Finite.word() as i64,
        ));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.pair.maximum.subtract_minimum(
            builder,
            self.workspace,
            self.pair.state,
            &self.pair.minimum,
            f,
        );
        f.instruction(&Instruction::End);
        self.pair.minimum.clear(builder, self.pair.state, f);
    }
}

/// These are only instruction-scan locals; no repetition is materialized.
pub(super) struct RequiredEmptyReplayLocals {
    pc: I64Local,
    address: I64Local,
    opcode: I64Local,
    temporary: I64Local,
    eligible: I32Local,
}

impl RequiredEmptyReplayLocals {
    fn intersect(&self, f: &mut Function) {
        self.eligible.load(f);
        f.instruction(&Instruction::I32And);
        self.eligible.store(f);
    }

    fn instruction(
        &self,
        current: &CountedMatcherLocals,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        current.program.load(f);
        self.pc.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        self.address.store(f);
        builder.emit_regexp_instruction_load(self.address, 0, self.opcode, f);
    }

    /// The caller owns the End dispatch flag and instruction-loop branch.
    pub(super) fn emit_if_proven<'body>(
        builder: &FunctionBuilder<'_>,
        workspace: &'body MatcherWorkspace,
        pair: &'body CountedLoopLocals,
        current: &CountedMatcherLocals,
        f: &mut Function,
        callback: impl for<'scope> FnOnce(
            ProvenRequiredEmptyReplay<'body, 'scope>,
            &FunctionBuilder<'_>,
            &mut Function,
        ),
    ) {
        let schema = builder.runtime_schema();
        let locals = Self {
            pc: schema.reserve_i64_local(f),
            address: schema.reserve_i64_local(f),
            opcode: schema.reserve_i64_local(f),
            temporary: schema.reserve_i64_local(f),
            eligible: schema.reserve_i32_local(f),
        };
        current.pc.load(f);
        pair.end.load(f);
        f.instruction(&Instruction::I64Eq);
        current.opcode.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_OPCODE_REPEAT_END as i64));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        builder.emit_regexp_scratch_load_word(
            pair.state,
            RegExpRepeatStateWord::Active.offset(),
            locals.temporary,
            f,
        );
        locals.temporary.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        builder.emit_regexp_scratch_load_word(
            pair.state,
            RegExpRepeatStateWord::Stage.offset(),
            locals.temporary,
            f,
        );
        locals.temporary.load(f);
        f.instruction(&Instruction::I64Const(
            RepeatIterationStage::Required.word() as i64,
        ));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        builder.emit_regexp_scratch_load_word(
            pair.state,
            RegExpRepeatStateWord::PreUtf16.offset(),
            locals.temporary,
            f,
        );
        locals.temporary.load(f);
        current.cursor.utf16.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        pair.minimum
            .greater_than_one(builder, workspace, pair.state, f);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I32Const(1));
        locals.eligible.store(f);
        pair.guard.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        locals.pc.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        locals.pc.load(f);
        pair.end.load(f);
        f.instruction(&Instruction::I64GeU);
        locals.eligible.load(f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::BrIf(1));
        locals.instruction(current, builder, f);
        // Unknown words remain refused even before the immutable gate is assumed.
        f.instruction(&Instruction::I32Const(0));
        locals.eligible.store(f);
        for opcode in RegExpOpcode::ALL {
            match opcode {
                RegExpOpcode::CaptureStart
                | RegExpOpcode::ClearCaptureRange
                | RegExpOpcode::CaptureEnd
                | RegExpOpcode::AssertStart
                | RegExpOpcode::AssertEnd
                | RegExpOpcode::WordBoundary
                | RegExpOpcode::NumberedBackreference
                | RegExpOpcode::NamedBackreference
                | RegExpOpcode::LookaroundStart
                | RegExpOpcode::Split
                | RegExpOpcode::ProgressSplit
                | RegExpOpcode::RepeatBegin
                | RegExpOpcode::RepeatGuard
                | RegExpOpcode::RepeatEnd
                | RegExpOpcode::RepeatExit => {}
                RegExpOpcode::Accept
                | RegExpOpcode::LiteralAscii
                | RegExpOpcode::PositiveAsciiClass
                | RegExpOpcode::Jump
                | RegExpOpcode::Whitespace
                | RegExpOpcode::Dot
                | RegExpOpcode::LiteralCodePoint
                | RegExpOpcode::UnicodeProperty
                | RegExpOpcode::NegativeAsciiClass
                | RegExpOpcode::NotWhitespace
                | RegExpOpcode::LookaroundEnd
                | RegExpOpcode::LookaroundFailure
                | RegExpOpcode::ProgressCheck => continue,
            }
            locals.opcode.load(f);
            f.instruction(&Instruction::I64Const(opcode as i64));
            f.instruction(&Instruction::I64Eq);
            f.instruction(&Instruction::If(BlockType::Empty));
            f.instruction(&Instruction::I32Const(1));
            locals.eligible.store(f);
            match opcode {
                RegExpOpcode::CaptureEnd => {
                    // A first successful forward End does not prove replay if a
                    // later Clear removes its inherited Start. Keep the actual
                    // post-body read precondition, conservatively in reverse too.
                    builder.emit_regexp_instruction_load(locals.address, 8, locals.temporary, f);
                    workspace.base.load(f);
                    locals.temporary.load(f);
                    f.instruction(&Instruction::I64Const(1));
                    f.instruction(&Instruction::I64Sub);
                    f.instruction(&Instruction::I64Const(16));
                    f.instruction(&Instruction::I64Mul);
                    f.instruction(&Instruction::I64Add);
                    locals.temporary.store(f);
                    builder.emit_regexp_scratch_load_word(locals.temporary, 0, locals.temporary, f);
                    locals.temporary.load(f);
                    f.instruction(&Instruction::I64Const(-1));
                    f.instruction(&Instruction::I64Ne);
                    locals.intersect(f);
                }
                RegExpOpcode::NumberedBackreference => {
                    references::emit_numbered(builder, workspace, pair, current, &locals, f);
                }
                RegExpOpcode::NamedBackreference => {
                    references::emit_named(builder, workspace, pair, current, &locals, f);
                }
                RegExpOpcode::LookaroundStart => {
                    input_only_assertion::emit_input_only_assertion(
                        builder,
                        workspace,
                        pair,
                        current,
                        locals.pc,
                        locals.eligible,
                        f,
                    );
                }
                RegExpOpcode::Split | RegExpOpcode::ProgressSplit => {
                    greedy::emit_completed_greedy_attempt(
                        builder, workspace, pair, current, &locals, f,
                    );
                }
                RegExpOpcode::RepeatBegin => {
                    nested::emit_exact_completed_child(
                        builder, workspace, pair, current, &locals, f,
                    );
                }
                RegExpOpcode::CaptureStart
                | RegExpOpcode::ClearCaptureRange
                | RegExpOpcode::AssertStart
                | RegExpOpcode::AssertEnd
                | RegExpOpcode::WordBoundary
                | RegExpOpcode::RepeatGuard
                | RegExpOpcode::RepeatEnd
                | RegExpOpcode::RepeatExit => {}
                RegExpOpcode::Accept
                | RegExpOpcode::LiteralAscii
                | RegExpOpcode::PositiveAsciiClass
                | RegExpOpcode::Jump
                | RegExpOpcode::Whitespace
                | RegExpOpcode::Dot
                | RegExpOpcode::LiteralCodePoint
                | RegExpOpcode::UnicodeProperty
                | RegExpOpcode::NegativeAsciiClass
                | RegExpOpcode::NotWhitespace
                | RegExpOpcode::LookaroundEnd
                | RegExpOpcode::LookaroundFailure
                | RegExpOpcode::ProgressCheck => {
                    unreachable!("unsupported replay opcode has no emitted arm")
                }
            }
            f.instruction(&Instruction::End);
        }
        locals.pc.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        locals.pc.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        locals.eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        let scope = ProofScope;
        callback(
            ProvenRequiredEmptyReplay {
                pair,
                workspace,
                _scope: &scope,
            },
            builder,
            f,
        );
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        schema.release_i32_local(locals.eligible, f);
        for local in [locals.temporary, locals.opcode, locals.address, locals.pc] {
            schema.release_i64_local(local, f);
        }
    }
}
