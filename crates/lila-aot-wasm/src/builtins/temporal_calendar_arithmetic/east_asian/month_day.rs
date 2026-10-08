//! The complete Stage 4 Table 6 reference policy, consumed by both actual
//! MonthDay factories. Native year metadata remains with the checked model.

use super::*;
use crate::gc_types::I64Local;

enum ReferenceYear {
    Unavailable,
    Fixed(i64),
    EleventhLeapByDay,
}

/// https://tc39.es/proposal-intl-era-monthcode/#sec-nonisomonthdaytoisoreferencedate
/// The first column describes days1..29; the second describes day30. Calendar
/// choice affects only the M03 day30 row. Storage encodings are never ordinals.
fn reference_years(
    kind: TemporalEastAsianCalendar,
    code: TemporalCalendarMonthCode,
) -> (ReferenceYear, Option<i64>) {
    use ReferenceYear::{EleventhLeapByDay, Fixed, Unavailable};
    match code {
        TemporalCalendarMonthCode::M01 => (Fixed(1972), Some(1970)),
        TemporalCalendarMonthCode::M02 => (Fixed(1972), Some(1972)),
        TemporalCalendarMonthCode::M03 => (
            Fixed(1972),
            Some(match kind {
                TemporalEastAsianCalendar::Chinese => 1966,
                TemporalEastAsianCalendar::Dangi => 1968,
            }),
        ),
        TemporalCalendarMonthCode::M04 => (Fixed(1972), Some(1970)),
        TemporalCalendarMonthCode::M05 => (Fixed(1972), Some(1972)),
        TemporalCalendarMonthCode::M06 => (Fixed(1972), Some(1971)),
        TemporalCalendarMonthCode::M07 => (Fixed(1972), Some(1972)),
        TemporalCalendarMonthCode::M08 => (Fixed(1972), Some(1971)),
        TemporalCalendarMonthCode::M09 => (Fixed(1972), Some(1972)),
        TemporalCalendarMonthCode::M10 => (Fixed(1972), Some(1972)),
        TemporalCalendarMonthCode::M11 => (Fixed(1972), Some(1970)),
        TemporalCalendarMonthCode::M12 => (Fixed(1972), Some(1972)),
        TemporalCalendarMonthCode::M13 => (Unavailable, None),
        TemporalCalendarMonthCode::M01L => (Unavailable, None),
        TemporalCalendarMonthCode::M02L => (Fixed(1947), None),
        TemporalCalendarMonthCode::M03L => (Fixed(1966), Some(1955)),
        TemporalCalendarMonthCode::M04L => (Fixed(1963), Some(1944)),
        TemporalCalendarMonthCode::M05L => (Fixed(1971), Some(1952)),
        TemporalCalendarMonthCode::M06L => (Fixed(1960), Some(1941)),
        TemporalCalendarMonthCode::M07L => (Fixed(1968), Some(1938)),
        TemporalCalendarMonthCode::M08L => (Fixed(1957), None),
        TemporalCalendarMonthCode::M09L => (Fixed(2014), None),
        TemporalCalendarMonthCode::M10L => (Fixed(1984), None),
        TemporalCalendarMonthCode::M11L => (EleventhLeapByDay, None),
        TemporalCalendarMonthCode::M12L => (Unavailable, None),
    }
}

