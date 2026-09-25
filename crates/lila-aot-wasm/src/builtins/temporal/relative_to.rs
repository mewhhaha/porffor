//! `GetTemporalRelativeToOption` (Temporal proposal 13.19).
//!
//! The option names either a PlainDate or a ZonedDateTime, and the
//! `Temporal.Duration` operations that read it need only the record fields,
//! never a heap object: a PlainDate is its ISO date and calendar, and a
//! ZonedDateTime is its exact time, its time-zone identifier and its calendar.
//! So the option is produced straight into locals, and nothing is allocated
//! for a property bag or a string.

use super::*;

/// Which record [`TemporalRelativeTo`] holds. A runtime value: whether the
/// option was absent, a plain date or a zoned date-time is only known once the
/// user's `relativeTo` has been read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::builtins) enum TemporalRelativeToKind {
    Undefined,
    Plain,
    Zoned,
}

impl TemporalRelativeToKind {
    pub(in crate::builtins) const fn code(self) -> i64 {
        match self {
            Self::Undefined => 0,
            Self::Plain => 1,
            Self::Zoned => 2,
        }
    }
}

/// `GetTemporalRelativeToOption`'s record, as locals.
///
/// `kind_local` holds a [`TemporalRelativeToKind`] code and says which fields
/// are meaningful:
///
/// - `Plain`: `date_locals` (ISO year, month, day) and the calendar.
/// - `Zoned`: the exact time as whole seconds plus a subsecond part in
///   `[0, 10^9)`, the time-zone identifier, and the calendar.
///
/// The locals are reserved together by
/// [`FunctionBuilder::reserve_temporal_relative_to`] and must be released with
/// [`FunctionBuilder::release_temporal_relative_to`].
pub(in crate::builtins) struct TemporalRelativeTo {
    pub(in crate::builtins) kind_local: u32,
    pub(in crate::builtins) date_locals: [u32; 3],
    pub(in crate::builtins) calendar_payload_local: u32,
    pub(in crate::builtins) epoch_seconds_local: u32,
    pub(in crate::builtins) epoch_subsecond_local: u32,
    pub(in crate::builtins) time_zone_payload_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(in crate::builtins) fn reserve_temporal_relative_to(&mut self) -> TemporalRelativeTo {
        TemporalRelativeTo {
            kind_local: self.reserve_temp_local(),
            date_locals: [
                self.reserve_temp_local(),
                self.reserve_temp_local(),
                self.reserve_temp_local(),
            ],
            calendar_payload_local: self.reserve_temp_local(),
            epoch_seconds_local: self.reserve_temp_local(),
            epoch_subsecond_local: self.reserve_temp_local(),
            time_zone_payload_local: self.reserve_temp_local(),
        }
    }

    pub(in crate::builtins) fn release_temporal_relative_to(
        &mut self,
        relative: TemporalRelativeTo,
    ) {
        for local in [
            relative.time_zone_payload_local,
            relative.epoch_subsecond_local,
            relative.epoch_seconds_local,
            relative.calendar_payload_local,
            relative.date_locals[2],
            relative.date_locals[1],
            relative.date_locals[0],
            relative.kind_local,
        ] {
            self.release_temp_local(local);
        }
    }

