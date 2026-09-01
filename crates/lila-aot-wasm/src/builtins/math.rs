use super::super::*;
use crate::control_flow::SyncIteratorConsumer;

enum MathBuiltin {
    Unary(MathUnaryBuiltin),
    Atan2,
    Hypot,
    Imul,
    Max,
    Min,
    Pow,
    Random,
    SumPrecise,
}

enum MathUnaryBuiltin {
    Abs,
    Acos,
    Acosh,
    Asin,
    Asinh,
    Atan,
    Atanh,
    Cbrt,
    Ceil,
    Clz32,
    Cos,
    Cosh,
    Exp,
    Expm1,
    F16Round,
    Floor,
    Fround,
    Log,
    Log10,
    Log1p,
    Log2,
    Round,
    Sign,
    Sin,
    Sinh,
    Sqrt,
    Tan,
    Tanh,
    Trunc,
}

enum MathExtremum {
    Minimum,
    Maximum,
}

const MATH_SUM_PRECISE_MAX_COUNT: i64 = (1_i64 << 53) - 1;
const MATH_SUM_PRECISE_MAX_EXACT_BITS: usize = 2_151;
const MATH_SUM_PRECISE_LIMB_BITS: usize = 64;
const MATH_SUM_PRECISE_LIMBS: usize =
    (MATH_SUM_PRECISE_MAX_EXACT_BITS + MATH_SUM_PRECISE_LIMB_BITS - 1) / MATH_SUM_PRECISE_LIMB_BITS;
const MATH_SUM_PRECISE_BYTES: u64 = (MATH_SUM_PRECISE_LIMBS * 8) as u64;
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
    ptr_local: u32,
}

#[must_use = "a completed Math.sumPrecise reduction must be finished"]
struct CompletedMathSumPreciseReduction {
    accumulator: MathSumPreciseAccumulator,
    state_local: u32,
}

#[must_use = "a completed Math.hypot reduction must be finished"]
struct CompletedMathHypotReduction {
    scale_local: u32,
    scaled_sum_local: u32,
    saw_infinity_local: u32,
    saw_nan_local: u32,
}

impl MathExtremum {
    const fn identity(&self) -> f64 {
        match self {
            Self::Minimum => f64::INFINITY,
            Self::Maximum => f64::NEG_INFINITY,
        }
    }

    fn emit_combine(&self, accumulator_local: u32, argument_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(accumulator_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(argument_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        match self {
            Self::Minimum => function.instruction(&Instruction::F64Min),
            Self::Maximum => function.instruction(&Instruction::F64Max),
        };
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(accumulator_local));
    }
}

impl<'a> FunctionBuilder<'a> {
    fn emit_math_sum_precise_state_store(
        state: MathSumPreciseState,
        state_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(state.abi_word()));
        function.instruction(&Instruction::LocalSet(state_local));
    }

    fn emit_math_sum_precise_load_limb(
        &self,
        accumulator: &MathSumPreciseAccumulator,
        index_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(accumulator.ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(dest_local));
    }

