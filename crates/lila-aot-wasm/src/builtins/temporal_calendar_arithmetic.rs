//! Calendar arithmetic over the existing ISO carriers. No JavaScript is read
//! here: callers retain their canonical calendar and completed field sweep.

mod east_asian;
mod hebrew;
mod islamic;
mod persian;
mod shared_arithmetic;
mod thirteen_month;
mod umalqura;
mod umalqura_data;

use crate::gc_types::*;

use super::super::*;
use super::temporal_options::{TemporalOverflow, TemporalUnit};
use super::temporal_plain_date::{
    MonthDayYearUse, TemporalCalendarArithmetic, TemporalCalendarId,
    TemporalCalendarMonthArithmetic, TemporalCalendarMonthCode, TemporalCalendarYearLength,
    TemporalIslamicCalendar, TemporalResolvedCalendarMonth, TemporalResolvedCalendarYear,
    TemporalThirteenMonthCalendar, TEMPORAL_PLAIN_DATE_MAXIMUM_EPOCH_DAY,
    TEMPORAL_PLAIN_DATE_MINIMUM_EPOCH_DAY,
};
use super::temporal_plain_year_month::TemporalPartialDateType;
use super::temporal_zone_provider::TemporalCalendarSlotLocals;
use crate::data::TEMPORAL_EAST_ASIAN_YEAR_ENVELOPE;
use crate::runtime_helpers::{
    CalendarDateCallResult, CalendarIsoDateCallResult, TemporalCalendarFieldsToIsoArguments,
    TemporalCalendarFieldsToIsoParameters, TemporalCalendarProjectDateArguments,
    TemporalCalendarProjectDateParameters,
};

/// One projection of the complete retained ISO date. All calendar-dependent
/// getters and receiver defaults borrow these same locals before release.
pub(super) struct TemporalCalendarDateLocals {
    calendar: I64Local,
    fields: [I64Local; 3],
    days_in_month: I64Local,
    day_of_year: I64Local,
    days_in_year: I64Local,
    leap: I64Local,
    months_in_year: I64Local,
}

impl TemporalCalendarDateLocals {
    fn reserve(
        builder: &mut FunctionBuilder<'_>,
        calendar: I64Local,
        function: &mut Function,
    ) -> Self {
        Self {
            calendar,
            fields: std::array::from_fn(|_| builder.runtime_schema().reserve_i64_local(function)),
            days_in_month: builder.runtime_schema().reserve_i64_local(function),
            day_of_year: builder.runtime_schema().reserve_i64_local(function),
            days_in_year: builder.runtime_schema().reserve_i64_local(function),
            leap: builder.runtime_schema().reserve_i64_local(function),
            months_in_year: builder.runtime_schema().reserve_i64_local(function),
        }
    }

    fn result_locals(&self) -> [I64Local; 8] {
        [
            self.fields[0],
            self.fields[1],
            self.fields[2],
            self.days_in_month,
            self.day_of_year,
            self.days_in_year,
            self.leap,
            self.months_in_year,
        ]
    }

