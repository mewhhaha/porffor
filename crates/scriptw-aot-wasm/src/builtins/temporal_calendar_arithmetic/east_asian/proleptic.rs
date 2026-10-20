//! Translation of the pinned ICU4X pingqi year model into real Wasm arithmetic.
//! ICU4X 2.1 revision 38a49da495248dd1ded84cf306e4ca42e64d5bb3,
//! retained at vendor/icu_calendar-2.0.6/src/cal/chinese_based/proleptic.rs.
//! Source portions copyright Unicode, Inc.; Unicode License v3 applies:
//! https://www.unicode.org/license.txt (vendor/icu_calendar-2.0.6/LICENSE).

use super::*;
use crate::data::{
    EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS, EAST_ASIAN_MEAN_SOLAR_TERM_MILLIS,
    EAST_ASIAN_MEAN_YEAR_MILLIS, EAST_ASIAN_MILLIS_PER_DAY,
};
use crate::gc_types::I64Local;

/// Canonical local moment: Unix epoch day plus milliseconds in [0, DAY). Every mutation
/// normalizes the pair. No full epoch-day times DAY multiplication occurs.
struct EastAsianMoment {
    day: I64Local,
    millis: I64Local,
}

impl EastAsianMoment {
    fn reserve(builder: &mut FunctionBuilder<'_>, function: &mut Function) -> Self {
        let result = Self {
            day: builder.runtime_schema().reserve_i64_local(function),
            millis: builder.runtime_schema().reserve_i64_local(function),
        };
        for local in [result.day, result.millis] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        result
    }
    fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        builder
            .runtime_schema()
            .release_i64_local(self.millis, function);
        builder
            .runtime_schema()
            .release_i64_local(self.day, function);
    }
}

