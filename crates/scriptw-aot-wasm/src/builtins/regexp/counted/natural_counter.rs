//! Exact source-sized base-10^9 counters inside the admitted repeat slab.
use super::*;
mod bounded;

mod sealed {
    pub trait Sealed {}
}

pub(super) struct MinimumCounter;
pub(super) struct MaximumCounter;
impl sealed::Sealed for MinimumCounter {}
impl sealed::Sealed for MaximumCounter {}

pub(super) trait CounterKind: sealed::Sealed {
    const USED_WORD: RegExpRepeatStateWord;
}
impl CounterKind for MinimumCounter {
    const USED_WORD: RegExpRepeatStateWord = RegExpRepeatStateWord::MinimumUsed;
}
impl CounterKind for MaximumCounter {
    const USED_WORD: RegExpRepeatStateWord = RegExpRepeatStateWord::MaximumUsed;
}

pub(super) struct NaturalCounterLocals<K: CounterKind> {
    limbs: I64Local,
    capacity: I64Local,
    kind: core::marker::PhantomData<K>,
}

/// A saved mandatory iteration either owns the current exact counter or the
/// preceding iteration's counter. This is arithmetic on all source-sized
/// limbs, not on a machine-word approximation of the decimal bound.
#[derive(Clone, Copy)]
pub(super) enum SnapshotCounterRelation {
    Current,
    PreviousIteration,
}

impl<K: CounterKind> NaturalCounterLocals<K> {
    pub(super) fn capacity(&self) -> I64Local {
        self.capacity
    }

    pub(super) fn limbs(&self) -> I64Local {
        self.limbs
    }

    pub(super) fn reserve(builder: &FunctionBuilder<'_>, f: &mut Function) -> Self {
        let schema = builder.runtime_schema();
        Self {
            limbs: schema.reserve_i64_local(f),
            capacity: schema.reserve_i64_local(f),
            kind: core::marker::PhantomData,
        }
    }

