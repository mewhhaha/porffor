//! Independent ordered body transitions stabilize above the UTF-16 input bound.
//! Retain their actual last iteration, alternatives and exact optional gap.
use super::*;
use crate::gc_types::I32Local;

struct ProofScope;

/// Minted only inside the actual Begin's all-path source proof. Its original
/// pair/workspace cannot be substituted when translating the exact counters.
pub(super) struct ProvenIndependentIterations<'body, 'scope> {
    pair: &'body CountedLoopLocals,
    workspace: &'body MatcherWorkspace,
    retained: I64Local,
    _scope: &'scope ProofScope,
}

impl ProvenIndependentIterations<'_, '_> {
    pub(super) fn translate_required_count(self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        self.pair.maximum_kind.load(f);
        f.instruction(&Instruction::I64Const(
            RegExpRepeatMaximumKind::Finite.word() as i64,
        ));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        // X - (M - retained), rather than independently clipping X and M. The
        // original optional count X-M and preference remain unchanged.
        self.pair.minimum.subtract_bounded(
            builder,
            self.workspace,
            self.pair.state,
            self.retained,
            f,
        );
        self.pair.maximum.subtract_minimum(
            builder,
            self.workspace,
            self.pair.state,
            &self.pair.minimum,
            f,
        );
        f.instruction(&Instruction::End);
        self.pair.minimum.retain_bounded(
            builder,
            self.workspace,
            self.pair.state,
            self.retained,
            f,
        );
    }
}

pub(super) struct IndependentIterationLocals;

impl IndependentIterationLocals {
    pub(super) fn emit_if_proven<'body>(
        builder: &FunctionBuilder<'_>,
        workspace: &'body MatcherWorkspace,
        pair: &'body CountedLoopLocals,
        current: &CountedMatcherLocals,
        f: &mut Function,
        callback: impl for<'scope> FnOnce(
            ProvenIndependentIterations<'body, 'scope>,
            &FunctionBuilder<'_>,
            &mut Function,
        ),
    ) {
        let schema = builder.runtime_schema();
        let eligible: I32Local = schema.reserve_i32_local(f);
        let zero_width = schema.reserve_i32_local(f);
        let retained = schema.reserve_i64_local(f);
        let pc = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let opcode = schema.reserve_i64_local(f);
        let clear_first = schema.reserve_i64_local(f);
        let clear_end = schema.reserve_i64_local(f);
        current.pc.load(f);
        pair.begin.load(f);
        f.instruction(&Instruction::I64Eq);
        current.opcode.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_OPCODE_REPEAT_BEGIN as i64));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        pair.minimum
            .greater_than_one(builder, workspace, pair.state, f);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I32Const(1));
        eligible.store(f);
        f.instruction(&Instruction::I32Const(1));
        zero_width.store(f);
        f.instruction(&Instruction::I64Const(0));
        clear_first.store(f);
        f.instruction(&Instruction::I64Const(0));
        clear_end.store(f);
        pair.guard.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        pc.store(f);
        pc.load(f);
        pair.end.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::If(BlockType::Empty));
        current.program.load(f);
        pc.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_INSTRUCTION_WIDTH as i64));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        address.store(f);
        builder.emit_regexp_instruction_load(address, 0, opcode, f);
        opcode.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_OPCODE_CLEAR_CAPTURE_RANGE as i64,
        ));
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_regexp_instruction_load(address, 8, clear_first, f);
        builder.emit_regexp_instruction_load(address, 16, clear_end, f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        // The actual entry Clear precedes every path. The original scanner
        // retains counted/progress/assertion owner checks and proves every
        // capture write lies in this interval. Without a Clear, no capture
        // writes are admitted. Reads of outside captures leave them untouched.
        input_only_assertion::emit_independent_iteration_body(
            builder,
            workspace,
            pair,
            current,
            clear_first,
            clear_end,
            zero_width,
            eligible,
            f,
        );
        eligible.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        zero_width.load(f);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::Else);
        current.input_utf16_len.load(f);
        f.instruction(&Instruction::I64Const(-1));
        f.instruction(&Instruction::I64Eq);
        CountedLoopLocals::reject(workspace, builder, f);
        current.input_utf16_len.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::End);
        retained.store(f);
        // A consuming success strictly changes UTF-16 in the parent direction;
        // assertions restore it. The original source scopes prevent an old
        // child counter/progress frame from substituting for fresh entry.
        pair.minimum
            .greater_than_bounded(builder, workspace, pair.state, retained, f);
        f.instruction(&Instruction::If(BlockType::Empty));
        let scope = ProofScope;
        callback(
            ProvenIndependentIterations {
                pair,
                workspace,
                retained,
                _scope: &scope,
            },
            builder,
            f,
        );
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [retained, clear_end, clear_first, opcode, address, pc] {
            schema.release_i64_local(local, f);
        }
        schema.release_i32_local(zero_width, f);
        schema.release_i32_local(eligible, f);
    }
}
