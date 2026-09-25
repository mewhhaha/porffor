//! The Temporal time-zone boundary.
//!
//! A Temporal time zone is stored as the identifier `ToTemporalTimeZoneIdentifier`
//! produced: a normalized `±HH:MM` offset or a case-normalized IANA name. Every
//! operation that depends on the zone's rules — the offset at an instant, the
//! exact time of a wall-clock time, the start of a day, offset matching and the
//! transition searches — is one [`TimeZoneCall`] answered by the pinned
//! time-zone kernel `Intl.DateTimeFormat` also uses. The kernel's
//! `TemporalTimeZone` domain is closed over the offset and named kinds, so no
//! emitted path chooses a kind itself and none can treat a named zone as UTC.
//!
//! Exact times and wall-clock times cross this boundary as a whole-second
//! count and a subsecond remainder in `[0, 10^9)`; offsets are whole seconds.

use lila_intl::{
    IntlHostCallOutcome, IntlHostOp, TemporalDisambiguation, TemporalOffsetMatch,
    TemporalOffsetMismatch, TemporalTimeZoneQueryKind, TemporalTimeZoneRangeError,
    TEMPORAL_EPOCH_SECONDS_LIMIT, TEMPORAL_TIME_ZONE_REQUEST_HEADER_BYTES,
    TEMPORAL_TIME_ZONE_REQUEST_KIND_OFFSET, TEMPORAL_TIME_ZONE_REQUEST_MATCH_OFFSET,
    TEMPORAL_TIME_ZONE_REQUEST_MODE_OFFSET, TEMPORAL_TIME_ZONE_REQUEST_OFFSET_NANOSECONDS_OFFSET,
    TEMPORAL_TIME_ZONE_REQUEST_SECONDS_OFFSET, TEMPORAL_TIME_ZONE_REQUEST_SUBSECOND_OFFSET,
    TEMPORAL_TIME_ZONE_RESPONSE_BYTES, TEMPORAL_TIME_ZONE_RESPONSE_STATUS_OFFSET,
    TEMPORAL_TIME_ZONE_RESPONSE_VALUE_OFFSET, TEMPORAL_TIME_ZONE_STATUS_NO_TRANSITION,
    TEMPORAL_TIME_ZONE_STATUS_SECONDS,
};

use super::super::*;
use super::intl_datetimeformat::NamedTimeZoneRejection;
use super::temporal_options::{Disambiguation, OffsetOption, StringValuedOption, TemporalOverflow};

/// `ISODateTimeWithinLimits` admits wall-clock times up to one day beyond the
/// exact-time range, exclusive at both ends.
pub(super) const TEMPORAL_ISO_DATE_TIME_SECONDS_LIMIT: i64 = TEMPORAL_EPOCH_SECONDS_LIMIT + 86_400;

pub(super) const TEMPORAL_INVALID_TIME_ZONE_MESSAGE: &str =
    "Invalid Temporal.ZonedDateTime time zone";

/// Every message the time-zone boundary and the zoned arithmetic can throw.
/// Any program that touches a ZonedDateTime or a zoned `relativeTo` can reach
/// them, so the pool carries them unconditionally.
pub(crate) fn temporal_time_zone_pool_strings() -> Vec<&'static str> {
    let mut strings: Vec<&'static str> = TemporalTimeZoneRangeError::ALL
        .into_iter()
        .map(TemporalTimeZoneRangeError::message)
        .collect();
    strings.extend([
        TEMPORAL_INVALID_TIME_ZONE_MESSAGE,
        "Temporal.ZonedDateTime options must be an object or undefined",
        "Temporal.PlainDateTime.prototype.toZonedDateTime options must be an object or undefined",
        "Temporal duration rounding window is empty",
        "Temporal.PlainDateTime is outside the supported date range",
        "Temporal.Instant epoch nanoseconds are outside the supported range",
        "Invalid Temporal.ZonedDateTime overflow option",
        "Invalid Temporal.ZonedDateTime disambiguation option",
        "Invalid Temporal.ZonedDateTime offset option",
        super::temporal_zoned_date_time_format::TEMPORAL_ZONED_DATE_TIME_VALUE_OF_MESSAGE,
        ":",
    ]);
    // The option names and values the zoned conversions read, from the
    // domains the readers match against.
    fn option_strings<O: StringValuedOption>(strings: &mut Vec<&'static str>) {
        strings.push(O::PROPERTY);
        strings.extend(O::ALLOWED.iter().map(|value| value.name()));
    }
    option_strings::<Disambiguation>(&mut strings);
    option_strings::<OffsetOption>(&mut strings);
    option_strings::<TemporalOverflow>(&mut strings);
    strings
}

