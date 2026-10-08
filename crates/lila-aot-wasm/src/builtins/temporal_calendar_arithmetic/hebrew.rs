//! Integer Hebrew new-year and canonical civil-month arithmetic. The parent
//! owns field envelopes, requested overflow and completed ISO publication.

use super::super::temporal_plain_date::{
    TemporalCalendarMonthCode, TEMPORAL_PLAIN_DATE_MAXIMUM_EPOCH_DAY,
    TEMPORAL_PLAIN_DATE_MINIMUM_EPOCH_DAY,
};
use super::*;
use crate::gc_types::I64Local;

// calendrical_calculations0.2.4's book algorithm uses RD -1373427. Unix
// epoch days subtract RD 719163; this convention includes its postponements.
const HEBREW_EPOCH_DAY: i64 = -2092590;
// ISO 1972-12-31 lies in Hebrew5733. Reference selection below checks the
// entire converted candidate; it never accepts this year merely by number.
const HEBREW_REFERENCE_YEAR: i64 = 5733;

impl FunctionBuilder<'_> {
    fn emit_temporal_hebrew_div_euclid(
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

    /// Field agreement precedes the native-year envelope. Reducing year first
    /// makes this rule safe even for an unbounded supplied i64 year.
    fn emit_temporal_hebrew_leap_i32(&mut self, year: I64Local, function: &mut Function) {
        let residue = self.runtime_schema().reserve_i64_local(function);
        (year).load(function);
        function.instruction(&Instruction::I64Const(19));
        function.instruction(&Instruction::I64RemS);
        (residue).store(function);
        (residue).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (residue).load(function);
        function.instruction(&Instruction::I64Const(19));
        function.instruction(&Instruction::I64Add);
        (residue).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (residue).load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(19));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64LtS);
        self.runtime_schema().release_i64_local(residue, function);
    }

    pub(in crate::builtins) fn emit_temporal_hebrew_months_in_year(
        &mut self,
        year: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        self.emit_temporal_hebrew_leap_i32(year, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Add);
        (out).store(function);
    }

    /// E(y) is only called after a native field envelope or the existing
    /// validated date-duration/component bounds have proved the multiplication.
    fn emit_temporal_hebrew_elapsed_months(
        &self,
        year: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        (year).load(function);
        function.instruction(&Instruction::I64Const(235));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(234));
        function.instruction(&Instruction::I64Sub);
        (out).store(function);
        self.emit_temporal_hebrew_div_euclid(out, 19, out, function);
    }

    pub(super) fn emit_temporal_hebrew_month_serial(
        &self,
        year: I64Local,
        ordinal: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        // Preserve ordinal when the requested output aliases it.
        (ordinal).load(function);
        self.emit_temporal_hebrew_elapsed_months(year, out, function);
        (out).load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (out).store(function);
    }

    /// Exact inverse of E(y)+ordinal-1, including negative serials. Keep the
    /// original serial live while overwriting either supplied year/month local.
    pub(super) fn emit_temporal_hebrew_year_month_from_serial(
        &mut self,
        serial: I64Local,
        year_out: I64Local,
        ordinal_out: I64Local,
        function: &mut Function,
    ) {
        let original = self.runtime_schema().reserve_i64_local(function);
        let elapsed = self.runtime_schema().reserve_i64_local(function);
        (serial).load(function);
        (original).store(function);
        (original).load(function);
        function.instruction(&Instruction::I64Const(19));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(252));
        function.instruction(&Instruction::I64Add);
        (elapsed).store(function);
        self.emit_temporal_hebrew_div_euclid(elapsed, 235, year_out, function);
        self.emit_temporal_hebrew_elapsed_months(year_out, elapsed, function);
        (original).load(function);
        (elapsed).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (ordinal_out).store(function);
        self.runtime_schema().release_i64_local(elapsed, function);
        self.runtime_schema().release_i64_local(original, function);
    }

    pub(in crate::builtins) fn emit_temporal_hebrew_month_code(
        &mut self,
        year: I64Local,
        ordinal: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        self.emit_temporal_hebrew_leap_i32(year, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (ordinal).load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(
            TemporalCalendarMonthCode::M05L.encoding(),
        ));
        function.instruction(&Instruction::Else);
        (ordinal).load(function);
        (ordinal).load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        (ordinal).load(function);
        function.instruction(&Instruction::End);
        (out).store(function);
    }

    /// Supplied-year agreement always constrains missing M05L to M06. The
    /// original code is retained by the caller for requested overflow later.
    pub(in crate::builtins) fn emit_temporal_hebrew_month_code_ordinal(
        &mut self,
        year: I64Local,
        code: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        (code).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalCalendarMonthCode::M05L.encoding(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::Else);
        (code).load(function);
        (code).load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64GeS);
        self.emit_temporal_hebrew_leap_i32(year, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        (out).store(function);
    }

    pub(super) fn emit_temporal_hebrew_month_code_present(
        &mut self,
        year: I64Local,
        code: I64Local,
        function: &mut Function,
    ) {
        (code).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalCalendarMonthCode::M05L.encoding(),
        ));
        function.instruction(&Instruction::I64Ne);
        self.emit_temporal_hebrew_leap_i32(year, function);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_temporal_hebrew_elapsed_days(
        &mut self,
        year: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        let months = self.runtime_schema().reserve_i64_local(function);
        let parts = self.runtime_schema().reserve_i64_local(function);
        let residue = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_hebrew_elapsed_months(year, months, function);
        (months).load(function);
        function.instruction(&Instruction::I64Const(13753));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(12084));
        function.instruction(&Instruction::I64Add);
        (parts).store(function);
        self.emit_temporal_hebrew_div_euclid(parts, 25920, parts, function);
        (months).load(function);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Mul);
        (parts).load(function);
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        (out).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64RemS);
        (residue).store(function);
        (residue).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (residue).load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        (residue).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (residue).load(function);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        (out).load(function);
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        for local in [residue, parts, months] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_temporal_hebrew_new_year(
        &mut self,
        year: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        let adjacent_year = self.runtime_schema().reserve_i64_local(function);
        let previous = self.runtime_schema().reserve_i64_local(function);
        let current = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        (year).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (adjacent_year).store(function);
        self.emit_temporal_hebrew_elapsed_days(adjacent_year, previous, function);
        self.emit_temporal_hebrew_elapsed_days(year, current, function);
        (year).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (adjacent_year).store(function);
        self.emit_temporal_hebrew_elapsed_days(adjacent_year, next, function);
        (next).load(function);
        (current).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(356));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::Else);
        (current).load(function);
        (previous).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(382));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::End);
        (current).load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(HEBREW_EPOCH_DAY));
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        for local in [next, current, previous, adjacent_year] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    fn emit_temporal_hebrew_year_length(
        &mut self,
        year: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        let start = self.runtime_schema().reserve_i64_local(function);
        let next_year = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_hebrew_new_year(year, start, function);
        (year).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (next_year).store(function);
        self.emit_temporal_hebrew_new_year(next_year, out, function);
        (out).load(function);
        (start).load(function);
        function.instruction(&Instruction::I64Sub);
        (out).store(function);
        self.runtime_schema().release_i64_local(next_year, function);
        self.runtime_schema().release_i64_local(start, function);
    }

    /// MonthDay admits a supplied native year if any day in that year
    /// intersects the full ISO date carrier, before looking up its month.
    /// The parent has already proved the native-year arithmetic envelope.
    pub(super) fn emit_temporal_hebrew_year_intersects_iso_limits_i32(
        &mut self,
        year: I64Local,
        function: &mut Function,
    ) {
        let next_year = self.runtime_schema().reserve_i64_local(function);
        let start = self.runtime_schema().reserve_i64_local(function);
        let next_start = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_hebrew_new_year(year, start, function);
        (year).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (next_year).store(function);
        self.emit_temporal_hebrew_new_year(next_year, next_start, function);
        (start).load(function);
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_DATE_MAXIMUM_EPOCH_DAY,
        ));
        function.instruction(&Instruction::I64LeS);
        (next_start).load(function);
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_DATE_MINIMUM_EPOCH_DAY,
        ));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        for local in [next_start, start, next_year] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Calendar-year length is already computed for this ordinal traversal.
    fn emit_temporal_hebrew_month_length_for_year(
        &mut self,
        year: I64Local,
        ordinal: I64Local,
        year_length: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        let code = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_hebrew_month_code(year, ordinal, code, function);
        (code).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (year_length).load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        (code).load(function);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (year_length).load(function);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::Else);
        (code).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalCalendarMonthCode::M05L.encoding(),
        ));
        function.instruction(&Instruction::I64Eq);
        (code).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        (out).store(function);
        self.runtime_schema().release_i64_local(code, function);
    }

    pub(super) fn emit_temporal_hebrew_days_in_month(
        &mut self,
        year: I64Local,
        ordinal: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        let length = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_hebrew_year_length(year, length, function);
        self.emit_temporal_hebrew_month_length_for_year(year, ordinal, length, out, function);
        self.runtime_schema().release_i64_local(length, function);
    }

    fn emit_temporal_hebrew_epoch_days(
        &mut self,
        fields: [I64Local; 3],
        out: I64Local,
        function: &mut Function,
    ) {
        let ordinal = self.runtime_schema().reserve_i64_local(function);
        let year_length = self.runtime_schema().reserve_i64_local(function);
        let month_length = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_hebrew_new_year(fields[0], out, function);
        self.emit_temporal_hebrew_year_length(fields[0], year_length, function);
        (out).load(function);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (out).store(function);
        function.instruction(&Instruction::I64Const(1));
        (ordinal).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (ordinal).load(function);
        (fields[1]).load(function);
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_hebrew_month_length_for_year(
            fields[0],
            ordinal,
            year_length,
            month_length,
            function,
        );
        (out).load(function);
        (month_length).load(function);
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        (ordinal).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (ordinal).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        for local in [month_length, year_length, ordinal] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_temporal_hebrew_to_iso(
        &mut self,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_hebrew_epoch_days(fields, epoch, function);
        self.emit_temporal_civil_from_days(epoch, fields[0], fields[1], fields[2], function);
        self.runtime_schema().release_i64_local(epoch, function);
    }

    pub(super) fn emit_temporal_hebrew_project_date(
        &mut self,
        iso: [I64Local; 3],
        result: &TemporalCalendarDateLocals,
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        let numerator = self.runtime_schema().reserve_i64_local(function);
        let start = self.runtime_schema().reserve_i64_local(function);
        let next_year = self.runtime_schema().reserve_i64_local(function);
        let next_start = self.runtime_schema().reserve_i64_local(function);
        let remaining = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_plain_date_epoch_days(iso[0], iso[1], iso[2], epoch, function);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(HEBREW_EPOCH_DAY));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(98496));
        function.instruction(&Instruction::I64Mul);
        (numerator).store(function);
        self.emit_temporal_hebrew_div_euclid(numerator, 35975351, result.fields[0], function);
        (result.fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.fields[0]).store(function);
        // Correct the native estimate against exact starts in both directions.
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        self.emit_temporal_hebrew_new_year(result.fields[0], start, function);
        (start).load(function);
        (epoch).load(function);
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::BrIf(1));
        (result.fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (result.fields[0]).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (result.fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (next_year).store(function);
        self.emit_temporal_hebrew_new_year(next_year, next_start, function);
        (next_start).load(function);
        (epoch).load(function);
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::BrIf(1));
        (next_year).load(function);
        (result.fields[0]).store(function);
        (next_start).load(function);
        (start).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (next_start).load(function);
        (start).load(function);
        function.instruction(&Instruction::I64Sub);
        (result.days_in_year).store(function);
        self.emit_temporal_hebrew_leap_i32(result.fields[0], function);
        function.instruction(&Instruction::I64ExtendI32U);
        (result.leap).store(function);
        self.emit_temporal_hebrew_months_in_year(result.fields[0], result.months_in_year, function);
        (epoch).load(function);
        (start).load(function);
        function.instruction(&Instruction::I64Sub);
        (remaining).store(function);
        (remaining).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.day_of_year).store(function);
        function.instruction(&Instruction::I64Const(1));
        (result.fields[1]).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        self.emit_temporal_hebrew_month_length_for_year(
            result.fields[0],
            result.fields[1],
            result.days_in_year,
            result.days_in_month,
            function,
        );
        (remaining).load(function);
        (result.days_in_month).load(function);
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(1));
        (remaining).load(function);
        (result.days_in_month).load(function);
        function.instruction(&Instruction::I64Sub);
        (remaining).store(function);
        (result.fields[1]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.fields[1]).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (remaining).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.fields[2]).store(function);
        for local in [remaining, next_start, next_year, start, numerator, epoch] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_temporal_hebrew_month_day_max_days(
        &self,
        code: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        // M02 and M03 each reach30 in long years; M05L is always30.
        (code).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        for special in [
            TemporalCalendarMonthCode::M02,
            TemporalCalendarMonthCode::M05L,
        ] {
            (code).load(function);
            function.instruction(&Instruction::I64Const(special.encoding()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Add);
        (out).store(function);
    }

    pub(super) fn emit_temporal_hebrew_missing_year_regulation_year(
        &mut self,
        code: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        let original_code = self.runtime_schema().reserve_i64_local(function);
        let ordinal = self.runtime_schema().reserve_i64_local(function);
        let maximum = self.runtime_schema().reserve_i64_local(function);
        let length = self.runtime_schema().reserve_i64_local(function);
        (code).load(function);
        (original_code).store(function);
        self.emit_temporal_hebrew_month_day_max_days(original_code, maximum, function);
        function.instruction(&Instruction::I64Const(HEBREW_REFERENCE_YEAR));
        (out).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        self.emit_temporal_hebrew_month_code_present(out, original_code, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_month_code_ordinal(out, original_code, ordinal, function);
        self.emit_temporal_hebrew_days_in_month(out, ordinal, length, function);
        (length).load(function);
        (maximum).load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(2));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (out).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (out).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        for local in [length, maximum, ordinal, original_code] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Select the latest FULL eligible date, preserving code across reference
    /// years. Regulated day<=the code maximum guarantees a nearby match.
    pub(super) fn emit_temporal_hebrew_month_day_reference(
        &mut self,
        fields: [I64Local; 3],
        code: I64Local,
        function: &mut Function,
    ) {
        let original_code = self.runtime_schema().reserve_i64_local(function);
        let length = self.runtime_schema().reserve_i64_local(function);
        let epoch = self.runtime_schema().reserve_i64_local(function);
        (code).load(function);
        (original_code).store(function);
        function.instruction(&Instruction::I64Const(HEBREW_REFERENCE_YEAR));
        (fields[0]).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        self.emit_temporal_hebrew_month_code_present(fields[0], original_code, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_month_code_ordinal(fields[0], original_code, fields[1], function);
        self.emit_temporal_hebrew_days_in_month(fields[0], fields[1], length, function);
        (fields[2]).load(function);
        (length).load(function);
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_epoch_days(fields, epoch, function);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(1095));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::BrIf(3));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (fields[0]).store(function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.emit_temporal_civil_from_days(epoch, fields[0], fields[1], fields[2], function);
        self.runtime_schema().release_i64_local(epoch, function);
        self.runtime_schema().release_i64_local(length, function);
        self.runtime_schema()
            .release_i64_local(original_code, function);
    }
}
