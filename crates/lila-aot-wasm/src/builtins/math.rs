use super::super::*;
use crate::gc_types::{CompletionLocals, I64Local, ValueLocals};
mod sum_precise;

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

#[must_use = "a completed Math.hypot reduction must be finished"]
struct CompletedMathHypotReduction {
    scale_local: I64Local,
    scaled_sum_local: I64Local,
    saw_infinity_local: I64Local,
    saw_nan_local: I64Local,
}

impl MathExtremum {
    const fn identity(&self) -> f64 {
        match self {
            Self::Minimum => f64::INFINITY,
            Self::Maximum => f64::NEG_INFINITY,
        }
    }

    fn emit_combine(
        &self,
        accumulator_local: I64Local,
        argument_local: I64Local,
        function: &mut Function,
    ) {
        accumulator_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        argument_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        match self {
            Self::Minimum => function.instruction(&Instruction::F64Min),
            Self::Maximum => function.instruction(&Instruction::F64Max),
        };
        function.instruction(&Instruction::I64ReinterpretF64);
        accumulator_local.store(function);
    }
}

impl<'a> FunctionBuilder<'a> {
    /// A native Math argument is usable only after the whole ToNumber result
    /// is Normal. A Throw branches to the caller's single cleanup edge.
    fn emit_math_coerce_number(
        &mut self,
        input: &ValueLocals,
        bits: I64Local,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(input, pending, function)?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(pending, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.value().scalar().load(function);
        bits.store(function);
        Ok(())
    }

    fn emit_math_hypot_argument_reduction(
        &mut self,
        arg_payload_local: I64Local,
        argument: &ValueLocals,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) -> Result<CompletedMathHypotReduction, EmitError> {
        let scale_local = self.runtime_schema().reserve_i64_local(function);
        let scaled_sum_local = self.runtime_schema().reserve_i64_local(function);
        let saw_infinity_local = self.runtime_schema().reserve_i64_local(function);
        let saw_nan_local = self.runtime_schema().reserve_i64_local(function);
        let schema = self.runtime_schema();
        let argument_index_local = schema.reserve_i32_local(function);
        let magnitude_local = self.runtime_schema().reserve_i64_local(function);
        let ratio_local = self.runtime_schema().reserve_i64_local(function);

        for local in [scale_local, scaled_sum_local] {
            function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
            function.instruction(&Instruction::I64ReinterpretF64);
            local.store(function);
        }
        for local in [saw_infinity_local, saw_nan_local] {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }

        function.instruction(&Instruction::I32Const(0));
        argument_index_local.store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        argument_index_local.load(function);
        schema.array_type::<crate::gc_types::ValueArray>().length(
            self.body_entry_locals()
                .expect("Math owns native entry")
                .arguments(),
            schema,
            function,
        );
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_argument_vector_entry_to_value(
            self.body_entry_locals()
                .expect("Math owns native entry")
                .arguments(),
            argument_index_local,
            argument,
            function,
        );
        self.emit_math_coerce_number(argument, arg_payload_local, pending, output, exit, function)?;

        arg_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::I64ReinterpretF64);
        magnitude_local.store(function);

        magnitude_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        saw_infinity_local.store(function);
        function.instruction(&Instruction::Else);

        arg_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        arg_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        saw_nan_local.store(function);
        function.instruction(&Instruction::Else);

        magnitude_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        magnitude_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        scale_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));

        scale_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        magnitude_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        ratio_local.store(function);
        scaled_sum_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        ratio_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        ratio_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        scaled_sum_local.store(function);
        magnitude_local.load(function);
        scale_local.store(function);

        function.instruction(&Instruction::Else);

        magnitude_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        scale_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        ratio_local.store(function);
        scaled_sum_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        ratio_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        ratio_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        scaled_sum_local.store(function);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        argument_index_local.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        argument_index_local.store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.runtime_schema()
            .release_i64_local(ratio_local, function);
        self.runtime_schema()
            .release_i64_local(magnitude_local, function);
        schema.release_i32_local(argument_index_local, function);

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
        output_bits: I64Local,
        function: &mut Function,
    ) {
        let CompletedMathHypotReduction {
            scale_local,
            scaled_sum_local,
            saw_infinity_local,
            saw_nan_local,
        } = reduction;

        saw_infinity_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        saw_nan_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        scale_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        output_bits.store(function);
        function.instruction(&Instruction::Else);
        scale_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        scaled_sum_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sqrt);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        output_bits.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        output_bits.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::I64ReinterpretF64);
        output_bits.store(function);
        function.instruction(&Instruction::End);

        self.runtime_schema()
            .release_i64_local(saw_nan_local, function);
        self.runtime_schema()
            .release_i64_local(saw_infinity_local, function);
        self.runtime_schema()
            .release_i64_local(scaled_sum_local, function);
        self.runtime_schema()
            .release_i64_local(scale_local, function);
    }

    fn emit_math_extremum_builtin(
        &mut self,
        extremum: MathExtremum,
        arg_payload_local: I64Local,
        argument: &ValueLocals,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        output_bits: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument_index_local = schema.reserve_i32_local(function);

        function.instruction(&Instruction::F64Const(Ieee64::from(extremum.identity())));
        function.instruction(&Instruction::I64ReinterpretF64);
        output_bits.store(function);
        function.instruction(&Instruction::I32Const(0));
        argument_index_local.store(function);

        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        argument_index_local.load(function);
        schema.array_type::<crate::gc_types::ValueArray>().length(
            self.body_entry_locals()
                .expect("Math owns native entry")
                .arguments(),
            schema,
            function,
        );
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));

        self.emit_argument_vector_entry_to_value(
            self.body_entry_locals()
                .expect("Math owns native entry")
                .arguments(),
            argument_index_local,
            argument,
            function,
        );
        self.emit_math_coerce_number(argument, arg_payload_local, pending, output, exit, function)?;
        extremum.emit_combine(output_bits, arg_payload_local, function);

        argument_index_local.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        argument_index_local.store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        schema.release_i32_local(argument_index_local, function);
        Ok(())
    }

    /// Call a unary `(f64) -> f64` Math host import, storing the reinterpreted
    /// result payload in `result_local`. The input is the ToNumber payload in
    /// `input_payload_local`; every caller keeps its spec fast paths around
    /// this call, so tabled inputs never change value.
    fn emit_math_host_unary_import_call(
        &self,
        input_payload_local: I64Local,
        import_function_index: u32,
        output_bits: I64Local,
        function: &mut Function,
    ) {
        input_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::Call(import_function_index));
        function.instruction(&Instruction::I64ReinterpretF64);
        output_bits.store(function);
    }

    /// Binary `(f64, f64) -> f64` counterpart for `Math.atan2(y, x)`.
    fn emit_math_host_binary_import_call(
        &self,
        lhs_payload_local: I64Local,
        rhs_payload_local: I64Local,
        import_function_index: u32,
        output_bits: I64Local,
        function: &mut Function,
    ) {
        lhs_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        rhs_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::Call(import_function_index));
        function.instruction(&Instruction::I64ReinterpretF64);
        output_bits.store(function);
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
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let output = schema.reserve_completion(function);
        output.initialize(function);
        let output_bits = schema.reserve_i64_local(function);
        let arg_payload_local = schema.reserve_i64_local(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        match builtin {
            MathBuiltin::SumPrecise => {
                self.emit_runtime_math_sum_precise(&output, function)?;
                self.emit_branch_to_target(exit, function);
            }
            MathBuiltin::Hypot => {
                let reduction = self.emit_math_hypot_argument_reduction(
                    arg_payload_local,
                    &argument,
                    &pending,
                    &output,
                    exit,
                    function,
                )?;
                self.emit_finish_math_hypot(reduction, output_bits, function);
            }
            MathBuiltin::Atan2 => {
                let y_payload_local = self.runtime_schema().reserve_i64_local(function);
                let import_function_index = self
                    .functions
                    .math_atan2_import_function_index()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "Math.atan2 requires the lila_host.math_atan2 import",
                        )
                    })?;
                self.emit_builtin_arg_to_value(0, &argument, function);
                self.emit_math_coerce_number(
                    &argument,
                    y_payload_local,
                    &pending,
                    &output,
                    exit,
                    function,
                )?;
                self.emit_builtin_arg_to_value(1, &argument, function);
                self.emit_math_coerce_number(
                    &argument,
                    arg_payload_local,
                    &pending,
                    &output,
                    exit,
                    function,
                )?;
                y_payload_local.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Gt);
                arg_payload_local.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::I32And);
                y_payload_local.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Eq);
                y_payload_local.load(function);
                function.instruction(&Instruction::I64Const(0.0f64.to_bits() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                arg_payload_local.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Ge);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                output_bits.store(function);
                function.instruction(&Instruction::Else);
                y_payload_local.load(function);
                function.instruction(&Instruction::I64Const((-0.0f64).to_bits() as i64));
                function.instruction(&Instruction::I64Eq);
                arg_payload_local.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Gt);
                arg_payload_local.load(function);
                function.instruction(&Instruction::I64Const(0.0f64.to_bits() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::F64Const(Ieee64::from(-0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                output_bits.store(function);
                function.instruction(&Instruction::Else);
                y_payload_local.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Lt);
                arg_payload_local.load(function);
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                function.instruction(&Instruction::F64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::F64Const(Ieee64::from(-0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                output_bits.store(function);
                function.instruction(&Instruction::Else);
                self.emit_math_host_binary_import_call(
                    y_payload_local,
                    arg_payload_local,
                    import_function_index,
                    output_bits,
                    function,
                );
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                self.runtime_schema()
                    .release_i64_local(y_payload_local, function);
            }
            MathBuiltin::Imul => {
                let lhs_uint32_local = self.runtime_schema().reserve_i64_local(function);
                self.emit_builtin_arg_to_value(0, &argument, function);
                self.emit_math_coerce_number(
                    &argument,
                    lhs_uint32_local,
                    &pending,
                    &output,
                    exit,
                    function,
                )?;
                self.emit_to_uint32_i64_from_number_payload(
                    lhs_uint32_local,
                    lhs_uint32_local,
                    function,
                );
                self.emit_builtin_arg_to_value(1, &argument, function);
                self.emit_math_coerce_number(
                    &argument,
                    arg_payload_local,
                    &pending,
                    &output,
                    exit,
                    function,
                )?;
                self.emit_to_uint32_i64_from_number_payload(
                    arg_payload_local,
                    arg_payload_local,
                    function,
                );
                arg_payload_local.load(function);
                function.instruction(&Instruction::I32WrapI64);
                lhs_uint32_local.load(function);
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32Mul);
                function.instruction(&Instruction::F64ConvertI32S);
                function.instruction(&Instruction::I64ReinterpretF64);
                output_bits.store(function);
                self.runtime_schema()
                    .release_i64_local(lhs_uint32_local, function);
            }
            MathBuiltin::Min => self.emit_math_extremum_builtin(
                MathExtremum::Minimum,
                arg_payload_local,
                &argument,
                &pending,
                &output,
                exit,
                output_bits,
                function,
            )?,
            MathBuiltin::Max => self.emit_math_extremum_builtin(
                MathExtremum::Maximum,
                arg_payload_local,
                &argument,
                &pending,
                &output,
                exit,
                output_bits,
                function,
            )?,
            MathBuiltin::Pow => {
                let exponent_payload_local = self.runtime_schema().reserve_i64_local(function);
                let base_payload_local = self.runtime_schema().reserve_i64_local(function);
                self.emit_builtin_arg_to_value(0, &argument, function);
                self.emit_math_coerce_number(
                    &argument,
                    base_payload_local,
                    &pending,
                    &output,
                    exit,
                    function,
                )?;
                self.emit_builtin_arg_to_value(1, &argument, function);
                self.emit_math_coerce_number(
                    &argument,
                    exponent_payload_local,
                    &pending,
                    &output,
                    exit,
                    function,
                )?;

                self.emit_number_pow_payload(
                    base_payload_local,
                    exponent_payload_local,
                    output_bits,
                    function,
                )?;

                self.runtime_schema()
                    .release_i64_local(base_payload_local, function);
                self.runtime_schema()
                    .release_i64_local(exponent_payload_local, function);
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
                output_bits.store(function);
            }
            MathBuiltin::Unary(unary) => {
                self.emit_builtin_arg_to_value(0, &argument, function);
                self.emit_math_coerce_number(
                    &argument,
                    output_bits,
                    &pending,
                    &output,
                    exit,
                    function,
                )?;
                output_bits.load(function);
                arg_payload_local.store(function);
                match unary {
                    MathUnaryBuiltin::Abs => {
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                    }
                    MathUnaryBuiltin::Ceil => {
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ceil);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                    }
                    MathUnaryBuiltin::Floor => {
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Floor);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                    }
                    MathUnaryBuiltin::Fround => {
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F32DemoteF64);
                        function.instruction(&Instruction::F64PromoteF32);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                    }
                    MathUnaryBuiltin::Sqrt => {
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Sqrt);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                    }
                    MathUnaryBuiltin::Trunc => {
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Trunc);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                    }
                    MathUnaryBuiltin::Clz32 => {
                        self.emit_to_uint32_i64_from_number_payload(
                            arg_payload_local,
                            arg_payload_local,
                            function,
                        );
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::I32WrapI64);
                        function.instruction(&Instruction::I32Clz);
                        function.instruction(&Instruction::F64ConvertI32U);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                    }
                    MathUnaryBuiltin::Exp => {
                        let import_function_index = self
                            .functions
                            .math_exp_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.exp requires the lila_host.math_exp import",
                                )
                            })?;
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Sinh => {
                        let import_function_index = self
                            .functions
                            .math_sinh_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.sinh requires the lila_host.math_sinh import",
                                )
                            })?;
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ne);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Log1p => {
                        let import_function_index = self
                            .functions
                            .math_log1p_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.log1p requires the lila_host.math_log1p import",
                                )
                            })?;
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Log => {
                        let import_function_index = self
                            .functions
                            .math_log_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.log requires the lila_host.math_log import",
                                )
                            })?;
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        for (input, output) in [
                            (0.0, f64::NEG_INFINITY),
                            (1.0, 0.0),
                            (f64::INFINITY, f64::INFINITY),
                        ] {
                            arg_payload_local.load(function);
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(input)));
                            function.instruction(&Instruction::F64Eq);
                            function.instruction(&Instruction::If(BlockType::Empty));
                            function.instruction(&Instruction::F64Const(Ieee64::from(output)));
                            function.instruction(&Instruction::I64ReinterpretF64);
                            output_bits.store(function);
                            function.instruction(&Instruction::End);
                        }
                    }
                    MathUnaryBuiltin::Atanh => {
                        let import_function_index = self
                            .functions
                            .math_atanh_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.atanh requires the lila_host.math_atanh import",
                                )
                            })?;
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Cosh => {
                        let import_function_index = self
                            .functions
                            .math_cosh_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.cosh requires the lila_host.math_cosh import",
                                )
                            })?;
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Log10 => {
                        let import_function_index = self
                            .functions
                            .math_log10_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.log10 requires the lila_host.math_log10 import",
                                )
                            })?;
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        for (input, output) in [
                            (0.0, f64::NEG_INFINITY),
                            (f64::INFINITY, f64::INFINITY),
                            (1.0, 0.0),
                            (10.0, 1.0),
                            (100.0, 2.0),
                            (1000.0, 3.0),
                        ] {
                            arg_payload_local.load(function);
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(input)));
                            function.instruction(&Instruction::F64Eq);
                            function.instruction(&Instruction::If(BlockType::Empty));
                            function.instruction(&Instruction::F64Const(Ieee64::from(output)));
                            function.instruction(&Instruction::I64ReinterpretF64);
                            output_bits.store(function);
                            function.instruction(&Instruction::End);
                        }
                    }
                    MathUnaryBuiltin::Log2 => {
                        let import_function_index = self
                            .functions
                            .math_log2_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.log2 requires the lila_host.math_log2 import",
                                )
                            })?;
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        for (input, output) in [
                            (0.0, f64::NEG_INFINITY),
                            (f64::INFINITY, f64::INFINITY),
                            (1.0, 0.0),
                            (2.0, 1.0),
                            (4.0, 2.0),
                            (8.0, 3.0),
                        ] {
                            arg_payload_local.load(function);
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(input)));
                            function.instruction(&Instruction::F64Eq);
                            function.instruction(&Instruction::If(BlockType::Empty));
                            function.instruction(&Instruction::F64Const(Ieee64::from(output)));
                            function.instruction(&Instruction::I64ReinterpretF64);
                            output_bits.store(function);
                            function.instruction(&Instruction::End);
                        }
                    }
                    MathUnaryBuiltin::Acosh => {
                        let import_function_index = self
                            .functions
                            .math_acosh_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.acosh requires the lila_host.math_acosh import",
                                )
                            })?;
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Cos => {
                        let import_function_index = self
                            .functions
                            .math_cos_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.cos requires the lila_host.math_cos import",
                                )
                            })?;
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Asin => {
                        let import_function_index = self
                            .functions
                            .math_asin_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.asin requires the lila_host.math_asin import",
                                )
                            })?;
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Atan => {
                        let import_function_index = self
                            .functions
                            .math_atan_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.atan requires the lila_host.math_atan import",
                                )
                            })?;
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Sin => {
                        let import_function_index = self
                            .functions
                            .math_sin_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.sin requires the lila_host.math_sin import",
                                )
                            })?;
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Round => {
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(4503599627370496.0)));
                        function.instruction(&Instruction::F64Ge);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(-0.5)));
                        function.instruction(&Instruction::F64Ge);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Lt);
                        function.instruction(&Instruction::I32And);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(-0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Ge);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.5)));
                        function.instruction(&Instruction::F64Lt);
                        function.instruction(&Instruction::I32And);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.5)));
                        function.instruction(&Instruction::F64Add);
                        function.instruction(&Instruction::F64Floor);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Expm1 => {
                        let import_function_index = self
                            .functions
                            .math_expm1_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.expm1 requires the lila_host.math_expm1 import",
                                )
                            })?;
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function
                            .instruction(&Instruction::F64Const(Ieee64::from(f64::NEG_INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Cbrt => {
                        let import_function_index = self
                            .functions
                            .math_cbrt_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.cbrt requires the lila_host.math_cbrt import",
                                )
                            })?;
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ne);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::F16Round => {
                        let half_bits_local = self.runtime_schema().reserve_i64_local(function);
                        let half_sign_local = self.runtime_schema().reserve_i64_local(function);
                        let half_exp_local = self.runtime_schema().reserve_i64_local(function);
                        let half_frac_local = self.runtime_schema().reserve_i64_local(function);
                        let half_remainder_local =
                            self.runtime_schema().reserve_i64_local(function);
                        let half_significand_local =
                            self.runtime_schema().reserve_i64_local(function);
                        self.emit_f64_payload_to_half_bits_local(
                            arg_payload_local,
                            half_bits_local,
                            half_sign_local,
                            half_exp_local,
                            half_frac_local,
                            output_bits,
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
                        output_bits.store(function);
                        self.runtime_schema()
                            .release_i64_local(half_significand_local, function);
                        self.runtime_schema()
                            .release_i64_local(half_remainder_local, function);
                        self.runtime_schema()
                            .release_i64_local(half_frac_local, function);
                        self.runtime_schema()
                            .release_i64_local(half_exp_local, function);
                        self.runtime_schema()
                            .release_i64_local(half_sign_local, function);
                        self.runtime_schema()
                            .release_i64_local(half_bits_local, function);
                    }
                    MathUnaryBuiltin::Sign => {
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ne);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Lt);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(-1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Tanh => {
                        let import_function_index = self
                            .functions
                            .math_tanh_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.tanh requires the lila_host.math_tanh import",
                                )
                            })?;
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        arg_payload_local.load(function);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                        for (input, output) in [(f64::NEG_INFINITY, -1.0), (f64::INFINITY, 1.0)] {
                            arg_payload_local.load(function);
                            function.instruction(&Instruction::F64ReinterpretI64);
                            function.instruction(&Instruction::F64Const(Ieee64::from(input)));
                            function.instruction(&Instruction::F64Eq);
                            function.instruction(&Instruction::If(BlockType::Empty));
                            function.instruction(&Instruction::F64Const(Ieee64::from(output)));
                            function.instruction(&Instruction::I64ReinterpretF64);
                            output_bits.store(function);
                            function.instruction(&Instruction::End);
                        }
                    }
                    MathUnaryBuiltin::Tan => {
                        let import_function_index = self
                            .functions
                            .math_tan_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.tan requires the lila_host.math_tan import",
                                )
                            })?;
                        output_bits.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Acos => {
                        let import_function_index = self
                            .functions
                            .math_acos_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.acos requires the lila_host.math_acos import",
                                )
                            })?;
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        output_bits.store(function);
                        function.instruction(&Instruction::End);
                    }
                    MathUnaryBuiltin::Asinh => {
                        let import_function_index = self
                            .functions
                            .math_asinh_import_function_index()
                            .ok_or_else(|| {
                                EmitError::unsupported(
                                    "Math.asinh requires the lila_host.math_asinh import",
                                )
                            })?;
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                        function.instruction(&Instruction::F64Eq);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                        function.instruction(&Instruction::F64Eq);
                        function.instruction(&Instruction::I32Or);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        arg_payload_local.load(function);
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ne);
                        function.instruction(&Instruction::I32Or);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::Else);
                        self.emit_math_host_unary_import_call(
                            arg_payload_local,
                            import_function_index,
                            output_bits,
                            function,
                        );
                        function.instruction(&Instruction::End);
                    }
                }
            }
        }
        argument.set_number(output_bits, function);
        output.set_normal(&argument, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        schema.release_i64_local(arg_payload_local, function);
        schema.release_i64_local(output_bits, function);
        output.clear(function);
        pending.clear(function);
        argument.clear(function);
        Ok(())
    }
}
