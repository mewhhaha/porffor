use super::super::*;

const DECIMAL_MAX_DIGITS: i64 = 768;
const DECIMAL_PRODUCT_CAPACITY: i64 = 800;
const DECIMAL_SCRATCH_SIZE: u64 = DECIMAL_MAX_DIGITS as u64 + DECIMAL_PRODUCT_CAPACITY as u64;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_decimal_to_binary64_payload(
        &self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper = self
            .decimal_to_binary64_helper_function_index()
            .ok_or_else(|| EmitError::unsupported("decimal converter helper is unavailable"))?;
        function.instruction(&Instruction::LocalGet(string_payload_local));
        for _ in 1..7 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Call(helper));
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::Drop);
        Ok(())
    }

    pub(crate) fn compile_decimal_to_binary64_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        let input_ptr = self.reserve_temp_local();
        let input_len = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let byte = self.reserve_temp_local();
        let negative = self.reserve_temp_local();
        let point_seen = self.reserve_temp_local();
        let fraction_digits = self.reserve_temp_local();
        let significant_started = self.reserve_temp_local();
        let significant_digits = self.reserve_temp_local();
        let num_digits = self.reserve_temp_local();
        let decimal_point = self.reserve_temp_local();
        let truncated = self.reserve_temp_local();
        let exponent_negative = self.reserve_temp_local();
        let exponent = self.reserve_temp_local();
        let saved_heap_ptr = self.reserve_temp_local();
        let digits_ptr = self.reserve_temp_local();
        let product_ptr = self.reserve_temp_local();
        let exp2 = self.reserve_temp_local();
        let shift = self.reserve_temp_local();
        let result_bits = self.reserve_temp_local();

        self.emit_unpack_string_payload(0, input_ptr, input_len, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(negative));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(point_seen));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(fraction_digits));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(significant_started));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(significant_digits));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(num_digits));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(truncated));
        function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(saved_heap_ptr));
        self.emit_heap_alloc_const(DECIMAL_SCRATCH_SIZE, &mut function)?;
        function.instruction(&Instruction::LocalSet(digits_ptr));
        function.instruction(&Instruction::LocalGet(digits_ptr));
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(product_ptr));

        // The three callers pass a scanner-validated decimal span. This parser
        // only separates its sign, digits, point, and exponent for conversion.
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_decimal_load_input_byte(input_ptr, index, byte, &mut function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(negative));
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_decimal_load_input_byte(input_ptr, index, byte, &mut function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'E' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(point_seen));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(point_seen));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_increment_local(fraction_digits, 1, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(significant_started));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(significant_started));
        function.instruction(&Instruction::LocalGet(significant_digits));
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_store_digit_from_byte(
            digits_ptr,
            significant_digits,
            byte,
            &mut function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(truncated));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_local(significant_digits, 1, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(significant_digits));
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(significant_digits));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(num_digits));
        self.emit_decimal_trim(digits_ptr, num_digits, &mut function);
        function.instruction(&Instruction::LocalGet(significant_digits));
        function.instruction(&Instruction::LocalGet(fraction_digits));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(decimal_point));

        // Parse the optional exponent, saturating before host integer overflow.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exponent));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exponent_negative));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_load_input_byte(input_ptr, index, byte, &mut function);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(exponent_negative));
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(input_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_decimal_load_input_byte(input_ptr, index, byte, &mut function);
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(exponent));
        function.instruction(&Instruction::End);
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(exponent_negative));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(decimal_point));
        function.instruction(&Instruction::End);

        self.emit_decimal_convert(
            digits_ptr,
            product_ptr,
            num_digits,
            decimal_point,
            truncated,
            exp2,
            shift,
            result_bits,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(result_bits));
        function.instruction(&Instruction::LocalGet(negative));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(saved_heap_ptr));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(result_bits));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_decimal_convert(
        &mut self,
        digits_ptr: u32,
        product_ptr: u32,
        num_digits: u32,
        decimal_point: u32,
        truncated: u32,
        exp2: u32,
        shift: u32,
        result_bits: u32,
        function: &mut Function,
    ) {
        let mantissa = self.reserve_temp_local();
        let power2 = self.reserve_temp_local();
        let first_digit = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Const(-324));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Const(310));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_decimal_fast_path(digits_ptr, num_digits, decimal_point, result_bits, function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exp2));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::BrIf(1));
        self.emit_decimal_select_shift(decimal_point, shift, function);
        self.emit_decimal_right_shift(
            digits_ptr,
            num_digits,
            decimal_point,
            truncated,
            shift,
            function,
        );
        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(exp2));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_digit));
        self.emit_decimal_load_digit(digits_ptr, first_digit, first_digit, function);
        function.instruction(&Instruction::LocalGet(first_digit));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(2));
        function.instruction(&Instruction::LocalGet(first_digit));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(shift));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(first_digit));
        self.emit_decimal_select_shift(first_digit, shift, function);
        function.instruction(&Instruction::End);
        self.emit_decimal_left_shift(
            digits_ptr,
            product_ptr,
            num_digits,
            decimal_point,
            truncated,
            shift,
            function,
        );
        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(exp2));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(exp2));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::I64Const(-1022));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(-1022));
        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1022));
        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(shift));
        self.emit_decimal_right_shift(
            digits_ptr,
            num_digits,
            decimal_point,
            truncated,
            shift,
            function,
        );
        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(exp2));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::I64Const(1024));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(53));
        function.instruction(&Instruction::LocalSet(shift));
        self.emit_decimal_left_shift(
            digits_ptr,
            product_ptr,
            num_digits,
            decimal_point,
            truncated,
            shift,
            function,
        );
        self.emit_decimal_round(
            digits_ptr,
            num_digits,
            decimal_point,
            truncated,
            mantissa,
            function,
        );
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(1_i64 << 53));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(shift));
        self.emit_decimal_right_shift(
            digits_ptr,
            num_digits,
            decimal_point,
            truncated,
            shift,
            function,
        );
        self.emit_increment_local(exp2, 1, function);
        self.emit_decimal_round(
            digits_ptr,
            num_digits,
            decimal_point,
            truncated,
            mantissa,
            function,
        );
        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::I64Const(1024));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(exp2));
        function.instruction(&Instruction::I64Const(1023));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(power2));
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(power2, -1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(power2));
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(power2));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const((1_i64 << 52) - 1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(first_digit);
        self.release_temp_local(power2);
        self.release_temp_local(mantissa);
    }

    fn emit_decimal_fast_path(
        &mut self,
        digits_ptr: u32,
        num_digits: u32,
        decimal_point: u32,
        result_bits: u32,
        function: &mut Function,
    ) {
        let exponent = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        let mantissa = self.reserve_temp_local();
        let power = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(exponent));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64Const(15));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(-22));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(22));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mantissa));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_decimal_load_digit(digits_ptr, index, digit, function);
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mantissa));
        self.emit_increment_local(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(power));
        for magnitude in 1..=22_i64 {
            function.instruction(&Instruction::LocalGet(exponent));
            function.instruction(&Instruction::I64Const(magnitude));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(exponent));
            function.instruction(&Instruction::I64Const(-magnitude));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::F64Const(Ieee64::from(
                10_f64.powi(magnitude as i32),
            )));
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(power));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::LocalGet(power));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::LocalGet(power));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_bits));
        function.instruction(&Instruction::End);
        // This method is emitted inside the converter's result block. The If
        // adds one label, so depth one exits conversion after the proven-safe
        // fast path has produced the final magnitude.
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.release_temp_local(power);
        self.release_temp_local(mantissa);
        self.release_temp_local(digit);
        self.release_temp_local(index);
        self.release_temp_local(exponent);
    }

    fn emit_decimal_select_shift(&self, distance: u32, shift: u32, function: &mut Function) {
        const POWERS: [i64; 19] = [
            0, 3, 6, 9, 13, 16, 19, 23, 26, 29, 33, 36, 39, 43, 46, 49, 53, 56, 59,
        ];
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::LocalSet(shift));
        for (index, selected_shift) in POWERS.into_iter().enumerate() {
            function.instruction(&Instruction::LocalGet(distance));
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(selected_shift));
            function.instruction(&Instruction::LocalSet(shift));
            function.instruction(&Instruction::End);
        }
    }

    fn emit_decimal_left_shift(
        &mut self,
        digits_ptr: u32,
        product_ptr: u32,
        num_digits: u32,
        decimal_point: u32,
        truncated: u32,
        shift: u32,
        function: &mut Function,
    ) {
        let read = self.reserve_temp_local();
        let write = self.reserve_temp_local();
        let carry = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        let quotient = self.reserve_temp_local();
        let old_num_digits = self.reserve_temp_local();
        let product_len = self.reserve_temp_local();
        let copy_len = self.reserve_temp_local();
        let source = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::LocalSet(old_num_digits));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::LocalSet(read));
        function.instruction(&Instruction::I64Const(DECIMAL_PRODUCT_CAPACITY));
        function.instruction(&Instruction::LocalSet(write));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(read));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_increment_local(read, -1, function);
        self.emit_increment_local(write, -1, function);
        self.emit_decimal_load_digit(digits_ptr, read, digit, function);
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(quotient));
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::LocalGet(quotient));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_decimal_store_digit(product_ptr, write, digit, function);
        function.instruction(&Instruction::LocalGet(quotient));
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_increment_local(write, -1, function);
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(quotient));
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::LocalGet(quotient));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_decimal_store_digit(product_ptr, write, digit, function);
        function.instruction(&Instruction::LocalGet(quotient));
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(DECIMAL_PRODUCT_CAPACITY));
        function.instruction(&Instruction::LocalGet(write));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(product_len));
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::LocalGet(product_len));
        function.instruction(&Instruction::LocalGet(old_num_digits));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(decimal_point));
        function.instruction(&Instruction::LocalGet(product_len));
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(product_len));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(copy_len));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(read));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(read));
        function.instruction(&Instruction::LocalGet(product_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write));
        function.instruction(&Instruction::LocalGet(read));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source));
        self.emit_decimal_load_digit(product_ptr, source, digit, function);
        function.instruction(&Instruction::LocalGet(read));
        function.instruction(&Instruction::LocalGet(copy_len));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_store_digit(digits_ptr, read, digit, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(truncated));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_local(read, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(copy_len));
        function.instruction(&Instruction::LocalSet(num_digits));
        self.emit_decimal_trim(digits_ptr, num_digits, function);
        self.release_temp_local(source);
        self.release_temp_local(copy_len);
        self.release_temp_local(product_len);
        self.release_temp_local(old_num_digits);
        self.release_temp_local(quotient);
        self.release_temp_local(digit);
        self.release_temp_local(carry);
        self.release_temp_local(write);
        self.release_temp_local(read);
    }

    fn emit_decimal_right_shift(
        &mut self,
        digits_ptr: u32,
        num_digits: u32,
        decimal_point: u32,
        truncated: u32,
        shift: u32,
        function: &mut Function,
    ) {
        let read = self.reserve_temp_local();
        let write = self.reserve_temp_local();
        let accumulator = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        let mask = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(read));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(accumulator));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(read));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_load_digit(digits_ptr, read, digit, function);
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(accumulator));
        self.emit_increment_local(read, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(2));
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(accumulator));
        self.emit_increment_local(read, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::LocalGet(read));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(decimal_point));
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Const(-2047));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(num_digits));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(decimal_point));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(truncated));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(mask));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(read));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_decimal_store_digit(digits_ptr, write, digit, function);
        self.emit_increment_local(write, 1, function);
        self.emit_decimal_load_digit(digits_ptr, read, digit, function);
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::LocalGet(mask));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(accumulator));
        self.emit_increment_local(read, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(digit));
        function.instruction(&Instruction::LocalGet(write));
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_store_digit(digits_ptr, write, digit, function);
        self.emit_increment_local(write, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(truncated));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(accumulator));
        function.instruction(&Instruction::LocalGet(mask));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(accumulator));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(write));
        function.instruction(&Instruction::LocalSet(num_digits));
        self.emit_decimal_trim(digits_ptr, num_digits, function);
        function.instruction(&Instruction::End);
        self.release_temp_local(mask);
        self.release_temp_local(digit);
        self.release_temp_local(accumulator);
        self.release_temp_local(write);
        self.release_temp_local(read);
    }

    fn emit_decimal_round(
        &mut self,
        digits_ptr: u32,
        num_digits: u32,
        decimal_point: u32,
        truncated: u32,
        result: u32,
        function: &mut Function,
    ) {
        let index = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Const(18));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(result));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(result));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(result));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_load_digit(digits_ptr, index, digit, function);
        function.instruction(&Instruction::LocalGet(result));
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(result));
        function.instruction(&Instruction::End);
        self.emit_increment_local(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_load_digit(digits_ptr, decimal_point, digit, function);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(decimal_point));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(truncated));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(result));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(result, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(digit);
        self.release_temp_local(index);
    }

    fn emit_decimal_trim(&mut self, digits_ptr: u32, num_digits: u32, function: &mut Function) {
        let last = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(num_digits));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(last));
        self.emit_decimal_load_digit(digits_ptr, last, digit, function);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(last));
        function.instruction(&Instruction::LocalSet(num_digits));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(digit);
        self.release_temp_local(last);
    }

    fn emit_decimal_load_input_byte(
        &self,
        input_ptr: u32,
        index: u32,
        result: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(input_ptr));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result));
    }

    fn emit_decimal_load_digit(
        &self,
        digits_ptr: u32,
        index: u32,
        result: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(digits_ptr));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(result));
    }

    fn emit_decimal_store_digit(
        &self,
        digits_ptr: u32,
        index: u32,
        digit: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(digits_ptr));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    fn emit_decimal_store_digit_from_byte(
        &self,
        digits_ptr: u32,
        index: u32,
        byte: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(digits_ptr));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }
}
