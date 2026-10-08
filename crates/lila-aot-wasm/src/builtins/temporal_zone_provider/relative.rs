//! RelativeTo stores whole rooted values privately and publishes its kind last.
//! A completed proof can be restored only inside the matching emitted branch.
use super::*;
use crate::builtins::temporal_options::TemporalConversionOverflowOptions;
mod bag;

pub(in crate::builtins) enum TemporalRelativeToKind {
    Absent,
    Zoned,
    PlainDate,
}
impl TemporalRelativeToKind {
    pub(in crate::builtins) const fn code(self) -> i64 {
        match self {
            Self::Absent => 0,
            Self::Zoned => 1,
            Self::PlainDate => 2,
        }
    }
}

pub(in crate::builtins) struct TemporalRelativeToLocals {
    kind: I64Local,
    epoch: ValueLocals,
    seconds: I64Local,
    nanosecond: I64Local,
    zone_identifier: ValueLocals,
    zone_primary: ValueLocals,
    zone_kind: I64Local,
    fixed_seconds: I64Local,
    calendar_identifier: ValueLocals,
    calendar_id: I64Local,
    date: [I64Local; 3],
}

/// The constructor is private to the checked Zoned branch below. Every value
/// and coordinate was copied from a completed epoch before kind publication.
pub(in crate::builtins) struct RelativeEpochView<'a> {
    value: &'a ValueLocals,
    seconds: I64Local,
    nanosecond: I64Local,
}
impl RelativeEpochView<'_> {
    pub(in crate::builtins) fn value(&self) -> &ValueLocals {
        self.value
    }
    pub(in crate::builtins) fn floor_seconds(&self) -> I64Local {
        self.seconds
    }
    pub(in crate::builtins) fn nanosecond(&self) -> I64Local {
        self.nanosecond
    }
}
/// This view cannot be constructed from arbitrary String/id operands.
pub(in crate::builtins) struct RelativeCalendarView<'a> {
    value: &'a ValueLocals,
    id: I64Local,
}
impl RelativeCalendarView<'_> {
    pub(in crate::builtins) fn value(&self) -> &ValueLocals {
        self.value
    }
    pub(in crate::builtins) fn calendar_id(&self) -> I64Local {
        self.id
    }
}

pub(in crate::builtins) struct TemporalZonedRelativeContextLocals {
    instant: NormalizedTemporalInstantLocals,
    zone: ResolvedTemporalZoneLocals,
    calendar: TemporalCalendarSlotLocals,
}
impl TemporalZonedRelativeContextLocals {
    pub(in crate::builtins) fn instant(&self) -> &NormalizedTemporalInstantLocals {
        &self.instant
    }
    pub(in crate::builtins) fn zone(&self) -> &ResolvedTemporalZoneLocals {
        &self.zone
    }
    pub(in crate::builtins) fn calendar(&self) -> &TemporalCalendarSlotLocals {
        &self.calendar
    }
}
pub(in crate::builtins) struct TemporalPlainRelativeContextLocals {
    date: TemporalIsoDateLocals,
    calendar: TemporalCalendarSlotLocals,
}
impl TemporalPlainRelativeContextLocals {
    pub(in crate::builtins) fn calendar(&self) -> &TemporalCalendarSlotLocals {
        &self.calendar
    }
    pub(in crate::builtins) fn date(&self) -> &TemporalIsoDateLocals {
        &self.date
    }
}

