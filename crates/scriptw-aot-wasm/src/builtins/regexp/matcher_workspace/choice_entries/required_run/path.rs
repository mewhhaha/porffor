//! A source-sized temporary path selects actual tree nodes without unfolding
//! their multiplicities. These addresses never become copied record authority.
use super::*;
use crate::gc_types::{ByteArray, GcLocal, GcOperand, RuntimeSchema};

#[derive(Clone, Copy)]
pub(super) enum PathWord {
    Definition,
    State,
    Context,
    Index,
    Older,
}

pub(super) struct PlaybackPath {
    bytes: GcLocal<ByteArray>,
    limit: I64Local,
}

impl PlaybackPath {
    pub(super) fn new(
        root: &RunLayout,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Self {
        let schema = builder.runtime_schema();
        let length = schema.reserve_i32_local(f);
        let limit = schema.reserve_i64_local(f);
        root.node_count.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        limit.store(f);
        limit.load(f);
        f.instruction(&Instruction::I64Const(40));
        f.instruction(&Instruction::I64Mul);
        workspace.maximum_capacity.load(f);
        f.instruction(&Instruction::I64GtU);
        reject(workspace, builder, f);
        limit.load(f);
        f.instruction(&Instruction::I64Const(40));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I32WrapI64);
        length.store(f);
        let bytes = schema.reserve_gc_local(f).initialize(
            schema
                .array_type::<ByteArray>()
                .filled(GcOperand::i32(0), length, f),
            f,
        );
        schema.release_i32_local(length, f);
        Self { bytes, limit }
    }

    fn address(&self, level: I64Local, word: PathWord, byte: u32, out: I32Local, f: &mut Function) {
        let offset = match word {
            PathWord::Definition => 0,
            PathWord::State => 8,
            PathWord::Context => 16,
            PathWord::Index => 24,
            PathWord::Older => 32,
        };
        level.load(f);
        f.instruction(&Instruction::I64Const(40));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Const(offset + byte as i64));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        out.store(f);
    }

    pub(super) fn read(
        &self,
        level: I64Local,
        word: PathWord,
        out: I64Local,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        let at = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I64Const(0));
        out.store(f);
        for n in 0..8 {
            self.address(level, word, n, at, f);
            schema
                .array_type::<ByteArray>()
                .read(&self.bytes, at, schema, f)
                .store(byte, f);
            out.load(f);
            byte.load(f);
            f.instruction(&Instruction::I64ExtendI32U);
            f.instruction(&Instruction::I64Const(n as i64 * 8));
            f.instruction(&Instruction::I64Shl);
            f.instruction(&Instruction::I64Or);
            out.store(f);
        }
        schema.release_i32_local(byte, f);
        schema.release_i32_local(at, f);
    }

    pub(super) fn write(
        &self,
        level: I64Local,
        word: PathWord,
        value: I64Local,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        let at = schema.reserve_i32_local(f);
        let byte = schema.reserve_i32_local(f);
        for n in 0..8 {
            self.address(level, word, n, at, f);
            value.load(f);
            f.instruction(&Instruction::I64Const(n as i64 * 8));
            f.instruction(&Instruction::I64ShrU);
            f.instruction(&Instruction::I32WrapI64);
            byte.store(f);
            schema.array_type::<ByteArray>().write(
                &self.bytes,
                at,
                GcOperand::i32_local(byte),
                schema,
                f,
            );
        }
        schema.release_i32_local(byte, f);
        schema.release_i32_local(at, f);
    }

    pub(super) fn initialize(
        &self,
        level: I64Local,
        definition: I64Local,
        state: I64Local,
        context: I64Local,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        level.load(f);
        self.limit.load(f);
        f.instruction(&Instruction::I64GeU);
        reject(workspace, builder, f);
        let schema = builder.runtime_schema();
        let value = schema.reserve_i64_local(f);
        for (word, input) in [
            (PathWord::Definition, definition),
            (PathWord::State, state),
            (PathWord::Context, context),
        ] {
            self.write(level, word, input, schema, f);
        }
        ChoiceStack::load(state, 32, f);
        value.store(f);
        self.write(level, PathWord::Index, value, schema, f);
        f.instruction(&Instruction::I64Const(0));
        value.store(f);
        self.write(level, PathWord::Older, value, schema, f);
        schema.release_i64_local(value, f);
    }

    pub(super) fn release(self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        self.bytes.clear(f);
        builder.runtime_schema().release_i64_local(self.limit, f);
    }
}
