use super::*;
use crate::control_flow::SyncIteratorConsumer;
use crate::gc_types::{
    CompletionLocals, GcLocal, GcOperand, I64Local, MathSumPreciseLimbArray, RuntimeSchema,
};

const MATH_SUM_PRECISE_MAX_COUNT: i64 = (1_i64 << 53) - 1;
const MATH_SUM_PRECISE_MAX_EXACT_BITS: usize = 2_151;
const MATH_SUM_PRECISE_LIMB_BITS: usize = 64;
const MATH_SUM_PRECISE_LIMBS: usize =
    (MATH_SUM_PRECISE_MAX_EXACT_BITS + MATH_SUM_PRECISE_LIMB_BITS - 1) / MATH_SUM_PRECISE_LIMB_BITS;
const _: () = assert!(MATH_SUM_PRECISE_LIMBS == 34);
const _: () = assert!(MATH_SUM_PRECISE_LIMBS * MATH_SUM_PRECISE_LIMB_BITS > 2_151);

enum MathSumPreciseState {
    MinusZero,
    Finite,
    PlusInfinity,
    MinusInfinity,
    NotANumber,
}

impl MathSumPreciseState {
    const fn abi_word(self) -> i64 {
        match self {
            Self::MinusZero => 0,
            Self::Finite => 1,
            Self::PlusInfinity => 2,
            Self::MinusInfinity => 3,
            Self::NotANumber => 4,
        }
    }
}

enum MathSumPreciseLimbOperation {
    Add,
    Subtract,
}

struct MathSumPreciseAccumulator {
    limbs: GcLocal<MathSumPreciseLimbArray>,
}

#[must_use = "a completed Math.sumPrecise reduction must be finished"]
struct CompletedMathSumPreciseReduction<'accumulator> {
    accumulator: &'accumulator MathSumPreciseAccumulator,
    state_local: I64Local,
}

impl MathSumPreciseAccumulator {
    fn new(schema: &RuntimeSchema, function: &mut Function) -> Self {
        let count = schema.reserve_i32_local(function);
        let zero = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I32Const(MATH_SUM_PRECISE_LIMBS as i32));
        count.store(function);
        function.instruction(&Instruction::I64Const(0));
        zero.store(function);
        let limbs = schema.reserve_gc_local(function).initialize(
            schema.array_type::<MathSumPreciseLimbArray>().filled(
                GcOperand::i64_local(zero),
                count,
                function,
            ),
            function,
        );
        schema.release_i64_local(zero, function);
        schema.release_i32_local(count, function);
        Self { limbs }
    }
    fn clear(self, function: &mut Function) {
        self.limbs.clear(function);
    }
}

