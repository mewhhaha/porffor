//! The property-bag relativeTo path retains whole roots and publishes kind last.
//! Calendar acquisition and the alphabetical field sweep precede regulation.
use super::*;
use crate::builtins::temporal::TemporalTimeZoneStringGoal;

/// Only a successfully resolved timeZone arm writes these roots. Restoration
/// is emitted inside that same present arm, never from unchecked JS values.
struct RelativeBagZoneBacking {
    identifier: ValueLocals,
    primary: ValueLocals,
    kind: I64Local,
    fixed: I64Local,
}
impl RelativeBagZoneBacking {
    fn reserve(builder: &mut FunctionBuilder<'_>, f: &mut Function) -> Self {
        let schema = builder.runtime_schema();
        let result = Self {
            identifier: schema.reserve_value_local(f),
            primary: schema.reserve_value_local(f),
            kind: schema.reserve_i64_local(f),
            fixed: schema.reserve_i64_local(f),
        };
        result.identifier.set_undefined(f);
        result.primary.set_undefined(f);
        for local in [result.kind, result.fixed] {
            f.instruction(&Instruction::I64Const(0));
            local.store(f);
        }
        result
    }
    fn copy_from(
        &self,
        zone: &ResolvedTemporalZoneLocals,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        self.identifier.set_reference(zone.identifier(), schema, f);
        self.primary
            .set_reference(zone.primary_identifier(), schema, f);
        zone.kind_local().load(f);
        self.kind.store(f);
        zone.fixed_offset_seconds().load(f);
        self.fixed.store(f);
    }
    fn restore(
        &self,
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
    ) -> ResolvedTemporalZoneLocals {
        let schema = builder.runtime_schema();
        let kind = schema.reserve_i64_local(f);
        let fixed_seconds = schema.reserve_i64_local(f);
        self.kind.load(f);
        kind.store(f);
        self.fixed.load(f);
        fixed_seconds.store(f);
        ResolvedTemporalZoneLocals {
            identifier: schema
                .reserve_gc_local(f)
                .initialize(self.identifier.cast_reference::<StringValue>(schema, f), f),
            primary: schema
                .reserve_gc_local(f)
                .initialize(self.primary.cast_reference::<StringValue>(schema, f), f),
            kind,
            fixed_seconds,
        }
    }
    fn clear(self, builder: &mut FunctionBuilder<'_>, f: &mut Function) {
        let schema = builder.runtime_schema();
        schema.release_i64_local(self.fixed, f);
        schema.release_i64_local(self.kind, f);
        self.primary.clear(f);
        self.identifier.clear(f);
    }
}
#[derive(Clone, Copy)]
enum RelativeBagInteger {
    Day,
    Hour,
    Microsecond,
    Millisecond,
    Minute,
    Month,
    Nanosecond,
    Second,
    Year,
}
impl RelativeBagInteger {
    fn field(self) -> (&'static str, usize, bool) {
        match self {
            Self::Day => ("day", 2, true),
            Self::Hour => ("hour", 3, false),
            Self::Microsecond => ("microsecond", 7, false),
            Self::Millisecond => ("millisecond", 6, false),
            Self::Minute => ("minute", 4, false),
            Self::Month => ("month", 1, true),
            Self::Nanosecond => ("nanosecond", 8, false),
            Self::Second => ("second", 5, false),
            Self::Year => ("year", 0, false),
        }
    }
}
impl FunctionBuilder<'_> {
    fn emit_temporal_relative_bag_integer(
        &mut self,
        field: RelativeBagInteger,
        bag: &ValueLocals,
        present: I64Local,
        fields: &[I64Local; 9],
        retain: Option<I64Local>,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let (name, index, positive) = field.field();
        if positive {
            self.emit_temporal_property_bag_positive_integer(
                bag,
                name,
                present,
                fields[index],
                0,
                RuntimeErrorMessage::TEMPORAL_DURATION_RELATIVETO_FIELD_MUST_BE_FINITE,
                RuntimeErrorMessage::TEMPORAL_DURATION_RELATIVETO_MONTH_AND_DAY_MUST_BE_POSITIVE,
                f,
            )?;
        } else {
            self.emit_temporal_property_bag_integer(
                bag,
                name,
                present,
                fields[index],
                0,
                RuntimeErrorMessage::TEMPORAL_DURATION_RELATIVETO_FIELD_MUST_BE_FINITE,
                f,
            )?;
        }
        if let Some(output) = retain {
            present.load(f);
            output.store(f);
        }
        Ok(())
    }
    pub(super) fn emit_temporal_duration_relative_bag_into(
        &mut self,
        relative: &mut TemporalRelativeToLocals,
        bag: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        let code = schema.reserve_value_local(f);
        let present = schema.reserve_i64_local(f);
        self.emit_temporal_duration_option_get(bag, "calendar", &value, f)?;
        let calendar = self.emit_temporal_calendar_slot_from_value(&value, f)?;
        let zone_present = schema.reserve_i64_local(f);
        let offset_present = schema.reserve_i64_local(f);
        let offset_nanos = schema.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        offset_nanos.store(f);
        let prepared_iso = self.reserve_temporal_iso_record_result(f);
        let zone_backing = RelativeBagZoneBacking::reserve(self, f);
        let fields = self.reserve_temporal_plain_date_time_field_locals(f);
        let day_present = schema.reserve_i64_local(f);
        let month_present = schema.reserve_i64_local(f);
        let year_present = schema.reserve_i64_local(f);
        let code_present = schema.reserve_i64_local(f);
        let encoded = schema.reserve_i64_local(f);
        let overflow = self.emit_temporal_constant_overflow(TemporalOverflow::Constrain, f);
        let era_slots = self.reserve_temporal_era_slots(f);
        self.emit_temporal_relative_bag_integer(
            RelativeBagInteger::Day,
            bag,
            present,
            &fields,
            Some(day_present),
            f,
        )?;
        let era = self.emit_temporal_read_era_fields(era_slots, bag, calendar.calendar_id(), f)?;
        for field in [
            RelativeBagInteger::Hour,
            RelativeBagInteger::Microsecond,
            RelativeBagInteger::Millisecond,
            RelativeBagInteger::Minute,
            RelativeBagInteger::Month,
        ] {
            let retain = if matches!(field, RelativeBagInteger::Month) {
                Some(month_present)
            } else {
                None
            };
            self.emit_temporal_relative_bag_integer(field, bag, present, &fields, retain, f)?;
        }
        self.emit_temporal_duration_option_get(bag, "monthCode", &code, f)?;
        code.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::I64ExtendI32U);
        code_present.store(f);
        self.emit_temporal_month_code_string(
            &code,
            RuntimeErrorMessage::TEMPORAL_DURATION_RELATIVETO_MONTHCODE_MUST_BE_A_STRING,
            RuntimeErrorMessage::INVALID_TEMPORAL_DURATION_RELATIVETO_MONTHCODE,
            f,
        )?;
        self.emit_temporal_relative_bag_integer(
            RelativeBagInteger::Nanosecond,
            bag,
            present,
            &fields,
            None,
            f,
        )?;
        self.emit_temporal_duration_option_get(bag, "offset", &value, f)?;
        value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::I64ExtendI32U);
        offset_present.store(f);
        self.emit_temporal_offset_string(
            &value,
            RuntimeErrorMessage::TEMPORAL_DURATION_RELATIVETO_OFFSET_MUST_BE_A_STRING,
            f,
        )?;
        offset_present.load(f);
        f.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, f);
        let text = schema
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<StringValue>(schema, f), f);
        self.emit_temporal_utc_offset_nanoseconds(&text, offset_nanos, f)?;
        text.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_temporal_relative_bag_integer(
            RelativeBagInteger::Second,
            bag,
            present,
            &fields,
            None,
            f,
        )?;
        self.emit_temporal_duration_option_get(bag, "timeZone", &value, f)?;
        value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::I64ExtendI32U);
        zone_present.store(f);
        zone_present.load(f);
        f.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, f);
        let zone = self.emit_temporal_zoned_date_time_time_zone(
            &value,
            TemporalTimeZoneStringGoal::Object,
            f,
        )?;
        zone_backing.copy_from(&zone, schema, f);
        zone.release(self, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_temporal_relative_bag_integer(
            RelativeBagInteger::Year,
            bag,
            present,
            &fields,
            Some(year_present),
            f,
        )?;
        let year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            fields[0],
            year_present,
            f,
        )?;
        self.emit_temporal_plain_date_resolve_fields(
            year,
            fields[1],
            month_present,
            &code,
            encoded,
            code_present,
            fields[2],
            day_present,
            overflow.local(),
            f,
        )?;
        self.emit_temporal_regulate_time(
            &Self::temporal_plain_date_time_time_locals(&fields),
            overflow.local(),
            f,
        )?;
        let iso = self.emit_temporal_regulated_iso_record_into(prepared_iso, &fields, f)?;
        overflow.release(self, f);
        for local in [
            encoded,
            code_present,
            year_present,
            month_present,
            day_present,
        ] {
            schema.release_i64_local(local, f);
        }
        self.release_temporal_plain_date_time_field_locals(fields, f);
        zone_present.load(f);
        f.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, f);
        let zone = zone_backing.restore(self, f);
        let compatible = self.emit_temporal_constant_disambiguation(Disambiguation::Compatible, f);
        let reject = self.emit_temporal_constant_offset_option(OffsetOption::Reject, f);
        let offset = self.emit_temporal_validated_offset_nanoseconds(offset_nanos, f)?;
        offset_present.load(f);
        f.instruction(&Instruction::I32WrapI64);
        self.open_frame(ControlFrameKind::If, f);
        let instant = self.emit_temporal_interpret_iso_date_time_offset(
            &iso,
            &zone,
            TemporalOffsetBehavior::Option,
            &offset,
            &compatible,
            &reject,
            TemporalOffsetMatchBehavior::MatchExactly,
            f,
        )?;
        self.emit_temporal_relative_store_zoned(relative, &instant, &zone, &calendar, f);
        instant.release(self, f);
        f.instruction(&Instruction::Else);
        let instant = self.emit_temporal_interpret_iso_date_time_offset(
            &iso,
            &zone,
            TemporalOffsetBehavior::Wall,
            &offset,
            &compatible,
            &reject,
            TemporalOffsetMatchBehavior::MatchExactly,
            f,
        )?;
        self.emit_temporal_relative_store_zoned(relative, &instant, &zone, &calendar, f);
        instant.release(self, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        offset.release(self, f);
        reject.release(self, f);
        compatible.release(self, f);
        zone.release(self, f);
        f.instruction(&Instruction::Else);
        let date = self.emit_temporal_iso_date_from_record(&iso, f);
        self.emit_temporal_relative_store_plain(relative, &date, &calendar, f);
        date.release(self, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        zone_backing.clear(self, f);
        iso.release(self, f);
        for local in [offset_nanos, offset_present, zone_present] {
            schema.release_i64_local(local, f);
        }
        calendar.release(self, f);
        schema.release_i64_local(present, f);
        code.clear(f);
        value.clear(f);
        Ok(())
    }
}
