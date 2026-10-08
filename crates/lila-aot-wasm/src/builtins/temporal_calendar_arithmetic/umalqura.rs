//! Validated Umm al-Qura table arithmetic over the existing ISO carrier.
//! Outside the exact table interval, the private Civil leaf owns conversion.

use super::super::temporal_plain_date::TemporalIslamicCalendar;
use super::umalqura_data::{
    ummalqura_day30_reference_year, UmmAlQuraYearInfo, UMMALQURA_EXCLUSIVE_END_DAY,
    UMMALQURA_FIRST_START_DAY, UMMALQURA_YEARS,
};
use super::*;
use crate::gc_types::I64Local;
use crate::runtime_helpers::{
    TemporalUmmAlQuraEpochArguments, TemporalUmmAlQuraEpochParameters,
    TemporalUmmAlQuraYearArguments, TemporalUmmAlQuraYearParameters, UmmAlQuraYearCallResult,
};

impl UmmAlQuraYearCallResult {
    fn store(self, year: I64Local, packed: I64Local, function: &mut Function) {
        packed.store(function);
        year.store(function);
    }
}

/// Only slices of the const-validated, nonempty table reach this emitter.
fn emit_umalqura_year_info_by_year(
    year: I64Local,
    rows: &[UmmAlQuraYearInfo],
    function: &mut Function,
) {
    if rows.len() == 1 {
        function.instruction(&Instruction::I64Const(rows[0].packed()));
        return;
    }
    let middle = rows.len() / 2;
    (year).load(function);
    function.instruction(&Instruction::I64Const(rows[middle].calendar_year()));
    function.instruction(&Instruction::I64LtS);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    emit_umalqura_year_info_by_year(year, &rows[..middle], function);
    function.instruction(&Instruction::Else);
    emit_umalqura_year_info_by_year(year, &rows[middle..], function);
    function.instruction(&Instruction::End);
}

/// The caller proves membership in the exact contiguous table interval.
/// Each terminal selects both the real arithmetic year and its packed info.
fn emit_umalqura_year_info_by_epoch(
    epoch: I64Local,
    year_out: I64Local,
    rows: &[UmmAlQuraYearInfo],
    function: &mut Function,
) {
    if rows.len() == 1 {
        function.instruction(&Instruction::I64Const(rows[0].calendar_year()));
        (year_out).store(function);
        function.instruction(&Instruction::I64Const(rows[0].packed()));
        return;
    }
    let middle = rows.len() / 2;
    (epoch).load(function);
    function.instruction(&Instruction::I64Const(rows[middle].start_day()));
    function.instruction(&Instruction::I64LtS);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    emit_umalqura_year_info_by_epoch(epoch, year_out, &rows[..middle], function);
    function.instruction(&Instruction::Else);
    emit_umalqura_year_info_by_epoch(epoch, year_out, &rows[middle..], function);
    function.instruction(&Instruction::End);
}

