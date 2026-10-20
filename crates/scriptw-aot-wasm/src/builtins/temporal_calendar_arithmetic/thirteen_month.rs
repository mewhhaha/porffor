//! Integer arithmetic for the three closed fixed thirteen-month calendars.
//! Parent factories regulate fields and retain the completed carrier owners.

use super::super::temporal_plain_date::TemporalThirteenMonthCalendar;
use super::*;
use crate::gc_types::I64Local;

impl FunctionBuilder<'_> {
    /// Positive literal divisor; numerator and output may alias because the
    /// original remainder is read before the quotient is stored.
    fn emit_temporal_thirteen_month_div_euclid(
        &self,
        numerator: I64Local,
        denominator: i64,
        out: I64Local,
        function: &mut Function,
    ) {
        (numerator).load(function);
        function.instruction(&Instruction::I64Const(denominator));
        function.instruction(&Instruction::I64DivS);
        (numerator).load(function);
        function.instruction(&Instruction::I64Const(denominator));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        (out).store(function);
    }

    /// The pinned Coptic/Ethiopian rule is Euclidean year mod 4 == 3,
    /// equivalently (year + 1) mod 4 == 0, including negative years.
    fn emit_temporal_thirteen_month_leap_i32(
        &self,
        kind: TemporalThirteenMonthCalendar,
        year: I64Local,
        function: &mut Function,
    ) {
        match kind {
            TemporalThirteenMonthCalendar::Coptic
            | TemporalThirteenMonthCalendar::Ethiopic
            | TemporalThirteenMonthCalendar::Ethioaa => {
                (year).load(function);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64Const(4));
                function.instruction(&Instruction::I64RemS);
                function.instruction(&Instruction::I64Eqz);
            }
        }
    }

    pub(super) fn emit_temporal_thirteen_month_days_in_month(
        &mut self,
        kind: TemporalThirteenMonthCalendar,
        year: I64Local,
        month: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        (month).load(function);
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_temporal_thirteen_month_leap_i32(kind, year, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::End);
        (out).store(function);
    }

    /// calendrical_calculations 0.2.4 coptic::fixed_from_coptic and
    /// ethiopian::fixed_from_ethiopian, translated from RD to Unix epoch days.
    /// The closed kind supplies its epoch; callers never supply raw offsets.
    fn emit_temporal_thirteen_month_epoch_days(
        &mut self,
        kind: TemporalThirteenMonthCalendar,
        fields: [I64Local; 3],
        out: I64Local,
        function: &mut Function,
    ) {
        let leap_years = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_thirteen_month_div_euclid(fields[0], 4, leap_years, function);
        function.instruction(&Instruction::I64Const(kind.epoch_day()));
        (fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(365));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (leap_years).load(function);
        function.instruction(&Instruction::I64Add);
        (fields[1]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        self.runtime_schema()
            .release_i64_local(leap_years, function);
    }

    pub(super) fn emit_temporal_thirteen_month_to_iso(
        &mut self,
        kind: TemporalThirteenMonthCalendar,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_thirteen_month_epoch_days(kind, fields, epoch, function);
        self.emit_temporal_civil_from_days(epoch, fields[0], fields[1], fields[2], function);
        self.runtime_schema().release_i64_local(epoch, function);
    }

    /// Populate the existing projection. The parent alone computes the shared
    /// days_in_year footer from the same leap flag after all domains join.
    pub(super) fn emit_temporal_thirteen_month_project_date(
        &mut self,
        kind: TemporalThirteenMonthCalendar,
        iso: [I64Local; 3],
        result: &TemporalCalendarDateLocals,
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        let numerator = self.runtime_schema().reserve_i64_local(function);
        let one = self.runtime_schema().reserve_i64_local(function);
        let year_start = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_plain_date_epoch_days(iso[0], iso[1], iso[2], epoch, function);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(kind.epoch_day()));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(1463));
        function.instruction(&Instruction::I64Add);
        (numerator).store(function);
        self.emit_temporal_thirteen_month_div_euclid(numerator, 1461, result.fields[0], function);
        function.instruction(&Instruction::I64Const(1));
        (one).store(function);
        self.emit_temporal_thirteen_month_epoch_days(
            kind,
            [result.fields[0], one, one],
            year_start,
            function,
        );
        (epoch).load(function);
        (year_start).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.day_of_year).store(function);
        // The within-year ordinal is positive, so ordinary division by 30
        // equals the native Euclidean month computation here.
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.fields[1]).store(function);
        (result.day_of_year).load(function);
        (result.fields[1]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (result.fields[2]).store(function);
        self.emit_temporal_thirteen_month_leap_i32(kind, result.fields[0], function);
        function.instruction(&Instruction::I64ExtendI32U);
        (result.leap).store(function);
        self.emit_temporal_thirteen_month_days_in_month(
            kind,
            result.fields[0],
            result.fields[1],
            result.days_in_month,
            function,
        );
        for local in [year_start, one, numerator, epoch] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Use the latest valid match on or before ISO 1972-12-31. Month 13 day 6
    /// must choose the earlier leap reference before any forward conversion.
    pub(super) fn emit_temporal_thirteen_month_month_day_reference(
        &mut self,
        kind: TemporalThirteenMonthCalendar,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(kind.reference_year()));
        (fields[0]).store(function);
        (fields[1]).load(function);
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Eq);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(kind.leap_reference_year()));
        (fields[0]).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_thirteen_month_epoch_days(kind, fields, epoch, function);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(1095));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(kind.reference_year() - 1));
        (fields[0]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_thirteen_month_to_iso(kind, fields, function);
        self.runtime_schema().release_i64_local(epoch, function);
    }
}
