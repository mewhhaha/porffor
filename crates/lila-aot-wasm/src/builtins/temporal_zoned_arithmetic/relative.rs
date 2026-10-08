//! Zone-aware relative Duration mathematics on retained exact epochs.

use super::*;
use crate::builtins::temporal_options::{
    Disambiguation, TemporalOverflow, TemporalRoundingMode, TemporalUnit,
};
use crate::builtins::temporal_zone_provider::TemporalZonedRelativeContextLocals;

use crate::builtins::temporal_duration_methods::{
    CompletedTemporalDurationRecordLocals, CompletedTemporalDurationRoundOptionsLocals,
    TemporalDurationTotalUnitLocals,
};
use crate::builtins::temporal_plain_date_time_methods::ResolvedTemporalDateTimeDifferenceSettings;

const NANOSECONDS_PER_DAY: i64 = 86_400_000_000_000;

/// Each variant carries an already completed option policy. A free local
/// integer cannot stand in for a rounding mode or an unrelated unit domain.
#[derive(Clone, Copy)]
pub(in crate::builtins) enum TemporalZonedRelativeRoundingSettings<'a> {
    Difference(&'a ResolvedTemporalDateTimeDifferenceSettings),
    Duration(&'a CompletedTemporalDurationRoundOptionsLocals),
}

impl TemporalZonedRelativeRoundingSettings<'_> {
    pub(in crate::builtins) fn largest_unit(&self) -> I64Local {
        match self {
            Self::Difference(settings) => settings.largest_unit(),
            Self::Duration(settings) => settings.largest_unit(),
        }
    }
    pub(in crate::builtins) fn smallest_unit(&self) -> I64Local {
        match self {
            Self::Difference(settings) => settings.smallest_unit(),
            Self::Duration(settings) => settings.smallest_unit(),
        }
    }
    pub(in crate::builtins) fn rounding_increment(&self) -> I64Local {
        match self {
            Self::Difference(settings) => settings.rounding_increment(),
            Self::Duration(settings) => settings.rounding_increment(),
        }
    }
    pub(in crate::builtins) fn rounding_mode(&self) -> I64Local {
        match self {
            Self::Difference(settings) => settings.rounding_mode(),
            Self::Duration(settings) => settings.rounding_mode(),
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::builtins) enum TemporalZonedRelativeLargestUnit<'a> {
    Rounding(TemporalZonedRelativeRoundingSettings<'a>),
    Total(&'a TemporalDurationTotalUnitLocals),
}

impl TemporalZonedRelativeLargestUnit<'_> {
    pub(in crate::builtins) fn local(&self) -> I64Local {
        match self {
            Self::Rounding(settings) => settings.largest_unit(),
            Self::Total(unit) => unit.local(),
        }
    }
}

/// Scratch endpoints from ComputeNudgeWindow. They are provisional: named
/// transitions can collapse or reverse a window, and the prescribed second
/// computation has not yet been selected.
struct ProvisionalTemporalZonedNudgeWindowLocals {
    r1: I64Local,
    r2: I64Local,
    start: [I64Local; 2],
    end: [I64Local; 2],
    start_date: [I64Local; 4],
    end_date: [I64Local; 4],
}

impl ProvisionalTemporalZonedNudgeWindowLocals {
    fn reserve(builder: &mut FunctionBuilder<'_>, function: &mut Function) -> Self {
        Self {
            r1: builder.runtime_schema().reserve_i64_local(function),
            r2: builder.runtime_schema().reserve_i64_local(function),
            start: std::array::from_fn(|_| builder.runtime_schema().reserve_i64_local(function)),
            end: std::array::from_fn(|_| builder.runtime_schema().reserve_i64_local(function)),
            start_date: std::array::from_fn(|_| {
                builder.runtime_schema().reserve_i64_local(function)
            }),
            end_date: std::array::from_fn(|_| builder.runtime_schema().reserve_i64_local(function)),
        }
    }

    fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in self
            .end_date
            .into_iter()
            .rev()
            .chain(self.start_date.into_iter().rev())
            .chain(self.end.into_iter().rev())
            .chain(self.start.into_iter().rev())
            .chain([self.r2, self.r1])
        {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}

/// Only final-window validation creates this authority. Rounding and exact
/// division accept it by value, so neither provisional endpoints nor a
/// different destination/sign can enter those consumers.
struct BracketedTemporalZonedNudgeWindowLocals {
    window: ProvisionalTemporalZonedNudgeWindowLocals,
    sign: I64Local,
    destination: [I64Local; 2],
}

impl FunctionBuilder<'_> {
    /// ToInternalDurationRecord on completed input fields. This private child
    /// can mint its parent's proof only after that exact mathematical step.
    fn emit_temporal_zoned_internal_from_duration_record(
        &mut self,
        record: &CompletedTemporalDurationRecordLocals,
        function: &mut Function,
    ) -> TemporalZonedInternalDurationLocals {
        let date = std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let subsecond = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_relative_internal_from_fields(
            record.fields(),
            date,
            seconds,
            subsecond,
            function,
        );
        TemporalZonedInternalDurationLocals {
            date,
            seconds,
            subsecond,
        }
    }
    // A selected Instant is converted to the existing exact epoch span
    // representation; negative seconds use Euclidean division.
    fn emit_temporal_zoned_epoch_span(
        &mut self,
        epoch: &NormalizedTemporalInstantLocals,
        days: I64Local,
        nanos: I64Local,
        function: &mut Function,
    ) {
        (epoch.floor_seconds()).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (days).store(function);
        (epoch.floor_seconds()).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (epoch.nanosecond()).load(function);
        function.instruction(&Instruction::I64Add);
        (nanos).store(function);
        (nanos).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (nanos).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_DAY));
        function.instruction(&Instruction::I64Add);
        (nanos).store(function);
        (days).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (days).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    /// CalendarDateAdd followed by the prescribed compatible inverse. The
    /// optional raw day shift is AddDaysToISODate, not a PlainDate creation.
    /// The retained calendar proof selects its actual native arithmetic.
    fn emit_temporal_zoned_calendar_window_endpoint(
        &mut self,
        iso: &RegulatedTemporalIsoRecordLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        date: [I64Local; 4],
        day_shift: Option<I64Local>,
        days: I64Local,
        nanos: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prepared = self.reserve_temporal_iso_record_result(function);
        let fields: [I64Local; 9] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        for (source, destination) in iso.fields().iter().zip(fields) {
            (*source).load(function);
            (destination).store(function);
        }
        let overflow = self.emit_temporal_constant_overflow(TemporalOverflow::Constrain, function);
        self.emit_temporal_add_calendar_date(
            calendar,
            fields[0],
            fields[1],
            fields[2],
            date[0],
            date[1],
            date[2],
            date[3],
            overflow.local(),
            function,
        )?;
        overflow.release(self, function);
        if let Some(shift) = day_shift {
            let epoch_days = self.runtime_schema().reserve_i64_local(function);
            self.emit_temporal_plain_date_epoch_days(
                fields[0], fields[1], fields[2], epoch_days, function,
            );
            (epoch_days).load(function);
            (shift).load(function);
            function.instruction(&Instruction::I64Add);
            (epoch_days).store(function);
            self.emit_temporal_civil_from_days(
                epoch_days, fields[0], fields[1], fields[2], function,
            );
            self.runtime_schema()
                .release_i64_local(epoch_days, function);
        }
        let completed = CompletedTemporalIsoArithmeticLocals { fields };
        let selected_iso =
            self.emit_temporal_iso_record_from_arithmetic_into(prepared, &completed, function)?;
        completed.release(self, function);
        let coordinate =
            self.emit_temporal_local_coordinate_from_iso_record(&selected_iso, function)?;
        let policy =
            self.emit_temporal_constant_disambiguation(Disambiguation::Compatible, function);
        let epoch =
            self.emit_temporal_get_epoch_nanoseconds_for(zone, &coordinate, &policy, function)?;
        self.emit_temporal_zoned_epoch_span(&epoch, days, nanos, function);
        epoch.release(self, function);
        policy.release(self, function);
        coordinate.release(self, function);
        selected_iso.release(self, function);
        Ok(())
    }

    // This checks the selected final window, after the prescribed recomputation.
    // Stage4 still leaves some real 24-hour-skip windows unresolved (issue 3310).
    // Keep that ownership outside JS completion before unsigned arithmetic.
    fn emit_temporal_zoned_require_window_direction(
        &mut self,
        sign: I64Local,
        start_days: I64Local,
        start_nanos: I64Local,
        end_days: I64Local,
        end_nanos: I64Local,
        function: &mut Function,
    ) {
        let direction = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_relative_span_cmp(
            end_days,
            end_nanos,
            start_days,
            start_nanos,
            direction,
            function,
        );
        (direction).load(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_reject_runtime_semantics(
            lila_ir::RuntimeSemanticRejection::Gap(
                lila_ir::RuntimeSemanticGap::TemporalZonedRoundingWindow,
            ),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(direction, function);
    }

    fn emit_temporal_zoned_compute_nudge_window(
        &mut self,
        date: [I64Local; 4],
        sign_local: I64Local,
        origin_days_local: I64Local,
        origin_nanos_local: I64Local,
        iso: &RegulatedTemporalIsoRecordLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        increment_local: I64Local,
        unit_local: I64Local,
        additional_shift_local: I64Local,
        window: &ProvisionalTemporalZonedNudgeWindowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let r1_local = window.r1;
        let r2_local = window.r2;
        let [start_days_local, start_nanos_local] = window.start;
        let [end_days_local, end_nanos_local] = window.end;
        let start_date = window.start_date;
        let end_date = window.end_date;
        let f = iso.fields();
        self.emit_temporal_relative_compute_nudge_date_durations(
            calendar,
            date,
            sign_local,
            [f[0], f[1], f[2]],
            increment_local,
            unit_local,
            additional_shift_local,
            r1_local,
            r2_local,
            start_date,
            end_date,
            function,
        )?;
        // PR3966 ComputeNudgeWindow at 3d4a6e7124a6878cb5af3132af7e01e01a88317f
        // tests DateDurationSign(startDateDuration), including larger fields.
        // The historical rendered proposal's r1 = 0 predicate is superseded
        // by the issue3316 correction; see temporal-selected-rounding-window.md.
        (start_date[0]).load(function);
        for local in &start_date[1..] {
            (*local).load(function);
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (origin_days_local).load(function);
        (start_days_local).store(function);
        (origin_nanos_local).load(function);
        (start_nanos_local).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_zoned_calendar_window_endpoint(
            iso,
            zone,
            calendar,
            start_date,
            None,
            start_days_local,
            start_nanos_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_zoned_calendar_window_endpoint(
            iso,
            zone,
            calendar,
            end_date,
            None,
            end_days_local,
            end_nanos_local,
            function,
        )?;
        Ok(())
    }
    fn emit_temporal_zoned_require_final_bracket(
        &mut self,
        sign: I64Local,
        start_days: I64Local,
        start_nanos: I64Local,
        dest_days: I64Local,
        dest_nanos: I64Local,
        end_days: I64Local,
        end_nanos: I64Local,
        function: &mut Function,
    ) {
        let near = self.runtime_schema().reserve_i64_local(function);
        let far = self.runtime_schema().reserve_i64_local(function);
        (sign).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_span_cmp(
            start_days,
            start_nanos,
            dest_days,
            dest_nanos,
            near,
            function,
        );
        self.emit_temporal_relative_span_cmp(
            dest_days, dest_nanos, end_days, end_nanos, far, function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_cmp(
            end_days, end_nanos, dest_days, dest_nanos, near, function,
        );
        self.emit_temporal_relative_span_cmp(
            dest_days,
            dest_nanos,
            start_days,
            start_nanos,
            far,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (near).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        (far).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_reject_runtime_semantics(
            lila_ir::RuntimeSemanticRejection::Gap(
                lila_ir::RuntimeSemanticGap::TemporalZonedRoundingWindow,
            ),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(far, function);
        self.runtime_schema().release_i64_local(near, function);
    }

    fn emit_temporal_zoned_complete_nudge_window(
        &mut self,
        window: ProvisionalTemporalZonedNudgeWindowLocals,
        sign: I64Local,
        destination: [I64Local; 2],
        function: &mut Function,
    ) -> BracketedTemporalZonedNudgeWindowLocals {
        self.emit_temporal_zoned_require_final_bracket(
            sign,
            window.start[0],
            window.start[1],
            destination[0],
            destination[1],
            window.end[0],
            window.end[1],
            function,
        );
        self.emit_temporal_zoned_require_window_direction(
            sign,
            window.start[0],
            window.start[1],
            window.end[0],
            window.end[1],
            function,
        );
        BracketedTemporalZonedNudgeWindowLocals {
            window,
            sign,
            destination,
        }
    }

    fn emit_temporal_zoned_nudge_to_calendar_unit(
        &mut self,
        sign_local: I64Local,
        date: [I64Local; 4],
        origin_days_local: I64Local,
        origin_nanos_local: I64Local,
        dest_days_local: I64Local,
        dest_nanos_local: I64Local,
        iso: &RegulatedTemporalIsoRecordLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        increment_local: I64Local,
        unit_local: I64Local,
        mode_local: I64Local,
        nudged_days_local: I64Local,
        nudged_nanos_local: I64Local,
        did_expand_local: I64Local,
        total_bits_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let shift_local = self.runtime_schema().reserve_i64_local(function);
        let window = ProvisionalTemporalZonedNudgeWindowLocals::reserve(self, function);
        let [start_days_local, start_nanos_local] = window.start;
        let [end_days_local, end_nanos_local] = window.end;
        let cmp_local = self.runtime_schema().reserve_i64_local(function);

        function.instruction(&Instruction::I64Const(0));
        (did_expand_local).store(function);
        function.instruction(&Instruction::I64Const(0));
        (shift_local).store(function);
        self.emit_temporal_zoned_compute_nudge_window(
            date,
            sign_local,
            origin_days_local,
            origin_nanos_local,
            iso,
            zone,
            calendar,
            increment_local,
            unit_local,
            shift_local,
            &window,
            function,
        )?;
        // The destination must sit inside the window; otherwise one
        // additional shift expands it outward.
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_span_cmp(
            start_days_local,
            start_nanos_local,
            dest_days_local,
            dest_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shift_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_relative_span_cmp(
            dest_days_local,
            dest_nanos_local,
            end_days_local,
            end_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shift_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_cmp(
            end_days_local,
            end_nanos_local,
            dest_days_local,
            dest_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shift_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_relative_span_cmp(
            dest_days_local,
            dest_nanos_local,
            start_days_local,
            start_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (shift_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (shift_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_zoned_compute_nudge_window(
            date,
            sign_local,
            origin_days_local,
            origin_nanos_local,
            iso,
            zone,
            calendar,
            increment_local,
            unit_local,
            shift_local,
            &window,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        (did_expand_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // ComputeNudgeWindow may return collapsed endpoints. Only the selected
        // final window must prove a nonzero signed span and bracket before any
        // distance becomes unsigned or enters the exact rational division.
        let window = self.emit_temporal_zoned_complete_nudge_window(
            window,
            sign_local,
            [dest_days_local, dest_nanos_local],
            function,
        );
        self.runtime_schema().release_i64_local(cmp_local, function);
        self.emit_temporal_zoned_round_calendar_window(
            window,
            date,
            increment_local,
            mode_local,
            nudged_days_local,
            nudged_nanos_local,
            did_expand_local,
            total_bits_local,
            function,
        );
        // Shift was reserved below the endpoint owner. The consumer releases
        // that owner before this final control local, preserving strict LIFO.
        self.runtime_schema()
            .release_i64_local(shift_local, function);
        Ok(())
    }

    fn emit_temporal_zoned_round_calendar_window(
        &mut self,
        window: BracketedTemporalZonedNudgeWindowLocals,
        date: [I64Local; 4],
        increment_local: I64Local,
        mode_local: I64Local,
        nudged_days_local: I64Local,
        nudged_nanos_local: I64Local,
        did_expand_local: I64Local,
        total_bits_local: I64Local,
        function: &mut Function,
    ) {
        let sign_local = window.sign;
        let [dest_days_local, dest_nanos_local] = window.destination;
        let r1_local = window.window.r1;
        let [start_days_local, start_nanos_local] = window.window.start;
        let [end_days_local, end_nanos_local] = window.window.end;
        let start_date = window.window.start_date;
        let end_date = window.window.end_date;
        let cmp_local = self.runtime_schema().reserve_i64_local(function);
        let umode_local = self.runtime_schema().reserve_i64_local(function);
        let dist_a_days_local = self.runtime_schema().reserve_i64_local(function);
        let dist_a_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let dist_b_days_local = self.runtime_schema().reserve_i64_local(function);
        let dist_b_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let pick_end_local = self.runtime_schema().reserve_i64_local(function);

        // `GetUnsignedRoundingMode`: zero and infinity decide between the
        // window ends, the half modes compare distances first.
        for (mode, positive, negative) in [
            (TemporalRoundingMode::Ceil, 1_i64, 0_i64),
            (TemporalRoundingMode::Floor, 0, 1),
            (TemporalRoundingMode::Expand, 1, 1),
            (TemporalRoundingMode::Trunc, 0, 0),
            (TemporalRoundingMode::HalfCeil, 3, 2),
            (TemporalRoundingMode::HalfFloor, 2, 3),
            (TemporalRoundingMode::HalfExpand, 3, 3),
            (TemporalRoundingMode::HalfTrunc, 2, 2),
            (TemporalRoundingMode::HalfEven, 4, 4),
        ] {
            (mode_local).load(function);
            function.instruction(&Instruction::I64Const(mode.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            (sign_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(positive));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(negative));
            function.instruction(&Instruction::End);
            (umode_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(0));
        (pick_end_local).store(function);
        self.emit_temporal_relative_span_cmp(
            dest_days_local,
            dest_nanos_local,
            end_days_local,
            end_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // `progress = 1`: the destination reached the far end.
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_cmp(
            dest_days_local,
            dest_nanos_local,
            start_days_local,
            start_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // Off the near end, `infinity` takes the far one.
        (umode_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // The half modes weigh the absolute distances to each end.
        (umode_local).load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_span_sub(
            dest_days_local,
            dest_nanos_local,
            start_days_local,
            start_nanos_local,
            dist_a_days_local,
            dist_a_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            end_days_local,
            end_nanos_local,
            dest_days_local,
            dest_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_sub(
            start_days_local,
            start_nanos_local,
            dest_days_local,
            dest_nanos_local,
            dist_a_days_local,
            dist_a_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            dest_days_local,
            dest_nanos_local,
            end_days_local,
            end_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_relative_span_cmp(
            dist_a_days_local,
            dist_a_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            cmp_local,
            function,
        );
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        function.instruction(&Instruction::Else);
        (cmp_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (umode_local).load(function);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // `half-even` takes the end whose `|r|` is odd's neighbor: an odd
        // `|r1|` rounds away from the start.
        (umode_local).load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        (r1_local).load(function);
        (increment_local).load(function);
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (pick_end_local).store(function);
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

        (pick_end_local).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        for (index, local) in end_date.iter().enumerate() {
            (*local).load(function);
            (date[index]).store(function);
        }
        (end_days_local).load(function);
        (nudged_days_local).store(function);
        (end_nanos_local).load(function);
        (nudged_nanos_local).store(function);
        function.instruction(&Instruction::I64Const(1));
        (did_expand_local).store(function);
        function.instruction(&Instruction::Else);
        for (index, local) in start_date.iter().enumerate() {
            (*local).load(function);
            (date[index]).store(function);
        }
        (start_days_local).load(function);
        (nudged_days_local).store(function);
        (start_nanos_local).load(function);
        (nudged_nanos_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // `[[Total]]`: `r1 + progress * increment * sign` left to right.
        (sign_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_span_sub(
            dest_days_local,
            dest_nanos_local,
            start_days_local,
            start_nanos_local,
            dist_a_days_local,
            dist_a_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            end_days_local,
            end_nanos_local,
            start_days_local,
            start_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_span_sub(
            start_days_local,
            start_nanos_local,
            dest_days_local,
            dest_nanos_local,
            dist_a_days_local,
            dist_a_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            start_days_local,
            start_nanos_local,
            end_days_local,
            end_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Accumulate the exact rational before one nearest-even division.
        self.emit_temporal_relative_nudge_total_exact(
            r1_local,
            dist_a_days_local,
            dist_a_nanos_local,
            dist_b_days_local,
            dist_b_nanos_local,
            increment_local,
            sign_local,
            total_bits_local,
            function,
        );

        for local in [
            pick_end_local,
            dist_b_nanos_local,
            dist_b_days_local,
            dist_a_nanos_local,
            dist_a_days_local,
            umode_local,
            cmp_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        window.window.release(self, function);
    }

    fn emit_temporal_zoned_nudge_to_time(
        &mut self,
        sign_local: I64Local,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        iso: &RegulatedTemporalIsoRecordLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        increment_local: I64Local,
        smallest_unit_local: I64Local,
        mode_local: I64Local,
        nudged_days_local: I64Local,
        nudged_nanos_local: I64Local,
        did_expand_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let start_days_local = self.runtime_schema().reserve_i64_local(function);
        let start_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let end_days_local = self.runtime_schema().reserve_i64_local(function);
        let end_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let span_days_local = self.runtime_schema().reserve_i64_local(function);
        let span_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let quantum_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_seconds_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_subsecond_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_sign_local = self.runtime_schema().reserve_i64_local(function);

        self.emit_temporal_zoned_calendar_window_endpoint(
            iso,
            zone,
            calendar,
            date,
            None,
            start_days_local,
            start_nanos_local,
            function,
        )?;
        self.emit_temporal_zoned_calendar_window_endpoint(
            iso,
            zone,
            calendar,
            date,
            Some(sign_local),
            end_days_local,
            end_nanos_local,
            function,
        )?;
        self.emit_temporal_zoned_require_window_direction(
            sign_local,
            start_days_local,
            start_nanos_local,
            end_days_local,
            end_nanos_local,
            function,
        );
        self.emit_temporal_relative_span_sub(
            end_days_local,
            end_nanos_local,
            start_days_local,
            start_nanos_local,
            span_days_local,
            span_nanos_local,
            function,
        );
        self.emit_temporal_duration_unit_quantum(
            smallest_unit_local,
            increment_local,
            quantum_local,
            function,
        );
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_round_seconds(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_round_subsecond(
            seconds_local,
            subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // `beyond` is the rounded time past the day span, renormalized to
        // the truncated convention before its sign is read.
        (seconds_local).load(function);
        (span_days_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (beyond_seconds_local).store(function);
        (subsecond_local).load(function);
        (span_nanos_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (beyond_subsecond_local).store(function);
        self.emit_temporal_duration_renormalize(
            beyond_seconds_local,
            beyond_subsecond_local,
            function,
        );
        self.emit_temporal_relative_time_sign(
            beyond_seconds_local,
            beyond_subsecond_local,
            beyond_sign_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        (sign_local).load(function);
        function.instruction(&Instruction::I64Sub);
        (beyond_sign_local).load(function);
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (did_expand_local).store(function);
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_round_seconds(
            beyond_seconds_local,
            beyond_subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_round_subsecond(
            beyond_seconds_local,
            beyond_subsecond_local,
            quantum_local,
            mode_local,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (beyond_seconds_local).load(function);
        (seconds_local).store(function);
        (beyond_subsecond_local).load(function);
        (subsecond_local).store(function);
        (date[3]).load(function);
        (sign_local).load(function);
        function.instruction(&Instruction::I64Add);
        (date[3]).store(function);
        (end_days_local).load(function);
        (nudged_days_local).store(function);
        (end_nanos_local).load(function);
        (nudged_nanos_local).store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        (did_expand_local).store(function);
        (start_days_local).load(function);
        (nudged_days_local).store(function);
        (start_nanos_local).load(function);
        (nudged_nanos_local).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_relative_span_add_duration(
            nudged_days_local,
            nudged_nanos_local,
            seconds_local,
            subsecond_local,
            function,
        );

        for local in [
            beyond_sign_local,
            beyond_subsecond_local,
            beyond_seconds_local,
            quantum_local,
            span_nanos_local,
            span_days_local,
            end_nanos_local,
            end_days_local,
            start_nanos_local,
            start_days_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    fn emit_temporal_zoned_bubble(
        &mut self,
        sign_local: I64Local,
        date: [I64Local; 4],
        seconds_local: I64Local,
        subsecond_local: I64Local,
        nudged_days_local: I64Local,
        nudged_nanos_local: I64Local,
        iso: &RegulatedTemporalIsoRecordLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        largest_unit_local: I64Local,
        start_unit_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let done_local = self.runtime_schema().reserve_i64_local(function);
        let end_years_local = self.runtime_schema().reserve_i64_local(function);
        let end_months_local = self.runtime_schema().reserve_i64_local(function);
        let end_weeks_local = self.runtime_schema().reserve_i64_local(function);
        let end_days_local = self.runtime_schema().reserve_i64_local(function);
        let end_span_days_local = self.runtime_schema().reserve_i64_local(function);
        let end_span_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_days_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_nanos_local = self.runtime_schema().reserve_i64_local(function);
        let beyond_sign_local = self.runtime_schema().reserve_i64_local(function);

        function.instruction(&Instruction::I64Const(0));
        (done_local).store(function);
        // Units bubble from just below `start_unit` down to `largest_unit`;
        // with three calendar units the cascade is explicit, descending.
        for unit in [TemporalUnit::Week, TemporalUnit::Month, TemporalUnit::Year] {
            (done_local).load(function);
            function.instruction(&Instruction::I64Eqz);
            (largest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::I32And);
            (start_unit_local).load(function);
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            // Week only bubbles toward a week largestUnit.
            function.instruction(&Instruction::I64Const(unit.code()));
            function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
            function.instruction(&Instruction::I64Ne);
            (largest_unit_local).load(function);
            function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            (date[0]).load(function);
            (end_years_local).store(function);
            (date[1]).load(function);
            (end_months_local).store(function);
            (date[2]).load(function);
            (end_weeks_local).store(function);
            function.instruction(&Instruction::I64Const(0));
            (end_days_local).store(function);
            // The bubbling unit increments by the sign; larger legs stay,
            // smaller legs clear.
            match unit {
                TemporalUnit::Year => {
                    (end_years_local).load(function);
                    (sign_local).load(function);
                    function.instruction(&Instruction::I64Add);
                    (end_years_local).store(function);
                    function.instruction(&Instruction::I64Const(0));
                    (end_months_local).store(function);
                    function.instruction(&Instruction::I64Const(0));
                    (end_weeks_local).store(function);
                }
                TemporalUnit::Month => {
                    (end_months_local).load(function);
                    (sign_local).load(function);
                    function.instruction(&Instruction::I64Add);
                    (end_months_local).store(function);
                    function.instruction(&Instruction::I64Const(0));
                    (end_weeks_local).store(function);
                }
                TemporalUnit::Week => {
                    (end_weeks_local).load(function);
                    (sign_local).load(function);
                    function.instruction(&Instruction::I64Add);
                    (end_weeks_local).store(function);
                }
                TemporalUnit::Day
                | TemporalUnit::Hour
                | TemporalUnit::Minute
                | TemporalUnit::Second
                | TemporalUnit::Millisecond
                | TemporalUnit::Microsecond
                | TemporalUnit::Nanosecond => unreachable!("closed calendar bubble cascade"),
            }
            self.emit_temporal_zoned_calendar_window_endpoint(
                iso,
                zone,
                calendar,
                [
                    end_years_local,
                    end_months_local,
                    end_weeks_local,
                    end_days_local,
                ],
                None,
                end_span_days_local,
                end_span_nanos_local,
                function,
            )?;
            self.emit_temporal_relative_span_sub(
                nudged_days_local,
                nudged_nanos_local,
                end_span_days_local,
                end_span_nanos_local,
                beyond_days_local,
                beyond_nanos_local,
                function,
            );
            // The difference normalizes its nanosecond leg nonnegative, so
            // the days decide the sign unless they are zero.
            function.instruction(&Instruction::I64Const(0));
            (beyond_sign_local).store(function);
            (beyond_days_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(-1));
            (beyond_sign_local).store(function);
            function.instruction(&Instruction::Else);
            (beyond_days_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            (beyond_days_local).load(function);
            function.instruction(&Instruction::I64Eqz);
            (beyond_nanos_local).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(1));
            (beyond_sign_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(0));
            (sign_local).load(function);
            function.instruction(&Instruction::I64Sub);
            (beyond_sign_local).load(function);
            function.instruction(&Instruction::I64Ne);
            self.open_frame(ControlFrameKind::If, function);
            (end_years_local).load(function);
            (date[0]).store(function);
            (end_months_local).load(function);
            (date[1]).store(function);
            (end_weeks_local).load(function);
            (date[2]).store(function);
            (end_days_local).load(function);
            (date[3]).store(function);
            function.instruction(&Instruction::I64Const(0));
            (seconds_local).store(function);
            function.instruction(&Instruction::I64Const(0));
            (subsecond_local).store(function);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            (done_local).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        for local in [
            beyond_sign_local,
            beyond_nanos_local,
            beyond_days_local,
            end_span_nanos_local,
            end_span_days_local,
            end_days_local,
            end_weeks_local,
            end_months_local,
            end_years_local,
            done_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// RoundRelativeDuration on a completed DifferenceZonedDateTime record.
    /// Every calendar boundary is interpreted through the retained zone.
    pub(in crate::builtins) fn emit_temporal_round_zoned_relative_duration(
        &mut self,
        origin: &NormalizedTemporalInstantLocals,
        dest: &NormalizedTemporalInstantLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        duration: &mut TemporalZonedInternalDurationLocals,
        settings: TemporalZonedRelativeRoundingSettings<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let origin_days = self.runtime_schema().reserve_i64_local(function);
        let origin_nanos = self.runtime_schema().reserve_i64_local(function);
        let dest_days = self.runtime_schema().reserve_i64_local(function);
        let dest_nanos = self.runtime_schema().reserve_i64_local(function);
        let nudged_days = self.runtime_schema().reserve_i64_local(function);
        let nudged_nanos = self.runtime_schema().reserve_i64_local(function);
        let sign = self.runtime_schema().reserve_i64_local(function);
        let expanded = self.runtime_schema().reserve_i64_local(function);
        let total_ignored = self.runtime_schema().reserve_i64_local(function);
        let start_unit = self.runtime_schema().reserve_i64_local(function);
        let snapshot = self.emit_temporal_zone_snapshot(zone, origin, function)?;
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        self.emit_temporal_zoned_epoch_span(origin, origin_days, origin_nanos, function);
        self.emit_temporal_zoned_epoch_span(dest, dest_days, dest_nanos, function);
        self.emit_temporal_relative_internal_sign(
            duration.date,
            duration.seconds,
            duration.subsecond,
            sign,
            function,
        );
        (settings.smallest_unit()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_zoned_nudge_to_calendar_unit(
            sign,
            duration.date,
            origin_days,
            origin_nanos,
            dest_days,
            dest_nanos,
            &iso,
            zone,
            calendar,
            settings.rounding_increment(),
            settings.smallest_unit(),
            settings.rounding_mode(),
            nudged_days,
            nudged_nanos,
            expanded,
            total_ignored,
            function,
        )?;
        for local in [duration.seconds, duration.subsecond] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        function.instruction(&Instruction::Else);
        self.emit_temporal_zoned_nudge_to_time(
            sign,
            duration.date,
            duration.seconds,
            duration.subsecond,
            &iso,
            zone,
            calendar,
            settings.rounding_increment(),
            settings.smallest_unit(),
            settings.rounding_mode(),
            nudged_days,
            nudged_nanos,
            expanded,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (expanded).load(function);
        function.instruction(&Instruction::I32WrapI64);
        (settings.smallest_unit()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        (settings.smallest_unit()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (settings.smallest_unit()).load(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::End);
        (start_unit).store(function);
        self.emit_temporal_zoned_bubble(
            sign,
            duration.date,
            duration.seconds,
            duration.subsecond,
            nudged_days,
            nudged_nanos,
            &iso,
            zone,
            calendar,
            settings.largest_unit(),
            start_unit,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        iso.release(self, function);
        snapshot.release(self, function);
        for local in [
            start_unit,
            total_ignored,
            expanded,
            sign,
            nudged_nanos,
            nudged_days,
            dest_nanos,
            dest_days,
            origin_nanos,
            origin_days,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// TemporalDurationFromInternal with date-category largestUnit changed
    /// to hour. Calendar days are copied separately, never scaled as 24h.
    pub(in crate::builtins) fn emit_temporal_zoned_balance_internal(
        &mut self,
        duration: &TemporalZonedInternalDurationLocals,
        largest: TemporalZonedRelativeLargestUnit<'_>,
        fields: &crate::builtins::temporal_duration::TemporalDurationFields,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_largest = self.runtime_schema().reserve_i64_local(function);
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let subsecond = self.runtime_schema().reserve_i64_local(function);
        (largest.local()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::Else);
        (largest.local()).load(function);
        function.instruction(&Instruction::End);
        (time_largest).store(function);
        for (source, destination) in [(duration.seconds, seconds), (duration.subsecond, subsecond)]
        {
            (source).load(function);
            (destination).store(function);
        }
        self.emit_temporal_duration_balance(seconds, subsecond, time_largest, fields, function)?;
        for (source, unit) in duration.date.into_iter().zip([
            TemporalUnit::Year,
            TemporalUnit::Month,
            TemporalUnit::Week,
            TemporalUnit::Day,
        ]) {
            (source).load(function);
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::I64ReinterpretF64);
            (fields.number_bits(unit)).store(function);
        }
        for local in [subsecond, seconds, time_largest] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    pub(in crate::builtins) fn emit_temporal_duration_compare_zoned_relative(
        &mut self,
        one: &CompletedTemporalDurationRecordLocals,
        two: &CompletedTemporalDurationRecordLocals,
        relative: &TemporalZonedRelativeContextLocals,
        out: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let first = self.emit_temporal_zoned_internal_from_duration_record(one, function);
        let second = self.emit_temporal_zoned_internal_from_duration_record(two, function);
        let overflow = self.emit_temporal_constant_overflow(TemporalOverflow::Constrain, function);
        let after_one = self.emit_temporal_add_zoned_internal_duration(
            relative.instant(),
            relative.zone(),
            relative.calendar(),
            &first,
            &overflow,
            function,
        )?;
        let after_two = self.emit_temporal_add_zoned_internal_duration(
            relative.instant(),
            relative.zone(),
            relative.calendar(),
            &second,
            &overflow,
            function,
        )?;
        self.emit_temporal_relative_span_cmp(
            after_one.floor_seconds(),
            after_one.nanosecond(),
            after_two.floor_seconds(),
            after_two.nanosecond(),
            out,
            function,
        );
        after_two.release(self, function);
        after_one.release(self, function);
        overflow.release(self, function);
        second.release(self, function);
        first.release(self, function);
        Ok(())
    }

    pub(in crate::builtins) fn emit_temporal_duration_round_zoned_relative(
        &mut self,
        record: &CompletedTemporalDurationRecordLocals,
        relative: &TemporalZonedRelativeContextLocals,
        options: &CompletedTemporalDurationRoundOptionsLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let original = self.emit_temporal_zoned_internal_from_duration_record(record, function);
        let overflow = self.emit_temporal_constant_overflow(TemporalOverflow::Constrain, function);
        let dest = self.emit_temporal_add_zoned_internal_duration(
            relative.instant(),
            relative.zone(),
            relative.calendar(),
            &original,
            &overflow,
            function,
        )?;
        let settings = TemporalZonedRelativeRoundingSettings::Duration(options);
        (options.largest_unit()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        let time_date: [I64Local; 4] =
            std::array::from_fn(|_| self.runtime_schema().reserve_i64_local(function));
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let subsecond = self.runtime_schema().reserve_i64_local(function);
        let quantum = self.runtime_schema().reserve_i64_local(function);
        for local in time_date {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.emit_temporal_zoned_elapsed_difference(
            relative.instant(),
            &dest,
            seconds,
            subsecond,
            function,
        );
        self.emit_temporal_duration_unit_quantum(
            options.smallest_unit(),
            options.rounding_increment(),
            quantum,
            function,
        );
        (options.smallest_unit()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Second.code()));
        function.instruction(&Instruction::I64LeS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_duration_round_seconds(
            seconds,
            subsecond,
            quantum,
            options.rounding_mode(),
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_round_subsecond(
            seconds,
            subsecond,
            quantum,
            options.rounding_mode(),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(quantum, function);
        let completed = TemporalZonedInternalDurationLocals {
            date: time_date,
            seconds,
            subsecond,
        };
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_zoned_balance_internal(
            &completed,
            TemporalZonedRelativeLargestUnit::Rounding(settings),
            &fields,
            function,
        )?;
        self.emit_create_temporal_duration(&fields, function)?;
        self.release_temporal_duration_field_locals(fields, function);
        completed.release(self, function);
        function.instruction(&Instruction::Else);
        let mut difference = self.emit_temporal_zoned_date_difference(
            relative.instant(),
            &dest,
            relative.zone(),
            relative.calendar(),
            TemporalZonedRelativeLargestUnit::Rounding(settings),
            function,
        )?;
        (options.smallest_unit()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::I64Ne);
        (options.rounding_increment()).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_round_zoned_relative_duration(
            relative.instant(),
            &dest,
            relative.zone(),
            relative.calendar(),
            &mut difference,
            settings,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let fields = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_zoned_balance_internal(
            &difference,
            TemporalZonedRelativeLargestUnit::Rounding(settings),
            &fields,
            function,
        )?;
        self.emit_create_temporal_duration(&fields, function)?;
        self.release_temporal_duration_field_locals(fields, function);
        difference.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        dest.release(self, function);
        overflow.release(self, function);
        original.release(self, function);
        Ok(())
    }

    /// TotalTimeDuration on a completed elapsed pair, accumulating integer
    /// nanoseconds before one division. Operands are copied scratch values.
    fn emit_temporal_zoned_total_time(
        &mut self,
        seconds: I64Local,
        subsecond: I64Local,
        unit: &TemporalDurationTotalUnitLocals,
        function: &mut Function,
    ) {
        let whole = self.runtime_schema().reserve_i64_local(function);
        let remainder = self.runtime_schema().reserve_i64_local(function);
        let negative = self.runtime_schema().reserve_i64_local(function);
        let high = self.runtime_schema().reserve_i64_local(function);
        let low = self.runtime_schema().reserve_i64_local(function);
        let divisor = self.runtime_schema().reserve_i64_local(function);
        let total = self.runtime_schema().reserve_i64_local(function);
        for (source, dest) in [(seconds, whole), (subsecond, remainder)] {
            (source).load(function);
            (dest).store(function);
        }
        self.emit_temporal_relative_abs_time(whole, remainder, negative, function);
        self.emit_temporal_relative_seconds_to_u128(whole, remainder, high, low, function);
        function.instruction(&Instruction::I64Const(1));
        (divisor).store(function);
        for (candidate, length) in [
            (TemporalUnit::Hour, 3_600_000_000_000),
            (TemporalUnit::Minute, 60_000_000_000),
            (TemporalUnit::Second, 1_000_000_000),
            (TemporalUnit::Millisecond, 1_000_000),
            (TemporalUnit::Microsecond, 1_000),
        ] {
            (unit.local()).load(function);
            function.instruction(&Instruction::I64Const(candidate.code()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            function.instruction(&Instruction::I64Const(length));
            (divisor).store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_u128_div_to_f64(high, low, divisor, total, function);
        (negative).load(function);
        function.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, function);
        (total).load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        (total).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().value().set_number(total, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        for local in [total, divisor, low, high, negative, remainder, whole] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// DifferenceZonedDateTimeWithTotal's date-unit branch. Time-unit totals
    /// use only the selected exact elapsed epochs and never inspect a window.
    pub(in crate::builtins) fn emit_temporal_total_zoned_relative_duration(
        &mut self,
        origin: &NormalizedTemporalInstantLocals,
        dest: &NormalizedTemporalInstantLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        unit: &TemporalDurationTotalUnitLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        (unit.local()).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::I64GeS);
        self.open_frame(ControlFrameKind::If, function);
        let seconds = self.runtime_schema().reserve_i64_local(function);
        let subsecond = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_zoned_elapsed_difference(origin, dest, seconds, subsecond, function);
        self.emit_temporal_zoned_total_time(seconds, subsecond, unit, function);
        self.runtime_schema().release_i64_local(subsecond, function);
        self.runtime_schema().release_i64_local(seconds, function);
        function.instruction(&Instruction::Else);
        let difference = self.emit_temporal_zoned_date_difference(
            origin,
            dest,
            zone,
            calendar,
            TemporalZonedRelativeLargestUnit::Total(unit),
            function,
        )?;
        let origin_days = self.runtime_schema().reserve_i64_local(function);
        let origin_nanos = self.runtime_schema().reserve_i64_local(function);
        let dest_days = self.runtime_schema().reserve_i64_local(function);
        let dest_nanos = self.runtime_schema().reserve_i64_local(function);
        let nudged_days = self.runtime_schema().reserve_i64_local(function);
        let nudged_nanos = self.runtime_schema().reserve_i64_local(function);
        let sign = self.runtime_schema().reserve_i64_local(function);
        let expanded = self.runtime_schema().reserve_i64_local(function);
        let total = self.runtime_schema().reserve_i64_local(function);
        let one = self.runtime_schema().reserve_i64_local(function);
        let trunc = self.runtime_schema().reserve_i64_local(function);
        let snapshot = self.emit_temporal_zone_snapshot(zone, origin, function)?;
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        self.emit_temporal_zoned_epoch_span(origin, origin_days, origin_nanos, function);
        self.emit_temporal_zoned_epoch_span(dest, dest_days, dest_nanos, function);
        self.emit_temporal_relative_internal_sign(
            difference.date,
            difference.seconds,
            difference.subsecond,
            sign,
            function,
        );
        function.instruction(&Instruction::I64Const(1));
        (one).store(function);
        function.instruction(&Instruction::I64Const(TemporalRoundingMode::Trunc.code()));
        (trunc).store(function);
        self.emit_temporal_zoned_nudge_to_calendar_unit(
            sign,
            difference.date,
            origin_days,
            origin_nanos,
            dest_days,
            dest_nanos,
            &iso,
            zone,
            calendar,
            one,
            unit.local(),
            trunc,
            nudged_days,
            nudged_nanos,
            expanded,
            total,
            function,
        )?;
        self.completion().value().set_number(total, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        iso.release(self, function);
        snapshot.release(self, function);
        for local in [
            trunc,
            one,
            total,
            expanded,
            sign,
            nudged_nanos,
            nudged_days,
            dest_nanos,
            dest_days,
            origin_nanos,
            origin_days,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        difference.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(in crate::builtins) fn emit_temporal_duration_total_zoned_relative(
        &mut self,
        record: &CompletedTemporalDurationRecordLocals,
        relative: &TemporalZonedRelativeContextLocals,
        unit: &TemporalDurationTotalUnitLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let duration = self.emit_temporal_zoned_internal_from_duration_record(record, function);
        let overflow = self.emit_temporal_constant_overflow(TemporalOverflow::Constrain, function);
        let dest = self.emit_temporal_add_zoned_internal_duration(
            relative.instant(),
            relative.zone(),
            relative.calendar(),
            &duration,
            &overflow,
            function,
        )?;
        self.emit_temporal_total_zoned_relative_duration(
            relative.instant(),
            &dest,
            relative.zone(),
            relative.calendar(),
            unit,
            function,
        )?;
        dest.release(self, function);
        overflow.release(self, function);
        duration.release(self, function);
        Ok(())
    }
}