    /// `GetTemporalRelativeToOption(options)`: reads `options.relativeTo` and
    /// converts it completely — property reads, string parsing and time-zone
    /// resolution included — before the caller reads its next option, which
    /// is the order `order-of-operations.js` observes.
    pub(in crate::builtins) fn emit_temporal_relative_to_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        relative: &TemporalRelativeTo,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            TemporalRelativeToKind::Undefined.code(),
        ));
        function.instruction(&Instruction::LocalSet(relative.kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        self.emit_temporal_duration_option_get(
            options_payload_local,
            options_tag_local,
            "relativeTo",
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        // Step 5: an Object. The three Temporal records are read through their
        // internal slots; anything else is a property bag.
        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        // [[InitializedTemporalZonedDateTime]]: the object itself.
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
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
                HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                relative.calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::LocalSet(relative.kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        // [[InitializedTemporalDate]]: the object itself.
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (offset, local) in [
            (
                HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET,
                relative.date_locals[0],
            ),
            (
                HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET,
                relative.date_locals[1],
            ),
            (
                HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET,
                relative.date_locals[2],
            ),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                relative.calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Plain.code()));
        function.instruction(&Instruction::LocalSet(relative.kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        // [[InitializedTemporalDateTime]]: its ISO date and calendar.
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (offset, local) in [
            (
                HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET,
                relative.date_locals[0],
            ),
            (
                HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET,
                relative.date_locals[1],
            ),
            (
                HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET,
                relative.date_locals[2],
            ),
            (
                HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
                relative.calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Plain.code()));
        function.instruction(&Instruction::LocalSet(relative.kind_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Steps 5.d-5.k: a property bag. `timeZone` decides the record.
        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let time_zone_present_local = self.reserve_temp_local();
        let offset_option_local = self.reserve_temp_local();
        let overflow_option_local = self.reserve_temp_local();
        let disambiguation_option_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        self.emit_temporal_zoned_date_time_from_property_bag(
            value_payload_local,
            value_tag_local,
            ZonedPropertyBagConsumer::RelativeTo {
                time_zone_present_local,
                date_destination_locals: relative.date_locals,
            },
            ZonedDateTimeOptionLocals {
                disambiguation: disambiguation_option_local,
                offset: offset_option_local,
                overflow: overflow_option_local,
            },
            epoch_payload_local,
            epoch_tag_local,
            time_zone_payload_local,
            time_zone_tag_local,
            relative.calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(time_zone_present_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::LocalSet(relative.kind_local));
        function.instruction(&Instruction::Else);
        // `CreateTemporalDate(isoDate, calendar)`.
        self.emit_temporal_iso_date_within_limits(
            relative.date_locals[0],
            relative.date_locals[1],
            relative.date_locals[2],
            days_local,
            "Temporal.PlainDate is outside the supported date range",
            function,
        )?;
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Plain.code()));
        function.instruction(&Instruction::LocalSet(relative.kind_local));
        function.instruction(&Instruction::End);
        for local in [
            days_local,
            disambiguation_option_local,
            overflow_option_local,
            offset_option_local,
            time_zone_present_local,
        ] {
            self.release_temp_local(local);
        }
        function.instruction(&Instruction::End);

        // Step 6: a String, parsed as a zoned date-time string exactly when it
        // carries a time-zone annotation.
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.Duration relativeTo must be a Temporal object, a property bag or a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        let annotated_local = self.reserve_temp_local();
        self.emit_temporal_string_has_time_zone_annotation(
            value_payload_local,
            annotated_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(annotated_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        // `InterpretISODateTimeOffset` with `compatible`, `reject` and
        // `match-minutes`: the same interpretation `ToTemporalZonedDateTime`
        // gives a string under default options.
        let offset_option_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(OffsetOption::Reject.code()));
        function.instruction(&Instruction::LocalSet(offset_option_local));
        self.emit_temporal_parse_iso_string(
            value_payload_local,
            epoch_payload_local,
            epoch_tag_local,
            TemporalIsoParseGoal::ZonedDateTime {
                offset_option_local,
                disambiguation: TemporalDisambiguationSource::Compatible,
                time_zone_payload_local,
                time_zone_tag_local,
                calendar_payload_local: relative.calendar_payload_local,
                calendar_tag_local,
            },
            function,
        )?;
        self.release_temp_local(offset_option_local);
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::LocalSet(relative.kind_local));
        function.instruction(&Instruction::Else);
        // `TemporalDateTimeString[~Zoned]`, then `CreateTemporalDate`.
        self.emit_temporal_parse_plain_date_string(
            value_payload_local,
            relative.date_locals[0],
            relative.date_locals[1],
            relative.date_locals[2],
            relative.calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        self.emit_temporal_reject_iso_date(
            relative.date_locals[0],
            relative.date_locals[1],
            relative.date_locals[2],
            function,
        )?;
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Plain.code()));
        function.instruction(&Instruction::LocalSet(relative.kind_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(annotated_local);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // A zoned record keeps its exact time as a normalized seconds pair and
        // its time-zone identifier.
        function.instruction(&Instruction::LocalGet(relative.kind_local));
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_epoch_nanoseconds_value_pair(
            epoch_payload_local,
            epoch_tag_local,
            relative.epoch_seconds_local,
            relative.epoch_subsecond_local,
            function,
        );
        self.emit_temporal_normalize_seconds_and_subseconds(
            relative.epoch_seconds_local,
            relative.epoch_subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(time_zone_payload_local));
        function.instruction(&Instruction::LocalSet(relative.time_zone_payload_local));
        function.instruction(&Instruction::End);

        for local in [
            calendar_tag_local,
            time_zone_tag_local,
            time_zone_payload_local,
            epoch_tag_local,
            epoch_payload_local,
            handled_local,
            record_local,
            brand_local,
            value_tag_local,
            value_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Whether an ISO string's first bracketed annotation is a time-zone
    /// annotation (`[UTC]`, `[!+01:00]`) rather than a `key=value` one. The
    /// grammar puts the time-zone annotation before every other annotation, so
    /// the first bracket decides; a malformed string is rejected by whichever
    /// parser the answer selects.
    fn emit_temporal_string_has_time_zone_annotation(
        &mut self,
        string_payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        self.emit_unpack_string_payload(string_payload_local, offset_local, length_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(cursor_local));
        // Find the first `[`.
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::BrIf(0));
        // Inside it: a time-zone annotation unless `=` precedes the `]`.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, cursor_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'=' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [byte_local, cursor_local, length_local, offset_local] {
            self.release_temp_local(local);
        }
    }
}
