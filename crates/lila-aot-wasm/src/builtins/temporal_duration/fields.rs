//! Integral Number fields and exact projections at the Duration arithmetic boundary.

use super::*;
use crate::gc_types::I64Local;

/// Wasm i64 locals containing canonical integral Number bits, never i64 field
/// values. There is deliberately no Index/Deref implementation: a caller must
/// name the representation before inspecting or writing a field.
pub(crate) struct TemporalDurationFields([I64Local; 10]);

impl TemporalDurationFields {
    pub(super) fn new(number_bits: [I64Local; 10]) -> Self {
        Self(number_bits)
    }

    pub(crate) fn number_bits(&self, unit: TemporalUnit) -> I64Local {
        self.0[unit.duration_field_index()]
    }

    pub(crate) fn number_bits_locals(&self) -> &[I64Local; 10] {
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
        number_bits: I64Local,
        function: &mut Function,
    ) {
        (number_bits).load(function);
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        (number_bits).store(function);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_temporal_duration_negate_fields(
        &mut self,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) {
        for local in fields.number_bits_locals() {
            (*local).load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            (*local).load(function);
            function.instruction(&Instruction::I64Const(i64::MIN));
            function.instruction(&Instruction::I64Xor);
            (*local).store(function);
            function.instruction(&Instruction::End);
        }
    }

    /// Calendar arithmetic consumes only the four bounded date fields. Wide
    /// time fields have no narrowing accessor and must use exact normalization.
    pub(crate) fn reserve_temporal_duration_date_field_locals(
        &mut self,
        fields: &TemporalDurationFields,
        function: &mut Function,
    ) -> [I64Local; 4] {
        std::array::from_fn(|index| {
            let local = self.runtime_schema().reserve_i64_local(function);
            (fields.number_bits(TemporalUnit::ALL[index])).load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            (local).store(function);
            local
        })
    }

    /// `Number` conversion happens once, before canonical validation/storage.
    pub(crate) fn emit_temporal_duration_set_integer_field(
        &mut self,
        fields: &TemporalDurationFields,
        unit: TemporalUnit,
        integer: I64Local,
        function: &mut Function,
    ) {
        (integer).load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        (fields.number_bits(unit)).store(function);
    }

    /// Divide the exact integer represented by a finite integral Number. The
    /// caller has bounded the field below 2^53 * divisor, so the quotient fits
    /// i64. Shifting the quotient/remainder pair avoids rounding a floating
    /// division or first narrowing a wide Number to i64.
    pub(super) fn emit_temporal_duration_number_divmod(
        &mut self,
        number_bits: I64Local,
        unit: TemporalDurationSubsecondUnit,
        quotient: I64Local,
        remainder: I64Local,
        function: &mut Function,
    ) {
        let divisor = unit.per_second();
        let magnitude = self.runtime_schema().reserve_i64_local(function);
        let significand = self.runtime_schema().reserve_i64_local(function);
        let shift = self.runtime_schema().reserve_i64_local(function);
        for local in [quotient, remainder] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        (number_bits).load(function);
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64And);
        magnitude.store(function);
        magnitude.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        (magnitude).load(function);
        function.instruction(&Instruction::I64Const((1_i64 << 52) - 1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(1_i64 << 52));
        function.instruction(&Instruction::I64Or);
        (significand).store(function);
        (magnitude).load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1075));
        function.instruction(&Instruction::I64Sub);
        (shift).store(function);
        (shift).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        (significand).load(function);
        function.instruction(&Instruction::I64Const(0));
        (shift).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        (significand).store(function);
        function.instruction(&Instruction::I64Const(0));
        (shift).store(function);
        function.instruction(&Instruction::End);
        for (destination, operation) in [
            (quotient, Instruction::I64DivU),
            (remainder, Instruction::I64RemU),
        ] {
            (significand).load(function);
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&operation);
            (destination).store(function);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        (shift).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        for local in [quotient, remainder] {
            (local).load(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Shl);
            (local).store(function);
        }
        (remainder).load(function);
        function.instruction(&Instruction::I64Const(divisor));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        (quotient).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (quotient).store(function);
        (remainder).load(function);
        function.instruction(&Instruction::I64Const(divisor));
        function.instruction(&Instruction::I64Sub);
        (remainder).store(function);
        function.instruction(&Instruction::End);
        (shift).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (shift).store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        (number_bits).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        for local in [quotient, remainder] {
            function.instruction(&Instruction::I64Const(0));
            (local).load(function);
            function.instruction(&Instruction::I64Sub);
            (local).store(function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(shift, function);
        self.runtime_schema()
            .release_i64_local(significand, function);
        self.runtime_schema().release_i64_local(magnitude, function);
    }

    /// Convert exact `(seconds * scale + remainder) / divisor` to Number with
    /// one rounding. Inputs are nonnegative, seconds are below 2^54, and the
    /// remainder is below the projection's scale. The closed scale is at most
    /// 10^9, so the product fits in 84 bits; the divisor is at most 10^6.
    /// Binary long division emits 53 significant bits, a guard bit, and exact
    /// sticky information. This also preserves fractional units in `total`.
    pub(crate) fn emit_temporal_duration_scaled_time_number(
        &mut self,
        seconds: I64Local,
        remainder: I64Local,
        projection: TemporalDurationNumberProjection,
        output_bits: I64Local,
        function: &mut Function,
    ) {
        let (scale, divisor) = projection.factors();
        let low = self.runtime_schema().reserve_i64_local(function);
        let high = self.runtime_schema().reserve_i64_local(function);
        let divisor_local = self.runtime_schema().reserve_i64_local(function);
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(scale));
        function.instruction(&Instruction::I64Mul);
        (remainder).load(function);
        function.instruction(&Instruction::I64Add);
        (low).store(function);
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(scale));
        function.instruction(&Instruction::I64Mul);
        (low).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        (high).store(function);
        (high).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        (low).load(function);
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        (low).store(function);
        (high).load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        (high).store(function);
        function.instruction(&Instruction::I64Const(divisor));
        (divisor_local).store(function);
        self.emit_u128_div_to_f64(high, low, divisor_local, output_bits, function);
        self.runtime_schema()
            .release_i64_local(divisor_local, function);
        self.runtime_schema().release_i64_local(high, function);
        self.runtime_schema().release_i64_local(low, function);
    }

    /// Binary long division of an unsigned 128-bit `(high, low)` numerator by
    /// a nonzero `u64` divisor, correctly rounded to one `f64`. Emits 53
    /// significant bits, a guard bit and exact sticky information; shared by
    /// `scaled_time_number` and the calendar-unit `total` projections, whose
    /// numerators and denominators both exceed `i64`.
    pub(crate) fn emit_u128_div_to_f64(
        &mut self,
        high: I64Local,
        low: I64Local,
        divisor_local: I64Local,
        output_bits: I64Local,
        function: &mut Function,
    ) {
        let bit_index = self.runtime_schema().reserve_i64_local(function);
        let significand = self.runtime_schema().reserve_i64_local(function);
        let division_remainder = self.runtime_schema().reserve_i64_local(function);
        let bit = self.runtime_schema().reserve_i64_local(function);
        let bit_count = self.runtime_schema().reserve_i64_local(function);
        let exponent = self.runtime_schema().reserve_i64_local(function);
        (high).load(function);
        (low).load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::F64)));
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::Else);
        for local in [significand, division_remainder, bit_count, exponent] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        function.instruction(&Instruction::I64Const(127));
        (bit_index).store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        (high).load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        (bit).store(function);
        (high).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (low).load(function);
        function.instruction(&Instruction::I64Const(63));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Or);
        (high).store(function);
        (low).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (low).store(function);
        (division_remainder).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (bit).load(function);
        function.instruction(&Instruction::I64Or);
        (division_remainder).store(function);
        (division_remainder).load(function);
        (divisor_local).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I64ExtendI32U);
        (bit).store(function);
        (bit).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        (division_remainder).load(function);
        (divisor_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (division_remainder).store(function);
        function.instruction(&Instruction::End);
        (bit_count).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        (bit).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        (bit_index).load(function);
        (exponent).store(function);
        function.instruction(&Instruction::I64Const(1));
        (bit_count).store(function);
        function.instruction(&Instruction::I64Const(1));
        (significand).store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Shl);
        (bit).load(function);
        function.instruction(&Instruction::I64Or);
        (significand).store(function);
        (bit_count).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (bit_count).store(function);
        function.instruction(&Instruction::End);
        (bit_count).load(function);
        function.instruction(&Instruction::I64Const(54));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        (bit_index).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (bit_index).store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        (bit).store(function);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        (significand).store(function);
        (bit).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        (division_remainder).load(function);
        (high).load(function);
        function.instruction(&Instruction::I64Or);
        (low).load(function);
        function.instruction(&Instruction::I64Or);
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        (significand).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (significand).store(function);
        function.instruction(&Instruction::End);
        (significand).load(function);
        function.instruction(&Instruction::F64ConvertI64U);
        (exponent).load(function);
        function.instruction(&Instruction::I64Const(971)); // 1023 - 52
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ReinterpretF64);
        (output_bits).store(function);
        for local in [
            exponent,
            bit_count,
            bit,
            division_remainder,
            significand,
            bit_index,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
