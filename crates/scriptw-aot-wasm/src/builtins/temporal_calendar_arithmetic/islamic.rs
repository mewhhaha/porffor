//! Integer Type-II tabular Islamic arithmetic over the existing ISO carrier.
//! The parent regulates fields and owns limits and completed publication.

use super::super::temporal_plain_date::TemporalIslamicCalendar;
use super::*;
use crate::gc_types::I64Local;

impl FunctionBuilder<'_> {
    /// Positive constant divisor; the remainder is read before storing out,
    /// so numerator and out may be the same local.
    fn emit_temporal_islamic_div_euclid(
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

    /// ICU's Type-II rule is rem_euclid(14 + 11 * year, 30) < 11,
    /// including signed arithmetic years before the Hijri epoch.
    fn emit_temporal_islamic_leap_i32(
        &mut self,
        kind: TemporalIslamicCalendar,
        year: I64Local,
        function: &mut Function,
    ) {
        let residue = self.runtime_schema().reserve_i64_local(function);
        match kind {
            TemporalIslamicCalendar::Civil | TemporalIslamicCalendar::Tbla => {
                (year).load(function);
                function.instruction(&Instruction::I64Const(11));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Const(14));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::I64Const(30));
                function.instruction(&Instruction::I64RemS);
                (residue).store(function);
                (residue).load(function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64LtS);
                self.open_frame(ControlFrameKind::If, function);
                (residue).load(function);
                function.instruction(&Instruction::I64Const(30));
                function.instruction(&Instruction::I64Add);
                (residue).store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                (residue).load(function);
                function.instruction(&Instruction::I64Const(11));
                function.instruction(&Instruction::I64LtS);
            }
        }
        self.runtime_schema().release_i64_local(residue, function);
    }

    pub(super) fn emit_temporal_islamic_days_in_month(
        &mut self,
        kind: TemporalIslamicCalendar,
        year: I64Local,
        month: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        (month).load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_temporal_islamic_leap_i32(kind, year, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        (month).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        (out).store(function);
    }

    /// Exact calendrical_calculations 0.2.4 forward rule, with the closed
    /// Friday/Thursday epoch already expressed as Unix epoch days.
    fn emit_temporal_islamic_epoch_days(
        &mut self,
        kind: TemporalIslamicCalendar,
        fields: [I64Local; 3],
        out: I64Local,
        function: &mut Function,
    ) {
        let leap_days = self.runtime_schema().reserve_i64_local(function);
        (fields[0]).load(function);
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        (leap_days).store(function);
        self.emit_temporal_islamic_div_euclid(leap_days, 30, leap_days, function);
        function.instruction(&Instruction::I64Const(kind.epoch_day()));
        (fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(354));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (leap_days).load(function);
        function.instruction(&Instruction::I64Add);
        (fields[1]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        // Regulated month 1..12 is positive, so truncation equals floor here.
        (fields[1]).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Add);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        self.runtime_schema().release_i64_local(leap_days, function);
    }

    pub(super) fn emit_temporal_islamic_to_iso(
        &mut self,
        kind: TemporalIslamicCalendar,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_islamic_epoch_days(kind, fields, epoch, function);
        self.emit_temporal_civil_from_days(epoch, fields[0], fields[1], fields[2], function);
        self.runtime_schema().release_i64_local(epoch, function);
    }

    /// The exact year inverse is floor((30*(date-epoch)+10646)/10631).
    /// At a year start its numerator is 10631*year+29-r, and at the final
    /// day it is 10631*(year+1)-1-r', with both residues in 0..29.
    /// The parent alone fills days_in_year from 354 plus this leap flag.
    pub(super) fn emit_temporal_islamic_project_date(
        &mut self,
        kind: TemporalIslamicCalendar,
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
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(10646));
        function.instruction(&Instruction::I64Add);
        (numerator).store(function);
        self.emit_temporal_islamic_div_euclid(numerator, 10631, result.fields[0], function);
        function.instruction(&Instruction::I64Const(1));
        (one).store(function);
        self.emit_temporal_islamic_epoch_days(
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
        // Pinned month extraction uses priorDays 0..354, so this division
        // is positive: floor((11*priorDays+330)/325) gives month 1..12.
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(330));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(325));
        function.instruction(&Instruction::I64DivS);
        (result.fields[1]).store(function);
        (result.day_of_year).load(function);
        (result.fields[1]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (result.fields[1]).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Sub);
        (result.fields[2]).store(function);
        self.emit_temporal_islamic_leap_i32(kind, result.fields[0], function);
        function.instruction(&Instruction::I64ExtendI32U);
        (result.leap).store(function);
        self.emit_temporal_islamic_days_in_month(
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

    /// Choose the latest valid match on or before ISO 1972-12-31. A leap
    /// month 12 day 30 selects 1390 before any conversion through a common year.
    pub(super) fn emit_temporal_islamic_month_day_reference(
        &mut self,
        kind: TemporalIslamicCalendar,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(kind.reference_year()));
        (fields[0]).store(function);
        (fields[1]).load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Eq);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(kind.leap_reference_year()));
        (fields[0]).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_islamic_epoch_days(kind, fields, epoch, function);
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
        self.emit_temporal_islamic_to_iso(kind, fields, function);
        self.runtime_schema().release_i64_local(epoch, function);
    }
}
