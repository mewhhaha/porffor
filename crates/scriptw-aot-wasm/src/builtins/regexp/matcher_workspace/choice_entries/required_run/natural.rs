//! Exact source-sized arithmetic for the consumed immutable choice template.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct NaturalRegion {
    pub(super) limbs: I64Local,
    pub(super) capacity: I64Local,
    pub(super) used: I64Local,
}

#[derive(Clone, Copy)]
pub(super) enum UnitDelta {
    Increment,
    Decrement,
}

impl NaturalRegion {
    fn load_limb(self, index: I64Local, function: &mut Function) {
        index.load(function);
        self.capacity.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.limbs.load(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load(FunctionBuilder::memarg32(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
    }

    fn store_limb(self, index: I64Local, value: I64Local, function: &mut Function) {
        self.limbs.load(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        value.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store(FunctionBuilder::memarg32(0)));
    }

    pub(super) fn validate(
        self,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let index = schema.reserve_i64_local(function);
        let value = schema.reserve_i64_local(function);
        self.used.load(function);
        self.capacity.load(function);
        function.instruction(&Instruction::I64GtU);
        reject(workspace, builder, function);
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        self.capacity.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.load_limb(index, function);
        value.store(function);
        value.load(function);
        function.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        function.instruction(&Instruction::I64GeU);
        index.load(function);
        self.used.load(function);
        function.instruction(&Instruction::I64GeU);
        value.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        self.used.load(function);
        function.instruction(&Instruction::I64Eq);
        value.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        reject(workspace, builder, function);
        index.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i64_local(value, function);
        schema.release_i64_local(index, function);
    }

    pub(super) fn set_one(self, function: &mut Function) {
        self.limbs.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        self.capacity.load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        self.limbs.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Store(FunctionBuilder::memarg32(0)));
        function.instruction(&Instruction::I64Const(1));
        self.used.store(function);
    }

    pub(super) fn copy_from(self, source: Self, function: &mut Function) {
        self.limbs.load(function);
        function.instruction(&Instruction::I32WrapI64);
        source.limbs.load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.capacity.load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: 0,
        });
        source.used.load(function);
        self.used.store(function);
    }

    /// The first pass detects overflow/borrow without publishing any mutation.
    pub(super) fn change_one(
        self,
        delta: UnitDelta,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        self.validate(workspace, builder, function);
        if matches!(delta, UnitDelta::Decrement) {
            self.used.load(function);
            function.instruction(&Instruction::I64Eqz);
            reject(workspace, builder, function);
        }
        let schema = builder.runtime_schema();
        let index = schema.reserve_i64_local(function);
        let carry = schema.reserve_i64_local(function);
        let value = schema.reserve_i64_local(function);
        let output = schema.reserve_i64_local(function);
        let used = schema.reserve_i64_local(function);
        for publish in [false, true] {
            for local in [index, used] {
                function.instruction(&Instruction::I64Const(0));
                local.store(function);
            }
            function.instruction(&Instruction::I64Const(1));
            carry.store(function);
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            index.load(function);
            self.capacity.load(function);
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            self.load_limb(index, function);
            value.store(function);
            match delta {
                UnitDelta::Increment => {
                    value.load(function);
                    carry.load(function);
                    function.instruction(&Instruction::I64Add);
                    output.store(function);
                    output.load(function);
                    function
                        .instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
                    function.instruction(&Instruction::I64DivU);
                    carry.store(function);
                    output.load(function);
                    function
                        .instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
                    function.instruction(&Instruction::I64RemU);
                    output.store(function);
                }
                UnitDelta::Decrement => {
                    value.load(function);
                    carry.load(function);
                    function.instruction(&Instruction::I64LtU);
                    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                    value.load(function);
                    function
                        .instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::Else);
                    value.load(function);
                    function.instruction(&Instruction::End);
                    carry.load(function);
                    function.instruction(&Instruction::I64Sub);
                    output.store(function);
                    value.load(function);
                    carry.load(function);
                    function.instruction(&Instruction::I64LtU);
                    function.instruction(&Instruction::I64ExtendI32U);
                    carry.store(function);
                }
            }
            if publish {
                self.store_limb(index, output, function);
                output.load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                index.load(function);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                used.store(function);
                function.instruction(&Instruction::End);
            }
            index.load(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            index.store(function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            carry.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            reject(workspace, builder, function);
            if publish {
                used.load(function);
                self.used.store(function);
            }
        }
        for local in [used, output, value, carry, index] {
            schema.release_i64_local(local, function);
        }
    }

    pub(super) fn add_into(
        self,
        right: Self,
        output: Self,
        workspace: &MatcherWorkspace,
        builder: &FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        self.validate(workspace, builder, function);
        right.validate(workspace, builder, function);
        self.used.load(function);
        output.capacity.load(function);
        function.instruction(&Instruction::I64GtU);
        right.used.load(function);
        output.capacity.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        reject(workspace, builder, function);
        let schema = builder.runtime_schema();
        let index = schema.reserve_i64_local(function);
        let carry = schema.reserve_i64_local(function);
        let value = schema.reserve_i64_local(function);
        let used = schema.reserve_i64_local(function);
        for publish in [false, true] {
            for local in [index, carry, used] {
                function.instruction(&Instruction::I64Const(0));
                local.store(function);
            }
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            index.load(function);
            output.capacity.load(function);
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::BrIf(1));
            self.load_limb(index, function);
            right.load_limb(index, function);
            function.instruction(&Instruction::I64Add);
            carry.load(function);
            function.instruction(&Instruction::I64Add);
            value.store(function);
            value.load(function);
            function.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
            function.instruction(&Instruction::I64DivU);
            carry.store(function);
            value.load(function);
            function.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
            function.instruction(&Instruction::I64RemU);
            value.store(function);
            if publish {
                output.store_limb(index, value, function);
                value.load(function);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                index.load(function);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                used.store(function);
                function.instruction(&Instruction::End);
            }
            index.load(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            index.store(function);
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            carry.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            reject(workspace, builder, function);
            if publish {
                used.load(function);
                output.used.store(function);
            }
        }
        for local in [used, value, carry, index] {
            schema.release_i64_local(local, function);
        }
    }
}

fn reject(workspace: &MatcherWorkspace, builder: &FunctionBuilder<'_>, function: &mut Function) {
    function.instruction(&Instruction::If(BlockType::Empty));
    workspace.fail(builder, RegExpMatcherFailure::CorruptProgram, function);
    function.instruction(&Instruction::End);
}