impl FunctionBuilder<'_> {
    fn emit_temporal_east_asian_reference_iso_year(
        &self,
        kind: TemporalEastAsianCalendar,
        code: I64Local,
        day: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        (out).store(function);
        for candidate in TemporalCalendarMonthCode::ALL {
            let (ordinary, day30) = reference_years(kind, candidate);
            (code).load(function);
            function.instruction(&Instruction::I64Const(candidate.encoding()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            (day).load(function);
            function.instruction(&Instruction::I64Const(30));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(day30.unwrap_or(0)));
            function.instruction(&Instruction::Else);
            match ordinary {
                ReferenceYear::Unavailable => {
                    function.instruction(&Instruction::I64Const(0));
                }
                ReferenceYear::Fixed(year) => {
                    function.instruction(&Instruction::I64Const(year));
                }
                ReferenceYear::EleventhLeapByDay => {
                    (day).load(function);
                    function.instruction(&Instruction::I64Const(10));
                    function.instruction(&Instruction::I64LeS);
                    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                    function.instruction(&Instruction::I64Const(2033));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(2034));
                    function.instruction(&Instruction::End);
                }
            }
            function.instruction(&Instruction::End);
            (out).store(function);
            function.instruction(&Instruction::End);
        }
    }

    /// Inputs already passed requested day overflow. Reference availability is
    /// a separate phase: Constrain changes only an unavailable leap code to its
    /// regular counterpart, then uses the corresponding complete table row.
    pub(in crate::builtins::temporal_calendar_arithmetic) fn emit_temporal_east_asian_month_day_reference(
        &mut self,
        kind: TemporalEastAsianCalendar,
        code: I64Local,
        day: I64Local,
        out: [I64Local; 3],
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let effective_code = self.runtime_schema().reserve_i64_local(function);
        let reference = self.runtime_schema().reserve_i64_local(function);
        let requested_day = self.runtime_schema().reserve_i64_local(function);
        let year = self.runtime_schema().reserve_i64_local(function);
        let ordinal = self.runtime_schema().reserve_i64_local(function);
        let maximum = self.runtime_schema().reserve_i64_local(function);
        let candidate: [I64Local; 3] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let epoch = self.runtime_schema().reserve_i64_local(function);
        let best_epoch = self.runtime_schema().reserve_i64_local(function);
        let found = self.runtime_schema().reserve_i64_local(function);
        // out may alias day. Snapshot before any candidate can publish fields.
        (day).load(function);
        (requested_day).store(function);
        (code).load(function);
        (effective_code).store(function);
        self.emit_temporal_east_asian_reference_iso_year(
            kind,
            effective_code,
            requested_day,
            reference,
            function,
        );
        (reference).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (overflow).load(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_MONTHCODE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // The closed code declaration owns the regular encoding as well as
        // spelling. Do not infer a regular code from a leap storage encoding.
        for candidate_code in TemporalCalendarMonthCode::ALL {
            let regular = TemporalCalendarMonthCode::ALL
                .into_iter()
                .find(|candidate| {
                    !candidate.is_leap()
                        && candidate.month_number() == candidate_code.month_number()
                })
                .expect("every closed MonthCode has a regular counterpart");
            (effective_code).load(function);
            function.instruction(&Instruction::I64Const(candidate_code.encoding()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(regular.encoding()));
            (effective_code).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_east_asian_reference_iso_year(
            kind,
            effective_code,
            requested_day,
            reference,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        (found).store(function);
        function.instruction(&Instruction::I64Const(0));
        (best_epoch).store(function);
        // Related Gregorian years agree with the ISO year of their New Year.
        // Thus only reference-1/reference can contain the selected ISO year.
        for delta in [-1, 0] {
            (reference).load(function);
            function.instruction(&Instruction::I64Const(delta));
            function.instruction(&Instruction::I64Add);
            (year).store(function);
            let model = self.emit_temporal_east_asian_year_model(kind, year, function);
            self.emit_temporal_east_asian_model_code_present_i32(&model, effective_code, function);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_model_code_ordinal(
                &model,
                effective_code,
                ordinal,
                function,
            );
            self.emit_temporal_east_asian_model_days_in_month(&model, ordinal, maximum, function);
            (requested_day).load(function);
            (maximum).load(function);
            function.instruction(&Instruction::I64LeS);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_model_epoch_days(
                &model,
                ordinal,
                requested_day,
                epoch,
                function,
            );
            self.emit_temporal_civil_from_days(
                epoch,
                candidate[0],
                candidate[1],
                candidate[2],
                function,
            );
            (candidate[0]).load(function);
            (reference).load(function);
            function.instruction(&Instruction::I64Eq);
            (found).load(function);
            function.instruction(&Instruction::I64Eqz);
            (epoch).load(function);
            (best_epoch).load(function);
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            (epoch).load(function);
            (best_epoch).store(function);
            function.instruction(&Instruction::I64Const(1));
            (found).store(function);
            for (source, destination) in candidate.into_iter().zip(out) {
                (source).load(function);
                (destination).store(function);
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            model.release(self, function);
        }
        // A malformed data/model join cannot publish an unrelated ISO date.
        (found).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_MONTHCODE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in [found, best_epoch, epoch] {
            self.runtime_schema().release_i64_local(local, function);
        }
        for local in candidate.into_iter().rev() {
            self.runtime_schema().release_i64_local(local, function);
        }
        for local in [
            maximum,
            ordinal,
            year,
            requested_day,
            reference,
            effective_code,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }
}