/// Where a `disambiguation` value comes from.
#[derive(Clone, Copy)]
pub(super) enum TemporalDisambiguationSource {
    /// An algorithm step that fixes `compatible` (`AddZonedDateTime`, the
    /// difference and rounding windows, `relativeTo` strings and bags).
    Compatible,
    /// The user's option: a local holding a [`Disambiguation::code`], which is
    /// the kernel's wire value.
    Option(u32),
}

/// `InterpretISODateTimeOffset`'s matchBehaviour.
#[derive(Clone, Copy)]
pub(super) enum TemporalOffsetMatchSource {
    Exactly,
    /// An ISO string: minutes unless the local, set to 1, says the offset
    /// string carried seconds.
    MinutesUnlessSeconds(u32),
}

/// `InterpretISODateTimeOffset`'s offsetBehaviour, as a runtime code shared
/// with the ISO parser's offset kind: none, `Z`, or a numeric offset.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TemporalOffsetBehaviour {
    Wall,
    Exact,
    Option,
}

impl TemporalOffsetBehaviour {
    pub(super) const fn code(self) -> i64 {
        match self {
            Self::Wall => 0,
            Self::Exact => 1,
            Self::Option => 2,
        }
    }
}

/// `GetNamedTimeZoneNextTransition` or `...PreviousTransition`.
#[derive(Clone, Copy)]
pub(super) enum TemporalTransitionSearch {
    Next,
    Previous,
}

/// One kernel query; the locals are its inputs.
#[derive(Clone, Copy)]
enum TimeZoneCall {
    OffsetAt {
        epoch_seconds: u32,
    },
    EpochFor {
        local_seconds: u32,
        subsecond: u32,
        disambiguation: TemporalDisambiguationSource,
    },
    EpochForOffset {
        local_seconds: u32,
        subsecond: u32,
        offset_nanoseconds: u32,
        /// An [`OffsetOption`] code that is `prefer` or `reject`.
        offset_option: u32,
        matching: TemporalOffsetMatchSource,
        disambiguation: TemporalDisambiguationSource,
    },
    StartOfDay {
        local_midnight: u32,
    },
    Transition {
        epoch_seconds: u32,
        subsecond: u32,
        search: TemporalTransitionSearch,
    },
}

impl TimeZoneCall {
    const fn kind(self) -> TemporalTimeZoneQueryKind {
        match self {
            Self::OffsetAt { .. } => TemporalTimeZoneQueryKind::OffsetAt,
            Self::EpochFor { .. } => TemporalTimeZoneQueryKind::EpochFor,
            Self::EpochForOffset { .. } => TemporalTimeZoneQueryKind::EpochForOffset,
            Self::StartOfDay { .. } => TemporalTimeZoneQueryKind::StartOfDay,
            Self::Transition {
                search: TemporalTransitionSearch::Next,
                ..
            } => TemporalTimeZoneQueryKind::NextTransition,
            Self::Transition {
                search: TemporalTransitionSearch::Previous,
                ..
            } => TemporalTimeZoneQueryKind::PreviousTransition,
        }
    }
}

