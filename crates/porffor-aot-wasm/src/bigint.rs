//! Arbitrary-precision BigInt arithmetic (ECMA-262 6.1.6.2).
//!
//! A BigInt value is either *inline* — tag [`ValueKind::BigInt`], payload a
//! two's-complement `i64` — or *heap-backed* — tag [`HEAP_BIGINT_VALUE_TAG`],
//! payload the address of a `{ sign, limbs_ptr, limbs_len, limbs_cap }` record
//! whose limbs are little-endian `u64` magnitude words.
//!
//! Both forms are decoded into a common working form before any operation: a
//! little-endian vector of base-2^32 digits, one digit per 8-byte slot. Keeping
//! digits at 32 bits means every partial product, sum and shift fits an `i64`,
//! so the whole implementation needs no 128-bit intermediates. The result is
//! re-encoded inline when it fits an `i64` and heap-backed otherwise, so the
//! narrow representation stays the common case and every operation is exact
//! regardless of magnitude.
//!
//! The whole state machine lives in one shared runtime helper reached with a
//! plain `call`, rather than being inlined at every arithmetic site — inlining
//! it would push single function bodies past Cranelift's per-function limits.

use super::*;
use crate::abi::COMPLETION_KIND_THROW;
use crate::emit::CompletionKind;
use crate::heap::{
    HEAP_BIGINT_LIMBS_CAP_OFFSET, HEAP_BIGINT_LIMBS_LEN_OFFSET, HEAP_BIGINT_LIMBS_PTR_OFFSET,
    HEAP_BIGINT_RECORD_SIZE, HEAP_BIGINT_SIGN_OFFSET, HEAP_BIGINT_VALUE_TAG,
};

/// Operation selector passed to the shared BigInt helper in parameter 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BigIntHelperOp {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
    Rem = 4,
    Exp = 5,
    /// Numeric three-way comparison; the result is a Number (-1, 0 or 1).
    Compare = 6,
    /// Unary negation of the left operand; the right operand is ignored.
    Negate = 7,
    /// Exact three-way comparison of a BigInt against a Number (right operand
    /// payload is f64 bits). Yields NaN when the Number is NaN, so the caller's
    /// float comparison against zero is false for every relational operator —
    /// which is what ECMA-262 7.2.13 requires.
    CompareWithNumber = 8,
}

impl BigIntHelperOp {
    pub(crate) const fn from_arithmetic(op: ArithmeticBinaryOp) -> Self {
        match op {
            ArithmeticBinaryOp::Add => Self::Add,
            ArithmeticBinaryOp::Sub => Self::Sub,
            ArithmeticBinaryOp::Mul => Self::Mul,
            ArithmeticBinaryOp::Div => Self::Div,
            ArithmeticBinaryOp::Mod => Self::Rem,
            ArithmeticBinaryOp::Exp => Self::Exp,
        }
    }
}

/// Every value that reaches the helper carries one of these two tags.
pub(crate) const BIGINT_DIVISION_BY_ZERO_MESSAGE: &str = "BigInt division by zero";
pub(crate) const BIGINT_EXPONENT_MESSAGE: &str = "BigInt exponent must be non-negative";

const DIGIT_MASK: i64 = 0xffff_ffff;

