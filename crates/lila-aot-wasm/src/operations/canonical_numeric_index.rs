//! CanonicalNumericIndexString (7.1.23) without a per-call allocation.
//!
//! Every String-keyed `[[Get]]`, `[[Set]]`, `[[HasProperty]]`,
//! `[[GetOwnProperty]]`, `[[DefineOwnProperty]]` and `[[Delete]]` on a
//! TypedArray starts here, so it runs once per element access in a loop such
//! as `for (…) dst[i] = src[i]`. The literal algorithm, `ToString(ToNumber(P))`
//! compared with `P`, materialises a fresh String on the bump heap every time;
//! with no collector that is a leak proportional to the number of accesses.
//!
//! The answer does not depend on that String, so it is computed in three
//! tiers, each exactly equivalent to the specification:
//!
//! 1. `P` is 1..=15 ASCII decimal digits with no leading zero (or exactly
//!    `"0"`): canonical, and its value is the digits read as an integer. Every
//!    such integer is below 10^15 < 2^53, so it is an exact Number whose
//!    ToString is precisely those digits.
//! 2. `P` is empty, or its first byte is none of `0-9`, `-`, `I` or `N`: not
//!    canonical. Number::toString only ever produces a String starting with a
//!    digit, `-` (`"-1"`, `"-Infinity"`, `"-1e-7"`), `I` (`"Infinity"`) or `N`
//!    (`"NaN"`), and `"-0"` also starts with `-`. This covers every ordinary
//!    property name (`"length"`, `"buffer"`, `"constructor"`, …). `"-0"`
//!    itself is then recognised by its two bytes.
//! 3. Everything else (`"-1"`, `"1.5"`, `"1e+21"`, `"01"`, `"NaN"`, 16+ digit
//!    strings, …): the literal algorithm. Number::toString allocates only its
//!    result (the static `"NaN"`/`"Infinity"`/`"0"` payloads allocate
//!    nothing), and that result is dead once compared, so the bump pointer is
//!    rewound over it and the region is cleared, because fresh heap memory is
//!    assumed to be zero — the same discipline as the decimal-to-binary64
//!    scratch.

use super::*;

/// Longest all-digit String whose integer value is certainly an exact Number.
/// 10^15 - 1 < 2^53; a 16-digit string may exceed 2^53 and is left to tier 3.
const EXACT_DECIMAL_INTEGER_DIGITS: i64 = 15;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_canonical_numeric_index_string(
        &mut self,
        string_payload_local: u32,
        number_payload_local: u32,
        is_canonical_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let integer_local = self.reserve_temp_local();
        let saved_heap_local = self.reserve_temp_local();
        let canonical_string_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(is_canonical_local));
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(number_payload_local));

        function.instruction(&Instruction::Block(BlockType::Empty));

        self.emit_unpack_string_payload(string_payload_local, offset_local, len_local, function);
        // Tier 2, empty half: ToString never produces "".
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        self.emit_load_string_byte(offset_local, byte_index_local, byte_local, function);

        // Tier 1: a short decimal integer without a leading zero.
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(EXACT_DECIMAL_INTEGER_DIGITS));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(integer_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, byte_index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        // A non-digit ("1.5", "1e+21") leaves the loop with the index short
        // of the length and falls through to tier 3.
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(integer_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(integer_local));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(integer_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_canonical_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        self.emit_load_string_byte(offset_local, byte_index_local, byte_local, function);
        function.instruction(&Instruction::End);

        // Tier 2, first-byte half.
        self.emit_byte_is_digit_i32(byte_local, function);
        for leading in [b'-', b'I', b'N'] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(leading as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));

        // Step 1: "-0" is the one canonical String that is not ToString of
        // its own value (ToString(-0) is "0").
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        self.emit_load_string_byte(offset_local, byte_index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(Ieee64::from(-0.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_canonical_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Tier 3: the literal algorithm, with its one allocation reclaimed.
        self.emit_string_to_number_payload(string_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(number_payload_local));
        function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(saved_heap_local));
        self.emit_number_to_string_payload(number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(canonical_string_payload_local));
        self.emit_string_payload_equality_i32(
            string_payload_local,
            canonical_string_payload_local,
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(is_canonical_local));
        function.instruction(&Instruction::LocalGet(saved_heap_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(saved_heap_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        function.instruction(&Instruction::LocalGet(saved_heap_local));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));

        function.instruction(&Instruction::End);

        self.release_temp_local(canonical_string_payload_local);
        self.release_temp_local(saved_heap_local);
        self.release_temp_local(integer_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(byte_index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(offset_local);
        Ok(())
    }
}
