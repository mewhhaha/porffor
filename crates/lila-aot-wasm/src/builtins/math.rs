use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MathBuiltin {
    Abs,
    Acos,
    Acosh,
    Asin,
    Asinh,
    Atan,
    Atan2,
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
    Hypot,
    Imul,
    Log,
    Log10,
    Log1p,
    Log2,
    Pow,
    Random,
    Round,
    Sign,
    Sin,
    Sinh,
    Sqrt,
    SumPrecise,
    Tan,
    Tanh,
    Trunc,
    Min,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MathExtremum {
    Minimum,
    Maximum,
}

#[must_use = "a completed Math.hypot reduction must be finished"]
struct CompletedMathHypotReduction {
    scale_local: u32,
    scaled_sum_local: u32,
    saw_infinity_local: u32,
    saw_nan_local: u32,
}

impl MathExtremum {
    const fn identity(self) -> f64 {
        match self {
            Self::Minimum => f64::INFINITY,
            Self::Maximum => f64::NEG_INFINITY,
        }
    }

    fn emit_combine(self, accumulator_local: u32, argument_local: u32, function: &mut Function) {
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

    pub(super) fn emit_math(
        &mut self,
        builtin: MathBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        match builtin {
            MathBuiltin::SumPrecise => {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Math.sumPrecise non-number element",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
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
            MathBuiltin::Abs
            | MathBuiltin::Acos
            | MathBuiltin::Acosh
            | MathBuiltin::Asin
            | MathBuiltin::Asinh
            | MathBuiltin::Atan
            | MathBuiltin::Atanh
            | MathBuiltin::Cbrt
            | MathBuiltin::Ceil
            | MathBuiltin::Clz32
            | MathBuiltin::Cos
            | MathBuiltin::Cosh
            | MathBuiltin::Exp
            | MathBuiltin::Expm1
            | MathBuiltin::F16Round
            | MathBuiltin::Floor
            | MathBuiltin::Fround
            | MathBuiltin::Log
            | MathBuiltin::Log10
            | MathBuiltin::Log1p
            | MathBuiltin::Log2
            | MathBuiltin::Round
            | MathBuiltin::Sign
            | MathBuiltin::Sin
            | MathBuiltin::Sinh
            | MathBuiltin::Sqrt
            | MathBuiltin::Tan
            | MathBuiltin::Tanh
            | MathBuiltin::Trunc => {
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_number_payload(arg_tag_local, arg_payload_local, function)?;
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalSet(arg_payload_local));
                match builtin {
                    MathBuiltin::Abs => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Abs);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathBuiltin::Ceil => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Ceil);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathBuiltin::Floor => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Floor);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathBuiltin::Fround => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F32DemoteF64);
                        function.instruction(&Instruction::F64PromoteF32);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathBuiltin::Sqrt => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Sqrt);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathBuiltin::Trunc => {
                        function.instruction(&Instruction::LocalGet(self.result_local));
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Trunc);
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathBuiltin::Clz32 => {
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
                    MathBuiltin::Exp => {
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
                    MathBuiltin::Sinh => {
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
                    MathBuiltin::Log1p => {
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
                    MathBuiltin::Log => {
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
                    MathBuiltin::Atanh => {
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
                    MathBuiltin::Cosh => {
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
                    MathBuiltin::Log10 => {
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
                    MathBuiltin::Log2 => {
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
                    MathBuiltin::Acosh => {
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
                    MathBuiltin::Cos => {
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
                    MathBuiltin::Asin | MathBuiltin::Atan | MathBuiltin::Sin => {
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
                    MathBuiltin::Round => {
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
                    MathBuiltin::Expm1 => {
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
                    MathBuiltin::Cbrt => {
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
                    MathBuiltin::F16Round => {
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
                    MathBuiltin::Sign => {
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
                    MathBuiltin::Tanh => {
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
                    MathBuiltin::Tan => {
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
                    MathBuiltin::Acos => {
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
                    MathBuiltin::Asinh => {
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
                    MathBuiltin::Atan2 | MathBuiltin::Imul => {
                        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                        function.instruction(&Instruction::I64ReinterpretF64);
                        function.instruction(&Instruction::LocalSet(self.result_local));
                    }
                    MathBuiltin::SumPrecise
                    | MathBuiltin::Hypot
                    | MathBuiltin::Min
                    | MathBuiltin::Max
                    | MathBuiltin::Pow
                    | MathBuiltin::Random => {
                        unreachable!("non-unary Math builtin reached unary dispatch")
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