    fn emit_math_sum_precise_store_limb(
        &self,
        accumulator: &MathSumPreciseAccumulator,
        index_local: u32,
        value_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(accumulator.ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
    }

    fn emit_math_sum_precise_initialize_accumulator(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_heap_alloc_const(MATH_SUM_PRECISE_BYTES, function)?;
        function.instruction(&Instruction::LocalSet(accumulator.ptr_local));
        for index in 0..MATH_SUM_PRECISE_LIMBS {
            self.store_i64_const_at_offset(accumulator.ptr_local, (index * 8) as u64, 0, function);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_math_sum_precise_fold_limbs(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        first_index_local: u32,
        low_local: u32,
        high_local: u32,
        operation: MathSumPreciseLimbOperation,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let addend_local = self.reserve_temp_local();
        let next_addend_local = self.reserve_temp_local();
        let carry_local = self.reserve_temp_local();
        let next_carry_local = self.reserve_temp_local();
        let old_local = self.reserve_temp_local();
        let updated_local = self.reserve_temp_local();
        let partial_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(first_index_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(low_local));
        function.instruction(&Instruction::LocalSet(addend_local));
        function.instruction(&Instruction::LocalGet(high_local));
        function.instruction(&Instruction::LocalSet(next_addend_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(carry_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MATH_SUM_PRECISE_LIMBS as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_math_sum_precise_load_limb(accumulator, index_local, old_local, function);
        function.instruction(&Instruction::LocalGet(old_local));
        function.instruction(&Instruction::LocalGet(addend_local));
        match &operation {
            MathSumPreciseLimbOperation::Add => function.instruction(&Instruction::I64Add),
            MathSumPreciseLimbOperation::Subtract => function.instruction(&Instruction::I64Sub),
        };
        function.instruction(&Instruction::LocalSet(partial_local));
        function.instruction(&Instruction::LocalGet(partial_local));
        function.instruction(&Instruction::LocalGet(old_local));
        match &operation {
            MathSumPreciseLimbOperation::Add => function.instruction(&Instruction::I64LtU),
            MathSumPreciseLimbOperation::Subtract => function.instruction(&Instruction::I64GtU),
        };
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(next_carry_local));

        function.instruction(&Instruction::LocalGet(partial_local));
        function.instruction(&Instruction::LocalGet(carry_local));
        match &operation {
            MathSumPreciseLimbOperation::Add => function.instruction(&Instruction::I64Add),
            MathSumPreciseLimbOperation::Subtract => function.instruction(&Instruction::I64Sub),
        };
        function.instruction(&Instruction::LocalSet(updated_local));
        function.instruction(&Instruction::LocalGet(updated_local));
        function.instruction(&Instruction::LocalGet(partial_local));
        match &operation {
            MathSumPreciseLimbOperation::Add => function.instruction(&Instruction::I64LtU),
            MathSumPreciseLimbOperation::Subtract => function.instruction(&Instruction::I64GtU),
        };
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(next_carry_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(next_carry_local));
        self.emit_math_sum_precise_store_limb(accumulator, index_local, updated_local, function);

        function.instruction(&Instruction::LocalGet(next_addend_local));
        function.instruction(&Instruction::LocalSet(addend_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(next_addend_local));
        function.instruction(&Instruction::LocalGet(next_carry_local));
        function.instruction(&Instruction::LocalSet(carry_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
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
            self.release_temp_local(local);
        }
    }

    fn emit_math_sum_precise_add_finite(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        number_bits_local: u32,
        function: &mut Function,
    ) {
        const FRACTION_MASK: i64 = ((1_u64 << 52) - 1) as i64;
        let exponent_local = self.reserve_temp_local();
        let significand_local = self.reserve_temp_local();
        let shift_local = self.reserve_temp_local();
        let first_index_local = self.reserve_temp_local();
        let bit_offset_local = self.reserve_temp_local();
        let low_local = self.reserve_temp_local();
        let high_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(exponent_local));
        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Const(FRACTION_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(significand_local));

        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(shift_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(significand_local));
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(first_index_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(bit_offset_local));
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::LocalGet(bit_offset_local));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(low_local));
        function.instruction(&Instruction::LocalGet(bit_offset_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(high_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::LocalGet(bit_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(high_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(number_bits_local));
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
            self.release_temp_local(local);
        }
    }

    fn emit_math_sum_precise_accept_number(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        state_local: u32,
        number_bits_local: u32,
        function: &mut Function,
    ) {
        const ABS_MASK: i64 = i64::MAX;
        const EXPONENT_MASK: i64 = 0x7ff0_0000_0000_0000_u64 as i64;
        const FRACTION_MASK: i64 = ((1_u64 << 52) - 1) as i64;

        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Const(EXPONENT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(EXPONENT_MASK));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Const(FRACTION_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(state_local));
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
        function.instruction(&Instruction::LocalGet(state_local));
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
        function.instruction(&Instruction::LocalGet(state_local));
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
        function.instruction(&Instruction::LocalGet(state_local));
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
        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Const(ABS_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(number_bits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusZero.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_math_sum_precise_state_store(MathSumPreciseState::Finite, state_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusZero.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(state_local));
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

    fn emit_math_sum_precise_reduction(
        &mut self,
        source_payload_local: u32,
        source_tag_local: u32,
        function: &mut Function,
    ) -> Result<CompletedMathSumPreciseReduction, EmitError> {
        let accumulator = MathSumPreciseAccumulator {
            ptr_local: self.reserve_temp_local(),
        };
        let state_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let iterator_locals = self.reserve_sync_iterator_locals();
        let done_local = self.reserve_temp_local();
        let consumer = SyncIteratorConsumer::MathSumPrecise;
        let count_local = self.reserve_temp_local();
        let close_saved_payload_local = self.reserve_temp_local();
        let close_saved_tag_local = self.reserve_temp_local();
        let close_saved_completion_local = self.reserve_temp_local();
        let close_saved_aux_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, source_payload_local, source_tag_local, function);
        self.emit_get_iterator_from_value_locals(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            source_payload_local,
            source_tag_local,
            method_payload_local,
            method_tag_local,
            &iterator_locals,
            &consumer,
            function,
        )?;

        self.emit_math_sum_precise_initialize_accumulator(&accumulator, function)?;
        Self::emit_math_sum_precise_state_store(
            MathSumPreciseState::MinusZero,
            state_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));

        let close = IteratorCloseOnThrowLocals {
            iterator_payload_local: iterator_locals.iterator_payload,
            iterator_tag_local: iterator_locals.iterator_tag,
            key_local: iterator_locals.key,
            return_payload_local: method_payload_local,
            return_tag_local: method_tag_local,
            result_payload_local: iterator_locals.result_payload,
            result_tag_local: iterator_locals.result_tag,
            saved_payload_local: close_saved_payload_local,
            saved_tag_local: close_saved_tag_local,
            saved_completion_local: close_saved_completion_local,
            saved_aux_local: close_saved_aux_local,
        };

        let break_target = self.open_frame(ControlFrameKind::Block, function);
        let loop_target = self.open_frame(ControlFrameKind::Loop, function);
        self.emit_sync_iterator_step_value(&iterator_locals, done_local, &consumer, function)?;
        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        self.emit_branch_if_to_target(break_target, function);

        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(MATH_SUM_PRECISE_MAX_COUNT));
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_range_error(
            "Math.sumPrecise iterable contains too many values",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_iterator_close_preserving_current_throw(close, function)?;
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(iterator_locals.value_tag));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            "Math.sumPrecise non-number element",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_iterator_close_preserving_current_throw(close, function)?;
        self.emit_return_current_completion(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.emit_math_sum_precise_accept_number(
            &accumulator,
            state_local,
            iterator_locals.value_payload,
            function,
        );
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        self.emit_branch_to_target(loop_target, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        for local in [
            close_saved_aux_local,
            close_saved_completion_local,
            close_saved_tag_local,
            close_saved_payload_local,
            count_local,
            done_local,
        ] {
            self.release_temp_local(local);
        }
        self.release_sync_iterator_locals(iterator_locals);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);

        Ok(CompletedMathSumPreciseReduction {
            accumulator,
            state_local,
        })
    }

    fn emit_math_sum_precise_make_magnitude(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        function: &mut Function,
    ) -> u32 {
        let negative_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let limb_local = self.reserve_temp_local();
        let inverted_local = self.reserve_temp_local();
        let carry_local = self.reserve_temp_local();
        let updated_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const((MATH_SUM_PRECISE_LIMBS - 1) as i64));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_math_sum_precise_load_limb(accumulator, index_local, limb_local, function);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(negative_local));

        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(carry_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(MATH_SUM_PRECISE_LIMBS as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_math_sum_precise_load_limb(accumulator, index_local, limb_local, function);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::LocalSet(inverted_local));
        function.instruction(&Instruction::LocalGet(inverted_local));
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(updated_local));
        function.instruction(&Instruction::LocalGet(updated_local));
        function.instruction(&Instruction::LocalGet(inverted_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(carry_local));
        self.emit_math_sum_precise_store_limb(accumulator, index_local, updated_local, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
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
            self.release_temp_local(local);
        }
        negative_local
    }

    fn emit_math_sum_precise_extract_bit(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        bit_index_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let limb_index_local = self.reserve_temp_local();
        let limb_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(bit_index_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(limb_index_local));
        self.emit_math_sum_precise_load_limb(accumulator, limb_index_local, limb_local, function);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::LocalGet(bit_index_local));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(result_local));
        self.release_temp_local(limb_local);
        self.release_temp_local(limb_index_local);
    }

    fn emit_math_sum_precise_sticky_below(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        exclusive_bit_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        let limit_limb_local = self.reserve_temp_local();
        let limit_offset_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let limb_local = self.reserve_temp_local();
        let mask_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(exclusive_bit_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(limit_limb_local));
        function.instruction(&Instruction::LocalGet(exclusive_bit_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(limit_offset_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(limit_limb_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_math_sum_precise_load_limb(accumulator, index_local, limb_local, function);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(limit_offset_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_math_sum_precise_load_limb(accumulator, limit_limb_local, limb_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(limit_offset_local));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(mask_local));
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::LocalGet(mask_local));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result_local));

        for local in [
            mask_local,
            limb_local,
            index_local,
            limit_offset_local,
            limit_limb_local,
        ] {
            self.release_temp_local(local);
        }
    }

    fn emit_math_sum_precise_round_finite(
        &mut self,
        accumulator: &MathSumPreciseAccumulator,
        negative_local: u32,
        function: &mut Function,
    ) {
        const FRACTION_MASK: i64 = ((1_u64 << 52) - 1) as i64;
        let index_local = self.reserve_temp_local();
        let limb_local = self.reserve_temp_local();
        let highest_bit_local = self.reserve_temp_local();
        let sign_bits_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(sign_bits_local));
        function.instruction(&Instruction::I64Const(MATH_SUM_PRECISE_LIMBS as i64));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(limb_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_math_sum_precise_load_limb(accumulator, index_local, limb_local, function);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::I64Clz);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(highest_bit_local));

        function.instruction(&Instruction::LocalGet(highest_bit_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::LocalGet(sign_bits_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);

        let shift_local = self.reserve_temp_local();
        let shift_limb_local = self.reserve_temp_local();
        let shift_offset_local = self.reserve_temp_local();
        let next_limb_local = self.reserve_temp_local();
        let significand_local = self.reserve_temp_local();
        let guard_index_local = self.reserve_temp_local();
        let guard_local = self.reserve_temp_local();
        let sticky_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(highest_bit_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(shift_limb_local));
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(shift_offset_local));
        self.emit_math_sum_precise_load_limb(accumulator, shift_limb_local, limb_local, function);
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::LocalGet(shift_offset_local));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(significand_local));

        function.instruction(&Instruction::LocalGet(shift_offset_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shift_limb_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_math_sum_precise_load_limb(accumulator, index_local, next_limb_local, function);
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::LocalGet(next_limb_local));
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::LocalGet(shift_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(significand_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const((1_i64 << 53) - 1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(significand_local));

        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(shift_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(guard_index_local));
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
        function.instruction(&Instruction::LocalGet(guard_local));
        function.instruction(&Instruction::LocalGet(sticky_local));
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(significand_local));

        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(1_i64 << 53));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(significand_local));
        function.instruction(&Instruction::LocalGet(highest_bit_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(highest_bit_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(highest_bit_local));
        function.instruction(&Instruction::I64Const(2_098));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_bits_local));
        function.instruction(&Instruction::I64Const(0x7ff0_0000_0000_0000_u64 as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(highest_bit_local));
        function.instruction(&Instruction::I64Const(51));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(significand_local));
        function.instruction(&Instruction::I64Const(FRACTION_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(sign_bits_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(self.result_local));
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
            self.release_temp_local(local);
        }

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(sign_bits_local);
        self.release_temp_local(highest_bit_local);
        self.release_temp_local(limb_local);
        self.release_temp_local(index_local);
    }

    fn emit_finish_math_sum_precise(
        &mut self,
        reduction: CompletedMathSumPreciseReduction,
        function: &mut Function,
    ) {
        let CompletedMathSumPreciseReduction {
            accumulator,
            state_local,
        } = reduction;

        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusZero.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const((-0.0_f64).to_bits() as i64));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::Finite.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let negative_local = self.emit_math_sum_precise_make_magnitude(&accumulator, function);
        self.emit_math_sum_precise_round_finite(&accumulator, negative_local, function);
        self.release_temp_local(negative_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::PlusInfinity.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::MinusInfinity.abi_word(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::NEG_INFINITY.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            MathSumPreciseState::NotANumber.abi_word(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(state_local);
        self.release_temp_local(accumulator.ptr_local);
    }

    fn emit_math_hypot_argument_reduction(
        &mut self,
        arg_payload_local: u32,
        arg_tag_local: u32,
        function: &mut Function,
    ) -> Result<CompletedMathHypotReduction, EmitError> {
        let scale_local = self.reserve_temp_local();
        let scaled_sum_local = self.reserve_temp_local();
        let saw_infinity_local = self.reserve_temp_local();
        let saw_nan_local = self.reserve_temp_local();
        let argument_index_local = self.reserve_temp_local();
        let magnitude_local = self.reserve_temp_local();
        let ratio_local = self.reserve_temp_local();

        for local in [scale_local, scaled_sum_local] {
            function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(local));
        }
        for local in [saw_infinity_local, saw_nan_local, argument_index_local] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            self.argv_param_local(),
            argument_index_local,
            arg_payload_local,
            arg_tag_local,
            function,
        );
        self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_payload_local));
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(magnitude_local));

        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(saw_infinity_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(saw_nan_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(ratio_local));
        function.instruction(&Instruction::LocalGet(scaled_sum_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(ratio_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(ratio_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scaled_sum_local));
        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::LocalSet(scale_local));

        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(magnitude_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(ratio_local));
        function.instruction(&Instruction::LocalGet(scaled_sum_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(ratio_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(ratio_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scaled_sum_local));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(argument_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(ratio_local);
        self.release_temp_local(magnitude_local);
        self.release_temp_local(argument_index_local);

        Ok(CompletedMathHypotReduction {
            scale_local,
            scaled_sum_local,
            saw_infinity_local,
            saw_nan_local,
        })
    }

    fn emit_finish_math_hypot(
        &mut self,
        reduction: CompletedMathHypotReduction,
        function: &mut Function,
    ) {
        let CompletedMathHypotReduction {
            scale_local,
            scaled_sum_local,
            saw_infinity_local,
            saw_nan_local,
        } = reduction;

        function.instruction(&Instruction::LocalGet(saw_infinity_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(saw_nan_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scale_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(scaled_sum_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sqrt);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(saw_nan_local);
        self.release_temp_local(saw_infinity_local);
        self.release_temp_local(scaled_sum_local);
        self.release_temp_local(scale_local);
    }

    fn emit_math_extremum_builtin(
        &mut self,
        extremum: MathExtremum,
        arg_payload_local: u32,
        arg_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_index_local = self.reserve_temp_local();

        function.instruction(&Instruction::F64Const(Ieee64::from(extremum.identity())));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(argument_index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_array_read(
            self.argv_param_local(),
            argument_index_local,
            arg_payload_local,
            arg_tag_local,
            function,
        );
        self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(arg_payload_local));
        self.emit_return_current_completion_if_throw(function);
        extremum.emit_combine(self.result_local, arg_payload_local, function);

        function.instruction(&Instruction::LocalGet(argument_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(argument_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(argument_index_local);
        Ok(())
    }

    pub(super) fn emit_math_abs_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Abs), function)
    }

    pub(super) fn emit_math_acos_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Acos), function)
    }

    pub(super) fn emit_math_acosh_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Acosh), function)
    }

    pub(super) fn emit_math_asin_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Asin), function)
    }