    /// The sole region construction follows checked descriptor admission.
    /// Maximum limbs follow the complete source-sized minimum region.
    fn configure_region(
        &self,
        state: I64Local,
        digit_length: I64Local,
        preceding_capacity: Option<I64Local>,
        f: &mut Function,
    ) {
        digit_length.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS as i64 - 1,
        ));
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS as i64,
        ));
        f.instruction(&Instruction::I64DivU);
        self.capacity.store(f);
        state.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_STATE_HEADER_SIZE as i64,
        ));
        f.instruction(&Instruction::I64Add);
        if let Some(previous) = preceding_capacity {
            previous.load(f);
            f.instruction(&Instruction::I64Const(
                REGEXP_REPEAT_COUNTER_LIMB_WIDTH as i64,
            ));
            f.instruction(&Instruction::I64Mul);
            f.instruction(&Instruction::I64Add);
        }
        self.limbs.store(f);
    }

    pub(super) fn load_used(
        &self,
        builder: &FunctionBuilder<'_>,
        state: I64Local,
        output: I64Local,
        f: &mut Function,
    ) {
        builder.emit_regexp_scratch_load_word(state, K::USED_WORD.offset(), output, f);
    }

    fn store_used(
        &self,
        builder: &FunctionBuilder<'_>,
        state: I64Local,
        value: I64Local,
        f: &mut Function,
    ) {
        builder.emit_regexp_scratch_store_word(state, K::USED_WORD.offset(), value, f);
    }

    fn limb_address(&self, index: I64Local, address: I64Local, f: &mut Function) {
        self.limbs.load(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_LIMB_WIDTH as i64,
        ));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        address.store(f);
    }

    fn store_limb(address: I64Local, value: I64Local, f: &mut Function) {
        address.load(f);
        f.instruction(&Instruction::I32WrapI64);
        value.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Store(FunctionBuilder::memarg32(0)));
    }

    fn load_limb(&self, index: I64Local, address: I64Local, value: I64Local, f: &mut Function) {
        self.limb_address(index, address, f);
        address.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Load(FunctionBuilder::memarg32(0)));
        f.instruction(&Instruction::I64ExtendI32U);
        value.store(f);
    }

    /// A batch operation reads a whole exact counter. Its used prefix has a
    /// nonzero high limb and every unused limb is zero, including canonical zero.
    pub(super) fn validate_canonical(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        state: I64Local,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let used = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        self.load_used(builder, state, used, f);
        used.load(f);
        self.capacity.load(f);
        f.instruction(&Instruction::I64GtU);
        CountedLoopLocals::reject(workspace, builder, f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        self.capacity.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        self.load_limb(index, address, value, f);
        value.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64GeU);
        index.load(f);
        used.load(f);
        f.instruction(&Instruction::I64GeU);
        value.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Or);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        used.load(f);
        f.instruction(&Instruction::I64Eq);
        value.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::I32Or);
        CountedLoopLocals::reject(workspace, builder, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [value, address, index, used] {
            schema.release_i64_local(local, f);
        }
    }

    /// Compare a workspace-owned snapshot row with this exact live counter.
    /// The snapshot's region is derived from the admitted row's same capacity;
    /// no caller supplies an independent limb count or a different counter role.
    pub(super) fn snapshot_equals(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        snapshot_state: I64Local,
        live_state: I64Local,
        relation: SnapshotCounterRelation,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let saved_limbs = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        let expected = schema.reserve_i64_local(f);
        let carry = schema.reserve_i64_local(f);
        let equal = schema.reserve_i32_local(f);
        self.validate_canonical(builder, workspace, live_state, f);
        // Rebase the region inside the same complete saved slab. The offset is
        // taken from this configured live row rather than guessed from the kind.
        snapshot_state.load(f);
        self.limbs.load(f);
        live_state.load(f);
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Add);
        saved_limbs.store(f);
        let saved = Self {
            limbs: saved_limbs,
            capacity: self.capacity,
            kind: core::marker::PhantomData,
        };
        saved.validate_canonical(builder, workspace, snapshot_state, f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::I64Const(match relation {
            SnapshotCounterRelation::Current => 0,
            SnapshotCounterRelation::PreviousIteration => 1,
        }));
        carry.store(f);
        f.instruction(&Instruction::I32Const(1));
        equal.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        self.capacity.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        self.load_limb(index, address, value, f);
        value.load(f);
        carry.load(f);
        f.instruction(&Instruction::I64Add);
        expected.store(f);
        expected.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::If(BlockType::Empty));
        expected.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64Sub);
        expected.store(f);
        f.instruction(&Instruction::I64Const(1));
        carry.store(f);
        f.instruction(&Instruction::Else);
        f.instruction(&Instruction::I64Const(0));
        carry.store(f);
        f.instruction(&Instruction::End);
        saved.load_limb(index, address, value, f);
        equal.load(f);
        value.load(f);
        expected.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        equal.store(f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        equal.load(f);
        carry.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32And);
        schema.release_i32_local(equal, f);
        for local in [carry, expected, value, address, index, saved_limbs] {
            schema.release_i64_local(local, f);
        }
    }

    /// Parse each complete canonical bound once at Begin. Every limb is written;
    /// zero has used length zero, rather than sharing a word with Unbounded.
    pub(super) fn initialize(
        &self,
        builder: &FunctionBuilder<'_>,
        state: I64Local,
        digits: I64Local,
        digit_length: I64Local,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let end = schema.reserve_i64_local(f);
        let start = schema.reserve_i64_local(f);
        let cursor = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        digit_length.load(f);
        end.store(f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        end.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(1));
        end.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS as i64,
        ));
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        f.instruction(&Instruction::I64Const(0));
        f.instruction(&Instruction::Else);
        end.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_DECIMAL_DIGITS as i64,
        ));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::End);
        start.store(f);
        start.load(f);
        cursor.store(f);
        f.instruction(&Instruction::I64Const(0));
        value.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        cursor.load(f);
        end.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        value.load(f);
        f.instruction(&Instruction::I64Const(10));
        f.instruction(&Instruction::I64Mul);
        digits.load(f);
        cursor.load(f);
        f.instruction(&Instruction::I64Add);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Load8U(FunctionBuilder::memarg8(0)));
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(i64::from(b'0')));
        f.instruction(&Instruction::I64Sub);
        f.instruction(&Instruction::I64Add);
        value.store(f);
        cursor.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        cursor.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        self.limb_address(index, address, f);
        Self::store_limb(address, value, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        start.load(f);
        end.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        self.capacity.load(f);
        value.store(f);
        // Canonicality admitted by the immutable descriptor means only "0"
        // begins with zero. Nonzero high limbs therefore retain full capacity.
        digits.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Load8U(FunctionBuilder::memarg8(0)));
        f.instruction(&Instruction::I32Const(i32::from(b'0')));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I64Const(0));
        value.store(f);
        f.instruction(&Instruction::End);
        self.store_used(builder, state, value, f);
        for local in [address, value, index, cursor, start, end] {
            schema.release_i64_local(local, f);
        }
    }

    /// Borrow exactly across low zero limbs. The high used length changes only
    /// when its last nonzero limb becomes zero; unused capacity remains intact.
    pub(super) fn decrement(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        state: I64Local,
        f: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let used = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        self.load_used(builder, state, used, f);
        used.load(f);
        f.instruction(&Instruction::I64Eqz);
        used.load(f);
        self.capacity.load(f);
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::I32Or);
        CountedLoopLocals::reject(workspace, builder, f);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        used.load(f);
        f.instruction(&Instruction::I64GeU);
        CountedLoopLocals::reject(workspace, builder, f);
        self.limb_address(index, address, f);
        address.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Load(FunctionBuilder::memarg32(0)));
        f.instruction(&Instruction::I64ExtendI32U);
        value.store(f);
        value.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64GeU);
        CountedLoopLocals::reject(workspace, builder, f);
        value.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_RADIX as i64 - 1,
        ));
        value.store(f);
        Self::store_limb(address, value, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(1));
        f.instruction(&Instruction::End);
        value.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        value.store(f);
        Self::store_limb(address, value, f);
        value.load(f);
        f.instruction(&Instruction::I64Eqz);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        used.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::I32And);
        f.instruction(&Instruction::If(BlockType::Empty));
        self.store_used(builder, state, index, f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::Br(1));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        for local in [address, value, index, used] {
            schema.release_i64_local(local, f);
        }
    }

    pub(super) fn release(self, builder: &FunctionBuilder<'_>, f: &mut Function) {
        let schema = builder.runtime_schema();
        schema.release_i64_local(self.capacity, f);
        schema.release_i64_local(self.limbs, f);
    }
}

