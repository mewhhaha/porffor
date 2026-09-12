use super::*;

pub(crate) const NUMBER_REMAINDER_TEMP_LOCALS: usize = 5;

impl FunctionBuilder<'_> {
    /// Number::remainder on already-converted Number payloads. Integer long
    /// division keeps the exact binary significands: a rounded floating-point
    /// quotient can overflow or lose the low bits needed by the remainder.
    pub(crate) fn emit_number_remainder_payload(
        &mut self,
        numerator_local: u32,
        denominator_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        const MAGNITUDE: i64 = i64::MAX;
        const INFINITY: i64 = 0x7ff0_0000_0000_0000;
        const HIDDEN_BIT: i64 = 1 << 52;
        const FRACTION: i64 = HIDDEN_BIT - 1;

        let sign_local = self.reserve_temp_local();
        let numerator_significand_local = self.reserve_temp_local();
        let denominator_significand_local = self.reserve_temp_local();
        let numerator_exponent_local = self.reserve_temp_local();
        let denominator_exponent_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(numerator_local));
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(sign_local));
        for (input, magnitude) in [
            (numerator_local, numerator_significand_local),
            (denominator_local, denominator_significand_local),
        ] {
            function.instruction(&Instruction::LocalGet(input));
            function.instruction(&Instruction::I64Const(MAGNITUDE));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(magnitude));
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::I64Const(INFINITY));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(denominator_significand_local));
        function.instruction(&Instruction::I64Const(INFINITY));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(denominator_significand_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        // This includes a finite numerator modulo infinity and either signed
        // zero modulo any non-zero denominator. Preserve its original bits.
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(denominator_significand_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(numerator_local));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(denominator_significand_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        // Both magnitudes are finite and non-zero. Normalize subnormals to the
        // same 53-bit significand as normal values, allowing negative exponents.
        for (significand, exponent) in [
            (numerator_significand_local, numerator_exponent_local),
            (denominator_significand_local, denominator_exponent_local),
        ] {
            function.instruction(&Instruction::LocalGet(significand));
            function.instruction(&Instruction::I64Const(52));
            function.instruction(&Instruction::I64ShrU);
            function.instruction(&Instruction::LocalTee(exponent));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(significand));
            function.instruction(&Instruction::I64Clz);
            function.instruction(&Instruction::I64Const(11));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(exponent));
            function.instruction(&Instruction::LocalGet(significand));
            function.instruction(&Instruction::LocalGet(exponent));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::LocalSet(significand));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalGet(exponent));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(exponent));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(significand));
            function.instruction(&Instruction::I64Const(FRACTION));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(HIDDEN_BIT));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(significand));
            function.instruction(&Instruction::End);
        }

        // The residual is below twice the divisor significand after every
        // shift, so subtraction and doubling fit in 54 bits without rounding.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(numerator_exponent_local));
        function.instruction(&Instruction::LocalGet(denominator_exponent_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(denominator_significand_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(denominator_significand_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(numerator_significand_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(numerator_exponent_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(numerator_exponent_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(denominator_significand_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(denominator_significand_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(numerator_significand_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::I64Clz);
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(numerator_exponent_local));
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(numerator_exponent_local));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(numerator_significand_local));
        function.instruction(&Instruction::LocalGet(denominator_exponent_local));
        function.instruction(&Instruction::LocalGet(numerator_exponent_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(denominator_exponent_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::I64Const(FRACTION));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalGet(denominator_exponent_local));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(numerator_significand_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(denominator_exponent_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(denominator_exponent_local);
        self.release_temp_local(numerator_exponent_local);
        self.release_temp_local(denominator_significand_local);
        self.release_temp_local(numerator_significand_local);
        self.release_temp_local(sign_local);
    }
}
