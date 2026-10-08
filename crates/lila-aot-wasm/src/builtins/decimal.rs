use super::super::*;
use crate::gc_types::*;
use crate::runtime_helpers::{
    DecimalToBinary64Parameters, HelperParameters, TransientByteAllocArguments,
};

const DECIMAL_MAX_DIGITS: i64 = 768;
const DECIMAL_PRODUCT_CAPACITY: i64 = 800;
const DECIMAL_SCRATCH_SIZE: u64 = DECIMAL_MAX_DIGITS as u64 + DECIMAL_PRODUCT_CAPACITY as u64;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_decimal_to_binary64_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::DecimalToBinary64);
        let schema = self.runtime_schema();
        let parameters = self.helper_parameters::<DecimalToBinary64Parameters>(&mut function);
        let input_units = schema
            .reserve_gc_local::<CodeUnitArray, NonNullable>(&mut function)
            .initialize(
                schema
                    .struct_type::<StringValue>()
                    .field(StringValueSchema::CODE_UNITS)
                    .read(&parameters.input, schema, &mut function)
                    .reference(),
                &mut function,
            );
        let input_len = self.runtime_schema().reserve_i64_local(&mut function);
        let index = self.runtime_schema().reserve_i64_local(&mut function);
        let byte = self.runtime_schema().reserve_i64_local(&mut function);
        let negative = self.runtime_schema().reserve_i64_local(&mut function);
        let point_seen = self.runtime_schema().reserve_i64_local(&mut function);
        let fraction_digits = self.runtime_schema().reserve_i64_local(&mut function);
        let significant_started = self.runtime_schema().reserve_i64_local(&mut function);
        let significant_digits = self.runtime_schema().reserve_i64_local(&mut function);
        let num_digits = self.runtime_schema().reserve_i64_local(&mut function);
        let decimal_point = self.runtime_schema().reserve_i64_local(&mut function);
        let truncated = self.runtime_schema().reserve_i64_local(&mut function);
        let exponent_negative = self.runtime_schema().reserve_i64_local(&mut function);
        let exponent = self.runtime_schema().reserve_i64_local(&mut function);
        let saved_heap_ptr = self.runtime_schema().reserve_i64_local(&mut function);
        let digits_ptr = self.runtime_schema().reserve_i64_local(&mut function);
        let product_ptr = self.runtime_schema().reserve_i64_local(&mut function);
        let exp2 = self.runtime_schema().reserve_i64_local(&mut function);
        let shift = self.runtime_schema().reserve_i64_local(&mut function);
        let result_bits = self.runtime_schema().reserve_i64_local(&mut function);

        schema
            .array_type::<CodeUnitArray>()
            .length(&input_units, schema, &mut function);
        function.instruction(&Instruction::I64ExtendI32U);
        input_len.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        index.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        negative.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        point_seen.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        fraction_digits.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        significant_started.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        significant_digits.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        num_digits.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        truncated.store(&mut function);
        function.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        saved_heap_ptr.store(&mut function);
        let size = schema.reserve_i64_local(&mut function);
        function.instruction(&Instruction::I64Const(DECIMAL_SCRATCH_SIZE as i64));
        size.store(&mut function);
        let allocation = schema.reserve_i64_local(&mut function);
        schema
            .call_helper(
                TransientByteAllocArguments::new(size),
                self.runtime_helper_base()?,
                &mut function,
            )
            .store(allocation, &mut function);
        allocation.load(&mut function);
        digits_ptr.store(&mut function);
        schema.release_i64_local(allocation, &mut function);
        schema.release_i64_local(size, &mut function);
        digits_ptr.load(&mut function);
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64Add);
        product_ptr.store(&mut function);

        // The three callers pass a scanner-validated decimal span. This parser
        // only separates its sign, digits, point, and exponent for conversion.
        input_len.load(&mut function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_decimal_load_input_byte(&input_units, index, byte, &mut function);
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        negative.store(&mut function);
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::Else);
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(&mut function);
        input_len.load(&mut function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_decimal_load_input_byte(&input_units, index, byte, &mut function);
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'E' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        point_seen.store(&mut function);
        function.instruction(&Instruction::Else);
        point_seen.load(&mut function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_increment_local(fraction_digits, 1, &mut function);
        function.instruction(&Instruction::End);
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Ne);
        significant_started.load(&mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        significant_started.store(&mut function);
        significant_digits.load(&mut function);
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
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        truncated.store(&mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_local(significant_digits, 1, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        significant_digits.load(&mut function);
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        significant_digits.load(&mut function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::End);
        num_digits.store(&mut function);
        self.emit_decimal_trim(digits_ptr, num_digits, &mut function);
        significant_digits.load(&mut function);
        fraction_digits.load(&mut function);
        function.instruction(&Instruction::I64Sub);
        decimal_point.store(&mut function);

        // Parse the optional exponent, saturating before host integer overflow.
        function.instruction(&Instruction::I64Const(0));
        exponent.store(&mut function);
        function.instruction(&Instruction::I64Const(0));
        exponent_negative.store(&mut function);
        index.load(&mut function);
        input_len.load(&mut function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index, 1, &mut function);
        index.load(&mut function);
        input_len.load(&mut function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_load_input_byte(&input_units, index, byte, &mut function);
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        exponent_negative.store(&mut function);
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::Else);
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(&mut function);
        input_len.load(&mut function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_decimal_load_input_byte(&input_units, index, byte, &mut function);
        exponent.load(&mut function);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        exponent.load(&mut function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        byte.load(&mut function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        exponent.store(&mut function);
        function.instruction(&Instruction::End);
        self.emit_increment_local(index, 1, &mut function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        exponent_negative.load(&mut function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        exponent.load(&mut function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        exponent.load(&mut function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        decimal_point.load(&mut function);
        function.instruction(&Instruction::I64Add);
        decimal_point.store(&mut function);
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
        result_bits.load(&mut function);
        negative.load(&mut function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        result_bits.store(&mut function);
        function.instruction(&Instruction::End);

        saved_heap_ptr.load(&mut function);
        function.instruction(&Instruction::GlobalSet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        result_bits.load(&mut function);
        for local in [
            result_bits,
            shift,
            exp2,
            product_ptr,
            digits_ptr,
            saved_heap_ptr,
            exponent,
            exponent_negative,
            truncated,
            decimal_point,
            num_digits,
            significant_digits,
            significant_started,
            fraction_digits,
            point_seen,
            negative,
            byte,
            index,
            input_len,
        ] {
            schema.release_i64_local(local, &mut function);
        }
        input_units.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn emit_decimal_convert(
        &mut self,
        digits_ptr: I64Local,
        product_ptr: I64Local,
        num_digits: I64Local,
        decimal_point: I64Local,
        truncated: I64Local,
        exp2: I64Local,
        shift: I64Local,
        result_bits: I64Local,
        function: &mut Function,
    ) {
        let mantissa = self.runtime_schema().reserve_i64_local(function);
        let power2 = self.runtime_schema().reserve_i64_local(function);
        let first_digit = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        result_bits.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        num_digits.load(function);
        function.instruction(&Instruction::I64Eqz);
        decimal_point.load(function);
        function.instruction(&Instruction::I64Const(-324));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(0));
        decimal_point.load(function);
        function.instruction(&Instruction::I64Const(310));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        result_bits.store(function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_decimal_fast_path(digits_ptr, num_digits, decimal_point, result_bits, function);

        function.instruction(&Instruction::I64Const(0));
        exp2.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        decimal_point.load(function);
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
        exp2.load(function);
        shift.load(function);
        function.instruction(&Instruction::I64Add);
        exp2.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        decimal_point.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::BrIf(1));
        decimal_point.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        first_digit.store(function);
        self.emit_decimal_load_digit(digits_ptr, first_digit, first_digit, function);
        first_digit.load(function);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(2));
        first_digit.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        shift.store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        decimal_point.load(function);
        function.instruction(&Instruction::I64Sub);
        first_digit.store(function);
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
        exp2.load(function);
        shift.load(function);
        function.instruction(&Instruction::I64Sub);
        exp2.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        exp2.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        exp2.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        exp2.load(function);
        function.instruction(&Instruction::I64Const(-1022));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(-1022));
        exp2.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1022));
        exp2.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::End);
        shift.store(function);
        self.emit_decimal_right_shift(
            digits_ptr,
            num_digits,
            decimal_point,
            truncated,
            shift,
            function,
        );
        exp2.load(function);
        shift.load(function);
        function.instruction(&Instruction::I64Add);
        exp2.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        exp2.load(function);
        function.instruction(&Instruction::I64Const(1024));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        result_bits.store(function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(53));
        shift.store(function);
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
        mantissa.load(function);
        function.instruction(&Instruction::I64Const(1_i64 << 53));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        shift.store(function);
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
        exp2.load(function);
        function.instruction(&Instruction::I64Const(1024));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        result_bits.store(function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        exp2.load(function);
        function.instruction(&Instruction::I64Const(1023));
        function.instruction(&Instruction::I64Add);
        power2.store(function);
        mantissa.load(function);
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(power2, -1, function);
        function.instruction(&Instruction::End);
        power2.load(function);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::INFINITY.to_bits() as i64));
        result_bits.store(function);
        function.instruction(&Instruction::Else);
        power2.load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        mantissa.load(function);
        function.instruction(&Instruction::I64Const((1_i64 << 52) - 1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        result_bits.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(first_digit, function);
        self.runtime_schema().release_i64_local(power2, function);
        self.runtime_schema().release_i64_local(mantissa, function);
    }

    fn emit_decimal_fast_path(
        &mut self,
        digits_ptr: I64Local,
        num_digits: I64Local,
        decimal_point: I64Local,
        result_bits: I64Local,
        function: &mut Function,
    ) {
        let exponent = self.runtime_schema().reserve_i64_local(function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        let mantissa = self.runtime_schema().reserve_i64_local(function);
        let power = self.runtime_schema().reserve_i64_local(function);
        decimal_point.load(function);
        num_digits.load(function);
        function.instruction(&Instruction::I64Sub);
        exponent.store(function);
        num_digits.load(function);
        function.instruction(&Instruction::I64Const(15));
        function.instruction(&Instruction::I64LeU);
        exponent.load(function);
        function.instruction(&Instruction::I64Const(-22));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32And);
        exponent.load(function);
        function.instruction(&Instruction::I64Const(22));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        function.instruction(&Instruction::I64Const(0));
        mantissa.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        num_digits.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_decimal_load_digit(digits_ptr, index, digit, function);
        mantissa.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        digit.load(function);
        function.instruction(&Instruction::I64Add);
        mantissa.store(function);
        self.emit_increment_local(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        power.store(function);
        for magnitude in 1..=22_i64 {
            exponent.load(function);
            function.instruction(&Instruction::I64Const(magnitude));
            function.instruction(&Instruction::I64Eq);
            exponent.load(function);
            function.instruction(&Instruction::I64Const(-magnitude));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::F64Const(Ieee64::from(
                10_f64.powi(magnitude as i32),
            )));
            function.instruction(&Instruction::I64ReinterpretF64);
            power.store(function);
            function.instruction(&Instruction::End);
        }
        exponent.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        mantissa.load(function);
        function.instruction(&Instruction::F64ConvertI64U);
        power.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::I64ReinterpretF64);
        result_bits.store(function);
        function.instruction(&Instruction::Else);
        mantissa.load(function);
        function.instruction(&Instruction::F64ConvertI64U);
        power.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        result_bits.store(function);
        function.instruction(&Instruction::End);
        // This method is emitted inside the converter's result block. The If
        // adds one label, so depth one exits conversion after the proven-safe
        // fast path has produced the final magnitude.
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(power, function);
        self.runtime_schema().release_i64_local(mantissa, function);
        self.runtime_schema().release_i64_local(digit, function);
        self.runtime_schema().release_i64_local(index, function);
        self.runtime_schema().release_i64_local(exponent, function);
    }

    fn emit_decimal_select_shift(
        &self,
        distance: I64Local,
        shift: I64Local,
        function: &mut Function,
    ) {
        const POWERS: [i64; 19] = [
            0, 3, 6, 9, 13, 16, 19, 23, 26, 29, 33, 36, 39, 43, 46, 49, 53, 56, 59,
        ];
        function.instruction(&Instruction::I64Const(60));
        shift.store(function);
        for (index, selected_shift) in POWERS.into_iter().enumerate() {
            distance.load(function);
            function.instruction(&Instruction::I64Const(index as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(selected_shift));
            shift.store(function);
            function.instruction(&Instruction::End);
        }
    }

    fn emit_decimal_left_shift(
        &mut self,
        digits_ptr: I64Local,
        product_ptr: I64Local,
        num_digits: I64Local,
        decimal_point: I64Local,
        truncated: I64Local,
        shift: I64Local,
        function: &mut Function,
    ) {
        let read = self.runtime_schema().reserve_i64_local(function);
        let write = self.runtime_schema().reserve_i64_local(function);
        let carry = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        let quotient = self.runtime_schema().reserve_i64_local(function);
        let old_num_digits = self.runtime_schema().reserve_i64_local(function);
        let product_len = self.runtime_schema().reserve_i64_local(function);
        let copy_len = self.runtime_schema().reserve_i64_local(function);
        let source = self.runtime_schema().reserve_i64_local(function);
        num_digits.load(function);
        old_num_digits.store(function);
        num_digits.load(function);
        read.store(function);
        function.instruction(&Instruction::I64Const(DECIMAL_PRODUCT_CAPACITY));
        write.store(function);
        function.instruction(&Instruction::I64Const(0));
        carry.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        read.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_increment_local(read, -1, function);
        self.emit_increment_local(write, -1, function);
        self.emit_decimal_load_digit(digits_ptr, read, digit, function);
        carry.load(function);
        digit.load(function);
        shift.load(function);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Add);
        carry.store(function);
        carry.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        quotient.store(function);
        carry.load(function);
        quotient.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        digit.store(function);
        self.emit_decimal_store_digit(product_ptr, write, digit, function);
        quotient.load(function);
        carry.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        carry.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_increment_local(write, -1, function);
        carry.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64DivU);
        quotient.store(function);
        carry.load(function);
        quotient.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        digit.store(function);
        self.emit_decimal_store_digit(product_ptr, write, digit, function);
        quotient.load(function);
        carry.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(DECIMAL_PRODUCT_CAPACITY));
        write.load(function);
        function.instruction(&Instruction::I64Sub);
        product_len.store(function);
        decimal_point.load(function);
        product_len.load(function);
        old_num_digits.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        decimal_point.store(function);
        product_len.load(function);
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        product_len.load(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::End);
        copy_len.store(function);
        function.instruction(&Instruction::I64Const(0));
        read.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        read.load(function);
        product_len.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        write.load(function);
        read.load(function);
        function.instruction(&Instruction::I64Add);
        source.store(function);
        self.emit_decimal_load_digit(product_ptr, source, digit, function);
        read.load(function);
        copy_len.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_store_digit(digits_ptr, read, digit, function);
        function.instruction(&Instruction::Else);
        digit.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        truncated.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_increment_local(read, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        copy_len.load(function);
        num_digits.store(function);
        self.emit_decimal_trim(digits_ptr, num_digits, function);
        self.runtime_schema().release_i64_local(source, function);
        self.runtime_schema().release_i64_local(copy_len, function);
        self.runtime_schema()
            .release_i64_local(product_len, function);
        self.runtime_schema()
            .release_i64_local(old_num_digits, function);
        self.runtime_schema().release_i64_local(quotient, function);
        self.runtime_schema().release_i64_local(digit, function);
        self.runtime_schema().release_i64_local(carry, function);
        self.runtime_schema().release_i64_local(write, function);
        self.runtime_schema().release_i64_local(read, function);
    }

    fn emit_decimal_right_shift(
        &mut self,
        digits_ptr: I64Local,
        num_digits: I64Local,
        decimal_point: I64Local,
        truncated: I64Local,
        shift: I64Local,
        function: &mut Function,
    ) {
        let read = self.runtime_schema().reserve_i64_local(function);
        let write = self.runtime_schema().reserve_i64_local(function);
        let accumulator = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        let mask = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        read.store(function);
        function.instruction(&Instruction::I64Const(0));
        write.store(function);
        function.instruction(&Instruction::I64Const(0));
        accumulator.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        accumulator.load(function);
        shift.load(function);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        read.load(function);
        num_digits.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_load_digit(digits_ptr, read, digit, function);
        accumulator.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        digit.load(function);
        function.instruction(&Instruction::I64Add);
        accumulator.store(function);
        self.emit_increment_local(read, 1, function);
        function.instruction(&Instruction::Else);
        accumulator.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(2));
        accumulator.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        accumulator.store(function);
        self.emit_increment_local(read, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        decimal_point.load(function);
        read.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Sub);
        decimal_point.store(function);
        decimal_point.load(function);
        function.instruction(&Instruction::I64Const(-2047));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        num_digits.store(function);
        function.instruction(&Instruction::I64Const(0));
        decimal_point.store(function);
        function.instruction(&Instruction::I64Const(0));
        truncated.store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        shift.load(function);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        mask.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        read.load(function);
        num_digits.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        accumulator.load(function);
        shift.load(function);
        function.instruction(&Instruction::I64ShrU);
        digit.store(function);
        self.emit_decimal_store_digit(digits_ptr, write, digit, function);
        self.emit_increment_local(write, 1, function);
        self.emit_decimal_load_digit(digits_ptr, read, digit, function);
        accumulator.load(function);
        mask.load(function);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        digit.load(function);
        function.instruction(&Instruction::I64Add);
        accumulator.store(function);
        self.emit_increment_local(read, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        accumulator.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        accumulator.load(function);
        shift.load(function);
        function.instruction(&Instruction::I64ShrU);
        digit.store(function);
        write.load(function);
        function.instruction(&Instruction::I64Const(DECIMAL_MAX_DIGITS));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_store_digit(digits_ptr, write, digit, function);
        self.emit_increment_local(write, 1, function);
        function.instruction(&Instruction::Else);
        digit.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        truncated.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        accumulator.load(function);
        mask.load(function);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        accumulator.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        write.load(function);
        num_digits.store(function);
        self.emit_decimal_trim(digits_ptr, num_digits, function);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(mask, function);
        self.runtime_schema().release_i64_local(digit, function);
        self.runtime_schema()
            .release_i64_local(accumulator, function);
        self.runtime_schema().release_i64_local(write, function);
        self.runtime_schema().release_i64_local(read, function);
    }

    fn emit_decimal_round(
        &mut self,
        digits_ptr: I64Local,
        num_digits: I64Local,
        decimal_point: I64Local,
        truncated: I64Local,
        result: I64Local,
        function: &mut Function,
    ) {
        let index = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        result.store(function);
        num_digits.load(function);
        function.instruction(&Instruction::I64Eqz);
        decimal_point.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        decimal_point.load(function);
        function.instruction(&Instruction::I64Const(18));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        result.store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        index.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        decimal_point.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        result.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        result.store(function);
        index.load(function);
        num_digits.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_load_digit(digits_ptr, index, digit, function);
        result.load(function);
        digit.load(function);
        function.instruction(&Instruction::I64Add);
        result.store(function);
        function.instruction(&Instruction::End);
        self.emit_increment_local(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        decimal_point.load(function);
        num_digits.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_decimal_load_digit(digits_ptr, decimal_point, digit, function);
        digit.load(function);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64GtU);
        digit.load(function);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        decimal_point.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        num_digits.load(function);
        function.instruction(&Instruction::I64LtU);
        truncated.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        result.load(function);
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
        self.runtime_schema().release_i64_local(digit, function);
        self.runtime_schema().release_i64_local(index, function);
    }

    fn emit_decimal_trim(
        &mut self,
        digits_ptr: I64Local,
        num_digits: I64Local,
        function: &mut Function,
    ) {
        let last = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        num_digits.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        num_digits.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        last.store(function);
        self.emit_decimal_load_digit(digits_ptr, last, digit, function);
        digit.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        last.load(function);
        num_digits.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(digit, function);
        self.runtime_schema().release_i64_local(last, function);
    }

    fn emit_decimal_load_input_byte(
        &self,
        input: &GcLocal<CodeUnitArray>,
        index: I64Local,
        result: I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let position = schema.reserve_i32_local(function);
        index.load(function);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        let unit = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .read(input, position, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        result.store(function);
        schema.release_i32_local(unit, function);
        schema.release_i32_local(position, function);
    }

    fn emit_decimal_load_digit(
        &self,
        digits_ptr: I64Local,
        index: I64Local,
        result: I64Local,
        function: &mut Function,
    ) {
        digits_ptr.load(function);
        index.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        result.store(function);
    }

    fn emit_decimal_store_digit(
        &self,
        digits_ptr: I64Local,
        index: I64Local,
        digit: I64Local,
        function: &mut Function,
    ) {
        digits_ptr.load(function);
        index.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        digit.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }

    fn emit_decimal_store_digit_from_byte(
        &self,
        digits_ptr: I64Local,
        index: I64Local,
        byte: I64Local,
        function: &mut Function,
    ) {
        digits_ptr.load(function);
        index.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        byte.load(function);
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
    }
}
