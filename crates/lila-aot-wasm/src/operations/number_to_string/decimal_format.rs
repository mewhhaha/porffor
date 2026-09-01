use super::*;

// A finite binary64 is `M * 2^e`, with at most 53 significand bits and
// `e >= -1074`. Rewriting a negative exponent as `M * 5^-e * 10^e` needs at
// most 767 decimal digits; positive exponents need at most 309. Requested
// formatting never enlarges those worst cases, while fixed formatting below
// 1e21 can append at most 100 digits.
const EXACT_DECIMAL_DIGIT_CAPACITY: i64 = 768;

/// The complete decimal formatting policy after argument coercion and range
/// validation. Only exponential formatting admits the shortest representation.
pub(in crate::operations) enum NumberDecimalFormat {
    Fixed { fraction_digits_local: u32 },
    Exponential(NumberExponentialFormat),
    Precision { significant_digits_local: u32 },
}

pub(in crate::operations) enum NumberExponentialFormat {
    Shortest,
    FractionDigits { fraction_digits_local: u32 },
}

impl<'a> FunctionBuilder<'a> {
    pub(in crate::operations) fn emit_number_decimal_format_payload(
        &mut self,
        payload_local: u32,
        format: NumberDecimalFormat,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let output_local = self.reserve_temp_local();
        match format {
            NumberDecimalFormat::Fixed {
                fraction_digits_local,
            } => {
                let scratch_local = self.reserve_temp_local();
                let digit_start_local = self.reserve_temp_local();
                let digit_count_local = self.reserve_temp_local();
                let exact_exponent_local = self.reserve_temp_local();
                let scientific_exponent_local = self.reserve_temp_local();
                let sign_local = self.reserve_temp_local();
                let decimal_shift_local = self.reserve_temp_local();
                self.emit_exact_binary64_decimal(
                    payload_local,
                    scratch_local,
                    digit_start_local,
                    digit_count_local,
                    exact_exponent_local,
                    scientific_exponent_local,
                    sign_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(exact_exponent_local));
                function.instruction(&Instruction::LocalGet(fraction_digits_local));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(decimal_shift_local));
                self.emit_round_exact_decimal(
                    scratch_local,
                    digit_start_local,
                    digit_count_local,
                    decimal_shift_local,
                    function,
                );
                self.emit_fixed_exact_decimal_payload(
                    sign_local,
                    scratch_local,
                    digit_start_local,
                    digit_count_local,
                    fraction_digits_local,
                    output_local,
                    function,
                )?;
                self.release_temp_local(decimal_shift_local);
                self.release_temp_local(sign_local);
                self.release_temp_local(scientific_exponent_local);
                self.release_temp_local(exact_exponent_local);
                self.release_temp_local(digit_count_local);
                self.release_temp_local(digit_start_local);
                self.release_temp_local(scratch_local);
            }
            NumberDecimalFormat::Exponential(exponential_format) => match exponential_format {
                NumberExponentialFormat::Shortest => {
                    self.emit_shortest_exponential_payload(payload_local, output_local, function)?;
                }
                NumberExponentialFormat::FractionDigits {
                    fraction_digits_local,
                } => {
                    let significant_digits_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalGet(fraction_digits_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(significant_digits_local));
                    self.emit_exact_significant_decimal_payload(
                        payload_local,
                        significant_digits_local,
                        ExactSignificantDecimalPlacement::Scientific,
                        output_local,
                        function,
                    )?;
                    self.release_temp_local(significant_digits_local);
                }
            },
            NumberDecimalFormat::Precision {
                significant_digits_local,
            } => {
                self.emit_exact_significant_decimal_payload(
                    payload_local,
                    significant_digits_local,
                    ExactSignificantDecimalPlacement::Precision,
                    output_local,
                    function,
                )?;
            }
        }
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(output_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_exact_binary64_decimal(
        &mut self,
        payload_local: u32,
        scratch_local: u32,
        digit_start_local: u32,
        digit_count_local: u32,
        exact_exponent_local: u32,
        scientific_exponent_local: u32,
        sign_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let capacity_local = self.reserve_temp_local();
        let absolute_bits_local = self.reserve_temp_local();
        let ieee_mantissa_local = self.reserve_temp_local();
        let ieee_exponent_local = self.reserve_temp_local();
        let binary_mantissa_local = self.reserve_temp_local();
        let binary_exponent_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(EXACT_DECIMAL_DIGIT_CAPACITY));
        function.instruction(&Instruction::LocalSet(capacity_local));
        self.emit_heap_alloc_from_local(capacity_local, function)?;
        function.instruction(&Instruction::LocalSet(scratch_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(digit_start_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(absolute_bits_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(absolute_bits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(binary_mantissa_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(binary_exponent_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(absolute_bits_local));
        function.instruction(&Instruction::I64Const((1_i64 << 52) - 1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(ieee_mantissa_local));
        function.instruction(&Instruction::LocalGet(absolute_bits_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(ieee_exponent_local));
        function.instruction(&Instruction::LocalGet(ieee_exponent_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(ieee_mantissa_local));
        function.instruction(&Instruction::LocalSet(binary_mantissa_local));
        function.instruction(&Instruction::I64Const(-1074));
        function.instruction(&Instruction::LocalSet(binary_exponent_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(ieee_mantissa_local));
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(binary_mantissa_local));
        function.instruction(&Instruction::LocalGet(ieee_exponent_local));
        function.instruction(&Instruction::I64Const(1023 + 52));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(binary_exponent_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_initialize_exact_decimal_digits(
            binary_mantissa_local,
            scratch_local,
            digit_count_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exact_exponent_local));
        function.instruction(&Instruction::LocalGet(binary_exponent_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_repeat_exact_decimal_multiply(
            scratch_local,
            digit_count_local,
            binary_exponent_local,
            2,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(binary_exponent_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        let multiplier_count_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(binary_exponent_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(multiplier_count_local));
        self.emit_repeat_exact_decimal_multiply(
            scratch_local,
            digit_count_local,
            multiplier_count_local,
            5,
            function,
        );
        function.instruction(&Instruction::LocalGet(binary_exponent_local));
        function.instruction(&Instruction::LocalSet(exact_exponent_local));
        self.release_temp_local(multiplier_count_local);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalGet(exact_exponent_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(scientific_exponent_local));

        self.release_temp_local(binary_exponent_local);
        self.release_temp_local(binary_mantissa_local);
        self.release_temp_local(ieee_exponent_local);
        self.release_temp_local(ieee_mantissa_local);
        self.release_temp_local(absolute_bits_local);
        self.release_temp_local(capacity_local);
        Ok(())
    }

    fn emit_initialize_exact_decimal_digits(
        &mut self,
        binary_mantissa_local: u32,
        scratch_local: u32,
        digit_count_local: u32,
        function: &mut Function,
    ) {
        let remaining_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::LocalGet(binary_mantissa_local));
        function.instruction(&Instruction::LocalSet(remaining_local));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(remaining_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(address_local);
        self.release_temp_local(digit_local);
        self.release_temp_local(remaining_local);
    }

    fn emit_repeat_exact_decimal_multiply(
        &mut self,
        scratch_local: u32,
        digit_count_local: u32,
        repetitions_local: u32,
        factor: i64,
        function: &mut Function,
    ) {
        let repetition_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(repetition_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(repetition_local));
        function.instruction(&Instruction::LocalGet(repetitions_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_exact_decimal_multiply(scratch_local, digit_count_local, factor, function);
        function.instruction(&Instruction::LocalGet(repetition_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(repetition_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(repetition_local);
    }

    fn emit_exact_decimal_multiply(
        &mut self,
        scratch_local: u32,
        digit_count_local: u32,
        factor: i64,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        let product_local = self.reserve_temp_local();
        let carry_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(carry_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(factor));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(product_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(product_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(product_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(carry_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(carry_local);
        self.release_temp_local(product_local);
        self.release_temp_local(address_local);
        self.release_temp_local(index_local);
    }

    fn emit_round_exact_decimal(
        &mut self,
        scratch_local: u32,
        digit_start_local: u32,
        digit_count_local: u32,
        decimal_shift_local: u32,
        function: &mut Function,
    ) {
        let shift_magnitude_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let address_local = self.reserve_temp_local();
        let rounding_digit_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(decimal_shift_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::LocalGet(decimal_shift_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(decimal_shift_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalGet(decimal_shift_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(decimal_shift_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift_magnitude_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rounding_digit_local));
        function.instruction(&Instruction::LocalGet(shift_magnitude_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(shift_magnitude_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(rounding_digit_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(shift_magnitude_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::LocalGet(shift_magnitude_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_start_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalGet(shift_magnitude_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(digit_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(rounding_digit_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address_local));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(address_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(digit_local);
        self.release_temp_local(rounding_digit_local);
        self.release_temp_local(address_local);
        self.release_temp_local(index_local);
        self.release_temp_local(shift_magnitude_local);
    }
}

enum ExactSignificantDecimalPlacement {
    Scientific,
    Precision,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_exact_significant_decimal_payload(
        &mut self,
        payload_local: u32,
        significant_digits_local: u32,
        placement: ExactSignificantDecimalPlacement,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let scratch_local = self.reserve_temp_local();
        let digit_start_local = self.reserve_temp_local();
        let digit_count_local = self.reserve_temp_local();
        let exact_exponent_local = self.reserve_temp_local();
        let scientific_exponent_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let decimal_shift_local = self.reserve_temp_local();
        self.emit_exact_binary64_decimal(
            payload_local,
            scratch_local,
            digit_start_local,
            digit_count_local,
            exact_exponent_local,
            scientific_exponent_local,
            sign_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(exact_exponent_local));
        function.instruction(&Instruction::LocalGet(significant_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(scientific_exponent_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(decimal_shift_local));
        self.emit_round_exact_decimal(
            scratch_local,
            digit_start_local,
            digit_count_local,
            decimal_shift_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalGet(significant_digits_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_start_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit_count_local));
        function.instruction(&Instruction::LocalGet(scientific_exponent_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scientific_exponent_local));
        function.instruction(&Instruction::End);

        match placement {
            ExactSignificantDecimalPlacement::Scientific => {
                self.emit_scientific_exact_decimal_payload(
                    sign_local,
                    scratch_local,
                    digit_start_local,
                    digit_count_local,
                    scientific_exponent_local,
                    output_local,
                    function,
                )?;
            }
            ExactSignificantDecimalPlacement::Precision => {
                function.instruction(&Instruction::LocalGet(scientific_exponent_local));
                function.instruction(&Instruction::I64Const(-6));
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::LocalGet(scientific_exponent_local));
                function.instruction(&Instruction::LocalGet(significant_digits_local));
                function.instruction(&Instruction::I64GeS);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_scientific_exact_decimal_payload(
                    sign_local,
                    scratch_local,
                    digit_start_local,
                    digit_count_local,
                    scientific_exponent_local,
                    output_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                let fraction_digits_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(significant_digits_local));
                function.instruction(&Instruction::LocalGet(scientific_exponent_local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(fraction_digits_local));
                self.emit_fixed_exact_decimal_payload(
                    sign_local,
                    scratch_local,
                    digit_start_local,
                    digit_count_local,
                    fraction_digits_local,
                    output_local,
                    function,
                )?;
                self.release_temp_local(fraction_digits_local);
                function.instruction(&Instruction::End);
            }
        }

        self.release_temp_local(decimal_shift_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(scientific_exponent_local);
        self.release_temp_local(exact_exponent_local);
        self.release_temp_local(digit_count_local);
        self.release_temp_local(digit_start_local);
        self.release_temp_local(scratch_local);
        Ok(())
    }

    fn emit_shortest_exponential_payload(
        &mut self,
        payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let absolute_bits_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let mantissa_local = self.reserve_temp_local();
        let exponent_local = self.reserve_temp_local();
        let mantissa_length_local = self.reserve_temp_local();
        let decimal_point_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(absolute_bits_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(absolute_bits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mantissa_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exponent_local));
        function.instruction(&Instruction::Else);
        self.emit_ryu_shortest_decimal(
            absolute_bits_local,
            mantissa_local,
            exponent_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_count_decimal_digits_u64(mantissa_local, mantissa_length_local, function);
        function.instruction(&Instruction::LocalGet(mantissa_length_local));
        function.instruction(&Instruction::LocalGet(exponent_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(decimal_point_local));
        self.emit_scientific_decimal_payload(
            sign_local,
            mantissa_local,
            mantissa_length_local,
            decimal_point_local,
            output_local,
            function,
        )?;
        self.release_temp_local(decimal_point_local);
        self.release_temp_local(mantissa_length_local);
        self.release_temp_local(exponent_local);
        self.release_temp_local(mantissa_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(absolute_bits_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_fixed_exact_decimal_payload(
        &mut self,
        sign_local: u32,
        scratch_local: u32,
        digit_start_local: u32,
        digit_count_local: u32,
        fraction_digits_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let total_length_local = self.reserve_temp_local();
        let output_offset_local = self.reserve_temp_local();
        let number_start_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_length_local));
        self.emit_heap_alloc_from_local(total_length_local, function)?;
        function.instruction(&Instruction::LocalSet(output_offset_local));
        self.emit_decimal_sign(
            sign_local,
            output_offset_local,
            number_start_local,
            function,
        );
        self.emit_write_exact_decimal_digits(
            scratch_local,
            digit_start_local,
            digit_count_local,
            number_start_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_length_local));
        self.emit_heap_alloc_from_local(total_length_local, function)?;
        function.instruction(&Instruction::LocalSet(output_offset_local));
        self.emit_decimal_sign(
            sign_local,
            output_offset_local,
            number_start_local,
            function,
        );
        let decimal_point_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(decimal_point_local));
        self.emit_write_exact_decimal_digits_with_point(
            scratch_local,
            digit_start_local,
            digit_count_local,
            decimal_point_local,
            number_start_local,
            function,
        );
        self.release_temp_local(decimal_point_local);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_length_local));
        self.emit_heap_alloc_from_local(total_length_local, function)?;
        function.instruction(&Instruction::LocalSet(output_offset_local));
        self.emit_decimal_sign(
            sign_local,
            output_offset_local,
            number_start_local,
            function,
        );
        self.store_ascii_byte_i64(number_start_local, b'0', function);
        let fraction_start_local = self.reserve_temp_local();
        let zero_count_local = self.reserve_temp_local();
        let digit_output_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(number_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_start_local));
        self.store_ascii_byte_i64(fraction_start_local, b'.', function);
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(zero_count_local));
        function.instruction(&Instruction::LocalGet(fraction_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_start_local));
        self.emit_repeated_ascii(fraction_start_local, zero_count_local, b'0', function);
        function.instruction(&Instruction::LocalGet(fraction_start_local));
        function.instruction(&Instruction::LocalGet(zero_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_output_local));
        self.emit_write_exact_decimal_digits(
            scratch_local,
            digit_start_local,
            digit_count_local,
            digit_output_local,
            function,
        );
        self.release_temp_local(digit_output_local);
        self.release_temp_local(zero_count_local);
        self.release_temp_local(fraction_start_local);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(output_offset_local, total_length_local, function);
        function.instruction(&Instruction::LocalSet(output_local));
        self.release_temp_local(number_start_local);
        self.release_temp_local(output_offset_local);
        self.release_temp_local(total_length_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_scientific_exact_decimal_payload(
        &mut self,
        sign_local: u32,
        scratch_local: u32,
        digit_start_local: u32,
        digit_count_local: u32,
        scientific_exponent_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let exponent_magnitude_local = self.reserve_temp_local();
        let exponent_digits_local = self.reserve_temp_local();
        let significand_length_local = self.reserve_temp_local();
        let total_length_local = self.reserve_temp_local();
        let output_offset_local = self.reserve_temp_local();
        let number_start_local = self.reserve_temp_local();
        let exponent_start_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(scientific_exponent_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(scientific_exponent_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(scientific_exponent_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(exponent_magnitude_local));
        self.emit_count_decimal_digits_u64(
            exponent_magnitude_local,
            exponent_digits_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(significand_length_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(significand_length_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(exponent_digits_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_length_local));
        self.emit_heap_alloc_from_local(total_length_local, function)?;
        function.instruction(&Instruction::LocalSet(output_offset_local));
        self.emit_decimal_sign(
            sign_local,
            output_offset_local,
            number_start_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_write_exact_decimal_digits(
            scratch_local,
            digit_start_local,
            digit_count_local,
            number_start_local,
            function,
        );
        function.instruction(&Instruction::Else);
        let decimal_point_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(decimal_point_local));
        self.emit_write_exact_decimal_digits_with_point(
            scratch_local,
            digit_start_local,
            digit_count_local,
            decimal_point_local,
            number_start_local,
            function,
        );
        self.release_temp_local(decimal_point_local);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(number_start_local));
        function.instruction(&Instruction::LocalGet(significand_length_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(exponent_start_local));
        self.store_ascii_byte_i64(exponent_start_local, b'e', function);
        function.instruction(&Instruction::LocalGet(exponent_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(exponent_start_local));
        function.instruction(&Instruction::LocalGet(scientific_exponent_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(exponent_start_local, b'-', function);
        function.instruction(&Instruction::Else);
        self.store_ascii_byte_i64(exponent_start_local, b'+', function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(exponent_start_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(exponent_start_local));
        self.emit_write_decimal_u64(
            exponent_magnitude_local,
            exponent_start_local,
            exponent_digits_local,
            function,
        );
        self.emit_pack_string_payload(output_offset_local, total_length_local, function);
        function.instruction(&Instruction::LocalSet(output_local));
        self.release_temp_local(exponent_start_local);
        self.release_temp_local(number_start_local);
        self.release_temp_local(output_offset_local);
        self.release_temp_local(total_length_local);
        self.release_temp_local(significand_length_local);
        self.release_temp_local(exponent_digits_local);
        self.release_temp_local(exponent_magnitude_local);
        Ok(())
    }

    fn emit_write_exact_decimal_digits(
        &mut self,
        scratch_local: u32,
        digit_start_local: u32,
        digit_count_local: u32,
        output_start_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let source_local = self.reserve_temp_local();
        let destination_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(source_local));
        function.instruction(&Instruction::LocalGet(source_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(output_start_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(destination_local));
        function.instruction(&Instruction::LocalGet(destination_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(digit_local);
        self.release_temp_local(destination_local);
        self.release_temp_local(source_local);
        self.release_temp_local(index_local);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_write_exact_decimal_digits_with_point(
        &mut self,
        scratch_local: u32,
        digit_start_local: u32,
        digit_count_local: u32,
        decimal_point_local: u32,
        output_start_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        let source_local = self.reserve_temp_local();
        let destination_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(scratch_local));
        function.instruction(&Instruction::LocalGet(digit_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(source_local));
        function.instruction(&Instruction::LocalGet(source_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(output_start_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(decimal_point_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(destination_local));
        function.instruction(&Instruction::LocalGet(destination_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(output_start_local));
        function.instruction(&Instruction::LocalGet(decimal_point_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(destination_local));
        self.store_ascii_byte_i64(destination_local, b'.', function);
        self.release_temp_local(digit_local);
        self.release_temp_local(destination_local);
        self.release_temp_local(source_local);
        self.release_temp_local(index_local);
    }
}