impl Disambiguation {
    /// The kernel's wire value; option locals hold exactly this.
    pub(super) const fn wire(self) -> TemporalDisambiguation {
        match self {
            Disambiguation::Compatible => TemporalDisambiguation::Compatible,
            Disambiguation::Earlier => TemporalDisambiguation::Earlier,
            Disambiguation::Later => TemporalDisambiguation::Later,
            Disambiguation::Reject => TemporalDisambiguation::Reject,
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    fn emit_store_i64_value_at(
        &mut self,
        base_local: u32,
        offset: u64,
        value: TimeZoneWord,
        function: &mut Function,
    ) {
        match value {
            TimeZoneWord::Local(local) => {
                self.store_i64_local_at_offset(base_local, offset, local, function)
            }
            TimeZoneWord::Constant(value) => {
                self.store_i64_const_at_offset(base_local, offset, value as u64, function)
            }
        }
    }

    /// Sends `call` about `time_zone_payload_local` to the kernel. The answer's
    /// seconds land in `value_local`; for a transition search `found_local`
    /// says whether there was one. A RangeError answer is thrown here.
    fn emit_temporal_time_zone_call(
        &mut self,
        time_zone_payload_local: u32,
        call: TimeZoneCall,
        value_local: u32,
        found_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let import = self.intl_call_import_function_index()?;
        let identifier_offset = self.reserve_temp_local();
        let identifier_length = self.reserve_temp_local();
        let request_length = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let response_length = self.reserve_temp_local();
        let status = self.reserve_temp_local();
        let match_word = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            time_zone_payload_local,
            identifier_offset,
            identifier_length,
            function,
        );
        function.instruction(&Instruction::LocalGet(identifier_length));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_TIME_ZONE_REQUEST_HEADER_BYTES as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(request_length));
        self.emit_heap_alloc_from_local(request_length, function)?;
        function.instruction(&Instruction::LocalSet(request));

        let zero = TimeZoneWord::Constant(0);
        let disambiguation_word = |source: TemporalDisambiguationSource| match source {
            TemporalDisambiguationSource::Compatible => {
                TimeZoneWord::Constant(TemporalDisambiguation::Compatible.wire())
            }
            TemporalDisambiguationSource::Option(local) => TimeZoneWord::Local(local),
        };
        let (seconds, subsecond, mode, offset_nanoseconds, matching) = match call {
            TimeZoneCall::OffsetAt { epoch_seconds } => {
                (TimeZoneWord::Local(epoch_seconds), zero, zero, zero, zero)
            }
            TimeZoneCall::EpochFor {
                local_seconds,
                subsecond,
                disambiguation,
            } => (
                TimeZoneWord::Local(local_seconds),
                TimeZoneWord::Local(subsecond),
                disambiguation_word(disambiguation),
                zero,
                zero,
            ),
            TimeZoneCall::EpochForOffset {
                local_seconds,
                subsecond,
                offset_nanoseconds,
                offset_option,
                matching,
                disambiguation,
            } => {
                // Low byte: prefer/reject; next byte: exact/minutes matching.
                function.instruction(&Instruction::LocalGet(offset_option));
                function.instruction(&Instruction::I64Const(OffsetOption::Reject.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(
                    TemporalOffsetMismatch::Reject.wire(),
                ));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(
                    TemporalOffsetMismatch::Prefer.wire(),
                ));
                function.instruction(&Instruction::End);
                match matching {
                    TemporalOffsetMatchSource::Exactly => {
                        function.instruction(&Instruction::I64Const(
                            TemporalOffsetMatch::Exactly.wire() << 8,
                        ));
                    }
                    TemporalOffsetMatchSource::MinutesUnlessSeconds(has_seconds) => {
                        function.instruction(&Instruction::LocalGet(has_seconds));
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                        function.instruction(&Instruction::I64Const(
                            TemporalOffsetMatch::Minutes.wire() << 8,
                        ));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::I64Const(
                            TemporalOffsetMatch::Exactly.wire() << 8,
                        ));
                        function.instruction(&Instruction::End);
                    }
                }
                function.instruction(&Instruction::I64Or);
                function.instruction(&Instruction::LocalSet(match_word));
                (
                    TimeZoneWord::Local(local_seconds),
                    TimeZoneWord::Local(subsecond),
                    disambiguation_word(disambiguation),
                    TimeZoneWord::Local(offset_nanoseconds),
                    TimeZoneWord::Local(match_word),
                )
            }
            TimeZoneCall::StartOfDay { local_midnight } => {
                (TimeZoneWord::Local(local_midnight), zero, zero, zero, zero)
            }
            TimeZoneCall::Transition {
                epoch_seconds,
                subsecond,
                ..
            } => (
                TimeZoneWord::Local(epoch_seconds),
                TimeZoneWord::Local(subsecond),
                zero,
                zero,
                zero,
            ),
        };
        for (offset, value) in [
            (
                TEMPORAL_TIME_ZONE_REQUEST_KIND_OFFSET,
                TimeZoneWord::Constant(call.kind().wire()),
            ),
            (TEMPORAL_TIME_ZONE_REQUEST_SECONDS_OFFSET, seconds),
            (TEMPORAL_TIME_ZONE_REQUEST_SUBSECOND_OFFSET, subsecond),
            (TEMPORAL_TIME_ZONE_REQUEST_MODE_OFFSET, mode),
            (
                TEMPORAL_TIME_ZONE_REQUEST_OFFSET_NANOSECONDS_OFFSET,
                offset_nanoseconds,
            ),
            (TEMPORAL_TIME_ZONE_REQUEST_MATCH_OFFSET, matching),
        ] {
            self.emit_store_i64_value_at(request, offset, value, function);
        }
        // The identifier bytes follow the fixed header.
        function.instruction(&Instruction::LocalGet(request));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_TIME_ZONE_REQUEST_HEADER_BYTES as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(identifier_offset));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(identifier_length));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: 0,
        });

        function.instruction(&Instruction::I64Const(
            TEMPORAL_TIME_ZONE_RESPONSE_BYTES as i64,
        ));
        function.instruction(&Instruction::LocalSet(response_length));
        self.emit_heap_alloc_from_local(response_length, function)?;
        function.instruction(&Instruction::LocalSet(response));
        function.instruction(&Instruction::I64Const(
            IntlHostOp::QueryTemporalTimeZone.wire(),
        ));
        self.emit_pack_string_payload(request, request_length, function);
        self.emit_pack_string_payload(response, response_length, function);
        function.instruction(&Instruction::Call(import));
        // The answer has a fixed extent; anything else is an ABI fault.
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::Written(TEMPORAL_TIME_ZONE_RESPONSE_BYTES as u32).wire(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            response,
            TEMPORAL_TIME_ZONE_RESPONSE_STATUS_OFFSET,
            status,
            function,
        );
        self.load_i64_to_local_from_offset(
            response,
            TEMPORAL_TIME_ZONE_RESPONSE_VALUE_OFFSET,
            value_local,
            function,
        );

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(status));
        function.instruction(&Instruction::I64Const(TEMPORAL_TIME_ZONE_STATUS_SECONDS));
        function.instruction(&Instruction::I64Eq);
        if let Some(found_local) = found_local {
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(found_local));
            function.instruction(&Instruction::LocalGet(found_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::BrIf(0));
            function.instruction(&Instruction::LocalGet(status));
            function.instruction(&Instruction::I64Const(
                TEMPORAL_TIME_ZONE_STATUS_NO_TRANSITION,
            ));
            function.instruction(&Instruction::I64Eq);
        }
        function.instruction(&Instruction::BrIf(0));
        for error in TemporalTimeZoneRangeError::ALL {
            function.instruction(&Instruction::LocalGet(status));
            function.instruction(&Instruction::I64Const(error.status()));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                error.message(),
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);

        for local in [
            match_word,
            status,
            response_length,
            response,
            request,
            request_length,
            identifier_length,
            identifier_offset,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `GetOffsetNanosecondsFor(timeZone, epochNs)` in seconds, for the exact
    /// time whose floored whole seconds are `epoch_seconds_local`.
    pub(super) fn emit_temporal_time_zone_offset_seconds(
        &mut self,
        time_zone_payload_local: u32,
        epoch_seconds_local: u32,
        offset_seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_time_zone_call(
            time_zone_payload_local,
            TimeZoneCall::OffsetAt {
                epoch_seconds: epoch_seconds_local,
            },
            offset_seconds_local,
            None,
            function,
        )
    }

    /// `GetOffsetNanosecondsFor`, in seconds, of the exact time whose floored
    /// epoch milliseconds are in `milliseconds_local`.
    pub(super) fn emit_temporal_time_zone_offset_seconds_at_milliseconds(
        &mut self,
        time_zone_payload_local: u32,
        milliseconds_local: u32,
        offset_seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let seconds_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(seconds_local));
        // Floor, not truncation, for instants before the epoch.
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_time_zone_offset_seconds(
            time_zone_payload_local,
            seconds_local,
            offset_seconds_local,
            function,
        )?;
        self.release_temp_local(seconds_local);
        Ok(())
    }

    /// The exact time of a Temporal record as floored whole seconds and a
    /// remainder in `[0, 10^9)`.
    pub(super) fn emit_temporal_epoch_value_seconds(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) {
        self.emit_temporal_epoch_nanoseconds_value_pair(
            payload_local,
            tag_local,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_normalize_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        );
    }

    /// `GetOffsetNanosecondsFor` of an epoch-nanoseconds value, in seconds.
    pub(super) fn emit_temporal_time_zone_offset_seconds_for_epoch_value(
        &mut self,
        time_zone_payload_local: u32,
        epoch_payload_local: u32,
        epoch_tag_local: u32,
        offset_seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        self.emit_temporal_epoch_value_seconds(
            epoch_payload_local,
            epoch_tag_local,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_time_zone_offset_seconds(
            time_zone_payload_local,
            seconds_local,
            offset_seconds_local,
            function,
        )?;
        self.release_temp_local(subsecond_local);
        self.release_temp_local(seconds_local);
        Ok(())
    }

    /// The nine ISO date-time fields of the local seconds `wall_local` plus
    /// the remainder `subsecond_local` (in `[0, 10^9)`).
    pub(super) fn emit_temporal_iso_date_time_from_local_seconds(
        &mut self,
        wall_seconds_local: u32,
        subsecond_local: u32,
        fields: &[u32; 9],
        function: &mut Function,
    ) {
        let wall_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(wall_seconds_local));
        function.instruction(&Instruction::LocalSet(wall_local));
        function.instruction(&Instruction::LocalGet(wall_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::LocalGet(wall_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(wall_local));
        function.instruction(&Instruction::LocalGet(wall_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(wall_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(wall_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_civil_from_days(days_local, fields[0], fields[1], fields[2], function);
        for (local, source, divisor, modulus) in [
            (fields[3], wall_local, 3_600, None),
            (fields[4], wall_local, 60, Some(60)),
            (fields[5], wall_local, 1, Some(60)),
            (fields[6], subsecond_local, 1_000_000, None),
            (fields[7], subsecond_local, 1_000, Some(1_000)),
            (fields[8], subsecond_local, 1, Some(1_000)),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivU);
            if let Some(modulus) = modulus {
                function.instruction(&Instruction::I64Const(modulus));
                function.instruction(&Instruction::I64RemU);
            }
            function.instruction(&Instruction::LocalSet(local));
        }
        self.release_temp_local(days_local);
        self.release_temp_local(wall_local);
    }

    /// `GetUTCEpochNanoseconds` of nine ISO date-time fields: local whole
    /// seconds and a remainder in `[0, 10^9)`.
    pub(super) fn emit_temporal_local_seconds_from_iso_date_time(
        &mut self,
        fields: &[u32; 9],
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) {
        self.emit_temporal_plain_date_epoch_days(
            fields[0],
            fields[1],
            fields[2],
            seconds_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        for (field, scale) in [(fields[3], 3_600), (fields[4], 60), (fields[5], 1)] {
            function.instruction(&Instruction::LocalGet(field));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
        }
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        for (field, scale) in [(fields[6], 1_000_000), (fields[7], 1_000), (fields[8], 1)] {
            function.instruction(&Instruction::LocalGet(field));
            function.instruction(&Instruction::I64Const(scale));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
        }
        function.instruction(&Instruction::LocalSet(subsecond_local));
    }

    /// `GetISODateTimeFor(timeZone, epochNs)` for the exact time
    /// `seconds + subsecond` (subsecond in `[0, 10^9)`).
    pub(super) fn emit_temporal_iso_date_time_for(
        &mut self,
        time_zone_payload_local: u32,
        seconds_local: u32,
        subsecond_local: u32,
        fields: &[u32; 9],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        self.emit_temporal_time_zone_offset_seconds(
            time_zone_payload_local,
            seconds_local,
            offset_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(offset_local));
        self.emit_temporal_iso_date_time_from_local_seconds(
            offset_local,
            subsecond_local,
            fields,
            function,
        );
        self.release_temp_local(offset_local);
        Ok(())
    }

    /// `GetEpochNanosecondsFor(timeZone, isoDateTime, disambiguation)`: the
    /// exact time of wall-clock `fields` as whole seconds and a remainder.
    /// Every RangeError of `GetPossibleEpochNanoseconds` and of the
    /// disambiguation is thrown.
    pub(super) fn emit_temporal_epoch_for_iso_date_time(
        &mut self,
        time_zone_payload_local: u32,
        fields: &[u32; 9],
        disambiguation: TemporalDisambiguationSource,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_local_seconds_from_iso_date_time(
            fields,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_time_zone_call(
            time_zone_payload_local,
            TimeZoneCall::EpochFor {
                local_seconds: seconds_local,
                subsecond: subsecond_local,
                disambiguation,
            },
            seconds_local,
            None,
            function,
        )
    }

    /// `GetStartOfDay(timeZone, isoDate)` in whole seconds.
    pub(super) fn emit_temporal_start_of_day(
        &mut self,
        time_zone_payload_local: u32,
        date: [u32; 3],
        seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_plain_date_epoch_days(
            date[0],
            date[1],
            date[2],
            seconds_local,
            function,
        );
        self.emit_temporal_start_of_epoch_day(
            time_zone_payload_local,
            seconds_local,
            seconds_local,
            function,
        )
    }

    /// `GetStartOfDay(timeZone, isoDate)` for the ISO date `day_local` days
    /// after 1970-01-01, in whole seconds.
    pub(super) fn emit_temporal_start_of_epoch_day(
        &mut self,
        time_zone_payload_local: u32,
        day_local: u32,
        seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(seconds_local));
        self.emit_temporal_time_zone_call(
            time_zone_payload_local,
            TimeZoneCall::StartOfDay {
                local_midnight: seconds_local,
            },
            seconds_local,
            None,
            function,
        )
    }

    /// `InterpretISODateTimeOffset(isoDate, time, offsetBehaviour,
    /// offsetNanoseconds, timeZone, disambiguation, offsetOption,
    /// matchBehaviour)` for wall-clock `fields`, as whole seconds and a
    /// remainder. `behaviour_local` holds a [`TemporalOffsetBehaviour`] code
    /// and `offset_option_local` an [`OffsetOption`] code.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_interpret_iso_date_time_offset(
        &mut self,
        fields: &[u32; 9],
        time_zone_payload_local: u32,
        behaviour_local: u32,
        offset_nanoseconds_local: u32,
        offset_option_local: u32,
        disambiguation: TemporalDisambiguationSource,
        matching: TemporalOffsetMatchSource,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_local_seconds_from_iso_date_time(
            fields,
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_interpret_local_seconds_offset(
            time_zone_payload_local,
            behaviour_local,
            offset_nanoseconds_local,
            offset_option_local,
            disambiguation,
            matching,
            seconds_local,
            subsecond_local,
            function,
        )
    }

    /// [`Self::emit_temporal_interpret_iso_date_time_offset`] for a wall-clock
    /// time already in local seconds and a remainder; both locals are replaced
    /// by the exact time's.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_interpret_local_seconds_offset(
        &mut self,
        time_zone_payload_local: u32,
        behaviour_local: u32,
        offset_nanoseconds_local: u32,
        offset_option_local: u32,
        disambiguation: TemporalDisambiguationSource,
        matching: TemporalOffsetMatchSource,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Exact, or `offset: "use"`: the wall clock minus the given offset.
        function.instruction(&Instruction::LocalGet(behaviour_local));
        function.instruction(&Instruction::I64Const(
            TemporalOffsetBehaviour::Exact.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(behaviour_local));
        function.instruction(&Instruction::I64Const(
            TemporalOffsetBehaviour::Option.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(offset_option_local));
        function.instruction(&Instruction::I64Const(OffsetOption::Use.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_apply_exact_offset(
            seconds_local,
            subsecond_local,
            offset_nanoseconds_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        // Wall-clock, or `offset: "ignore"`.
        function.instruction(&Instruction::LocalGet(behaviour_local));
        function.instruction(&Instruction::I64Const(TemporalOffsetBehaviour::Wall.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(offset_option_local));
        function.instruction(&Instruction::I64Const(OffsetOption::Ignore.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_time_zone_call(
            time_zone_payload_local,
            TimeZoneCall::EpochFor {
                local_seconds: seconds_local,
                subsecond: subsecond_local,
                disambiguation,
            },
            seconds_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::Else);
        // `prefer` or `reject`: match the zone's candidates.
        self.emit_temporal_time_zone_call(
            time_zone_payload_local,
            TimeZoneCall::EpochForOffset {
                local_seconds: seconds_local,
                subsecond: subsecond_local,
                offset_nanoseconds: offset_nanoseconds_local,
                offset_option: offset_option_local,
                matching,
                disambiguation,
            },
            seconds_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// The exact time of a local time at a given UTC offset: the local seconds
    /// and remainder minus `offset_nanoseconds_local`, normalized, and a
    /// RangeError unless `IsValidEpochNanoseconds`. `CheckISODaysRange` of the
    /// balanced date is subsumed by that check.
    pub(super) fn emit_temporal_apply_exact_offset(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        offset_nanoseconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(offset_nanoseconds_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::LocalGet(offset_nanoseconds_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_normalize_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_require_valid_epoch(seconds_local, subsecond_local, function)
    }

    /// `GetNamedTimeZoneNextTransition` / `PreviousTransition` of the exact
    /// time `seconds + subsecond`, limited to representable instants; always
    /// absent for an offset zone. `found_local` is 1 when `seconds_out_local`
    /// holds a transition.
    pub(super) fn emit_temporal_time_zone_transition(
        &mut self,
        time_zone_payload_local: u32,
        seconds_local: u32,
        subsecond_local: u32,
        search: TemporalTransitionSearch,
        found_local: u32,
        seconds_out_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_time_zone_call(
            time_zone_payload_local,
            TimeZoneCall::Transition {
                epoch_seconds: seconds_local,
                subsecond: subsecond_local,
                search,
            },
            seconds_out_local,
            Some(found_local),
            function,
        )
    }

    /// `TimeZoneEquals(one, two)` as 0 or 1 in `output_local`: equal
    /// identifiers, or two named zones with one primary identifier.
    pub(super) fn emit_temporal_time_zone_equals(
        &mut self,
        one_payload_local: u32,
        two_payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let one_local = self.reserve_temp_local();
        let two_local = self.reserve_temp_local();
        let one_primary_local = self.reserve_temp_local();
        let two_primary_local = self.reserve_temp_local();
        let named_local = self.reserve_temp_local();
        self.emit_string_payload_equality_i32(one_payload_local, two_payload_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_time_zone_is_offset(one_payload_local, named_local, function);
        self.emit_temporal_time_zone_is_offset(two_payload_local, one_local, function);
        function.instruction(&Instruction::LocalGet(named_local));
        function.instruction(&Instruction::LocalGet(one_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (source, identifier, primary) in [
            (one_payload_local, one_local, one_primary_local),
            (two_payload_local, two_local, two_primary_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(identifier));
            self.emit_intl_lookup_named_time_zone(
                identifier,
                Some(primary),
                NamedTimeZoneRejection::Throw(TEMPORAL_INVALID_TIME_ZONE_MESSAGE),
                function,
            )?;
        }
        self.emit_string_payload_equality_i32(one_primary_local, two_primary_local, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            named_local,
            two_primary_local,
            one_primary_local,
            two_local,
            one_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Whether a stored identifier is an offset time zone (`±HH:MM`): 0 or 1.
    fn emit_temporal_time_zone_is_offset(
        &mut self,
        time_zone_payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        self.emit_unpack_string_payload(
            time_zone_payload_local,
            offset_local,
            length_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_load_string_byte(offset_local, cursor_local, output_local, function);
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(cursor_local);
        self.release_temp_local(length_local);
        self.release_temp_local(offset_local);
    }

    /// `ParseTimeZoneIdentifier` of a time-zone annotation or a bare
    /// identifier known to be one or the other, then the normalization of
    /// `ToTemporalTimeZoneIdentifier`: an offset becomes `±HH:MM`, a name its
    /// available identifier. Anything else is a RangeError.
    pub(super) fn emit_temporal_time_zone_name_or_offset(
        &mut self,
        time_zone_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let is_offset_local = self.reserve_temp_local();
        self.emit_temporal_time_zone_is_offset(time_zone_payload_local, is_offset_local, function);
        function.instruction(&Instruction::LocalGet(is_offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_canonicalize_offset_time_zone(time_zone_payload_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_intl_lookup_named_time_zone(
            time_zone_payload_local,
            None,
            NamedTimeZoneRejection::Throw(TEMPORAL_INVALID_TIME_ZONE_MESSAGE),
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_temp_local(is_offset_local);
        Ok(())
    }

    /// `IsValidEpochNanoseconds` of `seconds + subsecond` (subsecond in
    /// `[0, 10^9)`), as a RangeError.
    pub(super) fn emit_temporal_require_valid_epoch(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(-TEMPORAL_EPOCH_SECONDS_LIMIT));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_EPOCH_SECONDS_LIMIT));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_EPOCH_SECONDS_LIMIT));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.Instant epoch nanoseconds are outside the supported range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `ISODateTimeWithinLimits`, as a RangeError.
    pub(super) fn emit_temporal_require_iso_date_time_within_limits(
        &mut self,
        fields: &[u32; 9],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        self.emit_temporal_local_seconds_from_iso_date_time(
            fields,
            seconds_local,
            subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(
            -TEMPORAL_ISO_DATE_TIME_SECONDS_LIMIT,
        ));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(
            -TEMPORAL_ISO_DATE_TIME_SECONDS_LIMIT,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_ISO_DATE_TIME_SECONDS_LIMIT));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDateTime is outside the supported date range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(subsecond_local);
        self.release_temp_local(seconds_local);
        Ok(())
    }
}

/// A request word: a local or a constant.
#[derive(Clone, Copy)]
enum TimeZoneWord {
    Local(u32),
    Constant(i64),
}
