//! Shared pure date arithmetic. The parent retains each original algorithm;
//! these facades only cross the registered scalar ABI and consume all results.
use super::*;
use crate::runtime_helpers::{
    CalendarDifferenceCallResult, CalendarYearMonthCallResult,
    TemporalCalendarBalanceYearMonthArguments, TemporalCalendarBalanceYearMonthParameters,
    TemporalCalendarDaysInMonthArguments, TemporalCalendarDaysInMonthParameters,
    TemporalCalendarDifferenceDateArguments, TemporalCalendarDifferenceDateParameters,
};

impl CalendarYearMonthCallResult {
    fn store(self, year: I64Local, month: I64Local, function: &mut Function) {
        month.store(function);
        year.store(function);
    }
}

impl CalendarDifferenceCallResult {
    fn store(self, fields: [I64Local; 4], function: &mut Function) {
        for local in fields.into_iter().rev() {
            local.store(function);
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_temporal_calendar_days_in_month(
        &mut self,
        calendar: I64Local,
        year: I64Local,
        month: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        self.runtime_schema()
            .call_helper(
                TemporalCalendarDaysInMonthArguments::new(calendar, year, month),
                self.runtime_helper_base()
                    .expect("Temporal calendar arithmetic has a registered helper plan"),
                function,
            )
            .store(out, function);
    }

    pub(crate) fn compile_temporal_calendar_days_in_month_helper(
        &mut self,
        real: bool,
    ) -> Function {
        if !real {
            return self
                .temporal_calendar_helper_stub(RuntimeHelperId::TemporalCalendarDaysInMonth);
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalCalendarDaysInMonth);
        let parameters =
            self.helper_parameters::<TemporalCalendarDaysInMonthParameters>(&mut function);
        let out = self.runtime_schema().reserve_i64_local(&mut function);
        self.emit_temporal_calendar_days_in_month_body(
            parameters.calendar,
            parameters.year,
            parameters.month,
            out,
            &mut function,
        );
        out.load(&mut function);
        self.runtime_schema().release_i64_local(out, &mut function);
        function.instruction(&Instruction::End);
        self.finish_function(function)
    }

    pub(super) fn emit_temporal_balance_calendar_year_month(
        &mut self,
        calendar: I64Local,
        year: I64Local,
        month: I64Local,
        function: &mut Function,
    ) {
        self.runtime_schema()
            .call_helper(
                TemporalCalendarBalanceYearMonthArguments::new(calendar, year, month),
                self.runtime_helper_base()
                    .expect("Temporal calendar arithmetic has a registered helper plan"),
                function,
            )
            .store(year, month, function);
    }

    pub(crate) fn compile_temporal_calendar_balance_year_month_helper(
        &mut self,
        real: bool,
    ) -> Function {
        if !real {
            return self
                .temporal_calendar_helper_stub(RuntimeHelperId::TemporalCalendarBalanceYearMonth);
        }
        let mut function =
            self.begin_helper_body(RuntimeHelperId::TemporalCalendarBalanceYearMonth);
        let parameters =
            self.helper_parameters::<TemporalCalendarBalanceYearMonthParameters>(&mut function);
        self.emit_temporal_balance_calendar_year_month_body(
            parameters.calendar,
            parameters.year,
            parameters.month,
            &mut function,
        );
        parameters.year.load(&mut function);
        parameters.month.load(&mut function);
        function.instruction(&Instruction::End);
        self.finish_function(function)
    }

    pub(super) fn emit_temporal_difference_projected_date(
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
        self.runtime_schema()
            .call_helper(
                TemporalCalendarDifferenceDateArguments::new(
                    calendar,
                    left_iso[0],
                    left_iso[1],
                    left_iso[2],
                    right_iso[0],
                    right_iso[1],
                    right_iso[2],
                    largest,
                ),
                self.runtime_helper_base()
                    .expect("Temporal calendar arithmetic has a registered helper plan"),
                function,
            )
            .store([years, months, weeks, days], function);
    }

    pub(crate) fn compile_temporal_calendar_difference_date_helper(
        &mut self,
        real: bool,
    ) -> Function {
        if !real {
            return self
                .temporal_calendar_helper_stub(RuntimeHelperId::TemporalCalendarDifferenceDate);
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalCalendarDifferenceDate);
        let parameters =
            self.helper_parameters::<TemporalCalendarDifferenceDateParameters>(&mut function);
        let out: [I64Local; 4] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(&mut function));
        self.emit_temporal_difference_projected_date_body(
            parameters.calendar,
            [
                parameters.left_year,
                parameters.left_month,
                parameters.left_day,
            ],
            [
                parameters.right_year,
                parameters.right_month,
                parameters.right_day,
            ],
            parameters.largest,
            out[0],
            out[1],
            out[2],
            out[3],
            &mut function,
        );
        for local in out {
            local.load(&mut function);
        }
        for local in out.into_iter().rev() {
            self.runtime_schema()
                .release_i64_local(local, &mut function);
        }
        function.instruction(&Instruction::End);
        self.finish_function(function)
    }
}