impl NaturalCounterLocals<MinimumCounter> {
    pub(super) fn configure(&self, state: I64Local, digits: I64Local, f: &mut Function) {
        self.configure_region(state, digits, None, f);
    }

    /// Emits an i32 predicate after checking the exact live counter.
    pub(super) fn greater_than_one(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        state: I64Local,
        f: &mut Function,
    ) {
        self.validate_canonical(builder, workspace, state, f);
        let schema = builder.runtime_schema();
        let used = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        self.load_used(builder, state, used, f);
        used.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::Else);
        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        self.load_limb(index, address, value, f);
        value.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64GtU);
        f.instruction(&Instruction::End);
        for local in [value, address, index, used] {
            schema.release_i64_local(local, f);
        }
    }

    pub(super) fn clear(&self, builder: &FunctionBuilder<'_>, state: I64Local, f: &mut Function) {
        self.limbs.load(f);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::I32Const(0));
        self.capacity.load(f);
        f.instruction(&Instruction::I64Const(
            REGEXP_REPEAT_COUNTER_LIMB_WIDTH as i64,
        ));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I32WrapI64);
        f.instruction(&Instruction::MemoryFill(0));
        builder.emit_regexp_scratch_store_word_const(
            state,
            RegExpRepeatStateWord::MinimumUsed.offset(),
            0,
            f,
        );
    }
}

impl NaturalCounterLocals<MaximumCounter> {
    pub(super) fn configure(
        &self,
        state: I64Local,
        digits: I64Local,
        minimum: &NaturalCounterLocals<MinimumCounter>,
        f: &mut Function,
    ) {
        self.configure_region(state, digits, Some(minimum.capacity), f);
    }