impl<'a> FunctionBuilder<'a> {
    /// Calls the shared BigInt helper, storing its value in
    /// `out_payload_local`/`out_tag_local` and propagating a throw (division by
    /// zero, negative exponent) to the enclosing handler.
    pub(crate) fn emit_bigint_binary_op_to_locals(
        &mut self,
        op: BigIntHelperOp,
        lhs_payload_local: u32,
        lhs_tag_local: u32,
        rhs_payload_local: u32,
        rhs_tag_local: u32,
        out_payload_local: u32,
        out_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper = self
            .bigint_arithmetic_helper_function_index()
            .ok_or_else(|| {
                EmitError::unsupported("BigInt arithmetic requires the runtime heap helpers")
            })?;
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(op as i64));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(helper));
        self.store_call_results(out_payload_local, out_tag_local, function);
        self.emit_propagate_throw_from_locals_if_needed(
            out_payload_local,
            out_tag_local,
            function,
        )?;
        Ok(())
    }

    /// Relational comparison of two BigInts in either representation. Leaves
    /// the boolean result as an i32 on the stack; the comparison itself cannot
    /// throw, so no completion handling is needed.
    pub(crate) fn emit_bigint_relational_i32(
        &mut self,
        op: RelationalBinaryOp,
        lhs_payload_local: u32,
        lhs_tag_local: u32,
        rhs_payload_local: u32,
        rhs_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_bigint_comparison_i32(
            BigIntHelperOp::Compare,
            op,
            lhs_payload_local,
            lhs_tag_local,
            rhs_payload_local,
            rhs_tag_local,
            function,
        )
    }

    /// Relational comparison of a BigInt (in either representation) against a
    /// Number, without the lossy detour through ToNumber that ECMA-262 7.2.13
    /// forbids. Pass the reversed operator for `number < bigint`.
    pub(crate) fn emit_bigint_number_relational_i32(
        &mut self,
        op: RelationalBinaryOp,
        bigint_payload_local: u32,
        bigint_tag_local: u32,
        number_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_bigint_comparison_i32(
            BigIntHelperOp::CompareWithNumber,
            op,
            bigint_payload_local,
            bigint_tag_local,
            number_payload_local,
            number_payload_local,
            function,
        )
    }

    fn emit_bigint_comparison_i32(
        &mut self,
        helper_op: BigIntHelperOp,
        op: RelationalBinaryOp,
        lhs_payload_local: u32,
        lhs_tag_local: u32,
        rhs_payload_local: u32,
        rhs_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper = self
            .bigint_arithmetic_helper_function_index()
            .ok_or_else(|| {
                EmitError::unsupported("BigInt comparison requires the runtime heap helpers")
            })?;
        function.instruction(&Instruction::LocalGet(lhs_payload_local));
        function.instruction(&Instruction::LocalGet(lhs_tag_local));
        function.instruction(&Instruction::LocalGet(rhs_payload_local));
        function.instruction(&Instruction::LocalGet(rhs_tag_local));
        function.instruction(&Instruction::I64Const(helper_op as i64));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(helper));
        // Results are (payload, tag, completion, completion_aux); keep the
        // payload, which is the -1/0/1 comparison as f64 bits.
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0_f64)));
        match op {
            RelationalBinaryOp::LessThan => function.instruction(&Instruction::F64Lt),
            RelationalBinaryOp::LessThanOrEqual => function.instruction(&Instruction::F64Le),
            RelationalBinaryOp::GreaterThan => function.instruction(&Instruction::F64Gt),
            RelationalBinaryOp::GreaterThanOrEqual => function.instruction(&Instruction::F64Ge),
        };
        Ok(())
    }

    /// Evaluates two statically-BigInt operands and applies `op` through the
    /// shared helper, leaving the value in `out_payload_local` and its runtime
    /// tag (inline or heap-backed) in `out_tag_local`.
    pub(crate) fn compile_bigint_arithmetic_to_locals(
        &mut self,
        op: BigIntHelperOp,
        lhs: &TypedExpr,
        rhs: &TypedExpr,
        out_payload_local: u32,
        out_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload = self.reserve_temp_local();
        let lhs_tag = self.reserve_temp_local();
        let rhs_payload = self.reserve_temp_local();
        let rhs_tag = self.reserve_temp_local();
        self.compile_expr_to_locals(lhs, lhs_payload, lhs_tag, function)?;
        self.compile_expr_to_locals(rhs, rhs_payload, rhs_tag, function)?;
        self.emit_bigint_binary_op_to_locals(
            op,
            lhs_payload,
            lhs_tag,
            rhs_payload,
            rhs_tag,
            out_payload_local,
            out_tag_local,
            function,
        )?;
        self.release_temp_local(rhs_tag);
        self.release_temp_local(rhs_payload);
        self.release_temp_local(lhs_tag);
        self.release_temp_local(lhs_payload);
        Ok(())
    }

    /// Compiles the shared BigInt arithmetic helper.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=lhs payload,
    /// 1=lhs tag, 2=rhs payload, 3=rhs tag, 4=[`BigIntHelperOp`]. Params 5-6
    /// are unused. Results are the standard four-i64 tuple.
    pub(crate) fn compile_bigint_arithmetic_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.push_scope();

        let lhs_sign = self.reserve_temp_local();
        let lhs_ptr = self.reserve_temp_local();
        let lhs_len = self.reserve_temp_local();
        let rhs_sign = self.reserve_temp_local();
        let rhs_ptr = self.reserve_temp_local();
        let rhs_len = self.reserve_temp_local();
        let res_sign = self.reserve_temp_local();
        let res_ptr = self.reserve_temp_local();
        let res_len = self.reserve_temp_local();
        let op_local = self.reserve_temp_local();
        let scratch = self.reserve_temp_local();
        let fraction_sign = self.reserve_temp_local();
        let number_class = self.reserve_temp_local();

        self.set_completion_kind(CompletionKind::Normal, &mut function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::LocalSet(op_local));

        self.emit_bigint_decode_operand(0, 1, lhs_sign, lhs_ptr, lhs_len, &mut function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(fraction_sign));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(number_class));
        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(
            BigIntHelperOp::CompareWithNumber as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_decode_number_operand(
            2,
            rhs_sign,
            rhs_ptr,
            rhs_len,
            fraction_sign,
            number_class,
            &mut function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_bigint_decode_operand(2, 3, rhs_sign, rhs_ptr, rhs_len, &mut function)?;
        function.instruction(&Instruction::End);

        // `a - b` is `a + (-b)`; negation only flips the sign word, so the
        // signed-addition path below covers both.
        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Sub as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(rhs_sign));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Add as i64));
        function.instruction(&Instruction::LocalSet(op_local));
        function.instruction(&Instruction::End);

        // `-a` is the left operand with a flipped sign; reuse the add path with
        // a zero right operand.
        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Negate as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(lhs_sign));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rhs_sign));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(rhs_len));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Add as i64));
        function.instruction(&Instruction::LocalSet(op_local));
        function.instruction(&Instruction::End);

        // `scratch` doubles as the "result already produced" flag: Compare
        // yields a Number directly and skips the BigInt packing tail.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scratch));

        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Compare as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(
            BigIntHelperOp::CompareWithNumber as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_signed_compare(
            lhs_sign,
            lhs_ptr,
            lhs_len,
            rhs_sign,
            rhs_ptr,
            rhs_len,
            res_sign,
            &mut function,
        );
        // A non-zero fractional part breaks a tie against the truncated
        // Number, and `±∞` decides the comparison outright.
        function.instruction(&Instruction::LocalGet(res_sign));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(fraction_sign));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(number_class));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(number_class));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(number_class));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(res_sign));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(scratch));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Add as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_signed_add(
            lhs_sign,
            lhs_ptr,
            lhs_len,
            rhs_sign,
            rhs_ptr,
            rhs_len,
            res_sign,
            res_ptr,
            res_len,
            &mut function,
        )?;
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Mul as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_magnitude_mul(
            lhs_ptr,
            lhs_len,
            rhs_ptr,
            rhs_len,
            res_ptr,
            res_len,
            &mut function,
        )?;
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Exp as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_pow(
            lhs_sign,
            lhs_ptr,
            lhs_len,
            rhs_sign,
            rhs_ptr,
            rhs_len,
            res_sign,
            res_ptr,
            res_len,
            &mut function,
        )?;
        function.instruction(&Instruction::Else);

        // Div and Rem share the long division; only the sign and which output
        // is kept differ.
        self.emit_bigint_divmod_op(
            op_local,
            lhs_sign,
            lhs_ptr,
            lhs_len,
            rhs_sign,
            rhs_ptr,
            rhs_len,
            res_sign,
            res_ptr,
            res_len,
            &mut function,
        )?;

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Skip packing when Compare already produced the value, and when an
        // operand check threw — the throw left the error in the result slots.
        function.instruction(&Instruction::LocalGet(scratch));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_pack_result(res_sign, res_ptr, res_len, &mut function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(number_class);
        self.release_temp_local(fraction_sign);
        self.release_temp_local(scratch);
        self.release_temp_local(op_local);
        self.release_temp_local(res_len);
        self.release_temp_local(res_ptr);
        self.release_temp_local(res_sign);
        self.release_temp_local(rhs_len);
        self.release_temp_local(rhs_ptr);
        self.release_temp_local(rhs_sign);
        self.release_temp_local(lhs_len);
        self.release_temp_local(lhs_ptr);
        self.release_temp_local(lhs_sign);
        self.pop_scope();

        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Pushes the byte address of digit `index_local` of the vector at
    /// `ptr_local` as an i32.
    fn emit_bigint_digit_address(&self, ptr_local: u32, index_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
    }

    fn emit_bigint_digit_load(
        &self,
        ptr_local: u32,
        index_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        self.emit_bigint_digit_address(ptr_local, index_local, function);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(dest_local));
    }

    /// Loads digit `index_local`, or zero when the index is at or past
    /// `len_local`. This lets every loop run to the widest operand without a
    /// separate tail.
    fn emit_bigint_digit_load_or_zero(
        &self,
        ptr_local: u32,
        len_local: u32,
        index_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_bigint_digit_address(ptr_local, index_local, function);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(dest_local));
    }

    fn emit_bigint_digit_store(
        &self,
        ptr_local: u32,
        index_local: u32,
        value_local: u32,
        function: &mut Function,
    ) {
        self.emit_bigint_digit_address(ptr_local, index_local, function);
        function.instruction(&Instruction::LocalGet(value_local));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
    }

    /// Allocates `count_local` zeroed digit slots into `dest_local`.
    fn emit_bigint_digits_alloc(
        &mut self,
        count_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let bytes = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let zero = self.reserve_temp_local();
        // A zero-length vector still needs a valid address, so always allocate
        // at least one slot.
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(bytes));
        self.emit_heap_alloc_from_local(bytes, function)?;
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_bigint_digit_store(dest_local, index, zero, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(zero);
        self.release_temp_local(index);
        self.release_temp_local(bytes);
        Ok(())
    }

    /// Drops high zero digits so every vector has a canonical length.
    fn emit_bigint_normalize_len(
        &mut self,
        ptr_local: u32,
        len_local: u32,
        function: &mut Function,
    ) {
        let digit = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_load(ptr_local, index, digit, function);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(index);
        self.release_temp_local(digit);
    }

    /// Decodes a finite Number into the exact integer `trunc(value)` as a sign
    /// and digit vector, plus the sign of the discarded fractional part.
    ///
    /// `class_local` reports the non-finite cases: 0 finite, 1 NaN, 2 `+∞`,
    /// 3 `-∞`.
    fn emit_bigint_decode_number_operand(
        &mut self,
        payload_param: u32,
        sign_local: u32,
        ptr_local: u32,
        len_local: u32,
        fraction_sign_local: u32,
        class_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let bits = self.reserve_temp_local();
        let exponent = self.reserve_temp_local();
        let mantissa = self.reserve_temp_local();
        let scale = self.reserve_temp_local();
        let words = self.reserve_temp_local();
        let shift = self.reserve_temp_local();
        let low = self.reserve_temp_local();
        let high = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        let index = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(payload_param));
        function.instruction(&Instruction::LocalSet(bits));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(class_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(fraction_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_bigint_digits_alloc(len_local, ptr_local, function)?;

        // sign := -1 for a set sign bit, else 1 (corrected to 0 for a zero).
        function.instruction(&Instruction::LocalGet(bits));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));

        function.instruction(&Instruction::LocalGet(bits));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(exponent));
        function.instruction(&Instruction::LocalGet(bits));
        function.instruction(&Instruction::I64Const(0x000f_ffff_ffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(mantissa));

        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(0x7ff));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(class_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Zero or subnormal: |value| < 1, so the integer part is zero and only
        // the fractional sign survives.
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(fraction_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::Else);

        // value == mantissa * 2^(exponent - 1075) with the implicit leading bit.
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(mantissa));
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(1075));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(scale));

        function.instruction(&Instruction::LocalGet(scale));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Exact integer: shift the 53-bit mantissa left by `scale` bits.
        function.instruction(&Instruction::LocalGet(scale));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(words));
        function.instruction(&Instruction::LocalGet(scale));
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(shift));
        function.instruction(&Instruction::LocalGet(words));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_bigint_digits_alloc(len_local, ptr_local, function)?;
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(low));
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(high));
        // digit[words] = low << shift
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit));
        function.instruction(&Instruction::LocalGet(words));
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_store(ptr_local, index, digit, function);
        // digit[words + 1] = (low >> (32 - shift)) | (high << shift)
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_store(ptr_local, index, digit, function);
        // digit[words + 2] = high >> (32 - shift)
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_store(ptr_local, index, digit, function);
        function.instruction(&Instruction::Else);
        // Fractional: the integer part is `mantissa >> -scale` and the shifted
        // out bits decide the fractional sign.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(scale));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(scale));
        function.instruction(&Instruction::LocalGet(scale));
        function.instruction(&Instruction::I64Const(64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalSet(fraction_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(mantissa));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(scale));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(fraction_sign_local));
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::LocalGet(scale));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(mantissa));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_bigint_digits_alloc(len_local, ptr_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_store(ptr_local, index, digit, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::LocalGet(mantissa));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_store(ptr_local, index, digit, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_bigint_normalize_len(ptr_local, len_local, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(index);
        self.release_temp_local(digit);
        self.release_temp_local(high);
        self.release_temp_local(low);
        self.release_temp_local(shift);
        self.release_temp_local(words);
        self.release_temp_local(scale);
        self.release_temp_local(mantissa);
        self.release_temp_local(exponent);
        self.release_temp_local(bits);
        Ok(())
    }

    /// Decodes one operand (either representation) into sign / digit vector.
    fn emit_bigint_decode_operand(
        &mut self,
        payload_param: u32,
        tag_param: u32,
        sign_local: u32,
        ptr_local: u32,
        len_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = self.reserve_temp_local();
        let magnitude = self.reserve_temp_local();
        let limbs = self.reserve_temp_local();
        let limb_count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let limb = self.reserve_temp_local();
        let digit_index = self.reserve_temp_local();
        let digit = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(tag_param));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            payload_param,
            HEAP_BIGINT_SIGN_OFFSET,
            sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            payload_param,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs,
            function,
        );
        self.load_i64_to_local_from_offset(
            payload_param,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count,
            function,
        );
        function.instruction(&Instruction::LocalGet(limb_count));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_bigint_digits_alloc(len_local, ptr_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(limb_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(limbs));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::LocalSet(limb));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(digit_index));
        function.instruction(&Instruction::LocalGet(limb));
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_store(ptr_local, digit_index, digit, function);
        function.instruction(&Instruction::LocalGet(digit_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(digit_index));
        function.instruction(&Instruction::LocalGet(limb));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_store(ptr_local, digit_index, digit, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(payload_param));
        function.instruction(&Instruction::LocalSet(value));
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));
        // `0 - i64::MIN` wraps to 2^63, which is exactly the unsigned magnitude
        // of the most negative inline value.
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(magnitude));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_bigint_digits_alloc(len_local, ptr_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(digit_index));
        function.instruction(&Instruction::LocalGet(magnitude));
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_store(ptr_local, digit_index, digit, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digit_index));
        function.instruction(&Instruction::LocalGet(magnitude));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_store(ptr_local, digit_index, digit, function);
        function.instruction(&Instruction::End);

        self.emit_bigint_normalize_len(ptr_local, len_local, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(digit);
        self.release_temp_local(digit_index);
        self.release_temp_local(limb);
        self.release_temp_local(index);
        self.release_temp_local(limb_count);
        self.release_temp_local(limbs);
        self.release_temp_local(magnitude);
        self.release_temp_local(value);
        Ok(())
    }

    /// Three-way magnitude comparison; leaves -1, 0 or 1 in `out_local`.
    fn emit_bigint_magnitude_compare(
        &mut self,
        a_ptr: u32,
        a_len: u32,
        b_ptr: u32,
        b_len: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        let index = self.reserve_temp_local();
        let a_digit = self.reserve_temp_local();
        let b_digit = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::LocalGet(b_len));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(b_len));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_load_or_zero(a_ptr, a_len, index, a_digit, function);
        self.emit_bigint_digit_load_or_zero(b_ptr, b_len, index, b_digit, function);
        function.instruction(&Instruction::LocalGet(a_digit));
        function.instruction(&Instruction::LocalGet(b_digit));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(a_digit));
        function.instruction(&Instruction::LocalGet(b_digit));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(b_digit);
        self.release_temp_local(a_digit);
        self.release_temp_local(index);
    }

    /// Signed three-way comparison of two decoded operands.
    fn emit_bigint_signed_compare(
        &mut self,
        lhs_sign: u32,
        lhs_ptr: u32,
        lhs_len: u32,
        rhs_sign: u32,
        rhs_ptr: u32,
        rhs_len: u32,
        out_local: u32,
        function: &mut Function,
    ) {
        let magnitude = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::Else);
        self.emit_bigint_magnitude_compare(lhs_ptr, lhs_len, rhs_ptr, rhs_len, magnitude, function);
        function.instruction(&Instruction::LocalGet(magnitude));
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(out_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(magnitude);
    }

    /// `out = a + b` over magnitudes.
    fn emit_bigint_magnitude_add(
        &mut self,
        a_ptr: u32,
        a_len: u32,
        b_ptr: u32,
        b_len: u32,
        out_ptr: u32,
        out_len: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index = self.reserve_temp_local();
        let carry = self.reserve_temp_local();
        let a_digit = self.reserve_temp_local();
        let b_digit = self.reserve_temp_local();
        let sum = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::LocalGet(b_len));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(b_len));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_len));
        self.emit_bigint_digits_alloc(out_len, out_ptr, function)?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(out_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_bigint_digit_load_or_zero(a_ptr, a_len, index, a_digit, function);
        self.emit_bigint_digit_load_or_zero(b_ptr, b_len, index, b_digit, function);
        function.instruction(&Instruction::LocalGet(a_digit));
        function.instruction(&Instruction::LocalGet(b_digit));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(sum));
        function.instruction(&Instruction::LocalGet(sum));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::LocalGet(sum));
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(sum));
        self.emit_bigint_digit_store(out_ptr, index, sum, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_bigint_normalize_len(out_ptr, out_len, function);

        self.release_temp_local(sum);
        self.release_temp_local(b_digit);
        self.release_temp_local(a_digit);
        self.release_temp_local(carry);
        self.release_temp_local(index);
        Ok(())
    }

    /// `out = a - b` over magnitudes; the caller guarantees `a >= b`.
    fn emit_bigint_magnitude_sub(
        &mut self,
        a_ptr: u32,
        a_len: u32,
        b_ptr: u32,
        b_len: u32,
        out_ptr: u32,
        out_len: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let index = self.reserve_temp_local();
        let borrow = self.reserve_temp_local();
        let a_digit = self.reserve_temp_local();
        let b_digit = self.reserve_temp_local();
        let diff = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::LocalSet(out_len));
        self.emit_bigint_digits_alloc(out_len, out_ptr, function)?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(borrow));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(out_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_bigint_digit_load_or_zero(a_ptr, a_len, index, a_digit, function);
        self.emit_bigint_digit_load_or_zero(b_ptr, b_len, index, b_digit, function);
        function.instruction(&Instruction::LocalGet(a_digit));
        function.instruction(&Instruction::LocalGet(b_digit));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(borrow));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(diff));
        function.instruction(&Instruction::LocalGet(diff));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(diff));
        function.instruction(&Instruction::I64Const(1 << 32));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(diff));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(borrow));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(borrow));
        function.instruction(&Instruction::End);
        self.emit_bigint_digit_store(out_ptr, index, diff, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_bigint_normalize_len(out_ptr, out_len, function);

        self.release_temp_local(diff);
        self.release_temp_local(b_digit);
        self.release_temp_local(a_digit);
        self.release_temp_local(borrow);
        self.release_temp_local(index);
        Ok(())
    }

    /// Signed addition: same signs add magnitudes, opposite signs subtract the
    /// smaller magnitude from the larger and keep the larger operand's sign.
    #[allow(clippy::too_many_arguments)]
    fn emit_bigint_signed_add(
        &mut self,
        lhs_sign: u32,
        lhs_ptr: u32,
        lhs_len: u32,
        rhs_sign: u32,
        rhs_ptr: u32,
        rhs_len: u32,
        res_sign: u32,
        res_ptr: u32,
        res_len: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let comparison = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::LocalGet(rhs_ptr));
        function.instruction(&Instruction::LocalSet(res_ptr));
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::LocalSet(res_len));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::LocalGet(lhs_ptr));
        function.instruction(&Instruction::LocalSet(res_ptr));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalSet(res_len));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_magnitude_add(
            lhs_ptr, lhs_len, rhs_ptr, rhs_len, res_ptr, res_len, function,
        )?;
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::Else);
        self.emit_bigint_magnitude_compare(
            lhs_ptr, lhs_len, rhs_ptr, rhs_len, comparison, function,
        );
        function.instruction(&Instruction::LocalGet(comparison));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(res_len));
        function.instruction(&Instruction::LocalGet(lhs_ptr));
        function.instruction(&Instruction::LocalSet(res_ptr));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(comparison));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_magnitude_sub(
            lhs_ptr, lhs_len, rhs_ptr, rhs_len, res_ptr, res_len, function,
        )?;
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::Else);
        self.emit_bigint_magnitude_sub(
            rhs_ptr, rhs_len, lhs_ptr, lhs_len, res_ptr, res_len, function,
        )?;
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(comparison);
        Ok(())
    }

    /// `out = a * b` over magnitudes (schoolbook, base 2^32).
    fn emit_bigint_magnitude_mul(
        &mut self,
        a_ptr: u32,
        a_len: u32,
        b_ptr: u32,
        b_len: u32,
        out_ptr: u32,
        out_len: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let i = self.reserve_temp_local();
        let j = self.reserve_temp_local();
        let k = self.reserve_temp_local();
        let carry = self.reserve_temp_local();
        let a_digit = self.reserve_temp_local();
        let b_digit = self.reserve_temp_local();
        let acc = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::LocalGet(b_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(out_len));
        self.emit_bigint_digits_alloc(out_len, out_ptr, function)?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(i));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(i));
        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_bigint_digit_load(a_ptr, i, a_digit, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(j));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(j));
        function.instruction(&Instruction::LocalGet(b_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(i));
        function.instruction(&Instruction::LocalGet(j));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(k));
        self.emit_bigint_digit_load(b_ptr, j, b_digit, function);
        // (2^32-1)^2 + 2*(2^32-1) == 2^64-1, so the accumulator never wraps.
        self.emit_bigint_digit_load(out_ptr, k, acc, function);
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::LocalGet(a_digit));
        function.instruction(&Instruction::LocalGet(b_digit));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(acc));
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(acc));
        self.emit_bigint_digit_store(out_ptr, k, acc, function);
        function.instruction(&Instruction::LocalGet(j));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(j));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // Flush the carry into the high digits.
        function.instruction(&Instruction::LocalGet(i));
        function.instruction(&Instruction::LocalGet(b_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(k));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(k));
        function.instruction(&Instruction::LocalGet(out_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_bigint_digit_load(out_ptr, k, acc, function);
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(acc));
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::LocalGet(acc));
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(acc));
        self.emit_bigint_digit_store(out_ptr, k, acc, function);
        function.instruction(&Instruction::LocalGet(k));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(k));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(i));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(i));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_bigint_normalize_len(out_ptr, out_len, function);

        self.release_temp_local(acc);
        self.release_temp_local(b_digit);
        self.release_temp_local(a_digit);
        self.release_temp_local(carry);
        self.release_temp_local(k);
        self.release_temp_local(j);
        self.release_temp_local(i);
        Ok(())
    }

    /// Binary long division over magnitudes: `q = a / b`, `r = a % b`.
    ///
    /// Restoring shift-and-subtract rather than Knuth D — one bit per
    /// iteration keeps every step to a digit-wide shift, compare and subtract,
    /// which needs no quotient-digit estimation or normalisation pass.
    #[allow(clippy::too_many_arguments)]
    fn emit_bigint_magnitude_divmod(
        &mut self,
        a_ptr: u32,
        a_len: u32,
        b_ptr: u32,
        b_len: u32,
        q_ptr: u32,
        q_len: u32,
        r_ptr: u32,
        r_len: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let bit = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let digit = self.reserve_temp_local();
        let carry = self.reserve_temp_local();
        let shifted = self.reserve_temp_local();
        let comparison = self.reserve_temp_local();
        let borrow = self.reserve_temp_local();
        let b_digit = self.reserve_temp_local();
        let working_len = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::LocalSet(q_len));
        self.emit_bigint_digits_alloc(q_len, q_ptr, function)?;
        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(working_len));
        self.emit_bigint_digits_alloc(working_len, r_ptr, function)?;

        function.instruction(&Instruction::LocalGet(a_len));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(bit));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(bit));

        // carry := bit `bit` of the dividend
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_load(a_ptr, index, digit, function);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(carry));

        // remainder := remainder * 2 + carry
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(working_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_bigint_digit_load(r_ptr, index, digit, function);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(carry));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(shifted));
        function.instruction(&Instruction::LocalGet(shifted));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(carry));
        function.instruction(&Instruction::LocalGet(shifted));
        function.instruction(&Instruction::I64Const(DIGIT_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(shifted));
        self.emit_bigint_digit_store(r_ptr, index, shifted, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // if remainder >= divisor { remainder -= divisor; set quotient bit }
        self.emit_bigint_magnitude_compare(r_ptr, working_len, b_ptr, b_len, comparison, function);
        function.instruction(&Instruction::LocalGet(comparison));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(borrow));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(working_len));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_bigint_digit_load(r_ptr, index, digit, function);
        self.emit_bigint_digit_load_or_zero(b_ptr, b_len, index, b_digit, function);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::LocalGet(b_digit));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(borrow));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shifted));
        function.instruction(&Instruction::LocalGet(shifted));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shifted));
        function.instruction(&Instruction::I64Const(1 << 32));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(shifted));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(borrow));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(borrow));
        function.instruction(&Instruction::End);
        self.emit_bigint_digit_store(r_ptr, index, shifted, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_load(q_ptr, index, digit, function);
        function.instruction(&Instruction::LocalGet(digit));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_store(q_ptr, index, digit, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(working_len));
        function.instruction(&Instruction::LocalSet(r_len));
        self.emit_bigint_normalize_len(q_ptr, q_len, function);
        self.emit_bigint_normalize_len(r_ptr, r_len, function);

        self.release_temp_local(working_len);
        self.release_temp_local(b_digit);
        self.release_temp_local(borrow);
        self.release_temp_local(comparison);
        self.release_temp_local(shifted);
        self.release_temp_local(carry);
        self.release_temp_local(digit);
        self.release_temp_local(index);
        self.release_temp_local(bit);
        Ok(())
    }

    /// `x / y` and `x % y`. Truncating division: the quotient takes the product
    /// of the signs and the remainder keeps the dividend's sign, per ECMA-262
    /// `BigInt::divide` / `BigInt::remainder`.
    #[allow(clippy::too_many_arguments)]
    fn emit_bigint_divmod_op(
        &mut self,
        op_local: u32,
        lhs_sign: u32,
        lhs_ptr: u32,
        lhs_len: u32,
        rhs_sign: u32,
        rhs_ptr: u32,
        rhs_len: u32,
        res_sign: u32,
        res_ptr: u32,
        res_len: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let q_ptr = self.reserve_temp_local();
        let q_len = self.reserve_temp_local();
        let r_ptr = self.reserve_temp_local();
        let r_len = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            BIGINT_DIVISION_BY_ZERO_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_bigint_magnitude_divmod(
            lhs_ptr, lhs_len, rhs_ptr, rhs_len, q_ptr, q_len, r_ptr, r_len, function,
        )?;
        function.instruction(&Instruction::LocalGet(op_local));
        function.instruction(&Instruction::I64Const(BigIntHelperOp::Div as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(q_ptr));
        function.instruction(&Instruction::LocalSet(res_ptr));
        function.instruction(&Instruction::LocalGet(q_len));
        function.instruction(&Instruction::LocalSet(res_len));
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(r_ptr));
        function.instruction(&Instruction::LocalSet(res_ptr));
        function.instruction(&Instruction::LocalGet(r_len));
        function.instruction(&Instruction::LocalSet(res_len));
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::LocalSet(res_sign));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(r_len);
        self.release_temp_local(r_ptr);
        self.release_temp_local(q_len);
        self.release_temp_local(q_ptr);
        Ok(())
    }

    /// `base ** exponent` by square-and-multiply. A negative exponent is a
    /// RangeError (ECMA-262 `BigInt::exponentiate` step 1).
    #[allow(clippy::too_many_arguments)]
    fn emit_bigint_pow(
        &mut self,
        lhs_sign: u32,
        lhs_ptr: u32,
        lhs_len: u32,
        rhs_sign: u32,
        rhs_ptr: u32,
        rhs_len: u32,
        res_sign: u32,
        res_ptr: u32,
        res_len: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let exponent = self.reserve_temp_local();
        let odd = self.reserve_temp_local();
        let base_ptr = self.reserve_temp_local();
        let base_len = self.reserve_temp_local();
        let square_ptr = self.reserve_temp_local();
        let square_len = self.reserve_temp_local();
        let one = self.reserve_temp_local();
        let digit = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(rhs_sign));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            BIGINT_EXPONENT_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);

        // Only exponents that fit 64 bits are representable as a loop count; a
        // wider one could never be materialised anyway.
        function.instruction(&Instruction::LocalGet(rhs_len));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            BIGINT_EXPONENT_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(exponent));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_load_or_zero(rhs_ptr, rhs_len, digit, exponent, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digit));
        self.emit_bigint_digit_load_or_zero(rhs_ptr, rhs_len, digit, one, function);
        function.instruction(&Instruction::LocalGet(one));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(exponent));
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(odd));

        // result := 1
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(res_len));
        self.emit_bigint_digits_alloc(res_len, res_ptr, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(digit));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(one));
        self.emit_bigint_digit_store(res_ptr, digit, one, function);
        function.instruction(&Instruction::LocalGet(lhs_ptr));
        function.instruction(&Instruction::LocalSet(base_ptr));
        function.instruction(&Instruction::LocalGet(lhs_len));
        function.instruction(&Instruction::LocalSet(base_len));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_bigint_magnitude_mul(
            res_ptr, res_len, base_ptr, base_len, square_ptr, square_len, function,
        )?;
        function.instruction(&Instruction::LocalGet(square_ptr));
        function.instruction(&Instruction::LocalSet(res_ptr));
        function.instruction(&Instruction::LocalGet(square_len));
        function.instruction(&Instruction::LocalSet(res_len));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(exponent));
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_bigint_magnitude_mul(
            base_ptr, base_len, base_ptr, base_len, square_ptr, square_len, function,
        )?;
        function.instruction(&Instruction::LocalGet(square_ptr));
        function.instruction(&Instruction::LocalSet(base_ptr));
        function.instruction(&Instruction::LocalGet(square_len));
        function.instruction(&Instruction::LocalSet(base_len));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // The result is negative only for a negative base raised to an odd
        // exponent.
        function.instruction(&Instruction::LocalGet(res_len));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(lhs_sign));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(odd));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(res_sign));

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(digit);
        self.release_temp_local(one);
        self.release_temp_local(square_len);
        self.release_temp_local(square_ptr);
        self.release_temp_local(base_len);
        self.release_temp_local(base_ptr);
        self.release_temp_local(odd);
        self.release_temp_local(exponent);
        Ok(())
    }

    /// Re-encodes a sign/digit-vector result: inline when it fits an `i64`,
    /// heap-backed otherwise.
    fn emit_bigint_pack_result(
        &mut self,
        sign_local: u32,
        ptr_local: u32,
        len_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let low = self.reserve_temp_local();
        let high = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let record = self.reserve_temp_local();
        let limbs = self.reserve_temp_local();
        let limb_count = self.reserve_temp_local();
        let limb = self.reserve_temp_local();
        let bytes = self.reserve_temp_local();

        self.emit_bigint_normalize_len(ptr_local, len_local, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_load_or_zero(ptr_local, len_local, index, low, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(index));
        self.emit_bigint_digit_load_or_zero(ptr_local, len_local, index, high, function);
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(value));

        // Inline when at most two digits are set and the signed magnitude is
        // representable: `[-2^63, 2^63-1]`.
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(i64::MIN));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(limb_count));
        function.instruction(&Instruction::LocalGet(limb_count));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(bytes));
        self.emit_heap_alloc_const(HEAP_BIGINT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record));
        self.emit_heap_alloc_from_local(bytes, function)?;
        function.instruction(&Instruction::LocalSet(limbs));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(limb_count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(low));
        self.emit_bigint_digit_load_or_zero(ptr_local, len_local, low, limb, function);
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low));
        self.emit_bigint_digit_load_or_zero(ptr_local, len_local, low, high, function);
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(limb));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(limb));
        function.instruction(&Instruction::LocalGet(limbs));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(limb));
        function.instruction(&Instruction::I64Store(Self::memarg64(0)));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(record, HEAP_BIGINT_SIGN_OFFSET, sign_local, function);
        self.store_i64_local_at_offset(record, HEAP_BIGINT_LIMBS_PTR_OFFSET, limbs, function);
        self.store_i64_local_at_offset(record, HEAP_BIGINT_LIMBS_LEN_OFFSET, limb_count, function);
        self.store_i64_local_at_offset(record, HEAP_BIGINT_LIMBS_CAP_OFFSET, limb_count, function);
        function.instruction(&Instruction::LocalGet(record));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(bytes);
        self.release_temp_local(limb);
        self.release_temp_local(limb_count);
        self.release_temp_local(limbs);
        self.release_temp_local(record);
        self.release_temp_local(index);
        self.release_temp_local(value);
        self.release_temp_local(high);
        self.release_temp_local(low);
        Ok(())
    }
}
