use super::*;

impl FunctionBuilder<'_> {
    /// Number::remainder on already-converted Number payloads. Integer long
    /// division keeps the exact binary significands: a rounded floating-point
    /// quotient can overflow or lose the low bits needed by the remainder.
    pub(crate) fn emit_number_remainder_payload(
        &mut self,
        numerator_local: crate::gc_types::I64Local,
        denominator_local: crate::gc_types::I64Local,
        output_local: crate::gc_types::I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        const MAGNITUDE: i64 = i64::MAX;
        const INFINITY: i64 = 0x7ff0_0000_0000_0000;
        const HIDDEN_BIT: i64 = 1 << 52;
        const FRACTION: i64 = HIDDEN_BIT - 1;

        let sign_local = schema.reserve_i64_local(function);
        let numerator_significand_local = schema.reserve_i64_local(function);
        let denominator_significand_local = schema.reserve_i64_local(function);
        let numerator_exponent_local = schema.reserve_i64_local(function);
        let denominator_exponent_local = schema.reserve_i64_local(function);

        numerator_local.load(function);
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64And);
        sign_local.store(function);
        for (input, magnitude) in [
            (numerator_local, numerator_significand_local),
            (denominator_local, denominator_significand_local),
        ] {
            input.load(function);
            function.instruction(&Instruction::I64Const(MAGNITUDE));
            function.instruction(&Instruction::I64And);
            magnitude.store(function);
        }

        function.instruction(&Instruction::Block(BlockType::Empty));
        numerator_significand_local.load(function);
        function.instruction(&Instruction::I64Const(INFINITY));
        function.instruction(&Instruction::I64GeU);
        denominator_significand_local.load(function);
        function.instruction(&Instruction::I64Const(INFINITY));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        denominator_significand_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        output_local.store(function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        // This includes a finite numerator modulo infinity and either signed
        // zero modulo any non-zero denominator. Preserve its original bits.
        numerator_significand_local.load(function);
        denominator_significand_local.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        numerator_local.load(function);
        output_local.store(function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        numerator_significand_local.load(function);
        denominator_significand_local.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        output_local.store(function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        // Both magnitudes are finite and non-zero. Normalize subnormals to the
        // same 53-bit significand as normal values, allowing negative exponents.
        for (significand, exponent) in [
            (numerator_significand_local, numerator_exponent_local),
            (denominator_significand_local, denominator_exponent_local),
        ] {
            significand.load(function);
            function.instruction(&Instruction::I64Const(52));
            function.instruction(&Instruction::I64ShrU);
            exponent.store(function);
            exponent.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            significand.load(function);
            function.instruction(&Instruction::I64Clz);
            function.instruction(&Instruction::I64Const(11));
            function.instruction(&Instruction::I64Sub);
            exponent.store(function);
            significand.load(function);
            exponent.load(function);
            function.instruction(&Instruction::I64Shl);
            significand.store(function);
            function.instruction(&Instruction::I64Const(1));
            exponent.load(function);
            function.instruction(&Instruction::I64Sub);
            exponent.store(function);
            function.instruction(&Instruction::Else);
            significand.load(function);
            function.instruction(&Instruction::I64Const(FRACTION));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Const(HIDDEN_BIT));
            function.instruction(&Instruction::I64Or);
            significand.store(function);
            function.instruction(&Instruction::End);
        }

        // The residual is below twice the divisor significand after every
        // shift, so subtraction and doubling fit in 54 bits without rounding.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        numerator_exponent_local.load(function);
        denominator_exponent_local.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        numerator_significand_local.load(function);
        denominator_significand_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        numerator_significand_local.load(function);
        denominator_significand_local.load(function);
        function.instruction(&Instruction::I64Sub);
        numerator_significand_local.store(function);
        function.instruction(&Instruction::End);
        numerator_significand_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        numerator_significand_local.store(function);
        numerator_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        numerator_exponent_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        numerator_significand_local.load(function);
        denominator_significand_local.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        numerator_significand_local.load(function);
        denominator_significand_local.load(function);
        function.instruction(&Instruction::I64Sub);
        numerator_significand_local.store(function);
        function.instruction(&Instruction::End);

        numerator_significand_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        output_local.store(function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        numerator_significand_local.load(function);
        function.instruction(&Instruction::I64Clz);
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Sub);
        numerator_exponent_local.store(function);
        numerator_significand_local.load(function);
        numerator_exponent_local.load(function);
        function.instruction(&Instruction::I64Shl);
        numerator_significand_local.store(function);
        denominator_exponent_local.load(function);
        numerator_exponent_local.load(function);
        function.instruction(&Instruction::I64Sub);
        denominator_exponent_local.store(function);
        denominator_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        numerator_significand_local.load(function);
        function.instruction(&Instruction::I64Const(FRACTION));
        function.instruction(&Instruction::I64And);
        denominator_exponent_local.load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::Else);
        numerator_significand_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        denominator_exponent_local.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::End);
        sign_local.load(function);
        function.instruction(&Instruction::I64Or);
        output_local.store(function);
        function.instruction(&Instruction::End);

        schema.release_i64_local(denominator_exponent_local, function);
        schema.release_i64_local(numerator_exponent_local, function);
        schema.release_i64_local(denominator_significand_local, function);
        schema.release_i64_local(numerator_significand_local, function);
        schema.release_i64_local(sign_local, function);
    }
}
