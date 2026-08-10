use super::super::*;
use super::temporal_options::{Disambiguation, OffsetOption, StringValuedOption, TemporalOverflow};
use super::temporal_plain_date::{TemporalCalendarId, TemporalEraField};
use super::temporal_plain_year_month_methods::TemporalPartialDateRewrite;

/// Which `Temporal.ZonedDateTime.prototype` accessor
/// [`FunctionBuilder::emit_temporal_zoned_date_time_iso_field`] is producing.
///
/// The emitter used to take a [`StandardBuiltinId`] and end its dispatch in
/// `_ => unreachable!()` — a catch-all over a several-hundred-variant enum,
/// which is precisely the shape `AGENTS.md` bans: "you added a getter and
/// forgot an arm" became a live `unreachable!()` in the compiler instead of a
/// compile error. The parameter is now a closed twelve-variant domain matched
/// with no catch-all, so the omission fails to build.
///
/// The other direction — `StandardBuiltinId -> ZonedDateTimeField` — lives in
/// `compile_standard_builtin`'s flat exhaustive match, which already fails to
/// build on a builtin with no arm. So a new ZonedDateTime accessor cannot reach
/// this emitter without naming its field here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ZonedDateTimeField {
    Era,
    EraYear,
    Year,
    Month,
    MonthCode,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

/// Where a [`ZonedDateTimeField`] arm leaves its answer.
///
/// Nine arms push exactly one `i64` that is an `f64` bit pattern and let the
/// tail of the emitter tag it `Number`. `MonthCode` writes a String result pair
/// itself; `Era` and `EraYear` delegate to
/// [`FunctionBuilder::emit_temporal_calendar_era_field`], which writes the pair
/// itself and whose answer may be a String, a Number *or* Undefined.
///
/// Before this enum the distinction was a `!=` against one specific builtin id.
/// Adding a second self-writing getter under that rule appended
/// `LocalSet(result_local)` + a `Number` tag after the callee had already
/// written the pair — so an `undefined` era would have been reported as a
/// number, from a stack the arm never pushed to. Two named states cannot be
/// confused that way, and the value is produced by the same `match` that emits
/// the arm, so the two cannot drift.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ZdtFieldResult {
    /// The arm left exactly one `i64` on the stack and did not touch the result
    /// pair.
    NumberOnStack,
    /// The arm wrote both halves of the result pair and left the stack empty.
    WrittenByCallee,
}

/// The three options `Temporal.ZonedDateTime.from` reads, in read order.
///
/// The option's identity used to be the `&str` the loop iterated, compared
/// again at two later points; a typo in the loop literal still compiled, routed
/// the code into the wrong destination local and reached a live `unreachable!()`
/// in the compiler. Every derived fact is now a total function of the variant.
#[derive(Clone, Copy)]
enum ZonedDateTimeOptionKey {
    Disambiguation,
    Offset,
    Overflow,
}

impl ZonedDateTimeOptionKey {
    const ALL: [ZonedDateTimeOptionKey; 3] = [
        ZonedDateTimeOptionKey::Disambiguation,
        ZonedDateTimeOptionKey::Offset,
        ZonedDateTimeOptionKey::Overflow,
    ];

    fn property(self) -> &'static str {
        match self {
            ZonedDateTimeOptionKey::Disambiguation => Disambiguation::PROPERTY,
            ZonedDateTimeOptionKey::Offset => OffsetOption::PROPERTY,
            ZonedDateTimeOptionKey::Overflow => TemporalOverflow::PROPERTY,
        }
    }

    fn range_error(self) -> &'static str {
        match self {
            ZonedDateTimeOptionKey::Disambiguation => {
                "Invalid Temporal.ZonedDateTime disambiguation option"
            }
            ZonedDateTimeOptionKey::Offset => "Invalid Temporal.ZonedDateTime offset option",
            ZonedDateTimeOptionKey::Overflow => "Invalid Temporal.ZonedDateTime overflow option",
        }
    }

    fn allowed(self) -> Vec<(&'static str, i64)> {
        fn pairs<O: StringValuedOption>() -> Vec<(&'static str, i64)> {
            O::ALLOWED
                .iter()
                .map(|value| (value.name(), value.code()))
                .collect()
        }
        match self {
            ZonedDateTimeOptionKey::Disambiguation => pairs::<Disambiguation>(),
            ZonedDateTimeOptionKey::Offset => pairs::<OffsetOption>(),
            ZonedDateTimeOptionKey::Overflow => pairs::<TemporalOverflow>(),
        }
    }

    fn default_code(self) -> i64 {
        match self {
            ZonedDateTimeOptionKey::Disambiguation => Disambiguation::DEFAULT.code(),
            ZonedDateTimeOptionKey::Offset => OffsetOption::DEFAULT.code(),
            ZonedDateTimeOptionKey::Overflow => StringValuedOption::code(TemporalOverflow::DEFAULT),
        }
    }

    /// Which caller-supplied local receives the code. `disambiguation` has no
    /// destination: this backend resolves only `UTC` and fixed offsets, so the
    /// value is validated and then deliberately dropped. That gap is now named
    /// rather than expressed as an absent code in a table.
    fn destination(self, offset_local: u32, overflow_local: u32) -> Option<u32> {
        match self {
            ZonedDateTimeOptionKey::Disambiguation => None,
            ZonedDateTimeOptionKey::Offset => Some(offset_local),
            ZonedDateTimeOptionKey::Overflow => Some(overflow_local),
        }
    }
}

const TEMPORAL_INSTANT_LIMIT_HIGH_LIMB: i64 = 468;
const TEMPORAL_INSTANT_LIMIT_LOW_LIMB: i64 = 6_923_773_503_929_843_712;
const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;
const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Copy)]
enum TemporalIsoParseGoal {
    Instant,
    TimeZoneIdentifier {
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
    },
    ZonedDateTimeSyntax {
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
    },
    ZonedDateTime {
        offset_option_local: u32,
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
    },
    /// `ParseISODateTime` with the `TemporalDateString` goal: the civil
    /// year/month/day are handed back instead of being collapsed into epoch
    /// nanoseconds, a trailing time and offset are optional, and a bracketed
    /// time zone is accepted without being resolved (a `PlainDate` has no time
    /// zone, so `[America/New_York]` is legal syntax even though this backend
    /// cannot resolve that identifier).
    PlainDate {
        year_destination_local: u32,
        month_destination_local: u32,
        day_destination_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
    },
    /// `ParseISODateTime` with the `TemporalDateTimeString` goal: the civil
    /// date *and* the wall-clock time are handed back. A missing time part
    /// defaults to midnight, a `Z` designator is forbidden (a `PlainDateTime`
    /// names no instant) and a bracketed time zone is accepted without being
    /// resolved.
    PlainDateTime {
        year_destination_local: u32,
        month_destination_local: u32,
        day_destination_local: u32,
        hour_destination_local: u32,
        minute_destination_local: u32,
        second_destination_local: u32,
        nanosecond_destination_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
    },
    /// `ParseTemporalTimeString`. The wrapper has already rewritten a bare
    /// time (`15:23`, `T15:23`, `152330-0800`) into `0000-01-01T` + the same
    /// tail, so by the time this goal is reached every input carries a date
    /// and only the time-of-day fields are wanted back. A date-only string
    /// still reaches here unrewritten, and is rejected here because a
    /// `PlainTime` never gets an implicit midnight.
    PlainTime {
        hour_destination_local: u32,
        minute_destination_local: u32,
        second_destination_local: u32,
        nanosecond_destination_local: u32,
    },
}

