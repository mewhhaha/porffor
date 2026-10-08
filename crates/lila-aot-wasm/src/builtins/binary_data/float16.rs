//! Shared binary16 kernels; all operands are typed scalar locals.
use super::super::super::*;
use crate::gc_types::I64Local;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_half_bits_to_f64_payload(
        &mut self,
        half_local: I64Local,
        sign_local: I64Local,
        exp_local: I64Local,
        frac_local: I64Local,
        f32_bits_local: I64Local,
        norm_exp_local: I64Local,
        function: &mut Function,
    ) {
        half_local.load(function);
        function.instruction(&Instruction::I64Const(0x8000));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Shl);
        sign_local.store(function);
        half_local.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x1f));
        function.instruction(&Instruction::I64And);
        exp_local.store(function);
        half_local.load(function);
        function.instruction(&Instruction::I64Const(0x03ff));
        function.instruction(&Instruction::I64And);
        frac_local.store(function);

        exp_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        frac_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        f32_bits_local.store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-14));
        norm_exp_local.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        frac_local.load(function);
        function.instruction(&Instruction::I64Const(0x0400));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(1));
        frac_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        frac_local.store(function);
        norm_exp_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        norm_exp_local.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        frac_local.load(function);
        function.instruction(&Instruction::I64Const(0x03ff));
        function.instruction(&Instruction::I64And);
        frac_local.store(function);
        sign_local.load(function);
        norm_exp_local.load(function);
        function.instruction(&Instruction::I64Const(127));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(23));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        frac_local.load(function);
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        f32_bits_local.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        exp_local.load(function);
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        function.instruction(&Instruction::I64Const(0x7f800000));
        function.instruction(&Instruction::I64Or);
        frac_local.load(function);
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        f32_bits_local.store(function);
        function.instruction(&Instruction::Else);
        sign_local.load(function);
        exp_local.load(function);
        function.instruction(&Instruction::I64Const(112));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(23));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        frac_local.load(function);
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        f32_bits_local.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        f32_bits_local.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::F32ReinterpretI32);
        function.instruction(&Instruction::F64PromoteF32);
    }

    pub(crate) fn emit_f64_payload_to_half_bits_local(
        &mut self,
        value_payload_local: I64Local,
        half_local: I64Local,
        sign_local: I64Local,
        exp_local: I64Local,
        fraction_local: I64Local,
        rounded_local: I64Local,
        remainder_local: I64Local,
        significand_local: I64Local,
        function: &mut Function,
    ) {
        // Binary16 must be rounded directly from f64. An f32 intermediate
        // double-rounds values immediately adjacent to binary16 midpoints.
        value_payload_local.load(function);
        function.instruction(&Instruction::I64Const(48));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x8000));
        function.instruction(&Instruction::I64And);
        sign_local.store(function);
        value_payload_local.load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64And);
        exp_local.store(function);
        value_payload_local.load(function);
        function.instruction(&Instruction::I64Const(0x000f_ffff_ffff_ffff));
        function.instruction(&Instruction::I64And);
        fraction_local.store(function);

        exp_local.load(function);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        fraction_local.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0x0200));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Or);
        half_local.store(function);
        function.instruction(&Instruction::Else);

        exp_local.load(function);
        function.instruction(&Instruction::I64Const(1009));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        value_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(16_777_216.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Nearest);
        function.instruction(&Instruction::I64TruncF64U);
        sign_local.load(function);
        function.instruction(&Instruction::I64Or);
        half_local.store(function);
        function.instruction(&Instruction::Else);

        exp_local.load(function);
        function.instruction(&Instruction::I64Const(1038));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        half_local.store(function);
        function.instruction(&Instruction::Else);

        fraction_local.load(function);
        function.instruction(&Instruction::I64Const(0x0010_0000_0000_0000));
        function.instruction(&Instruction::I64Or);
        significand_local.store(function);
        significand_local.load(function);
        function.instruction(&Instruction::I64Const(42));
        function.instruction(&Instruction::I64ShrU);
        rounded_local.store(function);
        significand_local.load(function);
        function.instruction(&Instruction::I64Const(0x0000_03ff_ffff_ffff));
        function.instruction(&Instruction::I64And);
        remainder_local.store(function);

        remainder_local.load(function);
        function.instruction(&Instruction::I64Const(0x0000_0200_0000_0000));
        function.instruction(&Instruction::I64GtU);
        remainder_local.load(function);
        function.instruction(&Instruction::I64Const(0x0000_0200_0000_0000));
        function.instruction(&Instruction::I64Eq);
        rounded_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        rounded_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        rounded_local.store(function);
        function.instruction(&Instruction::End);

        exp_local.load(function);
        function.instruction(&Instruction::I64Const(1008));
        function.instruction(&Instruction::I64Sub);
        exp_local.store(function);
        rounded_local.load(function);
        function.instruction(&Instruction::I64Const(2048));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1024));
        rounded_local.store(function);
        exp_local.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        exp_local.store(function);
        function.instruction(&Instruction::End);

        exp_local.load(function);
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        sign_local.load(function);
        function.instruction(&Instruction::I64Const(0x7c00));
        function.instruction(&Instruction::I64Or);
        half_local.store(function);
        function.instruction(&Instruction::Else);
        sign_local.load(function);
        exp_local.load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        rounded_local.load(function);
        function.instruction(&Instruction::I64Const(0x03ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        half_local.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }
}