impl<'a> FunctionBuilder<'a> {
    fn emit_math_sum_precise_state_store(
        state: MathSumPreciseState,
        state_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(state.abi_word()));
        state_local.store(function);
    }

    fn emit_math_sum_precise_load_limb(
        &self,
        accumulator: &MathSumPreciseAccumulator,
        index_local: I64Local,
        dest_local: I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let index = schema.reserve_i32_local(function);
        index_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        index.store(function);
        schema
            .array_type::<MathSumPreciseLimbArray>()
            .read(&accumulator.limbs, index, schema, function)
            .store_i64(dest_local, function);
        schema.release_i32_local(index, function);
    }

    fn emit_math_sum_precise_store_limb(
        &self,
        accumulator: &MathSumPreciseAccumulator,
        index_local: I64Local,
        value_local: I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let index = schema.reserve_i32_local(function);
        index_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        index.store(function);
        schema.array_type::<MathSumPreciseLimbArray>().write(
            &accumulator.limbs,
            index,
            GcOperand::i64_local(value_local),
            schema,
            function,
        );
        schema.release_i32_local(index, function);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_math_sum_precise_fold_limbs(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        first_index_local: I64Local,
        low_local: I64Local,
        high_local: I64Local,
        operation: MathSumPreciseLimbOperation,
        function: &mut Function,
    ) {
        let index_local = self.runtime_schema().reserve_i64_local(function);
        let addend_local = self.runtime_schema().reserve_i64_local(function);
        let next_addend_local = self.runtime_schema().reserve_i64_local(function);
        let carry_local = self.runtime_schema().reserve_i64_local(function);
        let next_carry_local = self.runtime_schema().reserve_i64_local(function);
        let old_local = self.runtime_schema().reserve_i64_local(function);
        let updated_local = self.runtime_schema().reserve_i64_local(function);
        let partial_local = self.runtime_schema().reserve_i64_local(function);

        first_index_local.load(function);
        index_local.store(function);
        low_local.load(function);
        addend_local.store(function);
        high_local.load(function);
        next_addend_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        carry_local.store(function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        function.instruction(&Instruction::I64Const(MATH_SUM_PRECISE_LIMBS as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_math_sum_precise_load_limb(accumulator, index_local, old_local, function);
        old_local.load(function);
        addend_local.load(function);
        match &operation {
            MathSumPreciseLimbOperation::Add => function.instruction(&Instruction::I64Add),
            MathSumPreciseLimbOperation::Subtract => function.instruction(&Instruction::I64Sub),
        };
        partial_local.store(function);
        partial_local.load(function);
        old_local.load(function);
        match &operation {
            MathSumPreciseLimbOperation::Add => function.instruction(&Instruction::I64LtU),
            MathSumPreciseLimbOperation::Subtract => function.instruction(&Instruction::I64GtU),
        };
        function.instruction(&Instruction::I64ExtendI32U);
        next_carry_local.store(function);

        partial_local.load(function);
        carry_local.load(function);
        match &operation {
            MathSumPreciseLimbOperation::Add => function.instruction(&Instruction::I64Add),
            MathSumPreciseLimbOperation::Subtract => function.instruction(&Instruction::I64Sub),
        };
        updated_local.store(function);
        updated_local.load(function);
        partial_local.load(function);
        match &operation {
            MathSumPreciseLimbOperation::Add => function.instruction(&Instruction::I64LtU),
            MathSumPreciseLimbOperation::Subtract => function.instruction(&Instruction::I64GtU),
        };
        function.instruction(&Instruction::I64ExtendI32U);
        next_carry_local.load(function);
        function.instruction(&Instruction::I64Or);
        next_carry_local.store(function);
        self.emit_math_sum_precise_store_limb(accumulator, index_local, updated_local, function);

        next_addend_local.load(function);
        addend_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        next_addend_local.store(function);
        next_carry_local.load(function);
        carry_local.store(function);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            partial_local,
            updated_local,
            old_local,
            next_carry_local,
            carry_local,
            next_addend_local,
            addend_local,
            index_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_math_sum_precise_add_finite(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        number_bits_local: I64Local,
        function: &mut Function,
    ) {
        const FRACTION_MASK: i64 = ((1_u64 << 52) - 1) as i64;
        let exponent_local = self.runtime_schema().reserve_i64_local(function);
        let significand_local = self.runtime_schema().reserve_i64_local(function);
        let shift_local = self.runtime_schema().reserve_i64_local(function);
        let first_index_local = self.runtime_schema().reserve_i64_local(function);
        let bit_offset_local = self.runtime_schema().reserve_i64_local(function);
        let low_local = self.runtime_schema().reserve_i64_local(function);
        let high_local = self.runtime_schema().reserve_i64_local(function);

        number_bits_local.load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64And);
        exponent_local.store(function);
        number_bits_local.load(function);
        function.instruction(&Instruction::I64Const(FRACTION_MASK));
        function.instruction(&Instruction::I64And);
        significand_local.store(function);

        exponent_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        shift_local.store(function);
        function.instruction(&Instruction::Else);
        significand_local.load(function);
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64Or);
        significand_local.store(function);
        exponent_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        shift_local.store(function);
        function.instruction(&Instruction::End);

        shift_local.load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        first_index_local.store(function);
        shift_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64And);
        bit_offset_local.store(function);
        significand_local.load(function);
        bit_offset_local.load(function);
        function.instruction(&Instruction::I64Shl);
        low_local.store(function);
        bit_offset_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        high_local.store(function);
        function.instruction(&Instruction::Else);
        significand_local.load(function);
        function.instruction(&Instruction::I64Const(64));
        bit_offset_local.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        high_local.store(function);
        function.instruction(&Instruction::End);

        number_bits_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_math_sum_precise_fold_limbs(
            accumulator,
            first_index_local,
            low_local,
            high_local,
            MathSumPreciseLimbOperation::Subtract,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_math_sum_precise_fold_limbs(
            accumulator,
            first_index_local,
            low_local,
            high_local,
            MathSumPreciseLimbOperation::Add,
            function,
        );
        function.instruction(&Instruction::End);

        for local in [
            high_local,
            low_local,
            bit_offset_local,
            first_index_local,
            shift_local,
            significand_local,
            exponent_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_math_sum_precise_accept_number(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        state_local: I64Local,
        number_bits_local: I64Local,
        function: &mut Function,
    ) {
        const ABS_MASK: i64 = i64::MAX;
        const EXPONENT_MASK: i64 = 0x7ff0_0000_0000_0000_u64 as i64;
        const FRACTION_MASK: i64 = ((1_u64 << 52) - 1) as i64;

        number_bits_local.load(function);
        function.instruction(&Instruction::I64Const(EXPONENT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(EXPONENT_MASK));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));

        number_bits_local.load(function);
        function.instruction(&Instruction::I64Const(FRACTION_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        number_bits_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::PlusInfinity.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_math_sum_precise_state_store(
            MathSumPreciseState::NotANumber,
            state_local,
            function,
        );
        function.instruction(&Instruction::Else);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::NotANumber.abi_word(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_math_sum_precise_state_store(
            MathSumPreciseState::MinusInfinity,
            state_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusInfinity.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_math_sum_precise_state_store(
            MathSumPreciseState::NotANumber,
            state_local,
            function,
        );
        function.instruction(&Instruction::Else);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::NotANumber.abi_word(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_math_sum_precise_state_store(
            MathSumPreciseState::PlusInfinity,
            state_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Else);
        Self::emit_math_sum_precise_state_store(
            MathSumPreciseState::NotANumber,
            state_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Else);
        number_bits_local.load(function);
        function.instruction(&Instruction::I64Const(ABS_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        number_bits_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusZero.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_math_sum_precise_state_store(MathSumPreciseState::Finite, state_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusZero.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::Finite.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_math_sum_precise_state_store(MathSumPreciseState::Finite, state_local, function);
        self.emit_math_sum_precise_add_finite(accumulator, number_bits_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_math_sum_precise_make_magnitude(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        function: &mut Function,
    ) -> I64Local {
        let negative_local = self.runtime_schema().reserve_i64_local(function);
        let index_local = self.runtime_schema().reserve_i64_local(function);
        let limb_local = self.runtime_schema().reserve_i64_local(function);
        let inverted_local = self.runtime_schema().reserve_i64_local(function);
        let carry_local = self.runtime_schema().reserve_i64_local(function);
        let updated_local = self.runtime_schema().reserve_i64_local(function);

        function.instruction(&Instruction::I64Const((MATH_SUM_PRECISE_LIMBS - 1) as i64));
        index_local.store(function);
        self.emit_math_sum_precise_load_limb(accumulator, index_local, limb_local, function);
        limb_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        negative_local.store(function);

        negative_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        index_local.store(function);
        function.instruction(&Instruction::I64Const(1));
        carry_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        function.instruction(&Instruction::I64Const(MATH_SUM_PRECISE_LIMBS as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_math_sum_precise_load_limb(accumulator, index_local, limb_local, function);
        limb_local.load(function);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Xor);
        inverted_local.store(function);
        inverted_local.load(function);
        carry_local.load(function);
        function.instruction(&Instruction::I64Add);
        updated_local.store(function);
        updated_local.load(function);
        inverted_local.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        carry_local.store(function);
        self.emit_math_sum_precise_store_limb(accumulator, index_local, updated_local, function);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            updated_local,
            carry_local,
            inverted_local,
            limb_local,
            index_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        negative_local
    }

    fn emit_math_sum_precise_extract_bit(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        bit_index_local: I64Local,
        result_local: I64Local,
        function: &mut Function,
    ) {
        let limb_index_local = self.runtime_schema().reserve_i64_local(function);
        let limb_local = self.runtime_schema().reserve_i64_local(function);
        bit_index_local.load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        limb_index_local.store(function);
        self.emit_math_sum_precise_load_limb(accumulator, limb_index_local, limb_local, function);
        limb_local.load(function);
        bit_index_local.load(function);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        result_local.store(function);
        self.runtime_schema()
            .release_i64_local(limb_local, function);
        self.runtime_schema()
            .release_i64_local(limb_index_local, function);
    }

    fn emit_math_sum_precise_sticky_below(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        exclusive_bit_local: I64Local,
        result_local: I64Local,
        function: &mut Function,
    ) {
        let limit_limb_local = self.runtime_schema().reserve_i64_local(function);
        let limit_offset_local = self.runtime_schema().reserve_i64_local(function);
        let index_local = self.runtime_schema().reserve_i64_local(function);
        let limb_local = self.runtime_schema().reserve_i64_local(function);
        let mask_local = self.runtime_schema().reserve_i64_local(function);

        function.instruction(&Instruction::I64Const(0));
        result_local.store(function);
        exclusive_bit_local.load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        limit_limb_local.store(function);
        exclusive_bit_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64And);
        limit_offset_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        index_local.store(function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        limit_limb_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_math_sum_precise_load_limb(accumulator, index_local, limb_local, function);
        result_local.load(function);
        limb_local.load(function);
        function.instruction(&Instruction::I64Or);
        result_local.store(function);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        limit_offset_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_math_sum_precise_load_limb(accumulator, limit_limb_local, limb_local, function);
        function.instruction(&Instruction::I64Const(1));
        limit_offset_local.load(function);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        mask_local.store(function);
        result_local.load(function);
        limb_local.load(function);
        mask_local.load(function);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        result_local.store(function);
        function.instruction(&Instruction::End);
        result_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        result_local.store(function);

        for local in [
            mask_local,
            limb_local,
            index_local,
            limit_offset_local,
            limit_limb_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_math_sum_precise_round_finite(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        negative_local: I64Local,
        output_bits: I64Local,
        function: &mut Function,
    ) {
        const FRACTION_MASK: i64 = ((1_u64 << 52) - 1) as i64;
        let index_local = self.runtime_schema().reserve_i64_local(function);
        let limb_local = self.runtime_schema().reserve_i64_local(function);
        let highest_bit_local = self.runtime_schema().reserve_i64_local(function);
        let sign_bits_local = self.runtime_schema().reserve_i64_local(function);

        negative_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64Shl);
        sign_bits_local.store(function);
        function.instruction(&Instruction::I64Const(MATH_SUM_PRECISE_LIMBS as i64));
        index_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        limb_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        index_local.store(function);
        self.emit_math_sum_precise_load_limb(accumulator, index_local, limb_local, function);
        limb_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        limb_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        output_bits.store(function);
        function.instruction(&Instruction::Else);

        index_local.load(function);
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(63));
        limb_local.load(function);
        function.instruction(&Instruction::I64Clz);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        highest_bit_local.store(function);

        highest_bit_local.load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        limb_local.load(function);
        sign_bits_local.load(function);
        function.instruction(&Instruction::I64Or);
        output_bits.store(function);
        function.instruction(&Instruction::Else);

        let shift_local = self.runtime_schema().reserve_i64_local(function);
        let shift_limb_local = self.runtime_schema().reserve_i64_local(function);
        let shift_offset_local = self.runtime_schema().reserve_i64_local(function);
        let next_limb_local = self.runtime_schema().reserve_i64_local(function);
        let significand_local = self.runtime_schema().reserve_i64_local(function);
        let guard_index_local = self.runtime_schema().reserve_i64_local(function);
        let guard_local = self.runtime_schema().reserve_i64_local(function);
        let sticky_local = self.runtime_schema().reserve_i64_local(function);
        let increment_local = self.runtime_schema().reserve_i64_local(function);

        highest_bit_local.load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Sub);
        shift_local.store(function);
        shift_local.load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        shift_limb_local.store(function);
        shift_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64And);
        shift_offset_local.store(function);
        self.emit_math_sum_precise_load_limb(accumulator, shift_limb_local, limb_local, function);
        limb_local.load(function);
        shift_offset_local.load(function);
        function.instruction(&Instruction::I64ShrU);
        significand_local.store(function);

        shift_offset_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        shift_limb_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        self.emit_math_sum_precise_load_limb(accumulator, index_local, next_limb_local, function);
        significand_local.load(function);
        next_limb_local.load(function);
        function.instruction(&Instruction::I64Const(64));
        shift_offset_local.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        significand_local.store(function);
        function.instruction(&Instruction::End);
        significand_local.load(function);
        function.instruction(&Instruction::I64Const((1_i64 << 53) - 1));
        function.instruction(&Instruction::I64And);
        significand_local.store(function);

        shift_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        increment_local.store(function);
        function.instruction(&Instruction::Else);
        shift_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        guard_index_local.store(function);
        self.emit_math_sum_precise_extract_bit(
            accumulator,
            guard_index_local,
            guard_local,
            function,
        );
        self.emit_math_sum_precise_sticky_below(
            accumulator,
            guard_index_local,
            sticky_local,
            function,
        );
        guard_local.load(function);
        sticky_local.load(function);
        significand_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64And);
        increment_local.store(function);
        function.instruction(&Instruction::End);
        significand_local.load(function);
        increment_local.load(function);
        function.instruction(&Instruction::I64Add);
        significand_local.store(function);

        significand_local.load(function);
        function.instruction(&Instruction::I64Const(1_i64 << 53));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        significand_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        significand_local.store(function);
        highest_bit_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        highest_bit_local.store(function);
        function.instruction(&Instruction::End);

        highest_bit_local.load(function);
        function.instruction(&Instruction::I64Const(2_098));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_bits_local.load(function);
        function.instruction(&Instruction::I64Const(0x7ff0_0000_0000_0000_u64 as i64));
        function.instruction(&Instruction::I64Or);
        output_bits.store(function);
        function.instruction(&Instruction::Else);
        highest_bit_local.load(function);
        function.instruction(&Instruction::I64Const(51));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        significand_local.load(function);
        function.instruction(&Instruction::I64Const(FRACTION_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        sign_bits_local.load(function);
        function.instruction(&Instruction::I64Or);
        output_bits.store(function);
        function.instruction(&Instruction::End);

        for local in [
            increment_local,
            sticky_local,
            guard_local,
            guard_index_local,
            significand_local,
            next_limb_local,
            shift_offset_local,
            shift_limb_local,
            shift_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.runtime_schema()
            .release_i64_local(sign_bits_local, function);
        self.runtime_schema()
            .release_i64_local(highest_bit_local, function);
        self.runtime_schema()
            .release_i64_local(limb_local, function);
        self.runtime_schema()
            .release_i64_local(index_local, function);
    }

    fn emit_finish_math_sum_precise(
        &mut self,
        reduction: CompletedMathSumPreciseReduction<'_>,
        output_bits: I64Local,
        function: &mut Function,
    ) {
        let CompletedMathSumPreciseReduction {
            accumulator,
            state_local,
        } = reduction;

        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusZero.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const((-0.0_f64).to_bits() as i64));
        output_bits.store(function);
        function.instruction(&Instruction::Else);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::Finite.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let negative_local = self.emit_math_sum_precise_make_magnitude(accumulator, function);
        self.emit_math_sum_precise_round_finite(accumulator, negative_local, output_bits, function);
        self.runtime_schema()
            .release_i64_local(negative_local, function);
        function.instruction(&Instruction::Else);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::PlusInfinity.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        output_bits.store(function);
        function.instruction(&Instruction::Else);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusInfinity.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::NEG_INFINITY.to_bits() as i64));
        output_bits.store(function);
        function.instruction(&Instruction::Else);
        state_local.load(function);
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::NotANumber.abi_word(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        output_bits.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_runtime_math_sum_precise(
        &mut self,
        output: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let source = schema.reserve_value_local(function);
        self.emit_builtin_arg_to_value(0, &source, function);
        // ToObject at the shared acquisition boundary performs the required
        // RequireObjectCoercible; Get and Call retain the original receiver.
        // Acquisition failure terminates this native invocation before any
        // accumulator exists.
        let iterator =
            self.emit_get_sync_iterator(&source, SyncIteratorConsumer::MathSumPrecise, function)?;
        source.clear(function);
        let accumulator = MathSumPreciseAccumulator::new(schema, function);
        let state = schema.reserve_i64_local(function);
        let count = schema.reserve_i64_local(function);
        let bits = schema.reserve_i64_local(function);
        let done = schema.reserve_i32_local(function);
        let value = schema.reserve_value_local(function);
        let rejection = schema.reserve_completion(function);
        Self::emit_math_sum_precise_state_store(MathSumPreciseState::MinusZero, state, function);
        function.instruction(&Instruction::I64Const(0));
        count.store(function);
        function.instruction(&Instruction::I32Const(0));
        done.store(function);
        let cleanup = self.open_frame(ControlFrameKind::Block, function);
        let reduced = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        // The shared step marks DONE and propagates its original whole Throw
        // directly. No IteratorClose is introduced for a failed step/value.
        self.emit_sync_iterator_step_value(&iterator, done, &value, function)?;
        done.load(function);
        self.emit_branch_if_to_target(reduced, function);
        count.load(function);
        function.instruction(&Instruction::I64Const(MATH_SUM_PRECISE_MAX_COUNT));
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_range_error(
            RuntimeErrorMessage::MATH_SUMPRECISE_ITERABLE_CONTAINS_TOO_MANY_VALUES,
            &rejection,
            function,
        )?;
        self.emit_sync_iterator_close(&iterator, &rejection, output, function)?;
        self.emit_branch_to_target(cleanup, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Number.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::MATH_SUMPRECISE_NON_NUMBER_ELEMENT,
            &rejection,
            function,
        )?;
        self.emit_sync_iterator_close(&iterator, &rejection, output, function)?;
        self.emit_branch_to_target(cleanup, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_math_sum_precise_accept_number(&accumulator, state, value.scalar(), function);
        count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        count.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_finish_math_sum_precise(
            CompletedMathSumPreciseReduction {
                accumulator: &accumulator,
                state_local: state,
            },
            bits,
            function,
        );
        value.set_number(bits, function);
        output.set_normal(&value, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        rejection.clear(function);
        value.clear(function);
        schema.release_i32_local(done, function);
        schema.release_i64_local(bits, function);
        schema.release_i64_local(count, function);
        schema.release_i64_local(state, function);
        accumulator.clear(function);
        iterator.clear(function);
        Ok(())
    }
}