impl<'a> FunctionBuilder<'a> {
    /// Temporal proposal 2.3.1 `Temporal.Now.timeZoneId`.
    ///
    /// This backend has no tzdata: `emit_temporal_zoned_date_time_time_zone`
    /// only resolves `UTC` and fixed `±HH:MM` offsets, so `UTC` is the only
    /// identifier the rest of the implementation can honour.
    pub(crate) fn emit_temporal_now_time_zone_id(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("UTC")));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        Ok(())
    }

    /// Reads the host wall clock and splits it into whole epoch seconds plus a
    /// non-negative nanosecond remainder, the shape
    /// `emit_temporal_epoch_nanoseconds_bigint` expects.
    ///
    /// The host import is millisecond resolution, which the spec permits: it
    /// only requires the clock not to go backwards within an execution.
    fn emit_temporal_now_epoch_seconds_and_subseconds(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let wall_clock_millis_import_function_index = self
            .functions
            .wall_clock_millis_import_function_index()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "Temporal.Now requires the porf_host.wall_clock_millis import",
                )
            })?;
        let milliseconds_local = self.reserve_temp_local();

        function.instruction(&Instruction::Call(wall_clock_millis_import_function_index));
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(milliseconds_local));

        // Floor-divide by 1000 so that pre-epoch clocks (test hosts can set
        // them) still produce a subsecond remainder in [0, 1e9).
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(subsecond_local));

        self.release_temp_local(milliseconds_local);
        Ok(())
    }

    /// Temporal proposal 2.3.2 `Temporal.Now.instant`.
    pub(crate) fn emit_temporal_now_instant(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_now_epoch_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        )?;
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            subsecond_local,
            seconds_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 2.3.4 `Temporal.Now.zonedDateTimeISO`.
    pub(crate) fn emit_temporal_now_zoned_date_time_iso(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        // An absent or `undefined` argument means SystemTimeZoneIdentifier();
        // anything else goes through the same resolution the ZonedDateTime
        // constructor uses, so named zones still reject.
        self.emit_builtin_arg_to_locals(0, time_zone_payload_local, time_zone_tag_local, function);
        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("UTC")));
        function.instruction(&Instruction::LocalSet(time_zone_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(time_zone_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("iso8601")));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));

        self.emit_temporal_now_epoch_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        )?;
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            subsecond_local,
            seconds_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let argument_brand_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            argument_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(argument_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            argument_brand_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            argument_brand_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            argument_brand_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(argument_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            argument_brand_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            argument_brand_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            argument_brand_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(argument_payload_local, argument_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(argument_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(argument_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant.from requires a string or Temporal.Instant",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_parse_iso_string(
            argument_payload_local,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            TemporalIsoParseGoal::Instant,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;

        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
        self.release_temp_local(argument_brand_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let argument_brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let offset_option_local = self.reserve_temp_local();
        let overflow_option_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            argument_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(argument_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_temporal_zoned_date_time_options(
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            function,
        )?;
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                calendar_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_zoned_date_time_from_property_bag(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_zoned_date_time_from_property_bag(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime.from requires a string or Temporal.ZonedDateTime",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_parse_iso_string(
            argument_payload_local,
            epoch_payload_local,
            epoch_tag_local,
            TemporalIsoParseGoal::ZonedDateTimeSyntax {
                time_zone_payload_local,
                time_zone_tag_local,
                calendar_payload_local,
                calendar_tag_local,
            },
            function,
        )?;
        self.emit_temporal_zoned_date_time_options(
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            function,
        )?;
        self.emit_temporal_parse_iso_string(
            argument_payload_local,
            epoch_payload_local,
            epoch_tag_local,
            TemporalIsoParseGoal::ZonedDateTime {
                offset_option_local,
                time_zone_payload_local,
                time_zone_tag_local,
                calendar_payload_local,
                calendar_tag_local,
            },
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            overflow_option_local,
            offset_option_local,
            options_tag_local,
            options_payload_local,
            record_local,
            argument_brand_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_zoned_date_time_from_property_bag(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        options_payload_local: u32,
        options_tag_local: u32,
        offset_option_local: u32,
        overflow_option_local: u32,
        epoch_payload_local: u32,
        epoch_tag_local: u32,
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let property_key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let day_present_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let microsecond_local = self.reserve_temp_local();
        let millisecond_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let month_present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let nanosecond_local = self.reserve_temp_local();
        let offset_payload_local = self.reserve_temp_local();
        let offset_present_local = self.reserve_temp_local();
        let second_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        // Last, so the resolver below — which runs before any further
        // reservation in this function — can release them off the top of the
        // LIFO temp stack.
        let era_slots = self.reserve_temporal_era_slots();

        function.instruction(&Instruction::I64Const(self.strings.payload("calendar")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        // Property bag: `ToTemporalCalendarIdentifier`, so an ISO date string
        // parses rather than throwing.
        self.emit_temporal_zoned_date_time_calendar(
            calendar_payload_local,
            calendar_tag_local,
            true,
            function,
        )?;

        // `day`, then the era pair, then `hour` .. `month`: the era keys sort
        // between `day` and `hour`, and a Proxy bag observes the reads in
        // exactly this order.
        self.emit_temporal_property_bag_positive_integer(
            argument_payload_local,
            argument_tag_local,
            "day",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            day_local,
            0,
            "Temporal.ZonedDateTime property bag field must be finite",
            "Temporal.ZonedDateTime month and day must be positive",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::LocalSet(day_present_local));

        let era = self.emit_temporal_read_era_fields(
            era_slots,
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            function,
        )?;

        for (property, output_local, output_present_local) in [
            ("hour", hour_local, None),
            ("microsecond", microsecond_local, None),
            ("millisecond", millisecond_local, None),
            ("minute", minute_local, None),
            ("month", month_local, Some(month_present_local)),
        ] {
            // `month` is the only remaining row of the calendar field table
            // whose conversion is `ToPositiveIntegerWithTruncation`; the
            // wall-clock rows accept zero.
            if property == "month" {
                self.emit_temporal_property_bag_positive_integer(
                    argument_payload_local,
                    argument_tag_local,
                    property,
                    property_key_local,
                    value_payload_local,
                    value_tag_local,
                    present_local,
                    output_local,
                    0,
                    "Temporal.ZonedDateTime property bag field must be finite",
                    "Temporal.ZonedDateTime month and day must be positive",
                    function,
                )?;
            } else {
                self.emit_temporal_property_bag_integer(
                    argument_payload_local,
                    argument_tag_local,
                    property,
                    property_key_local,
                    value_payload_local,
                    value_tag_local,
                    present_local,
                    output_local,
                    0,
                    "Temporal.ZonedDateTime property bag field must be finite",
                    function,
                )?;
            }
            if let Some(output_present_local) = output_present_local {
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::LocalSet(output_present_local));
            }
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("monthCode")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(month_code_present_local));
        self.emit_temporal_property_bag_string(
            value_payload_local,
            value_tag_local,
            "Temporal.ZonedDateTime monthCode must be a string",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(month_code_payload_local));

        for (property, output_local) in [("nanosecond", nanosecond_local)] {
            self.emit_temporal_property_bag_integer(
                argument_payload_local,
                argument_tag_local,
                property,
                property_key_local,
                value_payload_local,
                value_tag_local,
                present_local,
                output_local,
                0,
                "Temporal.ZonedDateTime property bag field must be finite",
                function,
            )?;
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("offset")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(offset_present_local));
        self.emit_temporal_property_bag_string(
            value_payload_local,
            value_tag_local,
            "Temporal.ZonedDateTime offset must be a string",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(offset_payload_local));

        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "second",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            second_local,
            0,
            "Temporal.ZonedDateTime property bag field must be finite",
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("timeZone")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "year",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            year_local,
            0,
            "Temporal.ZonedDateTime property bag field must be finite",
            function,
        )?;
        // KNOWN STEP-ORDER DIVERGENCE, and the only one of the five bag paths
        // that has it. `ToTemporalZonedDateTime` steps 2.h-2.k read the options
        // object — `GetOptionsObject`, then the disambiguation/offset/overflow
        // casts — *before* `InterpretTemporalDateTimeFields` reaches
        // `CalendarResolveFields`. Here the resolver, and the two "requires
        // year"/"requires day" TypeErrors below it, all run before
        // `emit_temporal_zoned_date_time_options` at the bottom of this
        // function, so an era `RangeError`/`TypeError` beats an observable
        // option read:
        //
        //   Temporal.ZonedDateTime.from(
        //     { month: 1, day: 1, timeZone: "UTC", era: "xyz", eraYear: 2025,
        //       calendar: "gregory" },
        //     { overflow: { get valueOf() { throw new Test262Error(); } } })
        //
        // throws the era RangeError where the specification throws from the
        // option read. Nothing in the pinned corpus is currently sensitive to
        // it. The repair is to move all three — resolver, `requires year`,
        // `requires day` — below the options call together; moving the resolver
        // alone would reorder it against the two pre-existing checks, which is a
        // different observable order again.
        let resolved_year = self.emit_temporal_resolve_era_to_year(
            era,
            calendar_payload_local,
            year_local,
            present_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(resolved_year.year_present_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime property bag requires year",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime property bag requires day",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_zoned_date_time_options(
            options_payload_local,
            options_tag_local,
            offset_option_local,
            overflow_option_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime property bag requires timeZone",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        for month in 1_i64..=12 {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&format!("M{month:02}")),
            ));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_string_payload_equality_i32(
                month_code_payload_local,
                value_tag_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.ZonedDateTime monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime month and monthCode must agree",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime property bag requires month or monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime month and day must be positive",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_regulate_property_bag_date_time(
            year_local,
            month_local,
            day_local,
            hour_local,
            minute_local,
            second_local,
            millisecond_local,
            microsecond_local,
            nanosecond_local,
            overflow_option_local,
            function,
        )?;

        let time_zone_offset_seconds_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let selected_offset_seconds_local = self.reserve_temp_local();
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            time_zone_offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
        function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
        function.instruction(&Instruction::LocalGet(offset_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_fixed_time_zone_offset_seconds(
            offset_payload_local,
            offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(offset_option_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime offset does not match its fixed time zone",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(offset_option_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        let adjusted_year_local = self.reserve_temp_local();
        let era_local = self.reserve_temp_local();
        let month_index_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        self.emit_temporal_days_from_civil(
            year_local,
            month_local,
            day_local,
            adjusted_year_local,
            era_local,
            month_index_local,
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(SECONDS_PER_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(second_local));
        function.instruction(&Instruction::I64Const(59));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(59));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(second_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(selected_offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(millisecond_local));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(microsecond_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(nanosecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            subsecond_local,
            seconds_local,
            days_local,
            month_index_local,
            era_local,
            adjusted_year_local,
            selected_offset_seconds_local,
            offset_seconds_local,
            time_zone_offset_seconds_local,
            year_local,
            second_local,
            offset_present_local,
            offset_payload_local,
            nanosecond_local,
            month_code_present_local,
            month_code_payload_local,
            month_present_local,
            month_local,
            minute_local,
            millisecond_local,
            microsecond_local,
            hour_local,
            day_present_local,
            day_local,
            present_local,
            value_tag_local,
            value_payload_local,
            property_key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_regulate_property_bag_date_time(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        hour_local: u32,
        minute_local: u32,
        second_local: u32,
        millisecond_local: u32,
        microsecond_local: u32,
        nanosecond_local: u32,
        overflow_option_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(-271_821));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(275_760));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime property bag year is outside the supported instant range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(overflow_option_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime property bag month is out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::LocalSet(maximum_day_local));
        for month in [4_i64, 6, 9, 11] {
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(30));
            function.instruction(&Instruction::LocalSet(maximum_day_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(28));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(maximum_day_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        for (local, minimum, maximum) in [
            (hour_local, 0_i64, 23_i64),
            (minute_local, 0, 59),
            (second_local, 0, 59),
            (millisecond_local, 0, 999),
            (microsecond_local, 0, 999),
            (nanosecond_local, 0, 999),
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(invalid_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(overflow_option_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.ZonedDateTime property bag date-time field is out of range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);
        for (local, minimum, maximum) in [
            (hour_local, 0_i64, 23_i64),
            (minute_local, 0, 59),
            (second_local, 0, 59),
            (millisecond_local, 0, 999),
            (microsecond_local, 0, 999),
            (nanosecond_local, 0, 999),
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(invalid_local);
        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_property_bag_integer(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        property: &str,
        property_key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        present_local: u32,
        output_local: u32,
        default: i64,
        not_finite_error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload(property)));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            argument_payload_local,
            argument_tag_local,
            argument_payload_local,
            argument_tag_local,
            property_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(present_local));
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(default));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::MAX)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            not_finite_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `IsPartialTemporalObject` step 2: a *branded* Temporal object is never a
    /// partial property bag, even though its prototype supplies every field
    /// name `PrepareCalendarFields` looks for. Without this, `.with(plainDate)`
    /// happily reads `plainDate.year` / `.month` / `.day` through the getters
    /// and succeeds where the specification demands a TypeError.
    ///
    /// Runs *before* the observable `calendar` / `timeZone` reads, matching the
    /// step order, and leaves non-objects alone — callers have already rejected
    /// those.
    ///
    /// Instant and Duration are deliberately absent: the specification's slot
    /// list is `[[InitializedTemporalDate]]`, `[[...DateTime]]`,
    /// `[[...MonthDay]]`, `[[...Time]]`, `[[...YearMonth]]`,
    /// `[[...ZonedDateTime]]`.
    pub(crate) fn emit_temporal_reject_branded_partial_object(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        for (index, brand) in [
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE,
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME,
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY,
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_TIME,
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH,
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME,
        ]
        .into_iter()
        .enumerate()
        {
            function.instruction(&Instruction::LocalGet(brand_local));
            function.instruction(&Instruction::I64Const(brand as i64));
            function.instruction(&Instruction::I64Eq);
            if index > 0 {
                function.instruction(&Instruction::I32Or);
            }
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(brand_local);
        Ok(())
    }

    /// `PrepareCalendarFields` with `ToPositiveIntegerWithTruncation`, the
    /// conversion the field table names for `day` and `month` (and only those).
    /// Identical to [`Self::emit_temporal_property_bag_integer`] except that a
    /// *present* field truncating to zero or below is a RangeError raised right
    /// here — before the next field is read and long before
    /// `GetTemporalOverflowOption`, which is why `{ day: -1 }` beats a primitive
    /// `options` argument to the throw in Test262's `with/options-wrong-type.js`
    /// and why `overflow: "constrain"` cannot rescue `{ month: 0 }`.
    ///
    /// An *absent* field takes `default` unvalidated, so callers may keep using
    /// `0` as the "no value" sentinel.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_property_bag_positive_integer(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        property: &str,
        property_key_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        present_local: u32,
        output_local: u32,
        default: i64,
        not_finite_error_message: &str,
        not_positive_error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            property,
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            output_local,
            default,
            not_finite_error_message,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            not_positive_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_temporal_property_bag_string(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        Ok(())
    }

    fn emit_temporal_zoned_date_time_options(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        offset_option_local: u32,
        overflow_option_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let property_key_local = self.reserve_temp_local();
        let option_payload_local = self.reserve_temp_local();
        let option_tag_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();
        let recognized_local = self.reserve_temp_local();

        for key in ZonedDateTimeOptionKey::ALL {
            if let Some(destination) = key.destination(offset_option_local, overflow_option_local) {
                function.instruction(&Instruction::I64Const(key.default_code()));
                function.instruction(&Instruction::LocalSet(destination));
            }
        }
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(options_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime.from options must be an object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for key in ZonedDateTimeOptionKey::ALL {
            let destination = key.destination(offset_option_local, overflow_option_local);
            function.instruction(&Instruction::I64Const(self.strings.payload(key.property())));
            function.instruction(&Instruction::LocalSet(property_key_local));
            self.emit_object_read(
                options_payload_local,
                options_tag_local,
                options_payload_local,
                options_tag_local,
                property_key_local,
                option_payload_local,
                option_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::LocalGet(option_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_value_to_string_payload(option_payload_local, option_tag_local, function)?;
            function.instruction(&Instruction::LocalSet(option_payload_local));
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(recognized_local));
            for (expected, code) in key.allowed() {
                function.instruction(&Instruction::I64Const(self.strings.payload(expected)));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                self.emit_string_payload_equality_i32(
                    option_payload_local,
                    expected_payload_local,
                    function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(recognized_local));
                if let Some(destination) = destination {
                    function.instruction(&Instruction::I64Const(code));
                    function.instruction(&Instruction::LocalSet(destination));
                }
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::LocalGet(recognized_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                key.range_error(),
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        self.release_temp_local(recognized_local);
        self.release_temp_local(expected_payload_local);
        self.release_temp_local(option_tag_local);
        self.release_temp_local(option_payload_local);
        self.release_temp_local(property_key_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let epoch_argument_payload_local = self.reserve_temp_local();
        let epoch_argument_tag_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(
            0,
            epoch_argument_payload_local,
            epoch_argument_tag_local,
            function,
        );
        self.emit_value_to_bigint_locals(
            epoch_argument_tag_local,
            epoch_argument_payload_local,
            false,
            epoch_payload_local,
            epoch_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_temporal_instant_validate_range(epoch_payload_local, epoch_tag_local, function)?;
        self.emit_builtin_arg_to_locals(1, time_zone_payload_local, time_zone_tag_local, function);
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(2, calendar_payload_local, calendar_tag_local, function);
        // Constructor: `CanonicalizeCalendar` only — an ISO date string is a
        // RangeError here.
        self.emit_temporal_zoned_date_time_calendar(
            calendar_payload_local,
            calendar_tag_local,
            false,
            function,
        )?;
        self.emit_error_new_target_prototype_to_local(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(calendar_tag_local);
        self.release_temp_local(calendar_payload_local);
        self.release_temp_local(time_zone_tag_local);
        self.release_temp_local(time_zone_payload_local);
        self.release_temp_local(epoch_tag_local);
        self.release_temp_local(epoch_payload_local);
        self.release_temp_local(epoch_argument_tag_local);
        self.release_temp_local(epoch_argument_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_time_zone(
        &mut self,
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_zone_offset_seconds_local = self.reserve_temp_local();
        let object_brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let first_byte_local = self.reserve_temp_local();
        let direct_identifier_local = self.reserve_temp_local();
        let unused_nanoseconds_payload_local = self.reserve_temp_local();
        let unused_nanoseconds_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            time_zone_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            object_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            time_zone_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
            time_zone_tag_local,
            function,
        );
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime time zone must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(direct_identifier_local));
        self.emit_unpack_string_payload(
            time_zone_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(
            string_offset_local,
            direct_identifier_local,
            first_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(first_byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(direct_identifier_local));
        function.instruction(&Instruction::End);

        let expected_utc_payload_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("UTC")));
        function.instruction(&Instruction::LocalSet(expected_utc_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(case_fold_local));
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            time_zone_payload_local,
            expected_utc_payload_local,
            Some(case_fold_local),
            function,
        );
        function.instruction(&Instruction::LocalGet(direct_identifier_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            time_zone_offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_parse_iso_string(
            time_zone_payload_local,
            unused_nanoseconds_payload_local,
            unused_nanoseconds_tag_local,
            TemporalIsoParseGoal::TimeZoneIdentifier {
                time_zone_payload_local,
                time_zone_tag_local,
            },
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(case_fold_local);
        self.release_temp_local(expected_utc_payload_local);
        self.release_temp_local(unused_nanoseconds_tag_local);
        self.release_temp_local(unused_nanoseconds_payload_local);
        self.release_temp_local(direct_identifier_local);
        self.release_temp_local(first_byte_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        self.release_temp_local(record_local);
        self.release_temp_local(object_brand_local);
        self.release_temp_local(time_zone_offset_seconds_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_fixed_time_zone_offset_seconds(
        &mut self,
        time_zone_payload_local: u32,
        time_zone_offset_seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected_payload_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("UTC")));
        function.instruction(&Instruction::LocalSet(expected_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(case_fold_local));
        self.emit_string_payload_equality_i32_with_ascii_case_folding(
            time_zone_payload_local,
            expected_payload_local,
            Some(case_fold_local),
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(expected_payload_local));
        function.instruction(&Instruction::LocalSet(time_zone_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(time_zone_offset_seconds_local));
        function.instruction(&Instruction::Else);
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let valid_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let has_minute_local = self.reserve_temp_local();
        self.emit_unpack_string_payload(
            time_zone_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(minute_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            string_len_local,
            byte_local,
            valid_local,
            hour_local,
            2,
            function,
        );
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            string_len_local,
            byte_local,
            valid_local,
            minute_local,
            2,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(23));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Const(59));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.ZonedDateTime time zone",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(time_zone_offset_seconds_local));
        self.emit_temporal_format_fixed_time_zone_offset(
            time_zone_offset_seconds_local,
            time_zone_payload_local,
            function,
        )?;
        self.release_temp_local(has_minute_local);
        self.release_temp_local(minute_local);
        self.release_temp_local(hour_local);
        self.release_temp_local(sign_local);
        self.release_temp_local(valid_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(cursor_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        function.instruction(&Instruction::End);
        self.release_temp_local(case_fold_local);
        self.release_temp_local(expected_payload_local);
        Ok(())
    }

    /// `Temporal.ZonedDateTime`'s calendar coercion.
    ///
    /// `parse_iso_strings` selects the specification operation, and the two are
    /// not interchangeable. The property-bag path
    /// (`emit_temporal_zoned_date_time_from_property_bag`) performs
    /// `ToTemporalCalendarIdentifier`, which runs `ParseTemporalCalendarString`
    /// and therefore accepts `"1111-11-11"`. The constructor performs
    /// `CanonicalizeCalendar` only, where the same string must stay a
    /// RangeError (`ZonedDateTime/calendar-invalid-iso-string.js`).
    fn emit_temporal_zoned_date_time_calendar(
        &mut self,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        parse_iso_strings: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if parse_iso_strings {
            return self.emit_temporal_to_temporal_calendar_identifier(
                calendar_payload_local,
                calendar_tag_local,
                "Temporal.ZonedDateTime calendar must be a string",
                function,
            );
        }
        // Same operation as the four date types' constructors, so it is the
        // same emitter; only the two error messages differ. Duplicating the
        // body here is how `gregory` could have been accepted by
        // `new Temporal.PlainDate` and rejected by `new Temporal.ZonedDateTime`.
        self.emit_temporal_canonicalize_calendar(
            calendar_payload_local,
            calendar_tag_local,
            "Temporal.ZonedDateTime calendar must be a string",
            "Invalid Temporal.ZonedDateTime calendar",
            function,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_alloc_temporal_zoned_date_time(
        &mut self,
        epoch_payload_local: u32,
        epoch_tag_local: u32,
        time_zone_payload_local: u32,
        time_zone_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_ZONED_DATE_TIME_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
                time_zone_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                time_zone_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                calendar_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.store_i64_local_at_offset(record_local, offset, local, function);
        }
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME,
            function,
        );
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(record_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_epoch_nanoseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_epoch_milliseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_heap_bigint_millisecond_quotient(
            nanoseconds_payload_local,
            quotient_local,
            remainder_local,
            negative_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(negative_local);
        self.release_temp_local(remainder_local);
        self.release_temp_local(quotient_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_offset(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_seconds_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_offset_seconds_from_receiver(
            offset_seconds_local,
            function,
        )?;
        self.emit_temporal_format_fixed_time_zone_offset(
            offset_seconds_local,
            output_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(output_payload_local);
        self.release_temp_local(offset_seconds_local);
        Ok(())
    }

    fn emit_temporal_format_fixed_time_zone_offset(
        &mut self,
        offset_seconds_local: u32,
        output_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let magnitude_seconds_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let separator_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("+")));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(magnitude_seconds_local));
        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(hour_payload_local));
        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(minute_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            hour_payload_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload(":")));
        function.instruction(&Instruction::LocalSet(separator_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            separator_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            minute_payload_local,
            2,
            function,
        )?;

        self.release_temp_local(separator_payload_local);
        self.release_temp_local(minute_payload_local);
        self.release_temp_local(hour_payload_local);
        self.release_temp_local(magnitude_seconds_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_offset_nanoseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_seconds_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_offset_seconds_from_receiver(
            offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(offset_seconds_local);
        Ok(())
    }

    fn emit_temporal_zoned_date_time_offset_seconds_from_receiver(
        &mut self,
        offset_seconds_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_payload_local,
            function,
        );
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            offset_seconds_local,
            function,
        )?;
        self.release_temp_local(time_zone_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_time_zone_id(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_string_slot(
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_TAG_OFFSET,
            function,
        )
    }

    pub(crate) fn emit_temporal_zoned_date_time_calendar_id(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_zoned_date_time_string_slot(
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
            function,
        )
    }

    /// `GetISODateTimeFor(timeZone, epochNanoseconds)`: the receiver's epoch
    /// nanoseconds shifted by its time zone's offset and split into the seven
    /// components `emit_date_components_from_time` produces, plus the
    /// sub-millisecond `remainder_local` those components do not carry.
    ///
    /// Extracted verbatim from `emit_temporal_zoned_date_time_iso_field` so
    /// `toPlainDateTime` runs the *same* sequence rather than a second copy of
    /// it. A copy would be free to drift on the two subtleties this body
    /// carries: the negative-remainder correction that turns a truncating
    /// `I64DivS` into a floor, and the two-limb heap-BigInt path for epoch
    /// values outside the inline range.
    ///
    /// The `component_locals` array is `[year, month, day, hour, minute,
    /// second, millisecond]` and every entry comes back as an **f64 bit
    /// pattern**, with `month` **0-based**. `remainder_local` is the exception:
    /// a plain non-negative `i64` nanosecond count inside the millisecond.
    /// Callers that want integers must reinterpret and truncate.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_zoned_date_time_local_components(
        &mut self,
        record_local: u32,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        milliseconds_local: u32,
        remainder_local: u32,
        negative_local: u32,
        offset_seconds_local: u32,
        time_zone_payload_local: u32,
        local_time_payload_local: u32,
        component_locals: [u32; 7],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [
            year_payload_local,
            month_payload_local,
            day_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            millisecond_payload_local,
        ] = component_locals;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
            time_zone_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_heap_bigint_millisecond_quotient(
            nanoseconds_payload_local,
            milliseconds_local,
            remainder_local,
            negative_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            offset_seconds_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(local_time_payload_local));
        self.emit_date_components_from_time(
            local_time_payload_local,
            year_payload_local,
            month_payload_local,
            day_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            millisecond_payload_local,
            function,
        );
        Ok(())
    }

    /// The local ISO date-time fields of a `Temporal.ZonedDateTime`, one
    /// accessor at a time.
    ///
    /// Every arm shares the epoch-nanoseconds -> local-components sequence at
    /// the top; `field` selects which component comes back and, through
    /// [`ZdtFieldResult`], how it was delivered.
    pub(crate) fn emit_temporal_zoned_date_time_iso_field(
        &mut self,
        field: ZonedDateTimeField,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let local_time_payload_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let day_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let millisecond_payload_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_temporal_zoned_date_time_local_components(
            record_local,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            milliseconds_local,
            remainder_local,
            negative_local,
            offset_seconds_local,
            time_zone_payload_local,
            local_time_payload_local,
            [
                year_payload_local,
                month_payload_local,
                day_payload_local,
                hour_payload_local,
                minute_payload_local,
                second_payload_local,
                millisecond_payload_local,
            ],
            function,
        )?;

        // Exhaustive over `ZonedDateTimeField`, no catch-all: a new accessor
        // must state both what it emits and how it delivered it, in the same
        // place.
        let delivery = match field {
            ZonedDateTimeField::Year => {
                function.instruction(&Instruction::LocalGet(year_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Month => {
                function.instruction(&Instruction::LocalGet(month_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
                function.instruction(&Instruction::F64Add);
                function.instruction(&Instruction::I64ReinterpretF64);
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Day => {
                function.instruction(&Instruction::LocalGet(day_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Hour => {
                function.instruction(&Instruction::LocalGet(hour_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Minute => {
                function.instruction(&Instruction::LocalGet(minute_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Second => {
                function.instruction(&Instruction::LocalGet(second_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Millisecond => {
                function.instruction(&Instruction::LocalGet(millisecond_payload_local));
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Microsecond => {
                function.instruction(&Instruction::LocalGet(remainder_local));
                function.instruction(&Instruction::I64Const(1_000));
                function.instruction(&Instruction::I64DivU);
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::Nanosecond => {
                function.instruction(&Instruction::LocalGet(remainder_local));
                function.instruction(&Instruction::I64Const(1_000));
                function.instruction(&Instruction::I64RemU);
                function.instruction(&Instruction::F64ConvertI64U);
                function.instruction(&Instruction::I64ReinterpretF64);
                ZdtFieldResult::NumberOnStack
            }
            ZonedDateTimeField::MonthCode => {
                let month_number_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(month_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncF64U);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(month_number_local));
                function.instruction(&Instruction::I64Const(self.strings.payload("M01")));
                function.instruction(&Instruction::LocalSet(self.result_local));
                for month in 2..=12 {
                    function.instruction(&Instruction::LocalGet(month_number_local));
                    function.instruction(&Instruction::I64Const(month));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(&format!("M{month:02}")),
                    ));
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(month_number_local);
                ZdtFieldResult::WrittenByCallee
            }
            ZonedDateTimeField::Era => {
                self.emit_temporal_zoned_date_time_era_field(
                    record_local,
                    year_payload_local,
                    TemporalEraField::Era,
                    function,
                );
                ZdtFieldResult::WrittenByCallee
            }
            ZonedDateTimeField::EraYear => {
                self.emit_temporal_zoned_date_time_era_field(
                    record_local,
                    year_payload_local,
                    TemporalEraField::EraYear,
                    function,
                );
                ZdtFieldResult::WrittenByCallee
            }
        };
        match delivery {
            ZdtFieldResult::NumberOnStack => {
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            ZdtFieldResult::WrittenByCallee => {}
        }

        for local in [
            millisecond_payload_local,
            second_payload_local,
            minute_payload_local,
            hour_payload_local,
            day_payload_local,
            month_payload_local,
            year_payload_local,
            local_time_payload_local,
            time_zone_payload_local,
            offset_seconds_local,
            negative_local,
            remainder_local,
            milliseconds_local,
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `era` / `eraYear` for a `Temporal.ZonedDateTime`, from the receiver's
    /// `[[Calendar]]` and its *local* ISO year.
    ///
    /// The answer comes out of [`Self::emit_temporal_calendar_era_field`], the
    /// same emitter `PlainDate`, `PlainDateTime` and `PlainYearMonth` use, so
    /// ZonedDateTime cannot disagree with them about where the year-0 boundary
    /// falls or which era codes exist. That emitter writes both halves of the
    /// result pair, which is why the caller reports
    /// [`ZdtFieldResult::WrittenByCallee`].
    ///
    /// The one unit conversion is load-bearing.
    /// `emit_date_components_from_time` leaves every component as an **f64 bit
    /// pattern** (each of its arms ends in `I64ReinterpretF64`), whereas
    /// `emit_temporal_calendar_era_field` takes a plain `i64` ISO year, the way
    /// `PlainDate` hands it one straight out of its record. Passing the bit
    /// pattern through unconverted would compare a reinterpreted double against
    /// `0` and put essentially every year in the `ce` branch — a bug invisible
    /// for positive years, which is exactly why
    /// `wasm_temporal_zoned_date_time_era.js` drives the `bce` side.
    /// `I64TruncF64S` is exact here: the value is an integral f64 well inside
    /// the `i64` range, since `ISODateTimeWithinLimits` bounds the year to
    /// ±275,760.
    fn emit_temporal_zoned_date_time_era_field(
        &mut self,
        record_local: u32,
        year_payload_local: u32,
        field: TemporalEraField,
        function: &mut Function,
    ) {
        let calendar_payload_local = self.reserve_temp_local();
        let iso_year_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(iso_year_local));
        self.emit_temporal_calendar_era_field(
            calendar_payload_local,
            iso_year_local,
            field,
            function,
        );
        self.release_temp_local(iso_year_local);
        self.release_temp_local(calendar_payload_local);
    }

    /// `Temporal.ZonedDateTime.prototype.toPlainDateTime`.
    ///
    /// The inverse of [`Self::emit_temporal_plain_date_time_to_zoned_date_time`]
    /// and the keystone of the era-boundary corpus: every
    /// `intl402/Temporal/ZonedDateTime/**/era-boundary-gregory.js` file asserts
    /// through `TemporalHelpers.assertPlainDateTime`, which requires a real
    /// `Temporal.PlainDateTime`, never the zoned value.
    ///
    /// It reuses the epoch-nanoseconds -> local-components sequence that
    /// [`Self::emit_temporal_zoned_date_time_iso_field`] already runs, then
    /// hands the nine ISO fields plus **the receiver's own calendar payload** to
    /// `CreateTemporalDateTime`. Carrying the calendar is what makes
    /// `result.era` answer `"bce"` rather than `undefined`; the ISO fields alone
    /// would produce an `iso8601` PlainDateTime that reports no era at all.
    pub(crate) fn emit_temporal_zoned_date_time_to_plain_date_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let local_time_payload_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let day_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let millisecond_payload_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_temporal_zoned_date_time_local_components(
            record_local,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            milliseconds_local,
            remainder_local,
            negative_local,
            offset_seconds_local,
            time_zone_payload_local,
            local_time_payload_local,
            [
                year_payload_local,
                month_payload_local,
                day_payload_local,
                hour_payload_local,
                minute_payload_local,
                second_payload_local,
                millisecond_payload_local,
            ],
            function,
        )?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            calendar_payload_local,
            function,
        );

        // The components arrive as f64 bit patterns; a `PlainDateTime` record
        // stores plain integers. `month_payload_local` is 0-based, matching
        // `emit_date_components_from_time`, so it gains the same `+ 1` the
        // `month` accessor applies.
        for (payload_local, field_local) in [
            (year_payload_local, field_locals[0]),
            (day_payload_local, field_locals[2]),
            (hour_payload_local, field_locals[3]),
            (minute_payload_local, field_locals[4]),
            (second_payload_local, field_locals[5]),
            (millisecond_payload_local, field_locals[6]),
        ] {
            function.instruction(&Instruction::LocalGet(payload_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::I64TruncF64S);
            function.instruction(&Instruction::LocalSet(field_local));
        }
        function.instruction(&Instruction::LocalGet(month_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(field_locals[1]));
        // `remainder_local` is the sub-millisecond nanosecond count, already a
        // plain non-negative `i64` — the one component that is not a bit
        // pattern, because it never went through `emit_date_components_from_time`.
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(field_locals[7]));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(field_locals[8]));

        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            None,
            function,
        )?;

        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            calendar_payload_local,
            millisecond_payload_local,
            second_payload_local,
            minute_payload_local,
            hour_payload_local,
            day_payload_local,
            month_payload_local,
            year_payload_local,
            local_time_payload_local,
            time_zone_payload_local,
            offset_seconds_local,
            negative_local,
            remainder_local,
            milliseconds_local,
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_record_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let other_payload_local = self.reserve_temp_local();
        let other_tag_local = self.reserve_temp_local();
        let other_record_local = self.reserve_temp_local();
        let receiver_epoch_payload_local = self.reserve_temp_local();
        let receiver_epoch_tag_local = self.reserve_temp_local();
        let other_epoch_payload_local = self.reserve_temp_local();
        let other_epoch_tag_local = self.reserve_temp_local();
        let receiver_time_zone_local = self.reserve_temp_local();
        let other_time_zone_local = self.reserve_temp_local();
        let receiver_calendar_local = self.reserve_temp_local();
        let other_calendar_local = self.reserve_temp_local();
        let equal_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(receiver_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalZonedDateTimeFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.ZonedDateTime.from`",
                )
            })?;
        self.emit_direct_js_call(
            &from_meta,
            None,
            &[(argument_payload_local, argument_tag_local)],
            other_payload_local,
            other_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            other_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            other_record_local,
            function,
        );
        for (record, offset, local) in [
            (
                receiver_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                receiver_epoch_payload_local,
            ),
            (
                receiver_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                receiver_epoch_tag_local,
            ),
            (
                other_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                other_epoch_payload_local,
            ),
            (
                other_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                other_epoch_tag_local,
            ),
            (
                receiver_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                receiver_time_zone_local,
            ),
            (
                other_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_TIME_ZONE_PAYLOAD_OFFSET,
                other_time_zone_local,
            ),
            (
                receiver_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                receiver_calendar_local,
            ),
            (
                other_record_local,
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                other_calendar_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record, offset, local, function);
        }
        function.instruction(&Instruction::LocalGet(receiver_epoch_tag_local));
        function.instruction(&Instruction::LocalGet(other_epoch_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_tagged_payload_equality_i32(
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_epoch_tag_local,
            other_epoch_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_mixed_bigint_equality_i32(
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_epoch_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(equal_local));
        self.emit_string_payload_equality_i32(
            receiver_time_zone_local,
            other_time_zone_local,
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(equal_local));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(equal_local));
        self.emit_string_payload_equality_i32(
            receiver_calendar_local,
            other_calendar_local,
            function,
        );
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalGet(equal_local));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            equal_local,
            other_calendar_local,
            receiver_calendar_local,
            other_time_zone_local,
            receiver_time_zone_local,
            other_epoch_tag_local,
            other_epoch_payload_local,
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_record_local,
            other_tag_local,
            other_payload_local,
            argument_tag_local,
            argument_payload_local,
            receiver_record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_temporal_zoned_date_time_string_slot(
        &mut self,
        payload_offset: u64,
        tag_offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            payload_offset,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            tag_offset,
            self.result_tag_local,
            function,
        );
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_to_instant(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            epoch_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
            epoch_tag_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_instant(
            epoch_payload_local,
            epoch_tag_local,
            prototype_payload_local,
            function,
        )?;
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(epoch_tag_local);
        self.release_temp_local(epoch_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_zoned_date_time_with_time_zone(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_zoned_date_time_record_from_receiver(record_local, function)?;
        self.emit_builtin_arg_to_locals(0, time_zone_payload_local, time_zone_tag_local, function);
        self.emit_temporal_zoned_date_time_time_zone(
            time_zone_payload_local,
            time_zone_tag_local,
            function,
        )?;
        for (offset, local) in [
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                epoch_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_EPOCH_NANOSECONDS_TAG_OFFSET,
                epoch_tag_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
            (
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_TAG_OFFSET,
                calendar_tag_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_zoned_date_time(
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            prototype_payload_local,
            function,
        )?;

        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(calendar_tag_local);
        self.release_temp_local(calendar_payload_local);
        self.release_temp_local(epoch_tag_local);
        self.release_temp_local(epoch_payload_local);
        self.release_temp_local(time_zone_tag_local);
        self.release_temp_local(time_zone_payload_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    fn emit_temporal_zoned_date_time_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime receiver does not have [[InitializedTemporalZonedDateTime]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.ZonedDateTime receiver does not have [[InitializedTemporalZonedDateTime]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_value_to_bigint_locals(
            argument_tag_local,
            argument_payload_local,
            false,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_temporal_instant_validate_range(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;
        self.emit_error_new_target_prototype_to_local(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_instant(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            prototype_payload_local,
            function,
        )?;

        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }

    fn emit_temporal_parse_iso_string(
        &mut self,
        string_payload_local: u32,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        parse_goal: TemporalIsoParseGoal,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let main_end_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let valid_local = self.reserve_temp_local();
        let negative_year_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let second_local = self.reserve_temp_local();
        let fraction_local = self.reserve_temp_local();
        let fraction_digits_local = self.reserve_temp_local();
        let date_separated_local = self.reserve_temp_local();
        let time_separated_local = self.reserve_temp_local();
        let has_minute_local = self.reserve_temp_local();
        let has_second_local = self.reserve_temp_local();
        let has_time_local = self.reserve_temp_local();
        let offset_kind_local = self.reserve_temp_local();
        let offset_sign_local = self.reserve_temp_local();
        let offset_hour_local = self.reserve_temp_local();
        let offset_minute_local = self.reserve_temp_local();
        let offset_second_local = self.reserve_temp_local();
        let offset_has_second_local = self.reserve_temp_local();
        let offset_fraction_local = self.reserve_temp_local();
        let offset_fraction_digits_local = self.reserve_temp_local();
        let maximum_day_local = self.reserve_temp_local();
        let calendar_count_local = self.reserve_temp_local();
        let calendar_critical_local = self.reserve_temp_local();
        let timezone_count_local = self.reserve_temp_local();
        let annotation_start_local = self.reserve_temp_local();
        let annotation_equals_local = self.reserve_temp_local();
        let annotation_critical_local = self.reserve_temp_local();
        let annotation_key_uppercase_local = self.reserve_temp_local();
        let annotation_numeric_timezone_local = self.reserve_temp_local();
        let annotation_colon_count_local = self.reserve_temp_local();
        let time_zone_start_local = self.reserve_temp_local();
        let time_zone_end_local = self.reserve_temp_local();
        let calendar_start_local = self.reserve_temp_local();
        let calendar_end_local = self.reserve_temp_local();
        let time_zone_offset_seconds_local = self.reserve_temp_local();
        let selected_offset_seconds_local = self.reserve_temp_local();
        let selected_offset_subsecond_local = self.reserve_temp_local();
        let offset_matches_time_zone_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let era_local = self.reserve_temp_local();
        let adjusted_year_local = self.reserve_temp_local();
        let month_index_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let parse_locals = [
            string_offset_local,
            string_len_local,
            main_end_local,
            cursor_local,
            byte_local,
            valid_local,
            negative_year_local,
            year_local,
            month_local,
            day_local,
            hour_local,
            minute_local,
            second_local,
            fraction_local,
            fraction_digits_local,
            date_separated_local,
            time_separated_local,
            has_minute_local,
            has_second_local,
            has_time_local,
            offset_kind_local,
            offset_sign_local,
            offset_hour_local,
            offset_minute_local,
            offset_second_local,
            offset_has_second_local,
            offset_fraction_local,
            offset_fraction_digits_local,
            maximum_day_local,
            calendar_count_local,
            calendar_critical_local,
            timezone_count_local,
            annotation_start_local,
            annotation_equals_local,
            annotation_critical_local,
            annotation_key_uppercase_local,
            annotation_numeric_timezone_local,
            annotation_colon_count_local,
            time_zone_start_local,
            time_zone_end_local,
            calendar_start_local,
            calendar_end_local,
            time_zone_offset_seconds_local,
            selected_offset_seconds_local,
            selected_offset_subsecond_local,
            offset_matches_time_zone_local,
            days_local,
            era_local,
            adjusted_year_local,
            month_index_local,
            seconds_local,
            subsecond_local,
        ];

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        for (local, value) in [
            (cursor_local, 0),
            (valid_local, 1),
            (negative_year_local, 0),
            (date_separated_local, 0),
            (time_separated_local, 0),
            (has_minute_local, 0),
            (has_second_local, 0),
            (has_time_local, 0),
            (offset_kind_local, 0),
            (fraction_local, 0),
            (fraction_digits_local, 0),
            (offset_sign_local, 0),
            (offset_hour_local, 0),
            (offset_minute_local, 0),
            (offset_second_local, 0),
            (offset_has_second_local, 0),
            (offset_fraction_local, 0),
            (offset_fraction_digits_local, 0),
            (calendar_count_local, 0),
            (calendar_critical_local, 0),
            (timezone_count_local, 0),
            (time_zone_start_local, -1),
            (time_zone_end_local, -1),
            (calendar_start_local, -1),
            (calendar_end_local, -1),
            (time_zone_offset_seconds_local, 0),
            (selected_offset_seconds_local, 0),
            (selected_offset_subsecond_local, 0),
            (offset_matches_time_zone_local, 0),
        ] {
            function.instruction(&Instruction::I64Const(value));
            function.instruction(&Instruction::LocalSet(local));
        }

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::LocalSet(main_end_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(main_end_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_year_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(cursor_local));
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            year_local,
            6,
            function,
        );
        function.instruction(&Instruction::LocalGet(negative_year_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            year_local,
            4,
            function,
        );
        function.instruction(&Instruction::End);

        self.emit_temporal_peek_byte(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(date_separated_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            month_local,
            2,
            function,
        );
        function.instruction(&Instruction::LocalGet(date_separated_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.emit_temporal_expect_byte(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            b'-',
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            day_local,
            2,
            function,
        );

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_time_local));
        self.emit_temporal_peek_byte(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'T' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor_local, function);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            hour_local,
            2,
            function,
        );

        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(time_separated_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_minute_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(minute_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            minute_local,
            2,
            function,
        );
        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(time_separated_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_second_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_second_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(has_second_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(second_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            second_local,
            2,
            function,
        );
        self.emit_temporal_parse_optional_fraction(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            fraction_local,
            fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'z' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(offset_kind_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(offset_kind_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(offset_sign_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            offset_hour_local,
            2,
            function,
        );
        self.emit_temporal_parse_offset_tail(
            string_offset_local,
            cursor_local,
            main_end_local,
            byte_local,
            valid_local,
            offset_minute_local,
            offset_second_local,
            offset_has_second_local,
            offset_fraction_local,
            offset_fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        if matches!(parse_goal, TemporalIsoParseGoal::Instant) {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        for local in [
            hour_local,
            minute_local,
            second_local,
            fraction_local,
            fraction_digits_local,
        ] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        if matches!(parse_goal, TemporalIsoParseGoal::Instant) {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_validate_annotations(
            string_offset_local,
            main_end_local,
            string_len_local,
            cursor_local,
            byte_local,
            valid_local,
            calendar_count_local,
            calendar_critical_local,
            timezone_count_local,
            annotation_start_local,
            annotation_equals_local,
            annotation_critical_local,
            annotation_key_uppercase_local,
            annotation_numeric_timezone_local,
            annotation_colon_count_local,
            time_zone_start_local,
            time_zone_end_local,
            calendar_start_local,
            calendar_end_local,
            function,
        );

        self.emit_temporal_validate_date_time(
            year_local,
            month_local,
            day_local,
            hour_local,
            minute_local,
            second_local,
            offset_hour_local,
            offset_minute_local,
            offset_second_local,
            maximum_day_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            match parse_goal {
                TemporalIsoParseGoal::Instant => "Invalid Temporal.Instant string",
                TemporalIsoParseGoal::TimeZoneIdentifier { .. } => {
                    "Invalid Temporal time zone identifier"
                }
                TemporalIsoParseGoal::ZonedDateTimeSyntax { .. }
                | TemporalIsoParseGoal::ZonedDateTime { .. } => {
                    "Invalid Temporal.ZonedDateTime string"
                }
                TemporalIsoParseGoal::PlainDate { .. } => "Invalid Temporal.PlainDate string",
                TemporalIsoParseGoal::PlainDateTime { .. } => {
                    "Invalid Temporal.PlainDateTime string"
                }
                TemporalIsoParseGoal::PlainTime { .. } => "Invalid Temporal.PlainTime string",
            },
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        if let TemporalIsoParseGoal::TimeZoneIdentifier {
            time_zone_payload_local,
            time_zone_tag_local,
        } = parse_goal
        {
            function.instruction(&Instruction::LocalGet(timezone_count_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal time zone string requires an offset or bracketed time zone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload("UTC")));
            function.instruction(&Instruction::LocalSet(time_zone_payload_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(offset_has_second_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(offset_fraction_digits_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal time zone offset must use minute precision",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(offset_sign_local));
            function.instruction(&Instruction::LocalGet(offset_hour_local));
            function.instruction(&Instruction::I64Const(3_600));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(offset_minute_local));
            function.instruction(&Instruction::I64Const(60));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(time_zone_offset_seconds_local));
            self.emit_temporal_format_fixed_time_zone_offset(
                time_zone_offset_seconds_local,
                time_zone_payload_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(time_zone_end_local));
            function.instruction(&Instruction::LocalGet(time_zone_start_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_string_slice_payload_from_locals(
                string_payload_local,
                time_zone_start_local,
                self.scratch_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(time_zone_payload_local));
            self.emit_temporal_fixed_time_zone_offset_seconds(
                time_zone_payload_local,
                time_zone_offset_seconds_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(time_zone_tag_local));

            for local in parse_locals.iter().rev() {
                self.release_temp_local(*local);
            }
            return Ok(());
        }

        if let TemporalIsoParseGoal::PlainTime {
            hour_destination_local,
            minute_destination_local,
            second_destination_local,
            nanosecond_destination_local,
        } = parse_goal
        {
            // `09:00:00Z` and `2019-10-01T09:00:00Z` both name an instant, not
            // a wall-clock time, so the UTC designator is a RangeError.
            // A numeric offset (`offset_kind == 2`) is merely ignored.
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.PlainTime string must not use the UTC designator",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            // A date-only string never gains an implicit midnight.
            function.instruction(&Instruction::LocalGet(has_time_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Invalid Temporal.PlainTime string",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(timezone_count_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64GtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Invalid Temporal.PlainTime string",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            // A parsed leap second is clamped, not rejected: `23:59:60` is
            // `23:59:59`.
            function.instruction(&Instruction::LocalGet(second_local));
            function.instruction(&Instruction::I64Const(60));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(59));
            function.instruction(&Instruction::LocalSet(second_local));
            function.instruction(&Instruction::End);
            for (source, destination) in [
                (hour_local, hour_destination_local),
                (minute_local, minute_destination_local),
                (second_local, second_destination_local),
            ] {
                function.instruction(&Instruction::LocalGet(source));
                function.instruction(&Instruction::LocalSet(destination));
            }
            self.emit_temporal_scale_fraction_to_nanoseconds(
                fraction_local,
                fraction_digits_local,
                function,
            );
            function.instruction(&Instruction::LocalSet(nanosecond_destination_local));

            for local in parse_locals.iter().rev() {
                self.release_temp_local(*local);
            }
            return Ok(());
        }

        if let TemporalIsoParseGoal::PlainDateTime {
            year_destination_local,
            month_destination_local,
            day_destination_local,
            hour_destination_local,
            minute_destination_local,
            second_destination_local,
            nanosecond_destination_local,
            calendar_payload_local,
            calendar_tag_local,
        } = parse_goal
        {
            // `2019-10-01T09:00:00Z` names an instant, not a wall-clock
            // date-time, so the UTC designator is a RangeError. A numeric
            // offset (`offset_kind == 2`) is merely ignored.
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.PlainDateTime string must not use the UTC designator",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            self.emit_temporal_iso_calendar_annotation(
                string_payload_local,
                calendar_count_local,
                calendar_start_local,
                calendar_end_local,
                calendar_payload_local,
                calendar_tag_local,
                "Invalid Temporal.PlainDateTime calendar annotation",
                function,
            )?;

            // A parsed leap second is clamped, not rejected: `23:59:60` is
            // `23:59:59`.
            function.instruction(&Instruction::LocalGet(second_local));
            function.instruction(&Instruction::I64Const(60));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(59));
            function.instruction(&Instruction::LocalSet(second_local));
            function.instruction(&Instruction::End);

            for (source, destination) in [
                (year_local, year_destination_local),
                (month_local, month_destination_local),
                (day_local, day_destination_local),
                (hour_local, hour_destination_local),
                (minute_local, minute_destination_local),
                (second_local, second_destination_local),
            ] {
                function.instruction(&Instruction::LocalGet(source));
                function.instruction(&Instruction::LocalSet(destination));
            }
            self.emit_temporal_scale_fraction_to_nanoseconds(
                fraction_local,
                fraction_digits_local,
                function,
            );
            function.instruction(&Instruction::LocalSet(nanosecond_destination_local));

            for local in parse_locals.iter().rev() {
                self.release_temp_local(*local);
            }
            return Ok(());
        }

        if let TemporalIsoParseGoal::PlainDate {
            year_destination_local,
            month_destination_local,
            day_destination_local,
            calendar_payload_local,
            calendar_tag_local,
        } = parse_goal
        {
            // A `PlainDate` has no instant, so the UTC designator is not just
            // redundant, it is forbidden: `2019-10-01T09:00:00Z` must throw.
            // `offset_kind == 1` is the `Z` form; `2` is an explicit numeric
            // offset, which is merely ignored.
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.PlainDate string must not use the UTC designator",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            self.emit_temporal_iso_calendar_annotation(
                string_payload_local,
                calendar_count_local,
                calendar_start_local,
                calendar_end_local,
                calendar_payload_local,
                calendar_tag_local,
                "Invalid Temporal.PlainDate calendar annotation",
                function,
            )?;

            for (source, destination) in [
                (year_local, year_destination_local),
                (month_local, month_destination_local),
                (day_local, day_destination_local),
            ] {
                function.instruction(&Instruction::LocalGet(source));
                function.instruction(&Instruction::LocalSet(destination));
            }

            for local in parse_locals.iter().rev() {
                self.release_temp_local(*local);
            }
            return Ok(());
        }

        if let TemporalIsoParseGoal::ZonedDateTimeSyntax {
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
        }
        | TemporalIsoParseGoal::ZonedDateTime {
            time_zone_payload_local,
            time_zone_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            ..
        } = parse_goal
        {
            function.instruction(&Instruction::LocalGet(timezone_count_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.ZonedDateTime string requires one bracketed time zone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::LocalGet(time_zone_end_local));
            function.instruction(&Instruction::LocalGet(time_zone_start_local));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_string_slice_payload_from_locals(
                string_payload_local,
                time_zone_start_local,
                self.scratch_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(time_zone_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(time_zone_tag_local));
            self.emit_temporal_fixed_time_zone_offset_seconds(
                time_zone_payload_local,
                time_zone_offset_seconds_local,
                function,
            )?;

            self.emit_temporal_iso_calendar_annotation(
                string_payload_local,
                calendar_count_local,
                calendar_start_local,
                calendar_end_local,
                calendar_payload_local,
                calendar_tag_local,
                "Invalid Temporal.ZonedDateTime calendar annotation",
                function,
            )?;
        }

        if matches!(parse_goal, TemporalIsoParseGoal::ZonedDateTimeSyntax { .. }) {
            for local in parse_locals.iter().rev() {
                self.release_temp_local(*local);
            }
            return Ok(());
        }

        self.emit_temporal_scale_fraction_to_nanoseconds(
            fraction_local,
            fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::LocalSet(fraction_local));
        self.emit_temporal_scale_fraction_to_nanoseconds(
            offset_fraction_local,
            offset_fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::LocalSet(offset_fraction_local));
        function.instruction(&Instruction::LocalGet(offset_sign_local));
        function.instruction(&Instruction::LocalGet(offset_hour_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(offset_minute_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(offset_second_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
        function.instruction(&Instruction::LocalGet(offset_sign_local));
        function.instruction(&Instruction::LocalGet(offset_fraction_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(selected_offset_subsecond_local));

        if let TemporalIsoParseGoal::ZonedDateTime {
            offset_option_local,
            ..
        } = parse_goal
        {
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
            function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(selected_offset_subsecond_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(offset_kind_local));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(selected_offset_seconds_local));
            function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(selected_offset_subsecond_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(offset_matches_time_zone_local));

            function.instruction(&Instruction::LocalGet(offset_option_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::LocalGet(offset_matches_time_zone_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Temporal.ZonedDateTime offset does not match its fixed time zone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::LocalGet(offset_option_local));
            function.instruction(&Instruction::I64Const(3));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(offset_option_local));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(offset_matches_time_zone_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(time_zone_offset_seconds_local));
            function.instruction(&Instruction::LocalSet(selected_offset_seconds_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(selected_offset_subsecond_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        self.emit_temporal_days_from_civil(
            year_local,
            month_local,
            day_local,
            adjusted_year_local,
            era_local,
            month_index_local,
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(SECONDS_PER_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(second_local));
        function.instruction(&Instruction::I64Const(59));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(59));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(second_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(selected_offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::LocalGet(selected_offset_subsecond_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_normalize_seconds_and_subseconds(
            seconds_local,
            subsecond_local,
            function,
        );
        self.emit_temporal_epoch_nanoseconds_bigint(
            seconds_local,
            subsecond_local,
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;
        self.emit_temporal_instant_validate_range(
            nanoseconds_payload_local,
            nanoseconds_tag_local,
            function,
        )?;

        for local in parse_locals.iter().rev() {
            self.release_temp_local(*local);
        }
        Ok(())
    }

    /// Resolves the `[u-ca=...]` annotation an ISO string may carry into a
    /// calendar payload. `emit_temporal_validate_annotations` has already
    /// captured only the FIRST annotation and rejected a repeated-and-critical
    /// pair, which is exactly the split Test262 asks for:
    /// `[u-ca=iso8601][u-ca=discord]` succeeds with the second ignored, while
    /// `[u-ca=iso8601][!u-ca=iso8601]` throws. Do not "fix" that asymmetry.
    ///
    /// The annotation value goes through the same
    /// [`TemporalCalendarId`] table as `CanonicalizeCalendar`, so
    /// `Temporal.PlainDate.from("2000-05-02[u-ca=gregory]").calendarId` is
    /// `"gregory"` and the round trip through `toString` is closed. An
    /// annotation naming no known calendar stays a RangeError, which is what
    /// `withCalendar/calendar-invalid-iso-string.js` pins with
    /// `"1997-12-04[u-ca=notacal]"`.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_iso_calendar_annotation(
        &mut self,
        string_payload_local: u32,
        calendar_count_local: u32,
        calendar_start_local: u32,
        calendar_end_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(
            self.strings
                .payload(TemporalCalendarId::DEFAULT.canonical()),
        ));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
        function.instruction(&Instruction::LocalGet(calendar_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        let calendar_annotation_payload_local = self.reserve_temp_local();
        let expected_calendar_payload_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();
        let matched_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(calendar_end_local));
        function.instruction(&Instruction::LocalGet(calendar_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_slice_payload_from_locals(
            string_payload_local,
            calendar_start_local,
            self.scratch_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(calendar_annotation_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(matched_local));
        for calendar in TemporalCalendarId::ALL {
            let canonical_payload = self.strings.payload(calendar.canonical());
            for &spelling in calendar.spellings() {
                function.instruction(&Instruction::I64Const(self.strings.payload(spelling)));
                function.instruction(&Instruction::LocalSet(expected_calendar_payload_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(case_fold_local));
                self.emit_string_payload_equality_i32_with_ascii_case_folding(
                    calendar_annotation_payload_local,
                    expected_calendar_payload_local,
                    Some(case_fold_local),
                    function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(canonical_payload));
                function.instruction(&Instruction::LocalSet(calendar_payload_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(matched_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(matched_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(matched_local);
        self.release_temp_local(case_fold_local);
        self.release_temp_local(expected_calendar_payload_local);
        self.release_temp_local(calendar_annotation_payload_local);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `ParseISODateTime` restricted to the `TemporalDateString` goal. Wraps
    /// the private parser so the `Temporal.PlainDate` emitters can reach it
    /// without the goal enum leaving this module.
    pub(crate) fn emit_temporal_parse_plain_date_string(
        &mut self,
        string_payload_local: u32,
        year_destination_local: u32,
        month_destination_local: u32,
        day_destination_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // The `nanoseconds_*` out-parameters are dead for this goal — the
        // PlainDate arm returns before the epoch-nanosecond tail runs — but the
        // parser signature still wants somewhere to point them.
        let unused_payload_local = self.reserve_temp_local();
        let unused_tag_local = self.reserve_temp_local();
        self.emit_temporal_parse_iso_string(
            string_payload_local,
            unused_payload_local,
            unused_tag_local,
            TemporalIsoParseGoal::PlainDate {
                year_destination_local,
                month_destination_local,
                day_destination_local,
                calendar_payload_local,
                calendar_tag_local,
            },
            function,
        )?;
        self.release_temp_local(unused_tag_local);
        self.release_temp_local(unused_payload_local);
        Ok(())
    }

    /// `ParseISODateTime` restricted to the `TemporalDateTimeString` goal.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_parse_plain_date_time_string(
        &mut self,
        string_payload_local: u32,
        year_destination_local: u32,
        month_destination_local: u32,
        day_destination_local: u32,
        hour_destination_local: u32,
        minute_destination_local: u32,
        second_destination_local: u32,
        nanosecond_destination_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let unused_payload_local = self.reserve_temp_local();
        let unused_tag_local = self.reserve_temp_local();
        self.emit_temporal_parse_iso_string(
            string_payload_local,
            unused_payload_local,
            unused_tag_local,
            TemporalIsoParseGoal::PlainDateTime {
                year_destination_local,
                month_destination_local,
                day_destination_local,
                hour_destination_local,
                minute_destination_local,
                second_destination_local,
                nanosecond_destination_local,
                calendar_payload_local,
                calendar_tag_local,
            },
            function,
        )?;
        self.release_temp_local(unused_tag_local);
        self.release_temp_local(unused_payload_local);
        Ok(())
    }

    /// `ParseTemporalTimeString`. A bare time (`15:23`, `T15:23`,
    /// `152330-0800`) is rewritten as `0000-01-01T` plus the same tail so the
    /// one ISO parser can serve both spellings; a string that already carries
    /// a date is passed through untouched.
    ///
    /// The `AmbiguousTemporalTimeString` rules are checked here, before the
    /// rewrite, because `1214` is a legal `MMDD` date *and* a legal `HHMM`
    /// time, and the proposal resolves that tie by demanding the `T`
    /// designator. `1232` is not ambiguous — there is no 32nd day — so it
    /// stays a time.
    pub(crate) fn emit_temporal_parse_plain_time_string(
        &mut self,
        string_payload_local: u32,
        hour_destination_local: u32,
        minute_destination_local: u32,
        second_destination_local: u32,
        nanosecond_destination_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let main_end_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let date_form_local = self.reserve_temp_local();
        let designated_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let maximum_day_local = self.reserve_temp_local();
        let piece_local = self.reserve_temp_local();
        let rewritten_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(string_payload_local, offset_local, length_local, function);

        // `main_end` is the first annotation bracket, or the whole string.
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::LocalSet(main_end_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(main_end_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // A date-shaped head is one of `±YYYYYY…`, `YYYY-MM-DD…` or eight
        // digits followed by a date/time separator. Everything else is a bare
        // time.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(date_form_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(designated_local));
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_load_byte_at_index(offset_local, 0, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(date_form_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'T' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(designated_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_load_byte_at_index(offset_local, 4, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(digits_local));
        self.emit_temporal_load_byte_at_index(offset_local, 7, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(date_form_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(digits_local));
        for index in 0..8 {
            self.emit_temporal_load_byte_at_index(
                offset_local,
                index,
                cursor_local,
                byte_local,
                function,
            );
            self.emit_temporal_byte_is_digit(byte_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(digits_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        self.emit_temporal_load_byte_at_index(offset_local, 8, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'T' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b' ' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(digits_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(date_form_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(date_form_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        // `AmbiguousTemporalTimeString`: only an undesignated bare time can
        // collide with a `MM-DD` or `YYYY-MM` date.
        function.instruction(&Instruction::LocalGet(designated_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // (length, month digit indices, day digit indices, separator index)
        for (length, month_indices, day_indices, separator) in [
            (4_i64, [0_i64, 1_i64], Some([2_i64, 3_i64]), None),
            (5, [0, 1], Some([3, 4]), Some(2_i64)),
            (6, [4, 5], None, None),
            (7, [5, 6], None, Some(4)),
        ] {
            function.instruction(&Instruction::LocalGet(main_end_local));
            function.instruction(&Instruction::I64Const(length));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(digits_local));
            for index in 0..length {
                if Some(index) == separator {
                    self.emit_temporal_load_byte_at_index(
                        offset_local,
                        index,
                        cursor_local,
                        byte_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(byte_local));
                    function.instruction(&Instruction::I64Const(b'-' as i64));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(digits_local));
                    function.instruction(&Instruction::End);
                    continue;
                }
                self.emit_temporal_load_byte_at_index(
                    offset_local,
                    index,
                    cursor_local,
                    byte_local,
                    function,
                );
                self.emit_temporal_byte_is_digit(byte_local, function);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(digits_local));
                function.instruction(&Instruction::End);
            }
            self.emit_temporal_two_digit_value(
                offset_local,
                month_indices,
                cursor_local,
                byte_local,
                month_local,
                function,
            );
            match day_indices {
                Some(indices) => self.emit_temporal_two_digit_value(
                    offset_local,
                    indices,
                    cursor_local,
                    byte_local,
                    day_local,
                    function,
                ),
                // A `YYYY-MM` collision has no day component; 1 is always in
                // range for a valid month.
                None => {
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(day_local));
                }
            }
            // February is treated as 29 days: `0229` is ambiguous even though
            // the year is unknown.
            function.instruction(&Instruction::I64Const(31));
            function.instruction(&Instruction::LocalSet(maximum_day_local));
            for (month, days) in [(4_i64, 30_i64), (6, 30), (9, 30), (11, 30), (2, 29)] {
                function.instruction(&Instruction::LocalGet(month_local));
                function.instruction(&Instruction::I64Const(month));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(days));
                function.instruction(&Instruction::LocalSet(maximum_day_local));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::LocalGet(digits_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(12));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(day_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::LocalGet(day_local));
            function.instruction(&Instruction::LocalGet(maximum_day_local));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                "Ambiguous Temporal.PlainTime string requires the T designator",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        // Rewrite: `0000-01-01T` + the tail, minus any `T` designator.
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::LocalGet(designated_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_string_slice_payload_from_locals(
            string_payload_local,
            designated_local,
            self.scratch_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(piece_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("0000-01-01T")));
        function.instruction(&Instruction::LocalSet(rewritten_local));
        self.emit_concat_string_payloads_local(rewritten_local, piece_local, function)?;
        function.instruction(&Instruction::LocalSet(rewritten_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(string_payload_local));
        function.instruction(&Instruction::LocalSet(rewritten_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_parse_iso_string(
            rewritten_local,
            piece_local,
            byte_local,
            TemporalIsoParseGoal::PlainTime {
                hour_destination_local,
                minute_destination_local,
                second_destination_local,
                nanosecond_destination_local,
            },
            function,
        )?;

        for local in [
            rewritten_local,
            piece_local,
            maximum_day_local,
            day_local,
            month_local,
            digits_local,
            designated_local,
            date_form_local,
            byte_local,
            cursor_local,
            main_end_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Loads the byte at a compile-time-known index, reusing `index_local` as
    /// the scratch cursor the byte loader wants.
    fn emit_temporal_load_byte_at_index(
        &mut self,
        string_offset_local: u32,
        index: i64,
        index_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(index));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
    }

    /// Two ASCII digits at fixed indices, read as a decimal number. The caller
    /// has already checked that both are digits.
    fn emit_temporal_two_digit_value(
        &mut self,
        string_offset_local: u32,
        indices: [i64; 2],
        index_local: u32,
        byte_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        self.emit_temporal_load_byte_at_index(
            string_offset_local,
            indices[0],
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_temporal_load_byte_at_index(
            string_offset_local,
            indices[1],
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_local));
    }

    fn emit_temporal_advance_cursor(&mut self, cursor_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
    }

    fn emit_temporal_byte_is_digit(&mut self, byte_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
    }

    fn emit_temporal_peek_byte(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_peek_byte_if_available(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::End);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_expect_byte(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        expected: u8,
        function: &mut Function,
    ) {
        self.emit_temporal_peek_byte(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(expected as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor_local, function);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_parse_fixed_decimal(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        destination_local: u32,
        width: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(destination_local));
        for _ in 0..width {
            self.emit_temporal_peek_byte(
                string_offset_local,
                cursor_local,
                end_local,
                byte_local,
                valid_local,
                function,
            );
            self.emit_temporal_byte_is_digit(byte_local, function);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(destination_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'0' as i64));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(destination_local));
            self.emit_temporal_advance_cursor(cursor_local, function);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_parse_optional_fraction(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        fraction_local: u32,
        fraction_digits_local: u32,
        function: &mut Function,
    ) {
        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(end_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_digits_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(fraction_digits_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_parse_offset_tail(
        &mut self,
        string_offset_local: u32,
        cursor_local: u32,
        end_local: u32,
        byte_local: u32,
        valid_local: u32,
        minute_local: u32,
        second_local: u32,
        has_second_local: u32,
        fraction_local: u32,
        fraction_digits_local: u32,
        function: &mut Function,
    ) {
        let separated_local = self.reserve_temp_local();
        let has_minute_local = self.reserve_temp_local();
        for local in [separated_local, has_minute_local, has_second_local] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(separated_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_minute_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_minute_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            valid_local,
            minute_local,
            2,
            function,
        );
        self.emit_temporal_peek_byte_if_available(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(separated_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_byte_is_digit(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_second_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(has_second_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(has_second_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_parse_fixed_decimal(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            valid_local,
            second_local,
            2,
            function,
        );
        self.emit_temporal_parse_optional_fraction(
            string_offset_local,
            cursor_local,
            end_local,
            byte_local,
            valid_local,
            fraction_local,
            fraction_digits_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.release_temp_local(has_minute_local);
        self.release_temp_local(separated_local);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_validate_annotations(
        &mut self,
        string_offset_local: u32,
        main_end_local: u32,
        string_len_local: u32,
        cursor_local: u32,
        byte_local: u32,
        valid_local: u32,
        calendar_count_local: u32,
        calendar_critical_local: u32,
        timezone_count_local: u32,
        annotation_start_local: u32,
        annotation_equals_local: u32,
        annotation_critical_local: u32,
        annotation_key_uppercase_local: u32,
        annotation_numeric_timezone_local: u32,
        annotation_colon_count_local: u32,
        time_zone_start_local: u32,
        time_zone_end_local: u32,
        calendar_start_local: u32,
        calendar_end_local: u32,
        function: &mut Function,
    ) {
        let annotation_end_local = self.reserve_temp_local();
        let key_is_calendar_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(main_end_local));
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_expect_byte(
            string_offset_local,
            cursor_local,
            string_len_local,
            byte_local,
            valid_local,
            b'[',
            function,
        );
        for (local, value) in [
            (annotation_critical_local, 0),
            (annotation_key_uppercase_local, 0),
            (annotation_numeric_timezone_local, 0),
            (annotation_colon_count_local, 0),
            (annotation_equals_local, -1),
        ] {
            function.instruction(&Instruction::I64Const(value));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_temporal_peek_byte(
            string_offset_local,
            cursor_local,
            string_len_local,
            byte_local,
            valid_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'!' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(annotation_critical_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(annotation_start_local));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(annotation_end_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(string_offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(annotation_end_local));
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(annotation_equals_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'=' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(annotation_equals_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'Z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(annotation_key_uppercase_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_colon_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(annotation_colon_count_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor_local, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(annotation_equals_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_key_uppercase_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_is_calendar_local));
        function.instruction(&Instruction::LocalGet(annotation_equals_local));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (offset, expected) in [(0, b'u'), (1, b'-'), (2, b'c'), (3, b'a')] {
            function.instruction(&Instruction::LocalGet(annotation_start_local));
            function.instruction(&Instruction::I64Const(offset));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.emit_load_string_byte(
                string_offset_local,
                self.scratch_local,
                byte_local,
                function,
            );
            // emit_load_string_byte consumes a local index, so materialize it.
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(expected as i64));
            function.instruction(&Instruction::I64Eq);
            if offset == 0 {
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(key_is_calendar_local));
            } else {
                function.instruction(&Instruction::LocalGet(key_is_calendar_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(key_is_calendar_local));
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_is_calendar_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(calendar_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_equals_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(calendar_start_local));
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalSet(calendar_end_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(calendar_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(calendar_count_local));
        function.instruction(&Instruction::LocalGet(calendar_critical_local));
        function.instruction(&Instruction::LocalGet(annotation_critical_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(calendar_critical_local));
        function.instruction(&Instruction::LocalGet(calendar_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(calendar_critical_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(annotation_critical_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(timezone_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::LocalSet(time_zone_start_local));
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalSet(time_zone_end_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(timezone_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalTee(timezone_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(
            string_offset_local,
            annotation_start_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(annotation_end_local));
        function.instruction(&Instruction::LocalGet(annotation_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(key_is_calendar_local);
        self.release_temp_local(annotation_end_local);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_validate_date_time(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        hour_local: u32,
        minute_local: u32,
        second_local: u32,
        offset_hour_local: u32,
        offset_minute_local: u32,
        offset_second_local: u32,
        maximum_day_local: u32,
        valid_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::LocalSet(maximum_day_local));
        for month in [4_i64, 6, 9, 11] {
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(30));
            function.instruction(&Instruction::LocalSet(maximum_day_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(28));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(maximum_day_local));
        function.instruction(&Instruction::End);

        for (local, minimum, maximum) in [
            (month_local, 1_i64, 12_i64),
            (day_local, 1, 31),
            (hour_local, 0, 23),
            (minute_local, 0, 59),
            (second_local, 0, 60),
            (offset_hour_local, 0, 23),
            (offset_minute_local, 0, 59),
            (offset_second_local, 0, 59),
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(minimum));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(valid_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_scale_fraction_to_nanoseconds(
        &mut self,
        fraction_local: u32,
        digit_count_local: u32,
        function: &mut Function,
    ) {
        let counter_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(digit_count_local));
        function.instruction(&Instruction::LocalSet(counter_local));
        for _ in 0..9 {
            function.instruction(&Instruction::LocalGet(counter_local));
            function.instruction(&Instruction::I64Const(9));
            function.instruction(&Instruction::I64LtU);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(fraction_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalSet(fraction_local));
            function.instruction(&Instruction::LocalGet(counter_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(counter_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(fraction_local));
        self.release_temp_local(counter_local);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_days_from_civil(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        adjusted_year_local: u32,
        era_local: u32,
        month_index_local: u32,
        days_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(adjusted_year_local));
        function.instruction(&Instruction::LocalGet(adjusted_year_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(adjusted_year_local));
        function.instruction(&Instruction::I64Const(399));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(adjusted_year_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(era_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-9));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(month_index_local));
        function.instruction(&Instruction::LocalGet(era_local));
        function.instruction(&Instruction::I64Const(146_097));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(adjusted_year_local));
        function.instruction(&Instruction::LocalGet(era_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(days_local));
        function.instruction(&Instruction::I64Const(365));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(month_index_local));
        function.instruction(&Instruction::I64Const(153));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(719_468));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
    }

    fn emit_temporal_normalize_seconds_and_subseconds(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_temporal_epoch_nanoseconds_bigint(
        &mut self,
        seconds_local: u32,
        subsecond_local: u32,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let negative_local = self.reserve_temp_local();
        let magnitude_seconds_local = self.reserve_temp_local();
        let magnitude_subsecond_local = self.reserve_temp_local();
        let low_word_local = self.reserve_temp_local();
        let low_product_local = self.reserve_temp_local();
        let high_product_local = self.reserve_temp_local();
        let low_limb_local = self.reserve_temp_local();
        let high_limb_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalSet(magnitude_seconds_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::LocalSet(magnitude_subsecond_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(magnitude_subsecond_local));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(magnitude_subsecond_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(low_word_local));
        function.instruction(&Instruction::LocalGet(low_word_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(low_product_local));
        function.instruction(&Instruction::LocalGet(magnitude_seconds_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_SECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(high_product_local));
        function.instruction(&Instruction::LocalGet(low_product_local));
        function.instruction(&Instruction::LocalGet(high_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low_limb_local));
        function.instruction(&Instruction::LocalGet(high_product_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::LocalGet(low_product_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high_limb_local));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::LocalGet(magnitude_subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(low_limb_local));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::LocalGet(magnitude_subsecond_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(high_limb_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::I64Const(i64::MAX));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::LocalSet(nanoseconds_tag_local));
        function.instruction(&Instruction::Else);
        let record_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(HEAP_BIGINT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.emit_heap_alloc_const(16, function)?;
        function.instruction(&Instruction::LocalSet(limbs_local));
        self.store_i64_local_at_offset(limbs_local, 0, low_limb_local, function);
        self.store_i64_local_at_offset(limbs_local, 8, high_limb_local, function);
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(limb_count_local));
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(low_word_local));
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_SIGN_OFFSET,
            low_word_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_BIGINT_LIMBS_CAP_OFFSET,
            limb_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(record_local));
        function.instruction(&Instruction::LocalSet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::LocalSet(nanoseconds_tag_local));
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(record_local);
        function.instruction(&Instruction::End);

        self.release_temp_local(high_limb_local);
        self.release_temp_local(low_limb_local);
        self.release_temp_local(high_product_local);
        self.release_temp_local(low_product_local);
        self.release_temp_local(low_word_local);
        self.release_temp_local(magnitude_subsecond_local);
        self.release_temp_local(magnitude_seconds_local);
        self.release_temp_local(negative_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_temporal_instant(
        &mut self,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let instant_payload_local = self.reserve_temp_local();
        let instant_record_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(instant_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_INSTANT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(instant_record_local));
        self.store_i64_local_at_offset(
            instant_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            instant_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            instant_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT,
            function,
        );
        self.store_i64_local_at_offset(
            instant_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            instant_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(instant_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(instant_record_local);
        self.release_temp_local(instant_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_epoch_nanoseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        self.emit_temporal_instant_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            self.result_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            self.result_tag_local,
            function,
        );
        self.release_temp_local(record_local);
        Ok(())
    }

    /// `floor(epochNanoseconds / 10^6)` as an f64, read out of the epoch
    /// nanoseconds slot pair of `record_local`.
    ///
    /// The division is a *floor*, not a truncation: `-1n` nanosecond is
    /// millisecond `-1`, not `0`, so the negative remainder is corrected
    /// explicitly. Both the small-integer and heap-BigInt representations of
    /// the slot are handled, because either can reach a record.
    ///
    /// `Temporal.Instant` and `Temporal.ZonedDateTime` lay their epoch
    /// nanoseconds out identically, which is why the offsets are parameters
    /// rather than baked in.
    pub(crate) fn emit_temporal_epoch_nanoseconds_record_to_milliseconds(
        &mut self,
        record_local: u32,
        payload_offset: u64,
        tag_offset: u64,
        dest_local: u32,
        function: &mut Function,
    ) {
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let quotient_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            record_local,
            payload_offset,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            tag_offset,
            nanoseconds_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_heap_bigint_millisecond_quotient(
            nanoseconds_payload_local,
            quotient_local,
            remainder_local,
            negative_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_local));

        self.release_temp_local(negative_local);
        self.release_temp_local(remainder_local);
        self.release_temp_local(quotient_local);
        self.release_temp_local(nanoseconds_tag_local);
        self.release_temp_local(nanoseconds_payload_local);
    }

    pub(crate) fn emit_temporal_instant_epoch_milliseconds(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();

        self.emit_temporal_instant_record_from_receiver(record_local, function)?;
        self.emit_temporal_epoch_nanoseconds_record_to_milliseconds(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            self.result_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(record_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_record_local = self.reserve_temp_local();
        let receiver_epoch_payload_local = self.reserve_temp_local();
        let receiver_epoch_tag_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let other_instant_payload_local = self.reserve_temp_local();
        let other_instant_tag_local = self.reserve_temp_local();
        let other_record_local = self.reserve_temp_local();
        let other_epoch_payload_local = self.reserve_temp_local();
        let other_epoch_tag_local = self.reserve_temp_local();

        self.emit_temporal_instant_record_from_receiver(receiver_record_local, function)?;
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            receiver_epoch_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            receiver_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            receiver_epoch_tag_local,
            function,
        );

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        let instant_from_meta = self
            .functions
            .get(&StandardBuiltinId::TemporalInstantFrom.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Temporal.Instant.from`",
                )
            })?;
        self.emit_direct_js_call(
            &instant_from_meta,
            None,
            &[(argument_payload_local, argument_tag_local)],
            other_instant_payload_local,
            other_instant_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            other_instant_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            other_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            other_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            other_epoch_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            other_record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            other_epoch_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(receiver_epoch_tag_local));
        function.instruction(&Instruction::LocalGet(other_epoch_tag_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_nonstring_tagged_payload_equality_i32(
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_epoch_tag_local,
            other_epoch_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_mixed_bigint_equality_i32(
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            other_epoch_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            other_epoch_tag_local,
            other_epoch_payload_local,
            other_record_local,
            other_instant_tag_local,
            other_instant_payload_local,
            argument_tag_local,
            argument_payload_local,
            receiver_epoch_tag_local,
            receiver_epoch_payload_local,
            receiver_record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_to_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let nanoseconds_payload_local = self.reserve_temp_local();
        let nanoseconds_tag_local = self.reserve_temp_local();
        let milliseconds_local = self.reserve_temp_local();
        let remainder_local = self.reserve_temp_local();
        let negative_local = self.reserve_temp_local();
        let time_payload_local = self.reserve_temp_local();
        let year_payload_local = self.reserve_temp_local();
        let month_payload_local = self.reserve_temp_local();
        let date_payload_local = self.reserve_temp_local();
        let hour_payload_local = self.reserve_temp_local();
        let minute_payload_local = self.reserve_temp_local();
        let second_payload_local = self.reserve_temp_local();
        let millisecond_payload_local = self.reserve_temp_local();
        let fraction_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let absolute_year_payload_local = self.reserve_temp_local();

        self.emit_temporal_instant_record_from_receiver(record_local, function)?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
            nanoseconds_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
            nanoseconds_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::LocalGet(nanoseconds_payload_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(remainder_local));
        // CONVENTION, shared with `emit_temporal_heap_bigint_millisecond_quotient`
        // in the `Else` arm: `remainder_local` is the **non-negative magnitude**
        // of the sub-millisecond part; the sign lives only in `negative_local`.
        // The floor correction below is written for that convention.
        //
        // `I64RemS` takes the sign of the dividend, so this arm handed the
        // correction a negative remainder while the heap-bigint arm handed it a
        // magnitude (it reduces the limbs with `I64RemU` and negates only the
        // quotient). `NANOSECONDS_PER_MILLISECOND - remainder` then *added*
        // where it meant to subtract:
        // `new Temporal.Instant(-13849764_999_999_999n).toJSON()` rendered
        // `1969-07-24T16:50:35.001999999Z` instead of `...35.000000001Z`,
        // because 1e6 - -999_999 is 1_999_999 rather than 1.
        //
        // This is the only one of the three sites that reads the remainder's
        // *value*: `emit_temporal_zoned_date_time_epoch_milliseconds` and
        // `emit_temporal_epoch_nanoseconds_record_to_milliseconds` only test it
        // with `I64Eqz`, for which the sign is immaterial. Caught by
        // `built-ins/Temporal/Instant/prototype/toJSON/negative-epochnanoseconds.js`.
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_heap_bigint_millisecond_quotient(
            nanoseconds_payload_local,
            milliseconds_local,
            remainder_local,
            negative_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(milliseconds_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(milliseconds_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(time_payload_local));
        self.emit_date_components_from_time(
            time_payload_local,
            year_payload_local,
            month_payload_local,
            date_payload_local,
            hour_payload_local,
            minute_payload_local,
            second_payload_local,
            millisecond_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(millisecond_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(fraction_local));

        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(absolute_year_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::LocalSet(absolute_year_payload_local));
        function.instruction(&Instruction::LocalGet(year_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(9_999.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("+")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            6,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_date_append_padded_decimal(
            output_payload_local,
            absolute_year_payload_local,
            4,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("-")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        function.instruction(&Instruction::LocalGet(month_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(month_payload_local));
        for (component_payload_local, minimum_width, separator) in [
            (month_payload_local, 2, "-"),
            (date_payload_local, 2, "T"),
            (hour_payload_local, 2, ":"),
            (minute_payload_local, 2, ":"),
        ] {
            self.emit_date_append_padded_decimal(
                output_payload_local,
                component_payload_local,
                minimum_width,
                function,
            )?;
            function.instruction(&Instruction::I64Const(self.strings.payload(separator)));
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_concat_string_payloads_local(
                output_payload_local,
                piece_payload_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(output_payload_local));
        }
        self.emit_date_append_padded_decimal(
            output_payload_local,
            second_payload_local,
            2,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(".")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        // `FormatFractionalSeconds` with `auto` precision: render the fraction
        // as nine digits and strip **every** trailing zero, not just whole
        // groups of three. The 3/6/9 cascade this replaces printed
        // `new Temporal.Instant(30_123_400_000n)` as `…:30.123400Z`, because
        // 123,400,000 is divisible by 1,000 but not by 1,000,000 — so it took
        // the six-digit arm — where the spec asks for `…:30.1234Z`
        // (`Instant/prototype/toJSON/basic.js` case 4).
        //
        // `emit_date_append_padded_decimal` takes a Rust-level width, so the
        // choice is a cascade over the nine possible widths rather than a
        // runtime loop: the first `width` whose divisor divides the fraction
        // exactly is the one with no trailing zero left. `fraction_local` is
        // known non-zero here (the enclosing `If` handles zero by emitting no
        // fraction at all), so the `width == 9` fallthrough is reached only by
        // a fraction with a significant nanosecond digit.
        const FRACTION_DIGITS: u32 = 9;
        for width in 1..FRACTION_DIGITS {
            let divisor = 10_i64.pow(FRACTION_DIGITS - width);
            function.instruction(&Instruction::LocalGet(fraction_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64RemU);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(fraction_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivU);
            function.instruction(&Instruction::F64ConvertI64U);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(piece_payload_local));
            self.emit_date_append_padded_decimal(
                output_payload_local,
                piece_payload_local,
                width,
                function,
            )?;
            function.instruction(&Instruction::Else);
        }
        function.instruction(&Instruction::LocalGet(fraction_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_date_append_padded_decimal(
            output_payload_local,
            piece_payload_local,
            FRACTION_DIGITS,
            function,
        )?;
        for _ in 1..FRACTION_DIGITS {
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("Z")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            absolute_year_payload_local,
            piece_payload_local,
            output_payload_local,
            fraction_local,
            millisecond_payload_local,
            second_payload_local,
            minute_payload_local,
            hour_payload_local,
            date_payload_local,
            month_payload_local,
            year_payload_local,
            time_payload_local,
            negative_local,
            remainder_local,
            milliseconds_local,
            nanoseconds_tag_local,
            nanoseconds_payload_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_temporal_instant_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant receiver does not have [[InitializedTemporalInstant]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Instant receiver does not have [[InitializedTemporalInstant]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    pub(crate) fn emit_temporal_instant_validate_range(
        &mut self,
        nanoseconds_payload_local: u32,
        nanoseconds_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let sign_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        let high_limb_local = self.reserve_temp_local();
        let low_limb_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(nanoseconds_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            nanoseconds_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            nanoseconds_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            nanoseconds_payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_instant_range_error(function)?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(limbs_local, 0, low_limb_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(high_limb_local));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(limbs_local, 8, high_limb_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_INSTANT_LIMIT_HIGH_LIMB));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(high_limb_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_INSTANT_LIMIT_HIGH_LIMB));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(low_limb_local));
        function.instruction(&Instruction::I64Const(TEMPORAL_INSTANT_LIMIT_LOW_LIMB));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_instant_range_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(low_limb_local);
        self.release_temp_local(high_limb_local);
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(sign_local);
        Ok(())
    }

    fn emit_temporal_instant_range_error(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error(
            RANGE_ERROR_NAME,
            "Temporal.Instant epoch nanoseconds are outside the supported range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_temporal_heap_bigint_millisecond_quotient(
        &mut self,
        bigint_payload_local: u32,
        quotient_local: u32,
        remainder_local: u32,
        negative_local: u32,
        function: &mut Function,
    ) {
        let sign_local = self.reserve_temp_local();
        let limbs_local = self.reserve_temp_local();
        let limb_count_local = self.reserve_temp_local();
        let limb_local = self.reserve_temp_local();
        let word_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            bigint_payload_local,
            HEAP_BIGINT_SIGN_OFFSET,
            sign_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            bigint_payload_local,
            HEAP_BIGINT_LIMBS_PTR_OFFSET,
            limbs_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            bigint_payload_local,
            HEAP_BIGINT_LIMBS_LEN_OFFSET,
            limb_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalTee(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(word_local));
        function.instruction(&Instruction::LocalGet(word_local));
        function.instruction(&Instruction::LocalGet(limb_count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(limbs_local));
        function.instruction(&Instruction::LocalGet(word_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg64(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(limb_local));
        function.instruction(&Instruction::LocalGet(limb_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(word_local));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(word_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::LocalGet(remainder_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_MILLISECOND));
        function.instruction(&Instruction::I64RemU);
        function.instruction(&Instruction::LocalSet(remainder_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(negative_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(quotient_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(quotient_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
        self.release_temp_local(word_local);
        self.release_temp_local(limb_local);
        self.release_temp_local(limb_count_local);
        self.release_temp_local(limbs_local);
        self.release_temp_local(sign_local);
    }

    /// Emits the five-byte stub body (`i64.const 0` ×4 + `end`) both Temporal
    /// calendar helpers use when no Temporal builtin is compiled. The slots are
    /// reserved unconditionally so the fixed helper offsets never shift with
    /// the shape of the program; only the bodies are elided.
    ///
    /// It takes the helper it is standing in for because a stub is still that
    /// helper's body: the elided and real bodies then enter through the same
    /// door, and neither arm of `if real` can start a body without naming which
    /// helper it belongs to.
    fn temporal_calendar_helper_stub(&mut self, helper: RuntimeHelperId) -> Function {
        let mut function = self.begin_helper_body(helper);
        for _ in 0..4 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::End);
        self.finish_function(function)
    }

    /// Compiles the `ParseTemporalCalendarString` date-shaped probe helper: one
    /// inlined copy of the ISO date parser that all three date-ish goals reuse
    /// by rewriting the string first.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=candidate string
    /// payload, 1=rewrite form (0 = none/`TemporalDateString`, 1 =
    /// `TemporalYearMonthString`, 2 = `TemporalMonthDayString`), 2..5 unused,
    /// 6=calling standard builtin's realm environment (or zero). On success the
    /// result tuple carries the resolved calendar payload with a normal
    /// completion; on a parse failure — including a `[u-ca=...]` annotation
    /// naming a calendar this backend does not ship — it carries a `Throw`
    /// completion the caller is expected to inspect and discard.
    ///
    /// The rewrite runs *before* the parse on purpose: that is what keeps this
    /// to a single inlined copy of `emit_temporal_parse_iso_string` instead of
    /// three, which is the difference between compiling and tripping
    /// Cranelift's per-function code-size limit.
    pub(crate) fn compile_temporal_calendar_iso_date_probe_helper(
        &mut self,
        real: bool,
    ) -> Result<Function, EmitError> {
        if !real {
            return Ok(
                self.temporal_calendar_helper_stub(RuntimeHelperId::TemporalCalendarIsoDateProbe)
            );
        }
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalCalendarIsoDateProbe);
        self.push_scope();
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);

        let normalized_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(normalized_local));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_partial_date_rewrite_string(
            0,
            TemporalPartialDateRewrite::YearMonth,
            normalized_local,
            &mut function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `None`: this is a calendar *probe*, not `ToTemporalMonthDay`. It
        // reports the annotation a candidate string carries and its caller
        // discards a `Throw` completion, so it must not acquire the step (g)
        // and (k) RangeErrors that depend on `result.[[Year]] is empty`.
        self.emit_temporal_month_day_rewrite_string(0, normalized_local, None, &mut function)?;
        function.instruction(&Instruction::End);

        self.emit_temporal_parse_plain_date_string(
            normalized_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            calendar_tag_local,
            &mut function,
        )?;

        function.instruction(&Instruction::LocalGet(calendar_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(calendar_tag_local);
        self.release_temp_local(calendar_payload_local);
        self.release_temp_local(day_local);
        self.release_temp_local(month_local);
        self.release_temp_local(year_local);
        self.release_temp_local(normalized_local);
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared `ToTemporalCalendarIdentifier` string-resolution
    /// helper — `ParseTemporalCalendarString` followed by
    /// `CanonicalizeCalendar`.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=calendar string
    /// payload, 1..5 unused, 6=calling standard builtin's realm environment (or
    /// zero). A normal completion carries the canonical `iso8601` payload; a
    /// `Throw` completion carries the RangeError.
    ///
    /// Order is load-bearing in both directions and must not be "tidied":
    ///
    /// * The three ISO date goals run first because `"152330"` is both a legal
    ///   `AnnotationValue` and a legal `HHMMSS`, and the specification resolves
    ///   that tie in favor of the ISO parse (`withCalendar/calendar-time-string.js`).
    /// * The bare `AnnotationValue` compare is hoisted *above* the time attempt
    ///   so the time parser's own RangeError can fall out as the final answer
    ///   without a fourth helper to catch it. The hoist is sound exactly while
    ///   no [`TemporalCalendarId::spellings`] entry is also a legal ISO date or
    ///   time string; `"iso8601"`, `"gregory"` and `"gregorian"` all satisfy
    ///   that (they are not `HHMMSS`, not `YYYYMMDD`, and contain letters an
    ///   ISO date cannot). A calendar spelled out of digits would break it, and
    ///   would have to move the compare below the time attempt.
    ///
    /// Known deviation: `emit_temporal_parse_plain_time_string` has no calendar
    /// out-parameter and never reaches `emit_temporal_iso_calendar_annotation`,
    /// so the time arm hard-codes `iso8601` and
    /// `pd.withCalendar("T11:30[u-ca=notacal]")` wrongly succeeds instead of
    /// throwing. No `built-ins` test covers it; the only tests in that shape
    /// are `intl402` files that need a non-ISO calendar and are out of scope.
    pub(crate) fn compile_temporal_calendar_identifier_helper(
        &mut self,
        real: bool,
    ) -> Result<Function, EmitError> {
        if !real {
            return Ok(
                self.temporal_calendar_helper_stub(RuntimeHelperId::TemporalCalendarIdentifier)
            );
        }
        let probe_helper_index = self
            .temporal_calendar_iso_date_probe_helper_function_index()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: \
                     Temporal calendar date-probe helper without heap",
                )
            })?;
        let mut function = self.begin_helper_body(RuntimeHelperId::TemporalCalendarIdentifier);
        self.push_scope();
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);

        let expected_payload_local = self.reserve_temp_local();
        let resolved_local = self.reserve_temp_local();
        let probe_payload_local = self.reserve_temp_local();
        let probe_tag_local = self.reserve_temp_local();
        let probe_completion_local = self.reserve_temp_local();
        let probe_aux_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let second_local = self.reserve_temp_local();
        let nanosecond_local = self.reserve_temp_local();
        // The canonical identifier this helper answers with. It starts at the
        // default and is overwritten by whichever arm resolves, so a `gregory`
        // annotation and a bare `"gregory"` both come back as `gregory` rather
        // than as the default the arms used to hard-code.
        let result_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings
                .payload(TemporalCalendarId::DEFAULT.canonical()),
        ));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(resolved_local));

        for form in 0..3_i64 {
            function.instruction(&Instruction::LocalGet(resolved_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(0));
            function.instruction(&Instruction::I64Const(form));
            for _ in 0..4 {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::LocalGet(6));
            function.instruction(&Instruction::Call(probe_helper_index));
            // Deliberately NOT `store_call_results` — the probe's throw must not
            // become this helper's completion. This is the whole "try and
            // recover": the four results land in scratch locals and a `Throw`
            // simply leaves `resolved` at zero so the next form runs.
            self.store_call_results_to(
                probe_payload_local,
                probe_tag_local,
                probe_completion_local,
                probe_aux_local,
                &mut function,
            );
            function.instruction(&Instruction::LocalGet(probe_completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            // The probe already ran `CanonicalizeCalendar` on whatever
            // `[u-ca=...]` the string carried, so its answer is the answer.
            function.instruction(&Instruction::LocalGet(probe_payload_local));
            function.instruction(&Instruction::LocalSet(result_payload_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(resolved_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        // `CanonicalizeCalendar` on a bare `AnnotationValue`, over the same
        // spelling table the constructors use.
        function.instruction(&Instruction::LocalGet(resolved_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for calendar in TemporalCalendarId::ALL {
            let canonical_payload = self.strings.payload(calendar.canonical());
            for &spelling in calendar.spellings() {
                function.instruction(&Instruction::I64Const(self.strings.payload(spelling)));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(case_fold_local));
                self.emit_string_payload_equality_i32_with_ascii_case_folding(
                    0,
                    expected_payload_local,
                    Some(case_fold_local),
                    &mut function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(canonical_payload));
                function.instruction(&Instruction::LocalSet(result_payload_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(resolved_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::End);

        // Last attempt: a bare `TemporalTimeString`. Its RangeError is the
        // final answer, so nothing catches this one.
        function.instruction(&Instruction::LocalGet(resolved_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_parse_plain_time_string(
            0,
            hour_local,
            minute_local,
            second_local,
            nanosecond_local,
            &mut function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            result_payload_local,
            nanosecond_local,
            second_local,
            minute_local,
            hour_local,
            case_fold_local,
            probe_aux_local,
            probe_completion_local,
            probe_tag_local,
            probe_payload_local,
            resolved_local,
            expected_payload_local,
        ] {
            self.release_temp_local(local);
        }
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