    /// Exact finite maximum minus the still-intact mandatory count. Distinct
    /// sealed counter roles make reversing the subtraction a Rust type error.
    pub(super) fn subtract_minimum(
        &self,
        builder: &FunctionBuilder<'_>,
        workspace: &MatcherWorkspace,
        state: I64Local,
        minimum: &NaturalCounterLocals<MinimumCounter>,
        f: &mut Function,
    ) {
        self.validate_canonical(builder, workspace, state, f);
        minimum.validate_canonical(builder, workspace, state, f);
        let schema = builder.runtime_schema();
        let used = schema.reserve_i64_local(f);
        let minimum_used = schema.reserve_i64_local(f);
        let index = schema.reserve_i64_local(f);
        let probe = schema.reserve_i64_local(f);
        let value = schema.reserve_i64_local(f);
        let subtrahend = schema.reserve_i64_local(f);
        let borrow = schema.reserve_i64_local(f);
        let address = schema.reserve_i64_local(f);
        let minimum_address = schema.reserve_i64_local(f);
        let comparison = schema.reserve_i64_local(f);
        self.load_used(builder, state, used, f);
        minimum.load_used(builder, state, minimum_used, f);
        minimum_used.load(f);
        used.load(f);
        f.instruction(&Instruction::I64GtU);
        CountedLoopLocals::reject(workspace, builder, f);
        minimum_used.load(f);
        used.load(f);
        f.instruction(&Instruction::I64Eq);
        f.instruction(&Instruction::If(BlockType::Empty));
        used.load(f);
        index.store(f);
        f.instruction(&Instruction::I64Const(0));
        comparison.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        f.instruction(&Instruction::I64Eqz);
        comparison.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::BrIf(1));
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        index.store(f);
        self.load_limb(index, address, value, f);
        minimum.load_limb(index, minimum_address, subtrahend, f);
        value.load(f);
        subtrahend.load(f);
        f.instruction(&Instruction::I64Ne);
        f.instruction(&Instruction::If(BlockType::Empty));
        value.load(f);
        subtrahend.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::Else);
        f.instruction(&Instruction::I64Const(2));
        f.instruction(&Instruction::End);
        comparison.store(f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        comparison.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Eq);
        CountedLoopLocals::reject(workspace, builder, f);
        f.instruction(&Instruction::End);

        f.instruction(&Instruction::I64Const(0));
        index.store(f);
        f.instruction(&Instruction::I64Const(0));
        borrow.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        used.load(f);
        f.instruction(&Instruction::I64GeU);
        f.instruction(&Instruction::BrIf(1));
        self.load_limb(index, address, value, f);
        f.instruction(&Instruction::I64Const(0));
        subtrahend.store(f);
        index.load(f);
        minimum_used.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::If(BlockType::Empty));
        minimum.load_limb(index, minimum_address, subtrahend, f);
        f.instruction(&Instruction::End);
        subtrahend.load(f);
        borrow.load(f);
        f.instruction(&Instruction::I64Add);
        subtrahend.store(f);
        value.load(f);
        subtrahend.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::If(BlockType::Empty));
        value.load(f);
        f.instruction(&Instruction::I64Const(REGEXP_REPEAT_COUNTER_RADIX as i64));
        f.instruction(&Instruction::I64Add);
        subtrahend.load(f);
        f.instruction(&Instruction::I64Sub);
        value.store(f);
        f.instruction(&Instruction::I64Const(1));
        borrow.store(f);
        f.instruction(&Instruction::Else);
        value.load(f);
        subtrahend.load(f);
        f.instruction(&Instruction::I64Sub);
        value.store(f);
        f.instruction(&Instruction::I64Const(0));
        borrow.store(f);
        f.instruction(&Instruction::End);
        Self::store_limb(address, value, f);
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        borrow.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        CountedLoopLocals::reject(workspace, builder, f);

        used.load(f);
        index.store(f);
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::BrIf(1));
        index.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        probe.store(f);
        self.load_limb(probe, address, value, f);
        value.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::BrIf(1));
        probe.load(f);
        index.store(f);
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        self.store_used(builder, state, index, f);
        for local in [
            comparison,
            minimum_address,
            address,
            borrow,
            subtrahend,
            value,
            probe,
            index,
            minimum_used,
            used,
        ] {
            schema.release_i64_local(local, f);
        }
    }
}