impl FunctionBuilder<'_> {
    fn emit_temporal_east_asian_div_euclid(
        &self,
        numerator: I64Local,
        divisor: i64,
        out: I64Local,
        function: &mut Function,
    ) {
        (numerator).load(function);
        function.instruction(&Instruction::I64Const(divisor));
        function.instruction(&Instruction::I64DivS);
        (numerator).load(function);
        function.instruction(&Instruction::I64Const(divisor));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        (out).store(function);
    }

    fn emit_temporal_east_asian_moment_from_millis(
        &mut self,
        value: I64Local,
        out: &EastAsianMoment,
        function: &mut Function,
    ) {
        self.emit_temporal_east_asian_div_euclid(
            value,
            EAST_ASIAN_MILLIS_PER_DAY,
            out.day,
            function,
        );
        (value).load(function);
        (out.day).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (out.millis).store(function);
    }

    /// k=a*DAY+r. Only r*period is multiplied; r<DAY and period<2^35,
    /// so the product is below 2^62 even for negative or billion-year indices.
    fn emit_temporal_east_asian_moment_product(
        &mut self,
        index: I64Local,
        period: i64,
        out: &EastAsianMoment,
        function: &mut Function,
    ) {
        let quotient = self.runtime_schema().reserve_i64_local(function);
        let product = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_east_asian_div_euclid(
            index,
            EAST_ASIAN_MILLIS_PER_DAY,
            quotient,
            function,
        );
        (index).load(function);
        (quotient).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(period));
        function.instruction(&Instruction::I64Mul);
        (product).store(function);
        (quotient).load(function);
        function.instruction(&Instruction::I64Const(period));
        function.instruction(&Instruction::I64Mul);
        (product).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        (out.day).store(function);
        (product).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64RemS);
        (out.millis).store(function);
        self.runtime_schema().release_i64_local(product, function);
        self.runtime_schema().release_i64_local(quotient, function);
    }

    fn emit_temporal_east_asian_moment_add(
        &self,
        left: &EastAsianMoment,
        right: &EastAsianMoment,
        function: &mut Function,
    ) {
        (left.millis).load(function);
        (right.millis).load(function);
        function.instruction(&Instruction::I64Add);
        (left.millis).store(function);
        (left.day).load(function);
        (right.day).load(function);
        function.instruction(&Instruction::I64Add);
        (left.millis).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        (left.day).store(function);
        (left.millis).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64RemS);
        (left.millis).store(function);
    }

    fn emit_temporal_east_asian_moment_advance(
        &self,
        moment: &EastAsianMoment,
        period: i64,
        function: &mut Function,
    ) {
        (moment.day).load(function);
        function.instruction(&Instruction::I64Const(period / EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64Add);
        (moment.day).store(function);
        (moment.millis).load(function);
        function.instruction(&Instruction::I64Const(period % EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64Add);
        (moment.millis).store(function);
        (moment.millis).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        (moment.day).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (moment.day).store(function);
        (moment.millis).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64Sub);
        (moment.millis).store(function);
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_east_asian_moment_copy(
        &self,
        from: &EastAsianMoment,
        to: &EastAsianMoment,
        function: &mut Function,
    ) {
        for (source, destination) in [(from.day, to.day), (from.millis, to.millis)] {
            (source).load(function);
            (destination).store(function);
        }
    }

    /// d=a*P+r. floor((d*DAY+c)/P)=a*DAY+floor((r*DAY+c)/P).
    /// Here r<P, 0<=c<DAY and every period is below2^35: no large d*DAY.
    fn emit_temporal_east_asian_period_on_or_before(
        &mut self,
        date: I64Local,
        base_millis: I64Local,
        period: i64,
        moment: &EastAsianMoment,
        index: I64Local,
        function: &mut Function,
    ) {
        let base = EastAsianMoment::reserve(self, function);
        let days = self.runtime_schema().reserve_i64_local(function);
        let quotient = self.runtime_schema().reserve_i64_local(function);
        let residue = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_east_asian_moment_from_millis(base_millis, &base, function);
        (date).load(function);
        (base.day).load(function);
        function.instruction(&Instruction::I64Sub);
        (days).store(function);
        self.emit_temporal_east_asian_div_euclid(days, period, quotient, function);
        (days).load(function);
        (quotient).load(function);
        function.instruction(&Instruction::I64Const(period));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY - 1));
        function.instruction(&Instruction::I64Add);
        (base.millis).load(function);
        function.instruction(&Instruction::I64Sub);
        (residue).store(function);
        (quotient).load(function);
        function.instruction(&Instruction::I64Const(EAST_ASIAN_MILLIS_PER_DAY));
        function.instruction(&Instruction::I64Mul);
        (residue).load(function);
        function.instruction(&Instruction::I64Const(period));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        (index).store(function);
        self.emit_temporal_east_asian_moment_product(index, period, moment, function);
        self.emit_temporal_east_asian_moment_add(moment, &base, function);
        self.runtime_schema().release_i64_local(residue, function);
        self.runtime_schema().release_i64_local(quotient, function);
        self.runtime_schema().release_i64_local(days, function);
        base.release(self, function);
    }

    fn emit_temporal_east_asian_unconsumed_term_i32(
        &self,
        term: I64Local,
        limit: i64,
        next: &EastAsianMoment,
        major: &EastAsianMoment,
        leap: I64Local,
        function: &mut Function,
    ) {
        (term).load(function);
        function.instruction(&Instruction::I64Const(limit));
        function.instruction(&Instruction::I64LtS);
        self.emit_temporal_east_asian_leap_interval_i32(next, major, leap, function);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_temporal_east_asian_leap_interval_i32(
        &self,
        next: &EastAsianMoment,
        major: &EastAsianMoment,
        leap: I64Local,
        function: &mut Function,
    ) {
        (next.day).load(function);
        (major.day).load(function);
        function.instruction(&Instruction::I64LeS);
        (leap).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
    }

    pub(super) fn emit_temporal_east_asian_proleptic_year(
        &mut self,
        model: &EastAsianYearLocals,
        function: &mut Function,
    ) {
        let image = self
            .strings
            .temporal_east_asian_year_image()
            .calendar(model.kind);
        let first = image.first_related_year();
        let anchors = [
            (image.lower_solstice_millis(), image.upper_solstice_millis()),
            (
                image.lower_reference_moon_millis(),
                image.upper_reference_moon_millis(),
            ),
            (image.lower_serial_offset(), image.upper_serial_offset()),
        ];
        let january = self.runtime_schema().reserve_i64_local(function);
        let one = self.runtime_schema().reserve_i64_local(function);
        let solstice_base = self.runtime_schema().reserve_i64_local(function);
        let moon_base = self.runtime_schema().reserve_i64_local(function);
        let serial_offset = self.runtime_schema().reserve_i64_local(function);
        let term = self.runtime_schema().reserve_i64_local(function);
        let leap_in_sui = self.runtime_schema().reserve_i64_local(function);
        let preceding = self.runtime_schema().reserve_i64_local(function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let length = self.runtime_schema().reserve_i64_local(function);
        let major = EastAsianMoment::reserve(self, function);
        let moon = EastAsianMoment::reserve(self, function);
        let next = EastAsianMoment::reserve(self, function);
        for ((lower, upper), out) in
            anchors
                .into_iter()
                .zip([solstice_base, moon_base, serial_offset])
        {
            (model.year).load(function);
            function.instruction(&Instruction::I64Const(first));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(lower));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(upper));
            function.instruction(&Instruction::End);
            (out).store(function);
        }
        function.instruction(&Instruction::I64Const(1));
        (one).store(function);
        self.emit_temporal_plain_date_epoch_days(model.year, one, one, january, function);
        (january).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (one).store(function);
        self.emit_temporal_east_asian_period_on_or_before(
            one,
            solstice_base,
            EAST_ASIAN_MEAN_YEAR_MILLIS,
            &major,
            index,
            function,
        );
        self.emit_temporal_east_asian_period_on_or_before(
            major.day,
            moon_base,
            EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS,
            &moon,
            index,
            function,
        );
        self.emit_temporal_east_asian_moment_copy(&moon, &next, function);
        self.emit_temporal_east_asian_moment_advance(
            &next,
            EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS,
            function,
        );
        function.instruction(&Instruction::I64Const(-2));
        (term).store(function);
        for local in [leap_in_sui, preceding] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        self.emit_temporal_east_asian_unconsumed_term_i32(
            term,
            0,
            &next,
            &major,
            leap_in_sui,
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        (preceding).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (preceding).store(function);
        (preceding).load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_east_asian_leap_interval_i32(&next, &major, leap_in_sui, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (leap_in_sui).store(function);
        function.instruction(&Instruction::Else);
        (term).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (term).store(function);
        self.emit_temporal_east_asian_moment_advance(
            &major,
            EAST_ASIAN_MEAN_SOLAR_TERM_MILLIS,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_east_asian_moment_copy(&next, &moon, function);
        self.emit_temporal_east_asian_moment_advance(
            &next,
            EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS,
            function,
        );
        (index).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (index).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (term).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (moon.day).load(function);
        (model.start).store(function);
        (index).load(function);
        (serial_offset).load(function);
        function.instruction(&Instruction::I64Add);
        (model.first_month_serial).store(function);
        for local in [model.month_count, model.month_mask, model.leap_ordinal] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        self.emit_temporal_east_asian_unconsumed_term_i32(
            term,
            12,
            &next,
            &major,
            leap_in_sui,
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        (model.month_count).load(function);
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (model.month_count).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (model.month_count).store(function);
        (next.day).load(function);
        (moon.day).load(function);
        function.instruction(&Instruction::I64Sub);
        (length).store(function);
        (length).load(function);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Ne);
        (length).load(function);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (length).load(function);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (model.month_mask).load(function);
        function.instruction(&Instruction::I64Const(1));
        (model.month_count).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Or);
        (model.month_mask).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (next.day).load(function);
        (model.next_start).store(function);
        self.emit_temporal_east_asian_leap_interval_i32(&next, &major, leap_in_sui, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (leap_in_sui).store(function);
        (model.month_count).load(function);
        (model.leap_ordinal).store(function);
        function.instruction(&Instruction::Else);
        (term).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (term).store(function);
        self.emit_temporal_east_asian_moment_advance(
            &major,
            EAST_ASIAN_MEAN_SOLAR_TERM_MILLIS,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_east_asian_moment_copy(&next, &moon, function);
        self.emit_temporal_east_asian_moment_advance(
            &next,
            EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS,
            function,
        );
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (term).load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Ne);
        (model.month_count).load(function);
        (model.leap_ordinal).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        (model.leap_ordinal).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (model.start).load(function);
        (january).load(function);
        function.instruction(&Instruction::I64Sub);
        (length).store(function);
        (length).load(function);
        function.instruction(&Instruction::I64Const(18));
        function.instruction(&Instruction::I64LtS);
        (length).load(function);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        next.release(self, function);
        moon.release(self, function);
        major.release(self, function);
        for local in [
            length,
            index,
            preceding,
            leap_in_sui,
            term,
            serial_offset,
            moon_base,
            solstice_base,
            one,
            january,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_temporal_east_asian_moon_epoch(
        &mut self,
        kind: TemporalEastAsianCalendar,
        index: I64Local,
        lower: bool,
        out: I64Local,
        function: &mut Function,
    ) {
        let image = self.strings.temporal_east_asian_year_image().calendar(kind);
        let base_millis = if lower {
            image.lower_reference_moon_millis()
        } else {
            image.upper_reference_moon_millis()
        };
        let scalar = self.runtime_schema().reserve_i64_local(function);
        let base = EastAsianMoment::reserve(self, function);
        let moon = EastAsianMoment::reserve(self, function);
        function.instruction(&Instruction::I64Const(base_millis));
        (scalar).store(function);
        self.emit_temporal_east_asian_moment_from_millis(scalar, &base, function);
        self.emit_temporal_east_asian_moment_product(
            index,
            EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS,
            &moon,
            function,
        );
        self.emit_temporal_east_asian_moment_add(&moon, &base, function);
        (moon.day).load(function);
        (out).store(function);
        moon.release(self, function);
        base.release(self, function);
        self.runtime_schema().release_i64_local(scalar, function);
    }
}