    pub(super) fn calendar_id(&self) -> I64Local {
        self.calendar
    }
    pub(super) fn months_in_year(&self) -> I64Local {
        self.months_in_year
    }
    pub(super) fn fields(&self) -> [I64Local; 3] {
        self.fields
    }
    pub(super) fn year(&self) -> I64Local {
        self.fields[0]
    }
    pub(super) fn month(&self) -> I64Local {
        self.fields[1]
    }
    pub(super) fn day(&self) -> I64Local {
        self.fields[2]
    }
    pub(super) fn days_in_month(&self) -> I64Local {
        self.days_in_month
    }
    pub(super) fn day_of_year(&self) -> I64Local {
        self.day_of_year
    }
    pub(super) fn days_in_year(&self) -> I64Local {
        self.days_in_year
    }
    pub(super) fn leap(&self) -> I64Local {
        self.leap
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in self.result_locals().into_iter().rev() {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl CalendarDateCallResult {
    fn store(self, result: &TemporalCalendarDateLocals, function: &mut Function) {
        for local in result.result_locals().into_iter().rev() {
            local.store(function);
        }
    }
}

impl CalendarIsoDateCallResult {
    fn store(self, fields: [I64Local; 3], function: &mut Function) {
        for local in fields.into_iter().rev() {
            local.store(function);
        }
    }
}

/// A calendar-derived reference ISO date and its matching partial-date kind.
/// The only factories below complete conversion before publication.
pub(super) struct CompletedTemporalPartialReferenceLocals {
    calendar: I64Local,
    kind: TemporalPartialDateType,
    fields: [I64Local; 3],
}

impl CompletedTemporalPartialReferenceLocals {
    pub(super) fn calendar_id(&self) -> I64Local {
        self.calendar
    }
    pub(super) fn kind(&self) -> TemporalPartialDateType {
        self.kind
    }
    pub(super) fn fields(&self) -> [I64Local; 3] {
        self.fields
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in self.fields.into_iter().rev() {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl FunctionBuilder<'_> {
    fn emit_temporal_calendar_year_to_iso(
        &self,
        calendar: I64Local,
        year: I64Local,
        function: &mut Function,
    ) {
        for id in TemporalCalendarId::ALL {
            match id.arithmetic() {
                TemporalCalendarArithmetic::ProlepticGregorian { year_offset } => {
                    if year_offset == 0 {
                        continue;
                    }
                    (calendar).load(function);
                    function.instruction(&Instruction::I64Const(id.runtime_code()));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    (year).load(function);
                    function.instruction(&Instruction::I64Const(year_offset));
                    function.instruction(&Instruction::I64Sub);
                    (year).store(function);
                    function.instruction(&Instruction::End);
                }
                TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => {}
            }
        }
    }

    fn emit_temporal_calendar_days_in_month_body(
        &mut self,
        calendar: I64Local,
        year: I64Local,
        month: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        self.emit_temporal_is_east_asian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_days_in_month(kind, year, month, out, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_days_in_month(year, month, out, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_indian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_indian_days_in_month(year, month, out, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_persian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_persian_days_in_month(year, month, out, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_thirteen_month_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::ThirteenMonthSolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_thirteen_month_days_in_month(kind, year, month, out, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_tabular_islamic_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::TabularIslamic(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(kind.calendar().runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_islamic_days_in_month(kind, year, month, out, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_umalqura_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_days_in_month(year, month, out, function);
        function.instruction(&Instruction::Else);
        let iso_year = self.runtime_schema().reserve_i64_local(function);
        (year).load(function);
        (iso_year).store(function);
        self.emit_temporal_calendar_year_to_iso(calendar, iso_year, function);
        self.emit_temporal_iso_days_in_month(iso_year, month, out, function);
        self.runtime_schema().release_i64_local(iso_year, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// Validate a conservative arithmetic-year envelope before integer offsets
    /// or ordinal conversion. The actual carrier's final limit is its caller's
    /// prescribed operation, not this intermediate conversion.
    fn emit_temporal_non_gregorian_year_envelope(
        &mut self,
        calendar: I64Local,
        year: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for id in TemporalCalendarId::ALL {
            let (minimum, maximum) = match id.arithmetic() {
                TemporalCalendarArithmetic::ProlepticGregorian { .. } => continue,
                TemporalCalendarArithmetic::IndianSolar => (-272_000, 276_000),
                TemporalCalendarArithmetic::PersianSolar => (-273_000, 276_000),
                TemporalCalendarArithmetic::EastAsianLunisolar(_) => (
                    -TEMPORAL_EAST_ASIAN_YEAR_ENVELOPE,
                    TEMPORAL_EAST_ASIAN_YEAR_ENVELOPE,
                ),
                TemporalCalendarArithmetic::ThirteenMonthSolar(_) => (-280_000, 285_000),
                TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => (-300_000, 300_000),
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (year).load(function);
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::I64LtS);
            (year).load(function);
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::RangeError,
                RuntimeErrorMessage::TEMPORAL_PLAINDATE_IS_OUTSIDE_THE_SUPPORTED_DATE_RANGE,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    fn emit_temporal_calendar_regulate(
        &mut self,
        calendar: I64Local,
        fields: [I64Local; 3],
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum = self.runtime_schema().reserve_i64_local(function);
        let month_count = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_non_gregorian_year_envelope(calendar, fields[0], function)?;
        self.emit_temporal_calendar_months_in_year_i64(calendar, fields[0], function);
        (month_count).store(function);
        (overflow).load(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        for (local, upper) in [(fields[1], Some(month_count)), (fields[2], None)] {
            (local).load(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64LtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(1));
            (local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            if let Some(upper) = upper {
                (local).load(function);
                (upper).load(function);
                function.instruction(&Instruction::I64GtS);
                self.open_frame(ControlFrameKind::If, function);
                (upper).load(function);
                (local).store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        self.emit_temporal_calendar_days_in_month(
            calendar, fields[0], fields[1], maximum, function,
        );
        (fields[2]).load(function);
        (maximum).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        (maximum).load(function);
        (fields[2]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_calendar_days_in_month(
            calendar, fields[0], fields[1], maximum, function,
        );
        for (local, lower, upper) in [(fields[1], 1, None), (fields[2], 1, Some(maximum))] {
            (local).load(function);
            function.instruction(&Instruction::I64Const(lower));
            function.instruction(&Instruction::I64LtS);
            if local == fields[1] {
                (local).load(function);
                (month_count).load(function);
                function.instruction(&Instruction::I64GtS);
                function.instruction(&Instruction::I32Or);
            } else if let Some(upper) = upper {
                (local).load(function);
                (upper).load(function);
                function.instruction(&Instruction::I64GtS);
                function.instruction(&Instruction::I32Or);
            }
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::RangeError,
                RuntimeErrorMessage::TEMPORAL_PLAINDATE_IS_NOT_A_VALID_ISO_DATE,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(month_count, function);
        self.runtime_schema().release_i64_local(maximum, function);
        Ok(())
    }

    fn emit_temporal_calendar_fields_to_iso(
        &mut self,
        calendar: I64Local,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        self.runtime_schema()
            .call_helper(
                TemporalCalendarFieldsToIsoArguments::new(
                    calendar, fields[0], fields[1], fields[2],
                ),
                self.runtime_helper_base()
                    .expect("Temporal calendar arithmetic has a registered helper plan"),
                function,
            )
            .store(fields, function);
    }

    pub(crate) fn compile_temporal_calendar_fields_to_iso_helper(
        &mut self,
        real: bool,
    ) -> Function {
        if !real {
            return self
                .temporal_calendar_helper_stub(RuntimeHelperId::TemporalCalendarFieldsToIso);
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalCalendarFieldsToIso);
        let parameters =
            self.helper_parameters::<TemporalCalendarFieldsToIsoParameters>(&mut function);
        let fields = [parameters.year, parameters.month, parameters.day];
        self.emit_temporal_calendar_fields_to_iso_body(parameters.calendar, fields, &mut function);
        for local in fields {
            local.load(&mut function);
        }
        function.instruction(&Instruction::End);
        self.finish_function(function)
    }

    fn emit_temporal_calendar_fields_to_iso_body(
        &mut self,
        calendar: I64Local,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        self.emit_temporal_is_east_asian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_to_iso(kind, fields, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_to_iso(fields, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_indian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_indian_to_iso(fields, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_persian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_persian_to_iso(fields, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_thirteen_month_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::ThirteenMonthSolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_thirteen_month_to_iso(kind, fields, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_tabular_islamic_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::TabularIslamic(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(kind.calendar().runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_islamic_to_iso(kind, fields, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_umalqura_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_to_iso(fields, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_calendar_year_to_iso(calendar, fields[0], function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// Consume the resolved year/code/ordinal together. Original code survives
    /// agreement until overflow succeeds; the completed coordinates become ISO.
    pub(super) fn emit_temporal_calendar_date_to_iso(
        &mut self,
        month: TemporalResolvedCalendarMonth,
        day: I64Local,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fields = [month.year_local(), month.month_local(), day];
        (month.month_code_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_calendar_apply_month_code(
            month.calendar_id(),
            fields[0],
            month.month_code_payload_local(),
            fields[1],
            overflow,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_calendar_regulate(month.calendar_id(), fields, overflow, function)?;
        self.emit_temporal_calendar_fields_to_iso(month.calendar_id(), fields, function);
        Ok(())
    }

    pub(super) fn emit_temporal_calendar_date_from_projection(
        &mut self,
        projected: &TemporalCalendarDateLocals,
        day: I64Local,
        overflow: I64Local,
        output: [I64Local; 3],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (source, destination) in [projected.year(), projected.month(), day]
            .into_iter()
            .zip(output)
        {
            (source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_calendar_regulate(projected.calendar, output, overflow, function)?;
        self.emit_temporal_calendar_fields_to_iso(projected.calendar, output, function);
        Ok(())
    }

    pub(super) fn emit_temporal_month_day_require_year_range(
        &mut self,
        resolved: &TemporalResolvedCalendarYear,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        (resolved.year_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let year = self.runtime_schema().reserve_i64_local(function);
        for id in TemporalCalendarId::ALL {
            match id.month_day_year_use() {
                MonthDayYearUse::OverflowOnly => continue,
                MonthDayYearUse::RangeChecked => {}
            }
            let (minimum, maximum) = match id.arithmetic() {
                TemporalCalendarArithmetic::ProlepticGregorian { year_offset } => {
                    (-271_821 + year_offset, 275_760 + year_offset)
                }
                TemporalCalendarArithmetic::IndianSolar => (-271_900, 275_682),
                // Full native-year intersection below supplies the final MD limit.
                TemporalCalendarArithmetic::PersianSolar => (-273_000, 276_000),
                TemporalCalendarArithmetic::EastAsianLunisolar(_) => (
                    -TEMPORAL_EAST_ASIAN_YEAR_ENVELOPE,
                    TEMPORAL_EAST_ASIAN_YEAR_ENVELOPE,
                ),
                TemporalCalendarArithmetic::ThirteenMonthSolar(_) => (-280_000, 285_000),
                TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => (-300_000, 300_000),
            };
            (resolved.calendar_id()).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (resolved.year_local()).load(function);
            (year).store(function);
            (year).load(function);
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::I64LtS);
            (year).load(function);
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::RangeError,
                RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_YEAR_IS_OUTSIDE_THE_SUPPORTED_RANGE,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // Envelopes above make every native year and next-year conversion safe.
        // The month factory invokes this before consulting month information.
        self.emit_temporal_month_day_year_intersects_iso_limits_i32(resolved, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_YEAR_IS_OUTSIDE_THE_SUPPORTED_RANGE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(year, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// A supplied non-ISO native year is eligible if any date in it can use
    /// the full ISO carrier. The requested month/day does not determine this
    /// admission. Borrow the resolved pairing; neither original input is changed.
    fn emit_temporal_month_day_year_intersects_iso_limits_i32(
        &mut self,
        resolved: &TemporalResolvedCalendarYear,
        function: &mut Function,
    ) {
        // Overflow-only domains admit the year without converting it. Derive
        // this exemption from the same exhaustive policy as the envelope gate.
        function.instruction(&Instruction::I32Const(0));
        for id in TemporalCalendarId::ALL {
            match id.month_day_year_use() {
                MonthDayYearUse::OverflowOnly => {
                    (resolved.calendar_id()).load(function);
                    function.instruction(&Instruction::I64Const(id.runtime_code()));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                }
                MonthDayYearUse::RangeChecked => {}
            }
        }
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_hebrew_calendar_i32(resolved.calendar_id(), function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_temporal_hebrew_year_intersects_iso_limits_i32(resolved.year_local(), function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_east_asian_calendar_i32(resolved.calendar_id(), function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        let eligible = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        (eligible).store(function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => continue,
            };
            (resolved.calendar_id()).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_year_intersects_iso_limits_i32(
                kind,
                resolved.year_local(),
                function,
            );
            function.instruction(&Instruction::I64ExtendI32U);
            (eligible).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (eligible).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.runtime_schema().release_i64_local(eligible, function);
        function.instruction(&Instruction::Else);
        let fields: [I64Local; 3] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let start = self.runtime_schema().reserve_i64_local(function);
        let next_start = self.runtime_schema().reserve_i64_local(function);
        for (year_delta, epoch) in [(0, start), (1, next_start)] {
            // Conversion rewrites the private triplet into ISO coordinates.
            // Reload the retained native year for BOTH starts, including y+1.
            (resolved.year_local()).load(function);
            function.instruction(&Instruction::I64Const(year_delta));
            function.instruction(&Instruction::I64Add);
            (fields[0]).store(function);
            for local in [fields[1], fields[2]] {
                function.instruction(&Instruction::I64Const(1));
                (local).store(function);
            }
            self.emit_temporal_calendar_fields_to_iso(resolved.calendar_id(), fields, function);
            self.emit_temporal_plain_date_epoch_days(
                fields[0], fields[1], fields[2], epoch, function,
            );
        }
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
        self.runtime_schema()
            .release_i64_local(next_start, function);
        self.runtime_schema().release_i64_local(start, function);
        for local in fields.into_iter().rev() {
            self.runtime_schema().release_i64_local(local, function);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_balance_calendar_year_month_body(
        &mut self,
        calendar: I64Local,
        year: I64Local,
        month: I64Local,
        function: &mut Function,
    ) {
        let serial = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_calendar_month_serial(calendar, year, month, serial, function);
        self.emit_temporal_calendar_year_month_from_serial(calendar, serial, year, month, function);
        self.runtime_schema().release_i64_local(serial, function);
    }

    pub(super) fn emit_temporal_add_calendar_date(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        year: I64Local,
        month: I64Local,
        day: I64Local,
        years: I64Local,
        months: I64Local,
        weeks: I64Local,
        days: I64Local,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_uses_non_gregorian_arithmetic_i32(calendar.calendar_id(), function);
        self.open_frame(ControlFrameKind::If, function);
        let projected = self.emit_temporal_project_calendar_date(
            calendar.calendar_id(),
            [year, month, day],
            function,
        );
        let fields = projected.fields();
        let epoch = self.runtime_schema().reserve_i64_local(function);
        let code = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_calendar_month_code(
            calendar.calendar_id(),
            fields[0],
            fields[1],
            code,
            function,
        );
        (fields[0]).load(function);
        (years).load(function);
        function.instruction(&Instruction::I64Add);
        (fields[0]).store(function);
        self.emit_temporal_calendar_apply_month_code(
            calendar.calendar_id(),
            fields[0],
            code,
            fields[1],
            overflow,
            function,
        )?;
        (fields[1]).load(function);
        (months).load(function);
        function.instruction(&Instruction::I64Add);
        (fields[1]).store(function);
        self.emit_temporal_balance_calendar_year_month(
            calendar.calendar_id(),
            fields[0],
            fields[1],
            function,
        );
        self.emit_temporal_calendar_regulate(calendar.calendar_id(), fields, overflow, function)?;
        self.emit_temporal_calendar_fields_to_iso(calendar.calendar_id(), fields, function);
        self.emit_temporal_plain_date_epoch_days(fields[0], fields[1], fields[2], epoch, function);
        (epoch).load(function);
        (weeks).load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (days).load(function);
        function.instruction(&Instruction::I64Add);
        (epoch).store(function);
        self.emit_temporal_civil_from_days(epoch, year, month, day, function);
        self.emit_temporal_reject_iso_date(year, month, day, function)?;
        self.runtime_schema().release_i64_local(code, function);
        self.runtime_schema().release_i64_local(epoch, function);
        projected.release(self, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_add_iso_date(
            year, month, day, years, months, weeks, days, overflow, function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(super) fn emit_temporal_difference_calendar_date(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        left: [I64Local; 3],
        right: [I64Local; 3],
        largest: I64Local,
        years: I64Local,
        months: I64Local,
        weeks: I64Local,
        days: I64Local,
        function: &mut Function,
    ) {
        self.emit_temporal_difference_projected_date(
            calendar.calendar_id(),
            left,
            right,
            largest,
            years,
            months,
            weeks,
            days,
            function,
        );
    }

    /// Complete a clone or a previously converted carrier without changing
    /// explicit constructor reference fields.
    pub(super) fn emit_temporal_complete_partial_reference(
        &mut self,
        calendar: I64Local,
        kind: TemporalPartialDateType,
        iso: [I64Local; 3],
        function: &mut Function,
    ) -> Result<CompletedTemporalPartialReferenceLocals, EmitError> {
        let result = CompletedTemporalPartialReferenceLocals {
            calendar,
            kind,
            fields: std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function)),
        };
        for (source, destination) in iso.into_iter().zip(result.fields) {
            (source).load(function);
            (destination).store(function);
        }
        match kind {
            TemporalPartialDateType::PlainYearMonth => self.emit_temporal_reject_iso_year_month(
                result.fields[0],
                result.fields[1],
                result.fields[2],
                function,
            )?,
            TemporalPartialDateType::PlainMonthDay => self.emit_temporal_reject_iso_date(
                result.fields[0],
                result.fields[1],
                result.fields[2],
                function,
            )?,
        }
        Ok(result)
    }

    /// Indian 1894 contains every possible month/day, including M01 day31.
    /// Its final months cross into ISO1973; those reference the preceding
    /// calendar year instead. No search or variable-duration data is needed.
    fn emit_temporal_indian_month_day_reference(
        &mut self,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let candidate: [I64Local; 3] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let epoch = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(1894));
        (fields[0]).store(function);
        for (source, destination) in fields.into_iter().zip(candidate) {
            (source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_indian_to_iso(candidate, function);
        self.emit_temporal_plain_date_epoch_days(
            candidate[0],
            candidate[1],
            candidate[2],
            epoch,
            function,
        );
        // ISO1972-12-31 is epoch day1095.
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(1095));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1893));
        (fields[0]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_indian_to_iso(fields, function);
        self.runtime_schema().release_i64_local(epoch, function);
        for local in candidate.into_iter().rev() {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_temporal_calendar_partial_reference(
        &mut self,
        calendar: I64Local,
        kind: TemporalPartialDateType,
        iso: [I64Local; 3],
        function: &mut Function,
    ) -> Result<CompletedTemporalPartialReferenceLocals, EmitError> {
        let result = CompletedTemporalPartialReferenceLocals {
            calendar,
            kind,
            fields: std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function)),
        };
        let projected = self.emit_temporal_project_calendar_date(calendar, iso, function);
        for (source, destination) in projected.fields().into_iter().zip(result.fields) {
            (source).load(function);
            (destination).store(function);
        }
        match kind {
            TemporalPartialDateType::PlainYearMonth => {
                function.instruction(&Instruction::I64Const(1));
                (result.fields[2]).store(function);
                self.emit_temporal_calendar_fields_to_iso(calendar, result.fields, function);
                self.emit_temporal_reject_iso_year_month(
                    result.fields[0],
                    result.fields[1],
                    result.fields[2],
                    function,
                )?;
            }
            TemporalPartialDateType::PlainMonthDay => {
                self.emit_temporal_is_east_asian_calendar_i32(calendar, function);
                self.open_frame(ControlFrameKind::If, function);
                let code = self.runtime_schema().reserve_i64_local(function);
                let overflow = self.runtime_schema().reserve_i64_local(function);
                self.emit_temporal_calendar_month_code(
                    calendar,
                    projected.year(),
                    projected.month(),
                    code,
                    function,
                );
                function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
                (overflow).store(function);
                self.emit_temporal_east_asian_reference_dispatch(
                    calendar,
                    code,
                    projected.day(),
                    result.fields,
                    overflow,
                    function,
                )?;
                self.runtime_schema().release_i64_local(overflow, function);
                self.runtime_schema().release_i64_local(code, function);
                function.instruction(&Instruction::Else);
                self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
                self.open_frame(ControlFrameKind::If, function);
                let code = self.runtime_schema().reserve_i64_local(function);
                self.emit_temporal_hebrew_month_code(
                    projected.year(),
                    projected.month(),
                    code,
                    function,
                );
                self.emit_temporal_hebrew_month_day_reference(result.fields, code, function);
                self.runtime_schema().release_i64_local(code, function);
                function.instruction(&Instruction::Else);
                self.emit_temporal_is_indian_calendar_i32(calendar, function);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_temporal_indian_month_day_reference(result.fields, function);
                function.instruction(&Instruction::Else);
                self.emit_temporal_is_persian_calendar_i32(calendar, function);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_temporal_persian_month_day_reference(result.fields, function);
                function.instruction(&Instruction::Else);
                self.emit_temporal_is_thirteen_month_calendar_i32(calendar, function);
                self.open_frame(ControlFrameKind::If, function);
                for id in TemporalCalendarId::ALL {
                    let kind = match id.arithmetic() {
                        TemporalCalendarArithmetic::ThirteenMonthSolar(kind) => kind,
                        TemporalCalendarArithmetic::ProlepticGregorian { .. }
                        | TemporalCalendarArithmetic::IndianSolar
                        | TemporalCalendarArithmetic::PersianSolar
                        | TemporalCalendarArithmetic::TabularIslamic(_)
                        | TemporalCalendarArithmetic::UmmAlQura
                        | TemporalCalendarArithmetic::HebrewLunisolar
                        | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
                    };
                    (calendar).load(function);
                    function.instruction(&Instruction::I64Const(id.runtime_code()));
                    function.instruction(&Instruction::I64Eq);
                    self.open_frame(ControlFrameKind::If, function);
                    self.emit_temporal_thirteen_month_month_day_reference(
                        kind,
                        result.fields,
                        function,
                    );
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::Else);
                self.emit_temporal_is_tabular_islamic_calendar_i32(calendar, function);
                self.open_frame(ControlFrameKind::If, function);
                for id in TemporalCalendarId::ALL {
                    let kind = match id.arithmetic() {
                        TemporalCalendarArithmetic::TabularIslamic(kind) => kind,
                        TemporalCalendarArithmetic::ProlepticGregorian { .. }
                        | TemporalCalendarArithmetic::IndianSolar
                        | TemporalCalendarArithmetic::PersianSolar
                        | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                        | TemporalCalendarArithmetic::UmmAlQura
                        | TemporalCalendarArithmetic::HebrewLunisolar
                        | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
                    };
                    (calendar).load(function);
                    function.instruction(&Instruction::I64Const(kind.calendar().runtime_code()));
                    function.instruction(&Instruction::I64Eq);
                    self.open_frame(ControlFrameKind::If, function);
                    self.emit_temporal_islamic_month_day_reference(kind, result.fields, function);
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::Else);
                self.emit_temporal_is_umalqura_calendar_i32(calendar, function);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_temporal_umalqura_month_day_reference(result.fields, function);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(1972));
                (result.fields[0]).store(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.emit_temporal_reject_iso_date(
                    result.fields[0],
                    result.fields[1],
                    result.fields[2],
                    function,
                )?;
            }
        }
        projected.release(self, function);
        Ok(result)
    }

    pub(super) fn emit_temporal_month_day_reference_from_fields(
        &mut self,
        month: TemporalResolvedCalendarMonth,
        day: I64Local,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<CompletedTemporalPartialReferenceLocals, EmitError> {
        let resolved = month.resolved_year();
        let calendar = month.calendar_id();
        let result = CompletedTemporalPartialReferenceLocals {
            calendar,
            kind: TemporalPartialDateType::PlainMonthDay,
            fields: std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function)),
        };
        for (source, destination) in [month.year_local(), month.month_local(), day]
            .into_iter()
            .zip(result.fields)
        {
            (source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_is_east_asian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        (month.year_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (month.month_code_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_calendar_apply_month_code(
            calendar,
            result.fields[0],
            month.month_code_payload_local(),
            result.fields[1],
            overflow,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_calendar_regulate(calendar, result.fields, overflow, function)?;
        // Only successful supplied-year overflow retires the original code.
        self.emit_temporal_calendar_month_code(
            calendar,
            result.fields[0],
            result.fields[1],
            month.month_code_payload_local(),
            function,
        );
        function.instruction(&Instruction::Else);
        // A yearless East Asian code has no ordinal/year yet. Its normative
        // maximum is 30, independent of whether Table 6 has that exact pair.
        (result.fields[2]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        (result.fields[2]).load(function);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        (overflow).load(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_IS_NOT_A_VALID_ISO_DATE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (result.fields[2]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::End);
        (result.fields[2]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_east_asian_reference_dispatch(
            calendar,
            month.month_code_payload_local(),
            result.fields[2],
            result.fields,
            overflow,
            function,
        )?;
        function.instruction(&Instruction::Else);
        (month.year_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (month.month_code_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (month.month_code_payload_local()).load(function);
        (result.fields[1]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_missing_year_regulation_year(
            month.month_code_payload_local(),
            result.fields[0],
            function,
        );
        self.emit_temporal_hebrew_month_code_ordinal(
            result.fields[0],
            month.month_code_payload_local(),
            result.fields[1],
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_uses_non_gregorian_arithmetic_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        (resolved.year_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1894));
        (result.fields[0]).store(function);
        self.emit_temporal_is_persian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1350));
        (result.fields[0]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::ThirteenMonthSolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(kind.leap_reference_year()));
            (result.fields[0]).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::TabularIslamic(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(kind.calendar().runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(kind.leap_reference_year()));
            (result.fields[0]).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_is_umalqura_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_month_day_regulation_year(
            result.fields[1],
            result.fields[0],
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (month.year_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        (month.month_code_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_calendar_apply_month_code(
            calendar,
            result.fields[0],
            month.month_code_payload_local(),
            result.fields[1],
            overflow,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_calendar_regulate(calendar, result.fields, overflow, function)?;
        // The consumed month proof already requires whole-native-year admission.
        // Do not restrict the requested date's ISO year before choosing the
        // independent reference: its native year may overlap either carrier edge.
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        // Overflow has now succeeded: retire the original missing code in
        // favor of the regulated year's actual code before reference selection.
        self.emit_temporal_hebrew_month_code(
            result.fields[0],
            result.fields[1],
            month.month_code_payload_local(),
            function,
        );
        self.emit_temporal_hebrew_month_day_reference(
            result.fields,
            month.month_code_payload_local(),
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_indian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_indian_month_day_reference(result.fields, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_persian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_persian_month_day_reference(result.fields, function);
        function.instruction(&Instruction::Else);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::ThirteenMonthSolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_thirteen_month_month_day_reference(kind, result.fields, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::TabularIslamic(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(kind.calendar().runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_islamic_month_day_reference(kind, result.fields, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_is_umalqura_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_month_day_reference(result.fields, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        (resolved.year_present_local()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1972));
        (result.fields[0]).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_calendar_year_to_iso(calendar, result.fields[0], function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_month_day_regulate(
            result.fields[0],
            result.fields[1],
            result.fields[2],
            overflow,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1972));
        (result.fields[0]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_reject_iso_date(
            result.fields[0],
            result.fields[1],
            result.fields[2],
            function,
        )?;
        Ok(result)
    }
    fn emit_temporal_east_asian_reference_dispatch(
        &mut self,
        calendar: I64Local,
        code: I64Local,
        day: I64Local,
        out: [I64Local; 3],
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_month_day_reference(
                kind, code, day, out, overflow, function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    fn emit_temporal_is_east_asian_calendar_i32(
        &self,
        calendar: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I32Const(0));
        for id in TemporalCalendarId::ALL {
            match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(_) => {
                    (calendar).load(function);
                    function.instruction(&Instruction::I64Const(id.runtime_code()));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                }
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => {}
            }
        }
    }

    fn emit_temporal_is_hebrew_calendar_i32(&self, calendar: I64Local, function: &mut Function) {
        (calendar).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalCalendarId::Hebrew.runtime_code(),
        ));
        function.instruction(&Instruction::I64Eq);
    }

    fn emit_temporal_calendar_apply_month_code(
        &mut self,
        calendar: I64Local,
        year: I64Local,
        code: I64Local,
        month: I64Local,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_month_code_present(year, code, function);
        function.instruction(&Instruction::I32Eqz);
        (overflow).load(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_MONTHCODE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_hebrew_month_code_ordinal(year, code, month, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_month_code_present(kind, year, code, function);
            function.instruction(&Instruction::I32Eqz);
            (overflow).load(function);
            function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(
                lila_ir::NativeErrorKind::RangeError,
                RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_MONTHCODE,
                function,
            )?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.emit_temporal_east_asian_month_code_ordinal(kind, year, code, month, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    pub(super) fn emit_temporal_calendar_month_code(
        &mut self,
        calendar: I64Local,
        year: I64Local,
        month: I64Local,
        code: I64Local,
        function: &mut Function,
    ) {
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_month_code(year, month, code, function);
        function.instruction(&Instruction::Else);
        (month).load(function);
        (code).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_month_code(kind, year, month, code, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
    }

    fn emit_temporal_calendar_month_rank(
        &self,
        code: I64Local,
        rank: I64Local,
        function: &mut Function,
    ) {
        // Code encoding is a storage key, never an ordinal or chronology rank.
        for candidate in TemporalCalendarMonthCode::ALL {
            (code).load(function);
            function.instruction(&Instruction::I64Const(candidate.encoding()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(candidate.rank()));
            (rank).store(function);
            function.instruction(&Instruction::End);
        }
    }

    /// The actual chronological month domain. Fixed calendars and Hebrew use
    /// the same serial consumer; the closed policy selects its sole native map.
    fn emit_temporal_calendar_month_serial(
        &mut self,
        calendar: I64Local,
        year: I64Local,
        month: I64Local,
        serial: I64Local,
        function: &mut Function,
    ) {
        for id in TemporalCalendarId::ALL {
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            match id.arithmetic().month_arithmetic() {
                TemporalCalendarMonthArithmetic::HebrewMetonic => {
                    self.emit_temporal_hebrew_month_serial(year, month, serial, function)
                }
                TemporalCalendarMonthArithmetic::EastAsianLunisolar(kind) => {
                    self.emit_temporal_east_asian_month_serial(kind, year, month, serial, function)
                }
                TemporalCalendarMonthArithmetic::Twelve
                | TemporalCalendarMonthArithmetic::Thirteen => {
                    let count = match id.arithmetic().month_arithmetic() {
                        TemporalCalendarMonthArithmetic::Twelve => 12,
                        TemporalCalendarMonthArithmetic::Thirteen => 13,
                        TemporalCalendarMonthArithmetic::HebrewMetonic
                        | TemporalCalendarMonthArithmetic::EastAsianLunisolar(_) => unreachable!(),
                    };
                    (year).load(function);
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::I64Const(count));
                    function.instruction(&Instruction::I64Mul);
                    (month).load(function);
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Sub);
                    (serial).store(function);
                }
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
    }

    fn emit_temporal_calendar_year_month_from_serial(
        &mut self,
        calendar: I64Local,
        serial: I64Local,
        year: I64Local,
        month: I64Local,
        function: &mut Function,
    ) {
        let quotient = self.runtime_schema().reserve_i64_local(function);
        for id in TemporalCalendarId::ALL {
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            match id.arithmetic().month_arithmetic() {
                TemporalCalendarMonthArithmetic::HebrewMetonic => {
                    self.emit_temporal_hebrew_year_month_from_serial(serial, year, month, function)
                }
                TemporalCalendarMonthArithmetic::EastAsianLunisolar(kind) => self
                    .emit_temporal_east_asian_year_month_from_serial(
                        kind, serial, year, month, function,
                    ),
                TemporalCalendarMonthArithmetic::Twelve
                | TemporalCalendarMonthArithmetic::Thirteen => {
                    let count = match id.arithmetic().month_arithmetic() {
                        TemporalCalendarMonthArithmetic::Twelve => 12,
                        TemporalCalendarMonthArithmetic::Thirteen => 13,
                        TemporalCalendarMonthArithmetic::HebrewMetonic
                        | TemporalCalendarMonthArithmetic::EastAsianLunisolar(_) => unreachable!(),
                    };
                    (serial).load(function);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64LtS);
                    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                    (serial).load(function);
                    function.instruction(&Instruction::I64Const(count - 1));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::Else);
                    (serial).load(function);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::I64Const(count));
                    function.instruction(&Instruction::I64DivS);
                    (quotient).store(function);
                    (quotient).load(function);
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    (year).store(function);
                    (serial).load(function);
                    (quotient).load(function);
                    function.instruction(&Instruction::I64Const(count));
                    function.instruction(&Instruction::I64Mul);
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    (month).store(function);
                }
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.runtime_schema().release_i64_local(quotient, function);
    }

    /// First phase of NonISODateSurpasses: compare the original code in the
    /// year after the year step, before the code is constrained or months move.
    /// Every year/month candidate combines this with its balanced ordinal phase.
    fn emit_temporal_calendar_original_year_surpasses(
        &mut self,
        source: &TemporalCalendarDateLocals,
        years: I64Local,
        target: [I64Local; 3],
        target_rank: I64Local,
        sign: I64Local,
        function: &mut Function,
    ) {
        let code = self.runtime_schema().reserve_i64_local(function);
        let rank = self.runtime_schema().reserve_i64_local(function);
        let year = self.runtime_schema().reserve_i64_local(function);
        let comparison = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_calendar_month_code(
            source.calendar,
            source.year(),
            source.month(),
            code,
            function,
        );
        self.emit_temporal_calendar_month_rank(code, rank, function);
        (source.year()).load(function);
        (years).load(function);
        function.instruction(&Instruction::I64Add);
        (year).store(function);
        self.emit_temporal_compare_iso_date(
            [target[0], target_rank, target[2]],
            [year, rank, source.day()],
            comparison,
            function,
        );
        (comparison).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        for local in [comparison, year, rank, code] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Retain the source code before changing the year. Its constrained target
    /// ordinal survives the month step. Original-code comparison remains the
    /// separate mandatory first phase; neither phase clamps the source day.
    fn emit_temporal_calendar_year_anchor(
        &mut self,
        source: &TemporalCalendarDateLocals,
        years: I64Local,
        year: I64Local,
        month: I64Local,
        rank: I64Local,
        function: &mut Function,
    ) {
        let code = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_calendar_month_code(
            source.calendar,
            source.year(),
            source.month(),
            code,
            function,
        );
        (source.year()).load(function);
        (years).load(function);
        function.instruction(&Instruction::I64Add);
        (year).store(function);
        self.emit_temporal_is_hebrew_calendar_i32(source.calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_month_code_ordinal(year, code, month, function);
        function.instruction(&Instruction::Else);
        (source.month()).load(function);
        (month).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => continue,
            };
            (source.calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_month_code_ordinal(kind, year, code, month, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_calendar_month_code(source.calendar, year, month, code, function);
        self.emit_temporal_calendar_month_rank(code, rank, function);
        self.runtime_schema().release_i64_local(code, function);
    }

    fn emit_temporal_calendar_month_anchor(
        &mut self,
        source: &TemporalCalendarDateLocals,
        years: I64Local,
        months: I64Local,
        year: I64Local,
        month: I64Local,
        rank: I64Local,
        function: &mut Function,
    ) {
        let serial = self.runtime_schema().reserve_i64_local(function);
        let code = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_calendar_year_anchor(source, years, year, month, rank, function);
        self.emit_temporal_calendar_month_serial(source.calendar, year, month, serial, function);
        (serial).load(function);
        (months).load(function);
        function.instruction(&Instruction::I64Add);
        (serial).store(function);
        self.emit_temporal_calendar_year_month_from_serial(
            source.calendar,
            serial,
            year,
            month,
            function,
        );
        self.emit_temporal_calendar_month_code(source.calendar, year, month, code, function);
        self.emit_temporal_calendar_month_rank(code, rank, function);
        self.runtime_schema().release_i64_local(code, function);
        self.runtime_schema().release_i64_local(serial, function);
    }

    /// Compare virtual calendar positions and return a real year/month pair.
    /// Only this owner interprets a year step; no consumer flattens it using a
    /// particular year's month count.
    fn emit_temporal_difference_calendar_position(
        &mut self,
        source: &TemporalCalendarDateLocals,
        target: [I64Local; 3],
        target_rank: I64Local,
        largest: I64Local,
        years: I64Local,
        months: I64Local,
        midpoint: [I64Local; 3],
        function: &mut Function,
    ) {
        let source_code = self.runtime_schema().reserve_i64_local(function);
        let source_rank = self.runtime_schema().reserve_i64_local(function);
        let sign = self.runtime_schema().reserve_i64_local(function);
        let comparison = self.runtime_schema().reserve_i64_local(function);
        let rank = self.runtime_schema().reserve_i64_local(function);
        let start_serial = self.runtime_schema().reserve_i64_local(function);
        let end_serial = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_calendar_month_code(
            source.calendar,
            source.year(),
            source.month(),
            source_code,
            function,
        );
        self.emit_temporal_calendar_month_rank(source_code, source_rank, function);
        self.emit_temporal_compare_iso_date(
            [target[0], target_rank, target[2]],
            [source.year(), source_rank, source.day()],
            sign,
            function,
        );
        years.set_constant(0, function);
        months.set_constant(0, function);
        (sign).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (largest).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (target[0]).load(function);
        (source.year()).load(function);
        function.instruction(&Instruction::I64Sub);
        (years).store(function);
        self.emit_temporal_calendar_year_anchor(
            source,
            years,
            midpoint[0],
            midpoint[1],
            rank,
            function,
        );
        self.emit_temporal_compare_iso_date(
            [target[0], target_rank, target[2]],
            [midpoint[0], rank, source.day()],
            comparison,
            function,
        );
        (comparison).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.emit_temporal_calendar_original_year_surpasses(
            source,
            years,
            target,
            target_rank,
            sign,
            function,
        );
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        (years).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Sub);
        (years).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_calendar_year_anchor(
            source,
            years,
            midpoint[0],
            midpoint[1],
            rank,
            function,
        );
        self.emit_temporal_calendar_month_serial(
            source.calendar,
            midpoint[0],
            midpoint[1],
            start_serial,
            function,
        );
        self.emit_temporal_calendar_month_serial(
            source.calendar,
            target[0],
            target[1],
            end_serial,
            function,
        );
        (end_serial).load(function);
        (start_serial).load(function);
        function.instruction(&Instruction::I64Sub);
        (months).store(function);
        self.emit_temporal_calendar_month_anchor(
            source,
            years,
            months,
            midpoint[0],
            midpoint[1],
            rank,
            function,
        );
        self.emit_temporal_compare_iso_date(
            [target[0], target_rank, target[2]],
            [midpoint[0], rank, source.day()],
            comparison,
            function,
        );
        (comparison).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.emit_temporal_calendar_original_year_surpasses(
            source,
            years,
            target,
            target_rank,
            sign,
            function,
        );
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        (months).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Sub);
        (months).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_calendar_month_anchor(
            source,
            years,
            months,
            midpoint[0],
            midpoint[1],
            rank,
            function,
        );
        (source.day()).load(function);
        (midpoint[2]).store(function);
        for local in [
            end_serial,
            start_serial,
            rank,
            comparison,
            sign,
            source_rank,
            source_code,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Bubble a rounded pair using its virtual calendar position. The next
    /// actual year anchor is validated first, as required by relative rounding.
    pub(super) fn emit_temporal_bubble_calendar_difference(
        &mut self,
        calendar: &TemporalCalendarSlotLocals,
        origin: [I64Local; 3],
        years: I64Local,
        months: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source =
            self.emit_temporal_project_calendar_date(calendar.calendar_id(), origin, function);
        let target: [I64Local; 3] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let midpoint: [I64Local; 3] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let rank = self.runtime_schema().reserve_i64_local(function);
        let sign = self.runtime_schema().reserve_i64_local(function);
        let next_year = self.runtime_schema().reserve_i64_local(function);
        let largest = self.runtime_schema().reserve_i64_local(function);
        let zero = self.runtime_schema().reserve_i64_local(function);
        let overflow = self.runtime_schema().reserve_i64_local(function);
        (months).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        (sign).store(function);
        (months).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(0));
        (zero).store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        (overflow).store(function);
        (years).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Add);
        (next_year).store(function);
        for (src, dst) in origin.into_iter().zip(midpoint) {
            (src).load(function);
            (dst).store(function);
        }
        self.emit_temporal_add_calendar_date(
            calendar,
            midpoint[0],
            midpoint[1],
            midpoint[2],
            next_year,
            zero,
            zero,
            zero,
            overflow,
            function,
        )?;
        self.emit_temporal_calendar_month_anchor(
            &source, years, months, target[0], target[1], rank, function,
        );
        (source.day()).load(function);
        (target[2]).store(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Year.code()));
        (largest).store(function);
        self.emit_temporal_difference_calendar_position(
            &source, target, rank, largest, years, months, midpoint, function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in [overflow, zero, largest, next_year, sign, rank] {
            self.runtime_schema().release_i64_local(local, function);
        }
        for local in midpoint.into_iter().rev().chain(target.into_iter().rev()) {
            self.runtime_schema().release_i64_local(local, function);
        }
        source.release(self, function);
        Ok(())
    }

    fn emit_temporal_is_indian_calendar_i32(&self, calendar: I64Local, function: &mut Function) {
        // The exhaustive domain match forces a new arithmetic domain to decide
        // this dispatch instead of silently borrowing either implementation.
        function.instruction(&Instruction::I32Const(0));
        for id in TemporalCalendarId::ALL {
            match id.arithmetic() {
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => {}
                TemporalCalendarArithmetic::IndianSolar => {
                    (calendar).load(function);
                    function.instruction(&Instruction::I64Const(id.runtime_code()));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                }
            }
        }
    }

    fn emit_temporal_is_persian_calendar_i32(&self, calendar: I64Local, function: &mut Function) {
        (calendar).load(function);
        function.instruction(&Instruction::I64Const(
            TemporalCalendarId::Persian.runtime_code(),
        ));
        function.instruction(&Instruction::I64Eq);
    }

    fn emit_temporal_is_thirteen_month_calendar_i32(
        &self,
        calendar: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I32Const(0));
        for id in TemporalCalendarId::ALL {
            match id.arithmetic() {
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => {}
                TemporalCalendarArithmetic::ThirteenMonthSolar(kind) => match kind {
                    TemporalThirteenMonthCalendar::Coptic
                    | TemporalThirteenMonthCalendar::Ethiopic
                    | TemporalThirteenMonthCalendar::Ethioaa => {
                        (calendar).load(function);
                        function.instruction(&Instruction::I64Const(id.runtime_code()));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::I32Or);
                    }
                },
            }
        }
    }

    fn emit_temporal_is_tabular_islamic_calendar_i32(
        &self,
        calendar: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I32Const(0));
        for id in TemporalCalendarId::ALL {
            match id.arithmetic() {
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => {}
                TemporalCalendarArithmetic::TabularIslamic(kind) => match kind {
                    TemporalIslamicCalendar::Civil | TemporalIslamicCalendar::Tbla => {
                        (calendar).load(function);
                        function
                            .instruction(&Instruction::I64Const(kind.calendar().runtime_code()));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::I32Or);
                    }
                },
            }
        }
    }

    fn emit_temporal_is_umalqura_calendar_i32(&self, calendar: I64Local, function: &mut Function) {
        function.instruction(&Instruction::I32Const(0));
        for id in TemporalCalendarId::ALL {
            match id.arithmetic() {
                TemporalCalendarArithmetic::UmmAlQura => {
                    (calendar).load(function);
                    function.instruction(&Instruction::I64Const(id.runtime_code()));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                }
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => {}
            }
        }
    }

    fn emit_temporal_uses_non_gregorian_arithmetic_i32(
        &self,
        calendar: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I32Const(0));
        for id in TemporalCalendarId::ALL {
            // Adding an arithmetic domain must classify its dispatch here.
            // A hand-maintained predicate chain omitted both lunisolar domains
            // and silently performed Gregorian arithmetic on their ISO slots.
            match id.arithmetic() {
                TemporalCalendarArithmetic::ProlepticGregorian { .. } => continue,
                TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => {}
            }
            calendar.load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    fn emit_temporal_indian_leap_i32(&mut self, year: I64Local, function: &mut Function) {
        let iso_year = self.runtime_schema().reserve_i64_local(function);
        (year).load(function);
        function.instruction(&Instruction::I64Const(78));
        function.instruction(&Instruction::I64Add);
        (iso_year).store(function);
        self.emit_temporal_iso_year_is_leap_i32(iso_year, function);
        self.runtime_schema().release_i64_local(iso_year, function);
    }

    fn emit_temporal_indian_days_in_month(
        &mut self,
        year: I64Local,
        month: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(30));
        (out).store(function);
        (month).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_indian_leap_i32(year, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        function.instruction(&Instruction::Else);
        (month).load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(31));
        (out).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_indian_ordinal(
        &mut self,
        fields: [I64Local; 3],
        out: I64Local,
        function: &mut Function,
    ) {
        let length = self.runtime_schema().reserve_i64_local(function);
        (fields[2]).load(function);
        (out).store(function);
        // At most eleven additions; no date-dependent source expansion.
        for month in 1..12 {
            (fields[1]).load(function);
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::I64GtS);
            self.open_frame(ControlFrameKind::If, function);
            if month == 1 {
                self.emit_temporal_indian_leap_i32(fields[0], function);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::I64Const(30));
                function.instruction(&Instruction::I64Add);
            } else {
                function.instruction(&Instruction::I64Const(if month <= 6 { 31 } else { 30 }));
            }
            (length).store(function);
            (out).load(function);
            (length).load(function);
            function.instruction(&Instruction::I64Add);
            (out).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.runtime_schema().release_i64_local(length, function);
    }

    fn emit_temporal_indian_to_iso(&mut self, fields: [I64Local; 3], function: &mut Function) {
        let ordinal = self.runtime_schema().reserve_i64_local(function);
        let epoch = self.runtime_schema().reserve_i64_local(function);
        let one = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_indian_ordinal(fields, ordinal, function);
        function.instruction(&Instruction::I64Const(1));
        (one).store(function);
        (fields[0]).load(function);
        function.instruction(&Instruction::I64Const(78));
        function.instruction(&Instruction::I64Add);
        (fields[0]).store(function);
        self.emit_temporal_plain_date_epoch_days(fields[0], one, one, epoch, function);
        (epoch).load(function);
        (ordinal).load(function);
        function.instruction(&Instruction::I64Const(79));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Add);
        (epoch).store(function);
        self.emit_temporal_civil_from_days(epoch, fields[0], fields[1], fields[2], function);
        self.runtime_schema().release_i64_local(one, function);
        self.runtime_schema().release_i64_local(epoch, function);
        self.runtime_schema().release_i64_local(ordinal, function);
    }

    pub(super) fn emit_temporal_project_calendar_date(
        &mut self,
        calendar: I64Local,
        iso: [I64Local; 3],
        function: &mut Function,
    ) -> TemporalCalendarDateLocals {
        let result = TemporalCalendarDateLocals::reserve(self, calendar, function);
        self.runtime_schema()
            .call_helper(
                TemporalCalendarProjectDateArguments::new(calendar, iso[0], iso[1], iso[2]),
                self.runtime_helper_base()
                    .expect("Temporal calendar arithmetic has a registered helper plan"),
                function,
            )
            .store(&result, function);
        result
    }

    pub(crate) fn compile_temporal_calendar_project_date_helper(&mut self, real: bool) -> Function {
        if !real {
            return self
                .temporal_calendar_helper_stub(RuntimeHelperId::TemporalCalendarProjectDate);
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalCalendarProjectDate);
        let parameters =
            self.helper_parameters::<TemporalCalendarProjectDateParameters>(&mut function);
        let result = self.emit_temporal_project_calendar_date_body(
            parameters.calendar,
            [parameters.year, parameters.month, parameters.day],
            &mut function,
        );
        for local in result.result_locals() {
            local.load(&mut function);
        }
        result.release(self, &mut function);
        function.instruction(&Instruction::End);
        self.finish_function(function)
    }

    fn emit_temporal_project_calendar_date_body(
        &mut self,
        calendar: I64Local,
        iso: [I64Local; 3],
        function: &mut Function,
    ) -> TemporalCalendarDateLocals {
        let result = TemporalCalendarDateLocals::reserve(self, calendar, function);
        for (source, destination) in iso.into_iter().zip(result.fields) {
            (source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_is_east_asian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::EastAsianLunisolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_east_asian_project_date(kind, iso, &result, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_hebrew_project_date(iso, &result, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_indian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_plain_date_day_of_year(
            iso[0],
            iso[1],
            iso[2],
            result.day_of_year,
            function,
        );
        (iso[0]).load(function);
        function.instruction(&Instruction::I64Const(78));
        function.instruction(&Instruction::I64Sub);
        (result.fields[0]).store(function);
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Const(80));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        (result.fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (result.fields[0]).store(function);
        self.emit_temporal_indian_leap_i32(result.fields[0], function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(365));
        function.instruction(&Instruction::I64Add);
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Add);
        (result.day_of_year).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Const(80));
        function.instruction(&Instruction::I64Sub);
        (result.day_of_year).store(function);
        (result.day_of_year).load(function);
        (result.fields[2]).store(function);
        function.instruction(&Instruction::I64Const(1));
        (result.fields[1]).store(function);
        for month in 1..12 {
            self.emit_temporal_indian_days_in_month(
                result.fields[0],
                result.fields[1],
                result.days_in_month,
                function,
            );
            (result.fields[2]).load(function);
            (result.days_in_month).load(function);
            function.instruction(&Instruction::I64GtS);
            self.open_frame(ControlFrameKind::If, function);
            (result.fields[2]).load(function);
            (result.days_in_month).load(function);
            function.instruction(&Instruction::I64Sub);
            (result.fields[2]).store(function);
            function.instruction(&Instruction::I64Const(month + 1));
            (result.fields[1]).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_indian_days_in_month(
            result.fields[0],
            result.fields[1],
            result.days_in_month,
            function,
        );
        self.emit_temporal_indian_leap_i32(result.fields[0], function);
        function.instruction(&Instruction::I64ExtendI32U);
        (result.leap).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_persian_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_persian_project_date(iso, &result, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_thirteen_month_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::ThirteenMonthSolar(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_thirteen_month_project_date(kind, iso, &result, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_tabular_islamic_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        for id in TemporalCalendarId::ALL {
            let kind = match id.arithmetic() {
                TemporalCalendarArithmetic::TabularIslamic(kind) => kind,
                TemporalCalendarArithmetic::ProlepticGregorian { .. }
                | TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => continue,
            };
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(kind.calendar().runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_islamic_project_date(kind, iso, &result, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_is_umalqura_calendar_i32(calendar, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_project_date(iso, &result, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_iso_days_in_month(iso[0], iso[1], result.days_in_month, function);
        self.emit_temporal_plain_date_day_of_year(
            iso[0],
            iso[1],
            iso[2],
            result.day_of_year,
            function,
        );
        self.emit_temporal_iso_year_is_leap_i32(iso[0], function);
        function.instruction(&Instruction::I64ExtendI32U);
        (result.leap).store(function);
        for id in TemporalCalendarId::ALL {
            match id.arithmetic() {
                TemporalCalendarArithmetic::ProlepticGregorian { year_offset } => {
                    if year_offset == 0 {
                        continue;
                    }
                    (calendar).load(function);
                    function.instruction(&Instruction::I64Const(id.runtime_code()));
                    function.instruction(&Instruction::I64Eq);
                    self.open_frame(ControlFrameKind::If, function);
                    (result.fields[0]).load(function);
                    function.instruction(&Instruction::I64Const(year_offset));
                    function.instruction(&Instruction::I64Add);
                    (result.fields[0]).store(function);
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                }
                TemporalCalendarArithmetic::IndianSolar
                | TemporalCalendarArithmetic::PersianSolar
                | TemporalCalendarArithmetic::ThirteenMonthSolar(_)
                | TemporalCalendarArithmetic::TabularIslamic(_)
                | TemporalCalendarArithmetic::UmmAlQura
                | TemporalCalendarArithmetic::HebrewLunisolar
                | TemporalCalendarArithmetic::EastAsianLunisolar(_) => {}
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for id in TemporalCalendarId::ALL {
            (calendar).load(function);
            function.instruction(&Instruction::I64Const(id.runtime_code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            match id.arithmetic().year_length() {
                TemporalCalendarYearLength::SolarCommonPlusLeap
                | TemporalCalendarYearLength::LunarCommonPlusLeap => {
                    (result.leap).load(function);
                    function.instruction(&Instruction::I64Const(
                        match id.arithmetic().year_length() {
                            TemporalCalendarYearLength::SolarCommonPlusLeap => 365,
                            TemporalCalendarYearLength::LunarCommonPlusLeap => 354,
                            TemporalCalendarYearLength::HebrewNewYearDifference
                            | TemporalCalendarYearLength::EastAsianNewYearDifference => {
                                unreachable!()
                            }
                        },
                    ));
                    function.instruction(&Instruction::I64Add);
                    (result.days_in_year).store(function);
                }
                TemporalCalendarYearLength::HebrewNewYearDifference
                | TemporalCalendarYearLength::EastAsianNewYearDifference => {}
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_temporal_is_hebrew_calendar_i32(calendar, function);
        self.emit_temporal_is_east_asian_calendar_i32(calendar, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_calendar_months_in_year_i64(calendar, result.year(), function);
        (result.months_in_year).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result
    }

    fn emit_temporal_difference_projected_date_body(
        &mut self,
        calendar: I64Local,
        left_iso: [I64Local; 3],
        right_iso: [I64Local; 3],
        largest: I64Local,
        years: I64Local,
        months: I64Local,
        weeks: I64Local,
        days: I64Local,
        function: &mut Function,
    ) {
        let left = self.emit_temporal_project_calendar_date(calendar, left_iso, function);
        let right = self.emit_temporal_project_calendar_date(calendar, right_iso, function);
        let midpoint: [I64Local; 3] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let code = self.runtime_schema().reserve_i64_local(function);
        let rank = self.runtime_schema().reserve_i64_local(function);
        let maximum = self.runtime_schema().reserve_i64_local(function);
        let left_epoch = self.runtime_schema().reserve_i64_local(function);
        let right_epoch = self.runtime_schema().reserve_i64_local(function);
        for local in [years, months, weeks, days] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.emit_temporal_plain_date_epoch_days(
            left_iso[0],
            left_iso[1],
            left_iso[2],
            left_epoch,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            right_iso[0],
            right_iso[1],
            right_iso[2],
            right_epoch,
            function,
        );
        (largest).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        (right_epoch).load(function);
        (left_epoch).load(function);
        function.instruction(&Instruction::I64Sub);
        (days).store(function);
        (largest).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (days).load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64DivS);
        (weeks).store(function);
        (days).load(function);
        (weeks).load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (days).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_calendar_month_code(
            calendar,
            right.year(),
            right.month(),
            code,
            function,
        );
        self.emit_temporal_calendar_month_rank(code, rank, function);
        self.emit_temporal_difference_calendar_position(
            &left,
            right.fields(),
            rank,
            largest,
            years,
            months,
            midpoint,
            function,
        );
        // The virtual comparison above uses the original day. Only now may
        // its concrete midpoint clamp the day before calculating the residue.
        self.emit_temporal_calendar_days_in_month(
            calendar,
            midpoint[0],
            midpoint[1],
            maximum,
            function,
        );
        (midpoint[2]).load(function);
        (maximum).load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        (maximum).load(function);
        (midpoint[2]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_calendar_fields_to_iso(calendar, midpoint, function);
        self.emit_temporal_plain_date_epoch_days(
            midpoint[0],
            midpoint[1],
            midpoint[2],
            left_epoch,
            function,
        );
        (right_epoch).load(function);
        (left_epoch).load(function);
        function.instruction(&Instruction::I64Sub);
        (days).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in [right_epoch, left_epoch, maximum, rank, code] {
            self.runtime_schema().release_i64_local(local, function);
        }
        for local in midpoint.into_iter().rev() {
            self.runtime_schema().release_i64_local(local, function);
        }
        right.release(self, function);
        left.release(self, function);
    }
}