impl TemporalRelativeToLocals {
    pub(in crate::builtins) fn kind_local(&self) -> I64Local {
        self.kind
    }
    pub(in crate::builtins) fn emit_zoned_branch<'a>(
        &self,
        builder: &mut FunctionBuilder<'a>,
        function: &mut Function,
        body: impl FnOnce(
            &mut FunctionBuilder<'a>,
            &mut Function,
            &TemporalZonedRelativeContextLocals,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let schema = builder.runtime_schema();
        self.kind.load(function);
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        function.instruction(&Instruction::I64Eq);
        builder.open_frame(ControlFrameKind::If, function);
        let epoch = RelativeEpochView {
            value: &self.epoch,
            seconds: self.seconds,
            nanosecond: self.nanosecond,
        };
        let calendar = RelativeCalendarView {
            value: &self.calendar_identifier,
            id: self.calendar_id,
        };
        let instant = builder.emit_temporal_normalized_instant_from_relative_view(&epoch, function);
        let calendar = builder.emit_temporal_calendar_from_relative_view(&calendar, function);
        let zone = ResolvedTemporalZoneLocals {
            identifier: schema.reserve_gc_local(function).initialize(
                self.zone_identifier
                    .cast_reference::<StringValue>(schema, function),
                function,
            ),
            primary: schema.reserve_gc_local(function).initialize(
                self.zone_primary
                    .cast_reference::<StringValue>(schema, function),
                function,
            ),
            kind: schema.reserve_i64_local(function),
            fixed_seconds: schema.reserve_i64_local(function),
        };
        self.zone_kind.load(function);
        zone.kind.store(function);
        self.fixed_seconds.load(function);
        zone.fixed_seconds.store(function);
        let context = TemporalZonedRelativeContextLocals {
            instant,
            zone,
            calendar,
        };
        body(builder, function, &context)?;
        context.calendar.release(builder, function);
        context.zone.release(builder, function);
        context.instant.release(builder, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    pub(in crate::builtins) fn emit_plain_branch<'a>(
        &self,
        builder: &mut FunctionBuilder<'a>,
        function: &mut Function,
        body: impl FnOnce(
            &mut FunctionBuilder<'a>,
            &mut Function,
            &TemporalPlainRelativeContextLocals,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let schema = builder.runtime_schema();
        self.kind.load(function);
        function.instruction(&Instruction::I64Const(
            TemporalRelativeToKind::PlainDate.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        builder.open_frame(ControlFrameKind::If, function);
        let view = RelativeCalendarView {
            value: &self.calendar_identifier,
            id: self.calendar_id,
        };
        let calendar = builder.emit_temporal_calendar_from_relative_view(&view, function);
        let fields = std::array::from_fn(|_| schema.reserve_i64_local(function));
        for (source, destination) in self.date.into_iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        let context = TemporalPlainRelativeContextLocals {
            date: TemporalIsoDateLocals { fields },
            calendar,
        };
        body(builder, function, &context)?;
        context.date.release(builder, function);
        context.calendar.release(builder, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    pub(in crate::builtins) fn release(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        for local in self.date.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        schema.release_i64_local(self.calendar_id, function);
        self.calendar_identifier.clear(function);
        schema.release_i64_local(self.fixed_seconds, function);
        schema.release_i64_local(self.zone_kind, function);
        self.zone_primary.clear(function);
        self.zone_identifier.clear(function);
        schema.release_i64_local(self.nanosecond, function);
        schema.release_i64_local(self.seconds, function);
        self.epoch.clear(function);
        schema.release_i64_local(self.kind, function);
    }
}

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_temporal_absent_relative_to(
        &mut self,
        function: &mut Function,
    ) -> TemporalRelativeToLocals {
        let schema = self.runtime_schema();
        let result = TemporalRelativeToLocals {
            kind: schema.reserve_i64_local(function),
            epoch: schema.reserve_value_local(function),
            seconds: schema.reserve_i64_local(function),
            nanosecond: schema.reserve_i64_local(function),
            zone_identifier: schema.reserve_value_local(function),
            zone_primary: schema.reserve_value_local(function),
            zone_kind: schema.reserve_i64_local(function),
            fixed_seconds: schema.reserve_i64_local(function),
            calendar_identifier: schema.reserve_value_local(function),
            calendar_id: schema.reserve_i64_local(function),
            date: std::array::from_fn(|_| schema.reserve_i64_local(function)),
        };
        for value in [
            &result.epoch,
            &result.zone_identifier,
            &result.zone_primary,
            &result.calendar_identifier,
        ] {
            value.set_undefined(function);
        }
        for local in [
            result.kind,
            result.seconds,
            result.nanosecond,
            result.zone_kind,
            result.fixed_seconds,
            result.calendar_id,
        ]
        .into_iter()
        .chain(result.date)
        {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        result
    }
    fn emit_temporal_relative_store_zoned(
        &self,
        result: &mut TemporalRelativeToLocals,
        instant: &NormalizedTemporalInstantLocals,
        zone: &ResolvedTemporalZoneLocals,
        calendar: &TemporalCalendarSlotLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        result
            .epoch
            .set_reference(instant.epoch_nanoseconds(), schema, function);
        result
            .zone_identifier
            .set_reference(zone.identifier(), schema, function);
        result
            .zone_primary
            .set_reference(zone.primary_identifier(), schema, function);
        result
            .calendar_identifier
            .set_reference(calendar.identifier(), schema, function);
        for (source, destination) in [
            (instant.floor_seconds(), result.seconds),
            (instant.nanosecond(), result.nanosecond),
            (zone.kind_local(), result.zone_kind),
            (zone.fixed_offset_seconds(), result.fixed_seconds),
            (calendar.calendar_id(), result.calendar_id),
        ] {
            source.load(function);
            destination.store(function);
        }
        function.instruction(&Instruction::I64Const(TemporalRelativeToKind::Zoned.code()));
        result.kind.store(function);
    }
    fn emit_temporal_relative_store_plain(
        &self,
        result: &mut TemporalRelativeToLocals,
        date: &TemporalIsoDateLocals,
        calendar: &TemporalCalendarSlotLocals,
        function: &mut Function,
    ) {
        for (source, destination) in date.fields().iter().copied().zip(result.date) {
            source.load(function);
            destination.store(function);
        }
        result.calendar_identifier.set_reference(
            calendar.identifier(),
            self.runtime_schema(),
            function,
        );
        calendar.calendar_id().load(function);
        result.calendar_id.store(function);
        function.instruction(&Instruction::I64Const(
            TemporalRelativeToKind::PlainDate.code(),
        ));
        result.kind.store(function);
    }
    fn emit_temporal_relative_zoned_value_into(
        &mut self,
        result: &mut TemporalRelativeToLocals,
        input: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let branded = self.emit_temporal_to_zoned_record(input, function)?;
        let instant =
            self.emit_temporal_normalized_instant_from_zoned_record(&branded, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&branded, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(&branded, function)?;
        self.emit_temporal_relative_store_zoned(result, &instant, &zone, &calendar, function);
        calendar.release(self, function);
        zone.release(self, function);
        instant.release(self, function);
        branded.release(function);
        Ok(())
    }
    fn emit_temporal_relative_plain_value_into(
        &mut self,
        result: &mut TemporalRelativeToLocals,
        input: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let calendar_value = schema.reserve_value_local(function);
        self.emit_temporal_to_temporal_date(
            input,
            TemporalConversionOverflowOptions::Omit,
            &result.date,
            &calendar_value,
            function,
        )?;
        let identifier = schema.reserve_gc_local(function).initialize(
            calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let calendar = self.emit_temporal_calendar_slot_from_identifier(&identifier, function)?;
        let date = TemporalIsoDateLocals {
            fields: std::array::from_fn(|_| schema.reserve_i64_local(function)),
        };
        for (source, destination) in result.date.into_iter().zip(date.fields) {
            source.load(function);
            destination.store(function);
        }
        self.emit_temporal_relative_store_plain(result, &date, &calendar, function);
        date.release(self, function);
        calendar.release(self, function);
        identifier.clear(function);
        calendar_value.clear(function);
        Ok(())
    }
    pub(in crate::builtins) fn emit_temporal_duration_relative_to_option_into(
        &mut self,
        result: &mut TemporalRelativeToLocals,
        input: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let zoned_annotation = schema.reserve_i64_local(function);
        input.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        input.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let string = schema.reserve_gc_local(function).initialize(
            input.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_relative_has_zone_annotation(&string, zoned_annotation, function);
        string.clear(function);
        zoned_annotation.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_plain_value_into(result, input, function)?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_zoned_value_into(result, input, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(input.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        input.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalZonedDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_zoned_value_into(result, input, function)?;
        function.instruction(&Instruction::Else);
        input.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        input.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_relative_plain_value_into(result, input, function)?;
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_relative_bag_into(result, input, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_temporal_relative_plain_value_into(result, input, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i64_local(zoned_annotation, function);
        Ok(())
    }
    fn emit_temporal_relative_has_zone_annotation(
        &mut self,
        input: &GcLocal<StringValue>,
        destination: I64Local,
        function: &mut Function,
    ) {
        let length = self.runtime_schema().reserve_i64_local(function);
        let cursor = self.runtime_schema().reserve_i64_local(function);
        let byte = self.runtime_schema().reserve_i64_local(function);
        let has_value = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_string_length(input, length, function);
        for local in [destination, cursor] {
            function.instruction(&Instruction::I64Const(0));
            (local).store(function);
        }
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (cursor).load(function);
        (length).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_load_code_unit(input, cursor, byte, function);
        (byte).load(function);
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_advance_cursor(cursor, function);
        function.instruction(&Instruction::I64Const(0));
        (has_value).store(function);
        self.open_frame(ControlFrameKind::Block, function);
        self.open_frame(ControlFrameKind::Loop, function);
        (cursor).load(function);
        (length).load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_temporal_load_code_unit(input, cursor, byte, function);
        (byte).load(function);
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        (byte).load(function);
        function.instruction(&Instruction::I64Const(b'=' as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (has_value).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor, function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        (has_value).load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1));
        (destination).store(function);
        function.instruction(&Instruction::Br(3));
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_advance_cursor(cursor, function);
        function.instruction(&Instruction::Br(0));
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        for local in [has_value, byte, cursor, length] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
