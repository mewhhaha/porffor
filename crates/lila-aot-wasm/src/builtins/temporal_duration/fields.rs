//! Integral Number fields and exact projections at the Duration arithmetic boundary.

use super::*;

/// Wasm i64 locals containing canonical integral Number bits, never i64 field
/// values. There is deliberately no Index/Deref implementation: a caller must
/// name the representation before inspecting or writing a field.
pub(crate) struct TemporalDurationFields([u32; 10]);

impl TemporalDurationFields {
    pub(super) fn new(number_bits: [u32; 10]) -> Self {
        Self(number_bits)
    }

    pub(crate) fn number_bits(&self, unit: TemporalUnit) -> u32 {
        self.0[unit.duration_field_index()]
    }

    pub(crate) fn number_bits_locals(&self) -> &[u32; 10] {
        &self.0
    }
}

/// The three units whose canonical fields need a wide integer projection.
#[derive(Clone, Copy)]
pub(crate) enum TemporalDurationSubsecondUnit {
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl TemporalDurationSubsecondUnit {
    pub(crate) const ALL: [Self; 3] = [Self::Millisecond, Self::Microsecond, Self::Nanosecond];

    pub(crate) const fn temporal_unit(self) -> TemporalUnit {
        match self {
            Self::Millisecond => TemporalUnit::Millisecond,
            Self::Microsecond => TemporalUnit::Microsecond,
            Self::Nanosecond => TemporalUnit::Nanosecond,
        }
    }

    const fn per_second(self) -> i64 {
        match self {
            Self::Millisecond => 1_000,
            Self::Microsecond => 1_000_000,
            Self::Nanosecond => 1_000_000_000,
        }
    }

    pub(crate) const fn nanoseconds(self) -> i64 {
        match self {
            Self::Millisecond => 1_000_000,
            Self::Microsecond => 1_000,
            Self::Nanosecond => 1,
        }
    }
}

/// A balanced field has already discarded its fractional unit; total retains
/// that fraction. The closed projection supplies both factors, so a caller
/// cannot select an inconsistent scale or an unsafe division bound.
pub(crate) enum TemporalDurationNumberProjection {
    BalancedField(TemporalDurationSubsecondUnit),
    Total(TemporalDurationSubsecondUnit),
}

impl TemporalDurationNumberProjection {
    const fn factors(self) -> (i64, i64) {
        match self {
            Self::BalancedField(unit) => (unit.per_second(), 1),
            Self::Total(unit) => (1_000_000_000, unit.nanoseconds()),
        }
    }
}

const _: () = {
    let mut index = 0;
    while index < TemporalDurationSubsecondUnit::ALL.len() {
        let unit = TemporalDurationSubsecondUnit::ALL[index];
        assert!(unit.per_second() * unit.nanoseconds() == 1_000_000_000);
        let field = TemporalDurationNumberProjection::BalancedField(unit).factors();
        let total = TemporalDurationNumberProjection::Total(unit).factors();
        // Products of 32-bit limbs stay in u64, and doubled long-division
        // remainders stay in i64 for every admitted projection.
        assert!(field.0 > 0 && field.0 <= 1_000_000_000 && field.1 == 1);
        assert!(total.0 == 1_000_000_000 && total.1 > 0 && total.1 <= 1_000_000);
        index += 1;
    }
};

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_temporal_duration_canonicalize_zero(
        &mut self,
        number_bits: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(number_bits));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(number_bits));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_temporal_duration_negate_fields(
        &mut self,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) {
        for local in fields.number_bits_locals() {
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(*local));
            function.instruction(&Instruction::I64Const(i64::MIN));
            function.instruction(&Instruction::I64Xor);
            function.instruction(&Instruction::LocalSet(*local));
            function.instruction(&Instruction::End);
        }
    }

    /// Calendar arithmetic consumes only the four bounded date fields. Wide
    /// time fields have no narrowing accessor and must use exact normalization.
    pub(crate) fn reserve_temporal_duration_date_field_locals(
        &mut self,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) -> [u32; 4] {
        std::array::from_fn(|index| {
            let local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(
                fields.number_bits(TemporalUnit::ALL[index]),
            ));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::LocalSet(local));
            local
        })
    }

    /// `Number` conversion happens once, before canonical validation/storage.
    pub(crate) fn emit_temporal_duration_set_integer_field(
        &mut self,
        fields: &TemporalDurationFields,
        unit: TemporalUnit,
        integer: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(integer));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(fields.number_bits(unit)));
    }

    /// Divide the exact integer represented by a finite integral Number. The
    /// caller has bounded the field below 2^53 * divisor, so the quotient fits
    /// i64. Shifting the quotient/remainder pair avoids rounding a floating
    /// division or first narrowing a wide Number to i64.
    pub(super) fn emit_temporal_duration_number_divmod(
        &mut self,
        number_bits: u32,
        unit: TemporalDurationSubsecondUnit,
        quotient: u32,
        remainder: u32,
        function: &mut Function,
    ) {
        let divisor = unit.per_second();
        let magnitude = self.reserve_temp_local();
        let significand = self.reserve_temp_local();
        let shift = self.reserve_temp_local();
        for local in [quotient, remainder] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(number_bits));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalTee(magnitude));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(magnitude));
        function.instruction(&Instruction::I64Const((1_i64 << 52) - 1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(significand));
        function.instruction(&Instruction::LocalGet(magnitude));
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1075));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(significand));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(significand));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(shift));
        function.instruction(&Instruction::End);
        for (destination, operation) in [
            (quotient, Instruction::I64DivU),
            (remainder, Instruction::I64RemU),
        ] {
            function.instruction(&Instruction::LocalGet(significand));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&operation);
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        for local in [quotient, remainder] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::LocalGet(remainder));
        function.instruction(&Instruction::I64Const(divisor));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(quotient));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(quotient));
        function.instruction(&Instruction::LocalGet(remainder));
        function.instruction(&Instruction::I64Const(divisor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(shift));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(shift));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(number_bits));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        for local in [quotient, remainder] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(shift);
        self.release_temp_local(significand);
        self.release_temp_local(magnitude);
    }

    /// Convert exact `(seconds * scale + remainder) / divisor` to Number with
    /// one rounding. Inputs are nonnegative, seconds are below 2^54, and the
    /// remainder is below the projection's scale. The closed scale is at most
    /// 10^9, so the product fits in 84 bits; the divisor is at most 10^6.
    /// Binary long division emits 53 significant bits, a guard bit, and exact
    /// sticky information. This also preserves fractional units in `total`.
    pub(crate) fn emit_temporal_duration_scaled_time_number(
        &mut self,
        seconds: u32,
        remainder: u32,
        projection: TemporalDurationNumberProjection,
        output_bits: u32,
        function: &mut Function,
    ) {
        let (scale, divisor) = projection.factors();
        let low = self.reserve_temp_local();
        let high = self.reserve_temp_local();
        let bit_index = self.reserve_temp_local();
        let significand = self.reserve_temp_local();
        let division_remainder = self.reserve_temp_local();
        let bit = self.reserve_temp_local();
        let bit_count = self.reserve_temp_local();
        let exponent = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(seconds));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(scale));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(remainder));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low));
        function.instruction(&Instruction::LocalGet(seconds));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(scale));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high));
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(low));
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(high));
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::Else);
        for local in [significand, division_remainder, bit_count, exponent] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::I64Const(127));
        function.instruction(&Instruction::LocalSet(bit_index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(bit));
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(high));
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(low));
        function.instruction(&Instruction::LocalGet(division_remainder));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(division_remainder));
        function.instruction(&Instruction::LocalGet(division_remainder));
        function.instruction(&Instruction::I64Const(divisor));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(bit));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(division_remainder));
        function.instruction(&Instruction::I64Const(divisor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(division_remainder));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(bit_count));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(bit_index));
        function.instruction(&Instruction::LocalSet(exponent));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(bit_count));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(significand));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(significand));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(significand));
        function.instruction(&Instruction::LocalGet(bit_count));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(bit_count));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(bit_count));
        function.instruction(&Instruction::I64Const(54));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(bit_index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(bit_index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(significand));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(bit));
        function.instruction(&Instruction::LocalGet(significand));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(significand));
        function.instruction(&Instruction::LocalGet(bit));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(division_remainder));
        function.instruction(&Instruction::LocalGet(high));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(low));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(significand));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(significand));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(significand));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(significand));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::LocalGet(exponent));
        function.instruction(&Instruction::I64Const(971)); // 1023 - 52
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(output_bits));
        for local in [
            exponent,
            bit_count,
            bit,
            division_remainder,
            significand,
            bit_index,
            high,
            low,
        ] {
            self.release_temp_local(local);
        }
    }
}
