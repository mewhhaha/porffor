//! Temporal policy remains compiled Wasm; the host supplies certified data only.

use super::responses::CheckedTemporalPossibleEpochsLocals;
use super::*;

#[derive(Clone, Copy)]
enum CandidateEnd {
    First,
    Last,
}

/// Parser-owned field flags, distinct from user options. Their actual grammar
/// branches are validated before entering the closed interpretation dispatch.
pub(crate) enum TemporalZonedParseFlags {
    String {
        has_time: I64Local,
        offset_kind: I64Local,
        offset_has_seconds: I64Local,
    },
    Bag {
        offset_present: I64Local,
    },
}

impl FunctionBuilder<'_> {
    fn emit_temporal_complete_selected_instant(
        &mut self,
        output: PreparedTemporalInstantLocals,
        function: &mut Function,
    ) -> Result<NormalizedTemporalInstantLocals, EmitError> {
        let value =
            self.emit_temporal_epoch_nanoseconds_bigint(output.seconds, output.nano, function);
        let epoch = self.emit_temporal_instant_validated_epoch(&value, function)?;
        value.clear(function);
        self.runtime_schema()
            .release_i64_local(output.nano, function);
        self.runtime_schema()
            .release_i64_local(output.seconds, function);
        Ok(NormalizedTemporalInstantLocals::from_epoch(epoch))
    }

    pub(in crate::builtins) fn emit_temporal_parsed_zoned_epoch_into(
        &mut self,
        output: PreparedTemporalInstantLocals,
        iso: &RegulatedTemporalIsoRecordLocals,
        zone: &ResolvedTemporalZoneLocals,
        supplied: &TemporalOffsetNanosecondsLocals,
        policies: &TemporalZonedOptionsLocals,
        flags: TemporalZonedParseFlags,
        function: &mut Function,
    ) -> Result<NormalizedTemporalInstantLocals, EmitError> {
        let has_time = self.runtime_schema().reserve_i64_local(function);
        let offset_kind = self.runtime_schema().reserve_i64_local(function);
        let offset_has_seconds = self.runtime_schema().reserve_i64_local(function);
        match flags {
            TemporalZonedParseFlags::String {
                has_time: parsed_time,
                offset_kind: parsed_offset,
                offset_has_seconds: parsed_seconds,
            } => {
                for (source, destination) in [
                    (parsed_time, has_time),
                    (parsed_offset, offset_kind),
                    (parsed_seconds, offset_has_seconds),
                ] {
                    (source).load(function);
                    (destination).store(function);
                }
            }
            TemporalZonedParseFlags::Bag { offset_present } => {
                function.instruction(&Instruction::I64Const(1));
                (has_time).store(function);
                (offset_present).load(function);
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::I64Mul);
                (offset_kind).store(function);
                function.instruction(&Instruction::I64Const(1));
                (offset_has_seconds).store(function);
            }
        }
        for (local, maximum) in [(has_time, 1), (offset_kind, 2), (offset_has_seconds, 1)] {
            (local).load(function);
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtU);
            self.emit_temporal_provider_corruption_if_i32(function);
        }
        let days = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_plain_date_epoch_days(
            iso.fields()[0],
            iso.fields()[1],
            iso.fields()[2],
            days,
            function,
        );
        (days).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        for (field, scale) in [
            (iso.fields()[3], 3600),
            (iso.fields()[4], 60),
            (iso.fields()[5], 1),
        ] {
            (field).load(function);
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
        }
        (output.seconds).store(function);
        // A syntactically valid six-digit year can exceed the provider's
        // contextual domain. Every offset is strictly less than one day, and
        // certified gap endpoints are within one day of their local window.
        // Beyond Instant +/- two days every prescribed candidate/endpoint or
        // exact-offset path necessarily throws RangeError. Emit that semantic
        // outcome after options, before the trusted coordinate constructor.
        let context_limit = 8_640_000_000_000 + 2 * 86_400;
        (output.seconds).load(function);
        function.instruction(&Instruction::I64Const(-context_limit));
        function.instruction(&Instruction::I64LtS);
        (output.seconds).load(function);
        function.instruction(&Instruction::I64Const(context_limit));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_instant_range_error(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(days, function);
        (has_time).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let date = self.emit_temporal_iso_date_from_record(iso, function);
        let start = self.emit_temporal_get_start_of_day(zone, &date, function)?;
        self.emit_temporal_copy_checked_instant(&start, &output, function);
        start.release(self, function);
        date.release(self, function);
        function.instruction(&Instruction::Else);
        for (code, behavior) in [
            (0, TemporalOffsetBehavior::Wall),
            (1, TemporalOffsetBehavior::Exact),
        ] {
            (offset_kind).load(function);
            function.instruction(&Instruction::I64Const(code));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, function);
            let instant = self.emit_temporal_interpret_iso_date_time_offset(
                iso,
                zone,
                behavior,
                supplied,
                policies.disambiguation(),
                policies.offset(),
                TemporalOffsetMatchBehavior::MatchExactly,
                function,
            )?;
            self.emit_temporal_copy_checked_instant(&instant, &output, function);
            instant.release(self, function);
            function.instruction(&Instruction::Else);
        }
        (offset_has_seconds).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        for match_behavior in [
            TemporalOffsetMatchBehavior::MatchMinutes,
            TemporalOffsetMatchBehavior::MatchExactly,
        ] {
            let instant = self.emit_temporal_interpret_iso_date_time_offset(
                iso,
                zone,
                TemporalOffsetBehavior::Option,
                supplied,
                policies.disambiguation(),
                policies.offset(),
                match_behavior,
                function,
            )?;
            self.emit_temporal_copy_checked_instant(&instant, &output, function);
            instant.release(self, function);
            if matches!(match_behavior, TemporalOffsetMatchBehavior::MatchMinutes) {
                function.instruction(&Instruction::Else);
            }
        }
        for _ in 0..4 {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        for local in [offset_has_seconds, offset_kind, has_time] {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.emit_temporal_complete_selected_instant(output, function)
    }

    fn emit_temporal_copy_checked_instant(
        &self,
        instant: &NormalizedTemporalInstantLocals,
        output: &PreparedTemporalInstantLocals,
        function: &mut Function,
    ) {
        for (source, destination) in [
            (instant.floor_seconds(), output.seconds),
            (instant.nanosecond(), output.nano),
        ] {
            (source).load(function);
            (destination).store(function);
        }
    }

    pub(in crate::builtins) fn emit_temporal_get_epoch_nanoseconds_for(
        &mut self,
        zone: &ResolvedTemporalZoneLocals,
        local: &TemporalLocalCoordinateLocals,
        disambiguation: &TemporalDisambiguationLocals,
        function: &mut Function,
    ) -> Result<NormalizedTemporalInstantLocals, EmitError> {
        let output = self.reserve_temporal_instant_result(function);
        let possible = self.emit_temporal_checked_possible_epochs(zone, local, function)?;
        self.emit_temporal_disambiguate_into(
            &output,
            &possible,
            zone,
            local,
            disambiguation,
            function,
        )?;
        possible.release(self, function);
        self.emit_temporal_complete_selected_instant(output, function)
    }
    pub(in crate::builtins) fn emit_temporal_get_start_of_day(
        &mut self,
        zone: &ResolvedTemporalZoneLocals,
        date: &TemporalIsoDateLocals,
        function: &mut Function,
    ) -> Result<NormalizedTemporalInstantLocals, EmitError> {
        let output = self.reserve_temporal_instant_result(function);
        let local = self.emit_temporal_midnight_coordinate_from_date(date, function);
        let possible = self.emit_temporal_checked_possible_epochs(zone, &local, function)?;
        (possible.kind()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // The immutable catalogue certificate proves this boundary is the
        // globally first valid local coordinate after midnight's empty gap,
        // and its inverse is sole. PR3966 explicitly validates that result.
        (possible.transition()).load(function);
        (output.seconds).store(function);
        function.instruction(&Instruction::I64Const(0));
        (output.nano).store(function);
        self.emit_temporal_require_instant_pair(output.seconds, output.nano, function)?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_candidate_end_into(
            &possible,
            CandidateEnd::First,
            output.seconds,
            output.nano,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        possible.release(self, function);
        local.release(self, function);
        self.emit_temporal_complete_selected_instant(output, function)
    }
    fn emit_temporal_candidate_end_into(
        &mut self,
        possible: &CheckedTemporalPossibleEpochsLocals,
        end: CandidateEnd,
        seconds: I64Local,
        nano: I64Local,
        function: &mut Function,
    ) {
        let index = self.runtime_schema().reserve_i64_local(function);
        match end {
            CandidateEnd::First => {
                function.instruction(&Instruction::I64Const(0));
            }
            CandidateEnd::Last => {
                (possible.count()).load(function);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
            }
        }
        (index).store(function);
        self.emit_temporal_candidate_word(possible, index, 0, seconds, function);
        self.emit_temporal_candidate_word(possible, index, 8, nano, function);
        self.runtime_schema().release_i64_local(index, function);
    }
    fn emit_temporal_disambiguate_into(
        &mut self,
        output: &PreparedTemporalInstantLocals,
        possible: &CheckedTemporalPossibleEpochsLocals,
        zone: &ResolvedTemporalZoneLocals,
        local: &TemporalLocalCoordinateLocals,
        disambiguation: &TemporalDisambiguationLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        (possible.kind()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (disambiguation.local()).load(function);
        function.instruction(&Instruction::I64Const(Disambiguation::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_disambiguation_range_error(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let before_seconds = self.runtime_schema().reserve_i64_local(function);
        let before_nano = self.runtime_schema().reserve_i64_local(function);
        let after_nano = self.runtime_schema().reserve_i64_local(function);
        let shifted_seconds = self.runtime_schema().reserve_i64_local(function);
        let shifted_nano = self.runtime_schema().reserve_i64_local(function);
        (possible.transition()).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (before_seconds).store(function);
        function.instruction(&Instruction::I64Const(999_999_999));
        (before_nano).store(function);
        function.instruction(&Instruction::I64Const(0));
        (after_nano).store(function);
        // These are exactly the sole inverses of the globally nearest local
        // endpoints. Their GetPossible checks precede offset use in the spec.
        self.emit_temporal_require_instant_pair(before_seconds, before_nano, function)?;
        self.emit_temporal_require_instant_pair(possible.transition(), after_nano, function)?;
        (local.seconds).load(function);
        (possible.after()).load(function);
        (possible.before()).load(function);
        function.instruction(&Instruction::I64Sub);
        (disambiguation.local()).load(function);
        function.instruction(&Instruction::I64Const(Disambiguation::Earlier.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (shifted_seconds).store(function);
        (local.nano).load(function);
        (shifted_nano).store(function);
        self.emit_temporal_require_local_coordinate_context(
            shifted_seconds,
            shifted_nano,
            function,
        );
        let shifted = TemporalLocalCoordinateLocals {
            seconds: shifted_seconds,
            nano: shifted_nano,
        };
        let after_possible =
            self.emit_temporal_checked_possible_epochs(zone, &shifted, function)?;
        (after_possible.kind()).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.emit_temporal_provider_corruption_if_i32(function);
        (disambiguation.local()).load(function);
        function.instruction(&Instruction::I64Const(Disambiguation::Earlier.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_candidate_end_into(
            &after_possible,
            CandidateEnd::First,
            output.seconds,
            output.nano,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_candidate_end_into(
            &after_possible,
            CandidateEnd::Last,
            output.seconds,
            output.nano,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        after_possible.release(self, function);
        shifted.release(self, function);
        for slot in [after_nano, before_nano, before_seconds] {
            self.runtime_schema().release_i64_local(slot, function);
        }
        function.instruction(&Instruction::Else);
        (possible.count()).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        (disambiguation.local()).load(function);
        function.instruction(&Instruction::I64Const(Disambiguation::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_disambiguation_range_error(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (disambiguation.local()).load(function);
        function.instruction(&Instruction::I64Const(Disambiguation::Later.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_candidate_end_into(
            possible,
            CandidateEnd::Last,
            output.seconds,
            output.nano,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_candidate_end_into(
            possible,
            CandidateEnd::First,
            output.seconds,
            output.nano,
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    fn emit_temporal_disambiguation_range_error(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            ZonedDateTimeOptionKey::Disambiguation.range_error(),
            function,
        )?;
        Ok(())
    }

    pub(in crate::builtins) fn emit_temporal_interpret_iso_date_time_offset(
        &mut self,
        iso: &RegulatedTemporalIsoRecordLocals,
        zone: &ResolvedTemporalZoneLocals,
        behavior: TemporalOffsetBehavior,
        supplied: &TemporalOffsetNanosecondsLocals,
        disambiguation: &TemporalDisambiguationLocals,
        offset_option: &TemporalOffsetOptionLocals,
        match_behavior: TemporalOffsetMatchBehavior,
        function: &mut Function,
    ) -> Result<NormalizedTemporalInstantLocals, EmitError> {
        let output = self.reserve_temporal_instant_result(function);
        let local = self.emit_temporal_local_coordinate_from_iso_record(iso, function)?;
        match behavior {
            TemporalOffsetBehavior::Wall => self.emit_temporal_policy_wall_into(
                &output,
                zone,
                &local,
                disambiguation,
                function,
            )?,
            TemporalOffsetBehavior::Exact => {
                self.emit_temporal_policy_exact_into(&output, &local, supplied, function)?
            }
            TemporalOffsetBehavior::Option => {
                (offset_option.local()).load(function);
                function.instruction(&Instruction::I64Const(OffsetOption::Ignore.code()));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_temporal_policy_wall_into(
                    &output,
                    zone,
                    &local,
                    disambiguation,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                (offset_option.local()).load(function);
                function.instruction(&Instruction::I64Const(OffsetOption::Use.code()));
                function.instruction(&Instruction::I64Eq);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_temporal_policy_exact_into(&output, &local, supplied, function)?;
                function.instruction(&Instruction::Else);
                self.emit_temporal_policy_offset_match_into(
                    &output,
                    zone,
                    &local,
                    supplied,
                    disambiguation,
                    offset_option,
                    match_behavior,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        local.release(self, function);
        self.emit_temporal_complete_selected_instant(output, function)
    }
    fn emit_temporal_policy_wall_into(
        &mut self,
        output: &PreparedTemporalInstantLocals,
        zone: &ResolvedTemporalZoneLocals,
        local: &TemporalLocalCoordinateLocals,
        disambiguation: &TemporalDisambiguationLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let possible = self.emit_temporal_checked_possible_epochs(zone, local, function)?;
        self.emit_temporal_disambiguate_into(
            output,
            &possible,
            zone,
            local,
            disambiguation,
            function,
        )?;
        possible.release(self, function);
        Ok(())
    }
    fn emit_temporal_policy_exact_into(
        &mut self,
        output: &PreparedTemporalInstantLocals,
        local: &TemporalLocalCoordinateLocals,
        supplied: &TemporalOffsetNanosecondsLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Avoid i64 total-epoch nanoseconds: exact pair subtraction and floor
        // normalization stay within the closed extended coordinate domain.
        (local.seconds).load(function);
        (supplied.local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Sub);
        (output.seconds).store(function);
        (local.nano).load(function);
        (supplied.local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Sub);
        (output.nano).store(function);
        self.emit_temporal_euclidean_epoch_pair(output.seconds, output.nano, function);
        (output.nano).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64GeU);
        self.open_frame(ControlFrameKind::If, function);
        (output.nano).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Sub);
        (output.nano).store(function);
        (output.seconds).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (output.seconds).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_validate_iso_days_pair(output.seconds, function)?;
        self.emit_temporal_require_instant_pair(output.seconds, output.nano, function)
    }
    fn emit_temporal_policy_offset_match_into(
        &mut self,
        output: &PreparedTemporalInstantLocals,
        zone: &ResolvedTemporalZoneLocals,
        local: &TemporalLocalCoordinateLocals,
        supplied: &TemporalOffsetNanosecondsLocals,
        disambiguation: &TemporalDisambiguationLocals,
        offset_option: &TemporalOffsetOptionLocals,
        match_behavior: TemporalOffsetMatchBehavior,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_validate_iso_days_coordinate(local, function)?;
        let possible = self.emit_temporal_checked_possible_epochs(zone, local, function)?;
        let found = self.runtime_schema().reserve_i64_local(function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let offset = self.runtime_schema().reserve_i64_local(function);
        let match_offset = self.runtime_schema().reserve_i64_local(function);
        for slot in [found, index] {
            function.instruction(&Instruction::I64Const(0));
            (slot).store(function);
        }
        self.open_frame(ControlFrameKind::Block, function);
        (possible.kind()).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(0));
        self.open_frame(ControlFrameKind::Loop, function);
        (index).load(function);
        (possible.count()).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_candidate_word(&possible, index, 16, offset, function);
        (offset).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        (match_offset).store(function);
        (match_offset).load(function);
        (supplied.local).load(function);
        function.instruction(&Instruction::I64Eq);
        if let TemporalOffsetMatchBehavior::MatchMinutes = match_behavior {
            // Offset seconds are bounded below one day, so signed halfExpand
            // minute rounding uses an exact i64 product and ties away from 0.
            (offset).load(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::End);
            (match_offset).store(function);
            (offset).load(function);
            (match_offset).load(function);
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Const(30));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const(60));
            function.instruction(&Instruction::I64DivU);
            function.instruction(&Instruction::I64Const(60_000_000_000));
            function.instruction(&Instruction::I64Mul);
            (match_offset).load(function);
            function.instruction(&Instruction::I64Mul);
            (supplied.local).load(function);
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_candidate_word(&possible, index, 0, output.seconds, function);
        self.emit_temporal_candidate_word(&possible, index, 8, output.nano, function);
        function.instruction(&Instruction::I64Const(1));
        (found).store(function);
        function.instruction(&Instruction::Br(2));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(index, function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (found).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        (offset_option.local()).load(function);
        function.instruction(&Instruction::I64Const(OffsetOption::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            ZonedDateTimeOptionKey::Offset.range_error(),
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_disambiguate_into(
            output,
            &possible,
            zone,
            local,
            disambiguation,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for slot in [match_offset, offset, index, found] {
            self.runtime_schema().release_i64_local(slot, function);
        }
        possible.release(self, function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_temporal_validate_iso_days_coordinate(
        &mut self,
        local: &TemporalLocalCoordinateLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_validate_iso_days_pair(local.floor_seconds(), function)
    }
    pub(super) fn emit_temporal_validate_iso_days_pair(
        &mut self,
        seconds: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // ValidateISODaysRange permits ±(100000000) ISO days, independently
        // of an eventual candidate's stricter Instant interval.
        let day = self.runtime_schema().reserve_i64_local(function);
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (day).store(function);
        (seconds).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (day).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (day).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (day).load(function);
        function.instruction(&Instruction::I64Const(-100_000_000));
        function.instruction(&Instruction::I64LtS);
        (day).load(function);
        function.instruction(&Instruction::I64Const(100_000_000));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_instant_range_error(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema().release_i64_local(day, function);
        Ok(())
    }
}