    pub(super) fn emit_math_asinh_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Asinh), function)
    }

    pub(super) fn emit_math_atan_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Atan), function)
    }

    pub(super) fn emit_math_atan2_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Atan2, function)
    }

    pub(super) fn emit_math_atanh_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Atanh), function)
    }

    pub(super) fn emit_math_cbrt_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Cbrt), function)
    }

    pub(super) fn emit_math_ceil_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Ceil), function)
    }

    pub(super) fn emit_math_clz32_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Clz32), function)
    }

    pub(super) fn emit_math_cos_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Cos), function)
    }

    pub(super) fn emit_math_cosh_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Cosh), function)
    }

    pub(super) fn emit_math_exp_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Exp), function)
    }

    pub(super) fn emit_math_expm1_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Expm1), function)
    }

    pub(super) fn emit_math_f16round_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::F16Round), function)
    }

    pub(super) fn emit_math_floor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Floor), function)
    }

    pub(super) fn emit_math_fround_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Fround), function)
    }

    pub(super) fn emit_math_hypot_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Hypot, function)
    }

    pub(super) fn emit_math_imul_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Imul, function)
    }

    pub(super) fn emit_math_log_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Log), function)
    }

    pub(super) fn emit_math_log10_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Log10), function)
    }

    pub(super) fn emit_math_log1p_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Log1p), function)
    }

    pub(super) fn emit_math_log2_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Log2), function)
    }

    pub(super) fn emit_math_pow_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Pow, function)
    }

    pub(super) fn emit_math_random_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Random, function)
    }

    pub(super) fn emit_math_round_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Round), function)
    }

    pub(super) fn emit_math_sign_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Sign), function)
    }

    pub(super) fn emit_math_sin_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Sin), function)
    }

    pub(super) fn emit_math_sinh_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Sinh), function)
    }

    pub(super) fn emit_math_sqrt_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Sqrt), function)
    }

    pub(super) fn emit_math_sum_precise_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::SumPrecise, function)
    }

    pub(super) fn emit_math_tan_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Tan), function)
    }

    pub(super) fn emit_math_tanh_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Tanh), function)
    }

    pub(super) fn emit_math_trunc_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Unary(MathUnaryBuiltin::Trunc), function)
    }

    pub(super) fn emit_math_min_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Min, function)
    }

    pub(super) fn emit_math_max_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_math(MathBuiltin::Max, function)
    }

    fn emit_math(
        &mut self,
        builtin: MathBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        match builtin {
            MathBuiltin::SumPrecise => {
                let reduction = self.emit_math_sum_precise_reduction(
                    arg_payload_local,
                    arg_tag_local,
                    function,
                )?;
                self.emit_finish_math_sum_precise(reduction, function);
            }
            MathBuiltin::Hypot => {
                let reduction = self.emit_math_hypot_argument_reduction(
                    arg_payload_local,
                    arg_tag_local,
                    function,
                )?;
                self.emit_finish_math_hypot(reduction, function);
            }
            MathBuiltin::Atan2 => {
                let y_payload_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, y_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, y_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(y_payload_local));
                self.emit_builtin_arg_to_locals(1, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(arg_payload_local));
                function.instruction(&Instruction::LocalGet(y_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Gt);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::LocalGet(y_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::LocalGet(y_payload_local));
                function.instruction(&Instruction::I64Const(0.0f64.to_bits() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Ge);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(y_payload_local));
                function.instruction(&Instruction::I64Const((-0.0f64).to_bits() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Gt);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::I64Const(0.0f64.to_bits() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::F64Const(Ieee64::from(-0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(y_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Lt);
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::F64Const(Ieee64::from(-0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                self.release_temp_local(y_payload_local);
            }
            MathBuiltin::Imul => {
                let lhs_uint32_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(lhs_uint32_local));
                self.emit_to_uint32_i64_from_number_payload(
                    lhs_uint32_local,
                    lhs_uint32_local,
                    function,
                );
                self.emit_builtin_arg_to_locals(1, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(arg_payload_local));
                self.emit_to_uint32_i64_from_number_payload(
                    arg_payload_local,
                    arg_payload_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(arg_payload_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::LocalGet(lhs_uint32_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Mul);
                function.instruction(&Instruction::F64ConvertI32S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                self.release_temp_local(lhs_uint32_local);
            }
            MathBuiltin::Min => self.emit_math_extremum_builtin(
                MathExtremum::Minimum,
                arg_payload_local,
                arg_tag_local,
                function,
            )?,
            MathBuiltin::Max => self.emit_math_extremum_builtin(
                MathExtremum::Maximum,
                arg_payload_local,
                arg_tag_local,
                function,
            )?,
            MathBuiltin::Pow => {
                let exponent_payload_local = self.reserve_temp_local();
                let base_payload_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(base_payload_local));
                self.emit_builtin_arg_to_locals(1, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(exponent_payload_local));

                self.emit_number_pow_payload(
                    base_payload_local,
                    exponent_payload_local,
                    self.result_local,
                    function,
                )?;

                self.release_temp_local(base_payload_local);
                self.release_temp_local(exponent_payload_local);
            }
            MathBuiltin::Random => {
                let random_f64_import_function_index = self
                    .functions
                    .random_f64_import_function_index()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "Math.random requires the lila_host.random_f64 import",
                        )
                    })?;
                function.instruction(&Instruction::Call(random_f64_import_function_index));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
            }
            MathBuiltin::Unary(unary) => {
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalSet(arg_payload_local));
                match unary {
                    MathUnaryBuiltin::Abs => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathUnaryBuiltin::Ceil => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ceil);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathUnaryBuiltin::Floor => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Floor);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathUnaryBuiltin::Fround => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F32DemoteF64);
                        function.instruction(&Instruction::F64PromoteF32);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathUnaryBuiltin::Sqrt => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Sqrt);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathUnaryBuiltin::Trunc => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Trunc);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathUnaryBuiltin::Clz32 => {
                        self.emit_to_uint32_i64_from_number_payload(
                            arg_payload_local,
                            arg_payload_local,
                            function,
                        );
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::I32WrapI64);
                        function.instruction(&Instruction::I32Clz);
                        function.instruction(&Instruction::F64ConvertI32U);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathUnaryBuiltin::Exp => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Sinh => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ne);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Log1p => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Log => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        for (input, output) in [
                            (0.0, f64::NEG_INFINITY),
                            (1.0, 0.0),
                            (f64::INFINITY, f64::INFINITY),
                        ] {
                            function.instruction(&Instruction::LocalGet(arg_payload_local));
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(input)));
                            function.instruction(&Instruction::F64Eq);
                            function.instruction(&Instruction::If(BlockType::Empty));
                            function.instruction(&Instruction::F64Const(Ieee64::from(output)));
                            function.instruction(&Instruction::I64ReinterpretF64);
                            function.instruction(&Instruction::LocalSet(self.result_local));
                            function.instruction(&Instruction::End);
                        }
                    }
                    MathUnaryBuiltin::Atanh => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Cosh => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Log10 => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        for (input, output) in [
                            (0.0, f64::NEG_INFINITY),
                            (f64::INFINITY, f64::INFINITY),
                            (1.0, 0.0),
                            (10.0, 1.0),
                            (100.0, 2.0),
                            (1000.0, 3.0),
                        ] {
                            function.instruction(&Instruction::LocalGet(arg_payload_local));
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(input)));
                            function.instruction(&Instruction::F64Eq);
                            function.instruction(&Instruction::If(BlockType::Empty));
                            function.instruction(&Instruction::F64Const(Ieee64::from(output)));
                            function.instruction(&Instruction::I64ReinterpretF64);
                            function.instruction(&Instruction::LocalSet(self.result_local));
                            function.instruction(&Instruction::End);
                        }
                    }
                    MathUnaryBuiltin::Log2 => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        for (input, output) in [
                            (0.0, f64::NEG_INFINITY),
                            (f64::INFINITY, f64::INFINITY),
                            (1.0, 0.0),
                            (2.0, 1.0),
                            (4.0, 2.0),
                            (8.0, 3.0),
                        ] {
                            function.instruction(&Instruction::LocalGet(arg_payload_local));
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(input)));
                            function.instruction(&Instruction::F64Eq);
                            function.instruction(&Instruction::If(BlockType::Empty));
                            function.instruction(&Instruction::F64Const(Ieee64::from(output)));
                            function.instruction(&Instruction::I64ReinterpretF64);
                            function.instruction(&Instruction::LocalSet(self.result_local));
                            function.instruction(&Instruction::End);
                        }
                    }
                    MathUnaryBuiltin::Acosh => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Cos => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Asin | MathUnaryBuiltin::Atan | MathUnaryBuiltin::Sin => {
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Round => {
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(4503599627370496.0)));
                        function.instruction(&Instruction::F64Ge);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(-0.5)));
                        function.instruction(&Instruction::F64Ge);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Lt);
                        function.instruction(&Instruction::I32And);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(-0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Ge);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.5)));
                        function.instruction(&Instruction::F64Lt);
                        function.instruction(&Instruction::I32And);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.5)));
                        function.instruction(&Instruction::F64Add);
                        function.instruction(&Instruction::F64Floor);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Expm1 => {
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Cbrt => {
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ne);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::F16Round => {
                        let half_bits_local = self.reserve_temp_local();
                        let half_sign_local = self.reserve_temp_local();
                        let half_exp_local = self.reserve_temp_local();
                        let half_frac_local = self.reserve_temp_local();
                        let half_remainder_local = self.reserve_temp_local();
                        let half_significand_local = self.reserve_temp_local();
                        self.emit_f64_payload_to_half_bits_local(
                            arg_payload_local,
                            half_bits_local,
                            half_sign_local,
                            half_exp_local,
                            half_frac_local,
                            self.result_local,
                            half_remainder_local,
                            half_significand_local,
                            function,
                        );
                        self.emit_half_bits_to_f64_payload(
                            half_bits_local,
                            half_sign_local,
                            half_exp_local,
                            half_frac_local,
                            half_significand_local,
                            half_remainder_local,
                            function,
                        );
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        self.release_temp_local(half_significand_local);
                        self.release_temp_local(half_remainder_local);
                        self.release_temp_local(half_frac_local);
                        self.release_temp_local(half_exp_local);
                        self.release_temp_local(half_sign_local);
                        self.release_temp_local(half_bits_local);
                    }
                    MathUnaryBuiltin::Sign => {
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ne);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Lt);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Tanh => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                        for (input, output) in [(f64::NEG_INFINITY, -1.0), (f64::INFINITY, 1.0)] {
                            function.instruction(&Instruction::LocalGet(arg_payload_local));
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(input)));
                            function.instruction(&Instruction::F64Eq);
                            function.instruction(&Instruction::If(BlockType::Empty));
                            function.instruction(&Instruction::F64Const(Ieee64::from(output)));
                            function.instruction(&Instruction::I64ReinterpretF64);
                            function.instruction(&Instruction::LocalSet(self.result_local));
                            function.instruction(&Instruction::End);
                        }
                    }
                    MathUnaryBuiltin::Tan => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Acos => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Asinh => {
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::LocalGet(arg_payload_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ne);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                        function.instruction(&Instruction::End);
                    }
                }
            }
        }
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }
}