impl FunctionBuilder<'_> {
    pub(crate) fn compile_temporal_umalqura_year_helper(&mut self, real: bool) -> Function {
        if !real {
            return self.temporal_calendar_helper_stub(RuntimeHelperId::TemporalUmmAlQuraYear);
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalUmmAlQuraYear);
        let parameters = self.helper_parameters::<TemporalUmmAlQuraYearParameters>(&mut function);
        emit_umalqura_year_info_by_year(parameters.year, &UMMALQURA_YEARS, &mut function);
        function.instruction(&Instruction::End);
        self.finish_function(function)
    }

    pub(crate) fn compile_temporal_umalqura_epoch_helper(&mut self, real: bool) -> Function {
        if !real {
            return self.temporal_calendar_helper_stub(RuntimeHelperId::TemporalUmmAlQuraEpoch);
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalUmmAlQuraEpoch);
        let parameters = self.helper_parameters::<TemporalUmmAlQuraEpochParameters>(&mut function);
        let year = self.runtime_schema().reserve_i64_local(&mut function);
        let packed = self.runtime_schema().reserve_i64_local(&mut function);
        emit_umalqura_year_info_by_epoch(parameters.epoch, year, &UMMALQURA_YEARS, &mut function);
        packed.store(&mut function);
        year.load(&mut function);
        packed.load(&mut function);
        self.runtime_schema()
            .release_i64_local(packed, &mut function);
        self.runtime_schema().release_i64_local(year, &mut function);
        function.instruction(&Instruction::End);
        self.finish_function(function)
    }

    fn emit_temporal_umalqura_year_info(
        &self,
        year: I64Local,
        packed: I64Local,
        function: &mut Function,
    ) {
        self.runtime_schema()
            .call_helper(
                TemporalUmmAlQuraYearArguments::new(year),
                self.runtime_helper_base()
                    .expect("Temporal calendar arithmetic has a registered helper plan"),
                function,
            )
            .store(packed, function);
    }

    fn emit_temporal_umalqura_epoch_info(
        &self,
        epoch: I64Local,
        year: I64Local,
        packed: I64Local,
        function: &mut Function,
    ) {
        self.runtime_schema()
            .call_helper(
                TemporalUmmAlQuraEpochArguments::new(epoch),
                self.runtime_helper_base()
                    .expect("Temporal calendar arithmetic has a registered helper plan"),
                function,
            )
            .store(year, packed, function);
    }

    fn emit_temporal_umalqura_has_table_year_i32(&self, year: I64Local, function: &mut Function) {
        (year).load(function);
        function.instruction(&Instruction::I64Const(UMMALQURA_YEARS[0].calendar_year()));
        function.instruction(&Instruction::I64GeS);
        (year).load(function);
        function.instruction(&Instruction::I64Const(
            UMMALQURA_YEARS[UMMALQURA_YEARS.len() - 1].calendar_year(),
        ));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I32And);
    }

    /// Regulated month1..12: each previous month contributes29 plus its bit.
    /// The prefix stays on the stack and does not mutate the result pair.
    fn emit_temporal_umalqura_month_prefix_i64(
        &self,
        packed: I64Local,
        month: I64Local,
        function: &mut Function,
    ) {
        (month).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Mul);
        (packed).load(function);
        function.instruction(&Instruction::I64Const(1));
        (month).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Popcnt);
        function.instruction(&Instruction::I64Add);
    }

    fn emit_temporal_umalqura_table_month_length_i64(
        &self,
        packed: I64Local,
        month: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(29));
        (packed).load(function);
        (month).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
    }

    pub(super) fn emit_temporal_umalqura_days_in_month(
        &mut self,
        year: I64Local,
        month: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        let packed = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_umalqura_has_table_year_i32(year, function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_year_info(year, packed, function);
        self.emit_temporal_umalqura_table_month_length_i64(packed, month, function);
        (out).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_islamic_days_in_month(
            TemporalIslamicCalendar::Civil,
            year,
            month,
            out,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(packed, function);
    }

    /// Used only with a proven table year and already regulated month/day.
    fn emit_temporal_umalqura_table_epoch_days(
        &mut self,
        fields: [I64Local; 3],
        out: I64Local,
        function: &mut Function,
    ) {
        let packed = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_umalqura_year_info(fields[0], packed, function);
        (packed).load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64ShrS);
        self.emit_temporal_umalqura_month_prefix_i64(packed, fields[1], function);
        function.instruction(&Instruction::I64Add);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        self.runtime_schema().release_i64_local(packed, function);
    }

    pub(super) fn emit_temporal_umalqura_to_iso(
        &mut self,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_umalqura_has_table_year_i32(fields[0], function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_table_epoch_days(fields, epoch, function);
        self.emit_temporal_civil_from_days(epoch, fields[0], fields[1], fields[2], function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_islamic_to_iso(TemporalIslamicCalendar::Civil, fields, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(epoch, function);
    }

    pub(super) fn emit_temporal_umalqura_project_date(
        &mut self,
        iso: [I64Local; 3],
        result: &TemporalCalendarDateLocals,
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        let packed = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_plain_date_epoch_days(iso[0], iso[1], iso[2], epoch, function);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(UMMALQURA_FIRST_START_DAY));
        function.instruction(&Instruction::I64GeS);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(UMMALQURA_EXCLUSIVE_END_DAY));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_epoch_info(epoch, result.fields[0], packed, function);
        (epoch).load(function);
        (packed).load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64ShrS);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.day_of_year).store(function);
        function.instruction(&Instruction::I64Const(1));
        (result.fields[1]).store(function);
        // Twelve months are bounded; table row selection above is logarithmic.
        for ordinal in 2_i64..=12 {
            (result.day_of_year).load(function);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            (packed).load(function);
            function.instruction(&Instruction::I64Const((1 << (ordinal - 1)) - 1));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Popcnt);
            function.instruction(&Instruction::I64Const(29 * (ordinal - 1)));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64GeS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(ordinal));
            (result.fields[1]).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        (result.day_of_year).load(function);
        self.emit_temporal_umalqura_month_prefix_i64(packed, result.fields[1], function);
        function.instruction(&Instruction::I64Sub);
        (result.fields[2]).store(function);
        self.emit_temporal_umalqura_table_month_length_i64(packed, result.fields[1], function);
        (result.days_in_month).store(function);
        // Native construction proves every row has six or seven long months.
        (packed).load(function);
        function.instruction(&Instruction::I64Const((1 << 12) - 1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Popcnt);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        (result.leap).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_islamic_project_date(
            TemporalIslamicCalendar::Civil,
            iso,
            result,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for local in [packed, epoch] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// The real absent-year owner still performs month/day regulation. This
    /// private lookup saturates only its selection, preserving that owner's
    /// overflow checks and leaving the original ordinal untouched.
    pub(super) fn emit_temporal_umalqura_month_day_regulation_year(
        &self,
        month: I64Local,
        year_out: I64Local,
        function: &mut Function,
    ) {
        for ordinal in 1..12 {
            (month).load(function);
            function.instruction(&Instruction::I64Const(ordinal as i64));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(ummalqura_day30_reference_year(
                ordinal,
            )));
            function.instruction(&Instruction::Else);
        }
        function.instruction(&Instruction::I64Const(ummalqura_day30_reference_year(12)));
        for _ in 1..12 {
            function.instruction(&Instruction::End);
        }
        (year_out).store(function);
    }

    /// Regulated day30 uses the latest eligible long month. Day1..29 uses
    /// 1392 unless its complete date exceeds the required ISO cutoff.
    pub(super) fn emit_temporal_umalqura_month_day_reference(
        &mut self,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_umalqura_month_day_regulation_year(fields[1], fields[0], function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1392));
        (fields[0]).store(function);
        self.emit_temporal_umalqura_table_epoch_days(fields, epoch, function);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(1095));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1391));
        (fields[0]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_umalqura_to_iso(fields, function);
        self.runtime_schema().release_i64_local(epoch, function);
    }
}
