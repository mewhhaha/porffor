use super::FormattingBuffer;
use super::*;
use crate::gc_types::{GcLocal, I64Local, StringValue};

// A finite binary64 is `M * 2^e`, with at most 53 significand bits and
// `e >= -1074`. Rewriting a negative exponent as `M * 5^-e * 10^e` needs at
// most 767 decimal digits; positive exponents need at most 309. Requested
// formatting never enlarges those worst cases, while fixed formatting below
// 1e21 can append at most 100 digits.
const EXACT_DECIMAL_DIGIT_CAPACITY: i64 = 768;

/// The complete decimal formatting policy after argument coercion and range
/// validation. Only exponential formatting admits the shortest representation.
pub(in crate::operations) enum NumberDecimalFormat {
    Fixed { fraction_digits_local: I64Local },
    Exponential(NumberExponentialFormat),
    Precision { significant_digits_local: I64Local },
}

pub(in crate::operations) enum NumberExponentialFormat {
    Shortest,
    FractionDigits { fraction_digits_local: I64Local },
}

impl<'a> FunctionBuilder<'a> {
    pub(in crate::operations) fn emit_number_decimal_format_payload(
        &mut self,
        payload_local: I64Local,
        format: NumberDecimalFormat,
        function: &mut Function,
    ) -> Result<crate::gc_types::GcStackReference<StringValue>, EmitError> {
        let schema = self.runtime_schema();
        let output_local = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("0", function)?,
            function,
        );
        match format {
            NumberDecimalFormat::Fixed {
                fraction_digits_local,
            } => {
                let scratch_local = FormattingBuffer::empty(schema, function);
                let digit_start_local = schema.reserve_i64_local(function);
                let digit_count_local = schema.reserve_i64_local(function);
                let exact_exponent_local = schema.reserve_i64_local(function);
                let scientific_exponent_local = schema.reserve_i64_local(function);
                let sign_local = schema.reserve_i64_local(function);
                let decimal_shift_local = schema.reserve_i64_local(function);
                self.emit_exact_binary64_decimal(
                    payload_local,
                    &scratch_local,
                    digit_start_local,
                    digit_count_local,
                    exact_exponent_local,
                    scientific_exponent_local,
                    sign_local,
                    function,
                )?;
                exact_exponent_local.load(function);
                fraction_digits_local.load(function);
                function.instruction(&Instruction::I64Add);
                decimal_shift_local.store(function);
                self.emit_round_exact_decimal(
                    &scratch_local,
                    digit_start_local,
                    digit_count_local,
                    decimal_shift_local,
                    function,
                );
                self.emit_fixed_exact_decimal_payload(
                    sign_local,
                    &scratch_local,
                    digit_start_local,
                    digit_count_local,
                    fraction_digits_local,
                    &output_local,
                    function,
                )?;
                schema.release_i64_local(decimal_shift_local, function);
                schema.release_i64_local(sign_local, function);
                schema.release_i64_local(scientific_exponent_local, function);
                schema.release_i64_local(exact_exponent_local, function);
                schema.release_i64_local(digit_count_local, function);
                schema.release_i64_local(digit_start_local, function);
                scratch_local.clear(function);
            }
            NumberDecimalFormat::Exponential(exponential_format) => match exponential_format {
                NumberExponentialFormat::Shortest => {
                    self.emit_shortest_exponential_payload(payload_local, &output_local, function)?;
                }
                NumberExponentialFormat::FractionDigits {
                    fraction_digits_local,
                } => {
                    let significant_digits_local = schema.reserve_i64_local(function);
                    fraction_digits_local.load(function);
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    significant_digits_local.store(function);
                    self.emit_exact_significant_decimal_payload(
                        payload_local,
                        significant_digits_local,
                        ExactSignificantDecimalPlacement::Scientific,
                        &output_local,
                        function,
                    )?;
                    schema.release_i64_local(significant_digits_local, function);
                }
            },
            NumberDecimalFormat::Precision {
                significant_digits_local,
            } => {
                self.emit_exact_significant_decimal_payload(
                    payload_local,
                    significant_digits_local,
                    ExactSignificantDecimalPlacement::Precision,
                    &output_local,
                    function,
                )?;
            }
        }
        let output = output_local.load(schema, function);
        output_local.clear(function);
        Ok(output)
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_exact_binary64_decimal(
        &mut self,
        payload_local: I64Local,
        scratch_local: &FormattingBuffer,
        digit_start_local: I64Local,
        digit_count_local: I64Local,
        exact_exponent_local: I64Local,
        scientific_exponent_local: I64Local,
        sign_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let capacity_local = schema.reserve_i64_local(function);
        let absolute_bits_local = schema.reserve_i64_local(function);
        let ieee_mantissa_local = schema.reserve_i64_local(function);
        let ieee_exponent_local = schema.reserve_i64_local(function);
        let binary_mantissa_local = schema.reserve_i64_local(function);
        let binary_exponent_local = schema.reserve_i64_local(function);

        function.instruction(&Instruction::I64Const(EXACT_DECIMAL_DIGIT_CAPACITY));
        capacity_local.store(function);
        scratch_local.resize(capacity_local, schema, function);
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.store(function);
        payload_local.load(function);
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        absolute_bits_local.store(function);
        payload_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        sign_local.store(function);
        absolute_bits_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        sign_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        binary_mantissa_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        binary_exponent_local.store(function);
        function.instruction(&Instruction::Else);
        absolute_bits_local.load(function);
        function.instruction(&Instruction::I64Const((1_i64 << 52) - 1));
        function.instruction(&Instruction::I64And);
        ieee_mantissa_local.store(function);
        absolute_bits_local.load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64And);
        ieee_exponent_local.store(function);
        ieee_exponent_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        ieee_mantissa_local.load(function);
        binary_mantissa_local.store(function);
        function.instruction(&Instruction::I64Const(-1074));
        binary_exponent_local.store(function);
        function.instruction(&Instruction::Else);
        ieee_mantissa_local.load(function);
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64Or);
        binary_mantissa_local.store(function);
        ieee_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(1023 + 52));
        function.instruction(&Instruction::I64Sub);
        binary_exponent_local.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_initialize_exact_decimal_digits(
            binary_mantissa_local,
            &scratch_local,
            digit_count_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        exact_exponent_local.store(function);
        binary_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_repeat_exact_decimal_multiply(
            &scratch_local,
            digit_count_local,
            binary_exponent_local,
            2,
            function,
        );
        function.instruction(&Instruction::Else);
        binary_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        let multiplier_count_local = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        binary_exponent_local.load(function);
        function.instruction(&Instruction::I64Sub);
        multiplier_count_local.store(function);
        self.emit_repeat_exact_decimal_multiply(
            &scratch_local,
            digit_count_local,
            multiplier_count_local,
            5,
            function,
        );
        binary_exponent_local.load(function);
        exact_exponent_local.store(function);
        schema.release_i64_local(multiplier_count_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        digit_count_local.load(function);
        exact_exponent_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        scientific_exponent_local.store(function);

        schema.release_i64_local(binary_exponent_local, function);
        schema.release_i64_local(binary_mantissa_local, function);
        schema.release_i64_local(ieee_exponent_local, function);
        schema.release_i64_local(ieee_mantissa_local, function);
        schema.release_i64_local(absolute_bits_local, function);
        schema.release_i64_local(capacity_local, function);
        Ok(())
    }

    fn emit_initialize_exact_decimal_digits(
        &mut self,
        binary_mantissa_local: I64Local,
        scratch_local: &FormattingBuffer,
        digit_count_local: I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let remaining_local = schema.reserve_i64_local(function);
        let digit_local = schema.reserve_i64_local(function);
        let address_local = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        scratch_local.write_from_stack(schema, function);
        function.instruction(&Instruction::I64Const(1));
        digit_count_local.store(function);
        binary_mantissa_local.load(function);
        remaining_local.store(function);
        remaining_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        digit_count_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        remaining_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        remaining_local.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        digit_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Add);
        address_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        digit_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.write_from_stack(schema, function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        digit_count_local.store(function);
        remaining_local.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        remaining_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i64_local(address_local, function);
        schema.release_i64_local(digit_local, function);
        schema.release_i64_local(remaining_local, function);
    }

    fn emit_repeat_exact_decimal_multiply(
        &mut self,
        scratch_local: &FormattingBuffer,
        digit_count_local: I64Local,
        repetitions_local: I64Local,
        factor: i64,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let repetition_local = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        repetition_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        repetition_local.load(function);
        repetitions_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_exact_decimal_multiply(scratch_local, digit_count_local, factor, function);
        repetition_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        repetition_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i64_local(repetition_local, function);
    }

    fn emit_exact_decimal_multiply(
        &mut self,
        scratch_local: &FormattingBuffer,
        digit_count_local: I64Local,
        factor: i64,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let index_local = schema.reserve_i64_local(function);
        let address_local = schema.reserve_i64_local(function);
        let product_local = schema.reserve_i64_local(function);
        let carry_local = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        index_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        carry_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        address_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.read_i32_from_stack(schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(factor));
        function.instruction(&Instruction::I64Mul);
        carry_local.load(function);
        function.instruction(&Instruction::I64Add);
        product_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        product_local.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.write_from_stack(schema, function);
        product_local.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        carry_local.store(function);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        carry_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Add);
        address_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        carry_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.write_from_stack(schema, function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        digit_count_local.store(function);
        function.instruction(&Instruction::End);
        schema.release_i64_local(carry_local, function);
        schema.release_i64_local(product_local, function);
        schema.release_i64_local(address_local, function);
        schema.release_i64_local(index_local, function);
    }

    fn emit_round_exact_decimal(
        &mut self,
        scratch_local: &FormattingBuffer,
        digit_start_local: I64Local,
        digit_count_local: I64Local,
        decimal_shift_local: I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let shift_magnitude_local = schema.reserve_i64_local(function);
        let index_local = schema.reserve_i64_local(function);
        let address_local = schema.reserve_i64_local(function);
        let rounding_digit_local = schema.reserve_i64_local(function);
        let digit_local = schema.reserve_i64_local(function);

        decimal_shift_local.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        digit_count_local.load(function);
        index_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        index_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.load(function);
        function.instruction(&Instruction::I64Add);
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        address_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.read_i32_from_stack(schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        digit_local.store(function);
        address_local.load(function);
        decimal_shift_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        digit_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.write_from_stack(schema, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        index_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        decimal_shift_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.load(function);
        function.instruction(&Instruction::I64Add);
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        address_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        scratch_local.write_from_stack(schema, function);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        digit_count_local.load(function);
        decimal_shift_local.load(function);
        function.instruction(&Instruction::I64Add);
        digit_count_local.store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        decimal_shift_local.load(function);
        function.instruction(&Instruction::I64Sub);
        shift_magnitude_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        rounding_digit_local.store(function);
        shift_magnitude_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.load(function);
        function.instruction(&Instruction::I64Add);
        shift_magnitude_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        address_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.read_i32_from_stack(schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        rounding_digit_local.store(function);
        function.instruction(&Instruction::End);
        shift_magnitude_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        digit_start_local.load(function);
        shift_magnitude_local.load(function);
        function.instruction(&Instruction::I64Add);
        digit_start_local.store(function);
        digit_count_local.load(function);
        shift_magnitude_local.load(function);
        function.instruction(&Instruction::I64Sub);
        digit_count_local.store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.store(function);
        function.instruction(&Instruction::I64Const(1));
        digit_count_local.store(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        scratch_local.write_from_stack(schema, function);
        function.instruction(&Instruction::End);

        rounding_digit_local.load(function);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        index_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.load(function);
        function.instruction(&Instruction::I64Add);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Add);
        address_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(1));
        scratch_local.write_from_stack(schema, function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        digit_count_local.store(function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.load(function);
        function.instruction(&Instruction::I64Add);
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        address_local.store(function);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.read_i32_from_stack(schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        digit_local.store(function);
        digit_local.load(function);
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        digit_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.write_from_stack(schema, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::Else);
        address_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        scratch_local.write_from_stack(schema, function);
        function.instruction(&Instruction::End);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        schema.release_i64_local(digit_local, function);
        schema.release_i64_local(rounding_digit_local, function);
        schema.release_i64_local(address_local, function);
        schema.release_i64_local(index_local, function);
        schema.release_i64_local(shift_magnitude_local, function);
    }
}

enum ExactSignificantDecimalPlacement {
    Scientific,
    Precision,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_exact_significant_decimal_payload(
        &mut self,
        payload_local: I64Local,
        significant_digits_local: I64Local,
        placement: ExactSignificantDecimalPlacement,
        output_local: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let scratch_local = FormattingBuffer::empty(schema, function);
        let digit_start_local = schema.reserve_i64_local(function);
        let digit_count_local = schema.reserve_i64_local(function);
        let exact_exponent_local = schema.reserve_i64_local(function);
        let scientific_exponent_local = schema.reserve_i64_local(function);
        let sign_local = schema.reserve_i64_local(function);
        let decimal_shift_local = schema.reserve_i64_local(function);
        self.emit_exact_binary64_decimal(
            payload_local,
            &scratch_local,
            digit_start_local,
            digit_count_local,
            exact_exponent_local,
            scientific_exponent_local,
            sign_local,
            function,
        )?;
        exact_exponent_local.load(function);
        significant_digits_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        scientific_exponent_local.load(function);
        function.instruction(&Instruction::I64Sub);
        decimal_shift_local.store(function);
        self.emit_round_exact_decimal(
            &scratch_local,
            digit_start_local,
            digit_count_local,
            decimal_shift_local,
            function,
        );
        digit_count_local.load(function);
        significant_digits_local.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        digit_start_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        digit_start_local.store(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        digit_count_local.store(function);
        scientific_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        scientific_exponent_local.store(function);
        function.instruction(&Instruction::End);

        match placement {
            ExactSignificantDecimalPlacement::Scientific => {
                self.emit_scientific_exact_decimal_payload(
                    sign_local,
                    &scratch_local,
                    digit_start_local,
                    digit_count_local,
                    scientific_exponent_local,
                    &output_local,
                    function,
                )?;
            }
            ExactSignificantDecimalPlacement::Precision => {
                scientific_exponent_local.load(function);
                function.instruction(&Instruction::I64Const(-6));
                function.instruction(&Instruction::I64LtS);
                scientific_exponent_local.load(function);
                significant_digits_local.load(function);
                function.instruction(&Instruction::I64GeS);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_scientific_exact_decimal_payload(
                    sign_local,
                    &scratch_local,
                    digit_start_local,
                    digit_count_local,
                    scientific_exponent_local,
                    &output_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                let fraction_digits_local = schema.reserve_i64_local(function);
                significant_digits_local.load(function);
                scientific_exponent_local.load(function);
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                fraction_digits_local.store(function);
                self.emit_fixed_exact_decimal_payload(
                    sign_local,
                    &scratch_local,
                    digit_start_local,
                    digit_count_local,
                    fraction_digits_local,
                    &output_local,
                    function,
                )?;
                schema.release_i64_local(fraction_digits_local, function);
                function.instruction(&Instruction::End);
            }
        }

        schema.release_i64_local(decimal_shift_local, function);
        schema.release_i64_local(sign_local, function);
        schema.release_i64_local(scientific_exponent_local, function);
        schema.release_i64_local(exact_exponent_local, function);
        schema.release_i64_local(digit_count_local, function);
        schema.release_i64_local(digit_start_local, function);
        scratch_local.clear(function);
        Ok(())
    }

    fn emit_shortest_exponential_payload(
        &mut self,
        payload_local: I64Local,
        output_local: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let absolute = schema.reserve_i64_local(function);
        let sign = schema.reserve_i64_local(function);
        let mantissa = schema.reserve_i64_local(function);
        let exponent = schema.reserve_i64_local(function);
        let capacity = schema.reserve_i64_local(function);
        let start = schema.reserve_i64_local(function);
        let digits = schema.reserve_i64_local(function);
        let scratch = FormattingBuffer::empty(schema, function);
        function.instruction(&Instruction::I64Const(EXACT_DECIMAL_DIGIT_CAPACITY));
        capacity.store(function);
        scratch.resize(capacity, schema, function);
        payload_local.load(function);
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        absolute.store(function);
        payload_local.load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        sign.store(function);
        absolute.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        sign.store(function);
        function.instruction(&Instruction::I64Const(0));
        mantissa.store(function);
        function.instruction(&Instruction::I64Const(0));
        exponent.store(function);
        function.instruction(&Instruction::Else);
        self.emit_ryu_shortest_decimal(absolute, mantissa, exponent, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_initialize_exact_decimal_digits(mantissa, &scratch, digits, function);
        function.instruction(&Instruction::I64Const(0));
        start.store(function);
        exponent.load(function);
        digits.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        exponent.store(function);
        self.emit_scientific_exact_decimal_payload(
            sign,
            &scratch,
            start,
            digits,
            exponent,
            output_local,
            function,
        )?;
        scratch.clear(function);
        for local in [digits, start, capacity, exponent, mantissa, sign, absolute] {
            schema.release_i64_local(local, function);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_fixed_exact_decimal_payload(
        &mut self,
        sign_local: I64Local,
        scratch_local: &FormattingBuffer,
        digit_start_local: I64Local,
        digit_count_local: I64Local,
        fraction_digits_local: I64Local,
        output_local: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let total_length_local = schema.reserve_i64_local(function);
        let output_offset_local = FormattingBuffer::empty(schema, function);
        let number_start_local = schema.reserve_i64_local(function);
        fraction_digits_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Add);
        total_length_local.store(function);
        output_offset_local.resize(total_length_local, schema, function);
        self.emit_decimal_sign(
            sign_local,
            &output_offset_local,
            number_start_local,
            function,
        );
        self.emit_write_exact_decimal_digits(
            &scratch_local,
            digit_start_local,
            digit_count_local,
            number_start_local,
            &output_offset_local,
            function,
        );
        function.instruction(&Instruction::Else);
        digit_count_local.load(function);
        fraction_digits_local.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        total_length_local.store(function);
        output_offset_local.resize(total_length_local, schema, function);
        self.emit_decimal_sign(
            sign_local,
            &output_offset_local,
            number_start_local,
            function,
        );
        let decimal_point_local = schema.reserve_i64_local(function);
        digit_count_local.load(function);
        fraction_digits_local.load(function);
        function.instruction(&Instruction::I64Sub);
        decimal_point_local.store(function);
        self.emit_write_exact_decimal_digits_with_point(
            &scratch_local,
            digit_start_local,
            digit_count_local,
            decimal_point_local,
            number_start_local,
            &output_offset_local,
            function,
        );
        schema.release_i64_local(decimal_point_local, function);
        function.instruction(&Instruction::Else);
        sign_local.load(function);
        fraction_digits_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        total_length_local.store(function);
        output_offset_local.resize(total_length_local, schema, function);
        self.emit_decimal_sign(
            sign_local,
            &output_offset_local,
            number_start_local,
            function,
        );
        self.store_ascii_byte_i64(&output_offset_local, number_start_local, b'0', function);
        let fraction_start_local = schema.reserve_i64_local(function);
        let zero_count_local = schema.reserve_i64_local(function);
        let digit_output_local = schema.reserve_i64_local(function);
        number_start_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        fraction_start_local.store(function);
        self.store_ascii_byte_i64(&output_offset_local, fraction_start_local, b'.', function);
        fraction_digits_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Sub);
        zero_count_local.store(function);
        fraction_start_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        fraction_start_local.store(function);
        self.emit_repeated_ascii(
            &output_offset_local,
            fraction_start_local,
            zero_count_local,
            b'0',
            function,
        );
        fraction_start_local.load(function);
        zero_count_local.load(function);
        function.instruction(&Instruction::I64Add);
        digit_output_local.store(function);
        self.emit_write_exact_decimal_digits(
            &scratch_local,
            digit_start_local,
            digit_count_local,
            digit_output_local,
            &output_offset_local,
            function,
        );
        schema.release_i64_local(digit_output_local, function);
        schema.release_i64_local(zero_count_local, function);
        schema.release_i64_local(fraction_start_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        output_local.replace(
            output_offset_local.publish_string(schema, function),
            function,
        );
        schema.release_i64_local(number_start_local, function);
        output_offset_local.clear(function);
        schema.release_i64_local(total_length_local, function);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_scientific_exact_decimal_payload(
        &mut self,
        sign_local: I64Local,
        scratch_local: &FormattingBuffer,
        digit_start_local: I64Local,
        digit_count_local: I64Local,
        scientific_exponent_local: I64Local,
        output_local: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let exponent_magnitude_local = schema.reserve_i64_local(function);
        let exponent_digits_local = schema.reserve_i64_local(function);
        let significand_length_local = schema.reserve_i64_local(function);
        let total_length_local = schema.reserve_i64_local(function);
        let output_offset_local = FormattingBuffer::empty(schema, function);
        let number_start_local = schema.reserve_i64_local(function);
        let exponent_start_local = schema.reserve_i64_local(function);
        scientific_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        scientific_exponent_local.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        scientific_exponent_local.load(function);
        function.instruction(&Instruction::End);
        exponent_magnitude_local.store(function);
        self.emit_count_decimal_digits_u64(
            exponent_magnitude_local,
            exponent_digits_local,
            function,
        );
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        significand_length_local.store(function);
        sign_local.load(function);
        significand_length_local.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        exponent_digits_local.load(function);
        function.instruction(&Instruction::I64Add);
        total_length_local.store(function);
        output_offset_local.resize(total_length_local, schema, function);
        self.emit_decimal_sign(
            sign_local,
            &output_offset_local,
            number_start_local,
            function,
        );
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_write_exact_decimal_digits(
            &scratch_local,
            digit_start_local,
            digit_count_local,
            number_start_local,
            &output_offset_local,
            function,
        );
        function.instruction(&Instruction::Else);
        let decimal_point_local = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(1));
        decimal_point_local.store(function);
        self.emit_write_exact_decimal_digits_with_point(
            &scratch_local,
            digit_start_local,
            digit_count_local,
            decimal_point_local,
            number_start_local,
            &output_offset_local,
            function,
        );
        schema.release_i64_local(decimal_point_local, function);
        function.instruction(&Instruction::End);
        number_start_local.load(function);
        significand_length_local.load(function);
        function.instruction(&Instruction::I64Add);
        exponent_start_local.store(function);
        self.store_ascii_byte_i64(&output_offset_local, exponent_start_local, b'e', function);
        exponent_start_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        exponent_start_local.store(function);
        scientific_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.store_ascii_byte_i64(&output_offset_local, exponent_start_local, b'-', function);
        function.instruction(&Instruction::Else);
        self.store_ascii_byte_i64(&output_offset_local, exponent_start_local, b'+', function);
        function.instruction(&Instruction::End);
        exponent_start_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        exponent_start_local.store(function);
        self.emit_write_decimal_u64(
            exponent_magnitude_local,
            exponent_start_local,
            exponent_digits_local,
            &output_offset_local,
            function,
        );
        output_local.replace(
            output_offset_local.publish_string(schema, function),
            function,
        );
        schema.release_i64_local(exponent_start_local, function);
        schema.release_i64_local(number_start_local, function);
        output_offset_local.clear(function);
        schema.release_i64_local(total_length_local, function);
        schema.release_i64_local(significand_length_local, function);
        schema.release_i64_local(exponent_digits_local, function);
        schema.release_i64_local(exponent_magnitude_local, function);
        Ok(())
    }

    fn emit_write_exact_decimal_digits(
        &mut self,
        scratch_local: &FormattingBuffer,
        digit_start_local: I64Local,
        digit_count_local: I64Local,
        output_start_local: I64Local,
        output_buffer: &FormattingBuffer,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let index_local = schema.reserve_i64_local(function);
        let source_local = schema.reserve_i64_local(function);
        let destination_local = schema.reserve_i64_local(function);
        let digit_local = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        index_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.load(function);
        function.instruction(&Instruction::I64Add);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Add);
        index_local.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        source_local.store(function);
        source_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.read_i32_from_stack(schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        digit_local.store(function);
        output_start_local.load(function);
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        destination_local.store(function);
        destination_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        digit_local.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        output_buffer.write_from_stack(schema, function);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i64_local(digit_local, function);
        schema.release_i64_local(destination_local, function);
        schema.release_i64_local(source_local, function);
        schema.release_i64_local(index_local, function);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_write_exact_decimal_digits_with_point(
        &mut self,
        scratch_local: &FormattingBuffer,
        digit_start_local: I64Local,
        digit_count_local: I64Local,
        decimal_point_local: I64Local,
        output_start_local: I64Local,
        output_buffer: &FormattingBuffer,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let index_local = schema.reserve_i64_local(function);
        let source_local = schema.reserve_i64_local(function);
        let destination_local = schema.reserve_i64_local(function);
        let digit_local = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        index_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index_local.load(function);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(0));
        digit_start_local.load(function);
        function.instruction(&Instruction::I64Add);
        digit_count_local.load(function);
        function.instruction(&Instruction::I64Add);
        index_local.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        source_local.store(function);
        source_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        scratch_local.read_i32_from_stack(schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        digit_local.store(function);
        output_start_local.load(function);
        index_local.load(function);
        function.instruction(&Instruction::I64Add);
        index_local.load(function);
        decimal_point_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        destination_local.store(function);
        destination_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        digit_local.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        output_buffer.write_from_stack(schema, function);
        index_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        index_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        output_start_local.load(function);
        decimal_point_local.load(function);
        function.instruction(&Instruction::I64Add);
        destination_local.store(function);
        self.store_ascii_byte_i64(output_buffer, destination_local, b'.', function);
        schema.release_i64_local(digit_local, function);
        schema.release_i64_local(destination_local, function);
        schema.release_i64_local(source_local, function);
        schema.release_i64_local(index_local, function);
    }
}
