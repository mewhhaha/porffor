//! ToDateTimeFormattable completes both range observations before either HandleDateTimeValue.
use super::*;
pub(super) struct ObservedDtfValue {
    value: ValueLocals,
    kind: GcI32DomainLocal<DateTimeValueKind>,
    zoned: I32Local,
}
impl ObservedDtfValue {
    fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        schema.release_i32_local(self.zoned, f);
        self.kind.clear(schema, f);
        self.value.clear(f);
    }
}
/// Only completed TimeClip, a validated Temporal brand, or the Zoned locale
/// owner can produce this private ten-word native input.
pub(super) struct CompletedDtfInput {
    pub(super) words: [I64Local; 10],
}
impl CompletedDtfInput {
    pub(super) fn new(schema: &RuntimeSchema, f: &mut Function) -> Self {
        let words = core::array::from_fn(|_| schema.reserve_i64_local(f));
        for word in words {
            f.instruction(&Instruction::I64Const(0));
            word.store(f);
        }
        Self { words }
    }
    pub(super) fn append(
        &self,
        message: &IntlByteArrayBuilder,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        for word in self.words {
            message.append_u64(word, schema, f);
        }
    }
    pub(super) fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        for word in self.words.into_iter().rev() {
            schema.release_i64_local(word, f);
        }
    }
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_dtf_require_temporal_receiver(
        &mut self,
        kind: DtfTemporalKind,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let (heap,message)=match kind{
DtfTemporalKind::PlainDate=>(schema.reference_type::<TemporalPlainDateObject>(GcNullability::NonNullable).heap_type,RuntimeErrorMessage::TEMPORAL_PLAINDATE_RECEIVER_DOES_NOT_HAVE_INITIALIZEDTEMPORALDATE),
DtfTemporalKind::PlainYearMonth=>(schema.reference_type::<TemporalPlainYearMonthObject>(GcNullability::NonNullable).heap_type,RuntimeErrorMessage::TEMPORAL_PLAINYEARMONTH_RECEIVER_DOES_NOT_HAVE_INITIALIZEDTEMPORALYEARMONTH),
DtfTemporalKind::PlainMonthDay=>(schema.reference_type::<TemporalPlainMonthDayObject>(GcNullability::NonNullable).heap_type,RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_RECEIVER_DOES_NOT_HAVE_INITIALIZEDTEMPORALMONTHDAY),
DtfTemporalKind::PlainTime=>(schema.reference_type::<TemporalPlainTimeObject>(GcNullability::NonNullable).heap_type,RuntimeErrorMessage::TEMPORAL_PLAINTIME_RECEIVER_DOES_NOT_HAVE_INITIALIZEDTEMPORALTIME),
DtfTemporalKind::PlainDateTime=>(schema.reference_type::<TemporalPlainDateTimeObject>(GcNullability::NonNullable).heap_type,RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_RECEIVER_DOES_NOT_HAVE_INITIALIZEDTEMPORALDATETIME),
DtfTemporalKind::Instant=>(schema.reference_type::<TemporalInstantObject>(GcNullability::NonNullable).heap_type,RuntimeErrorMessage::TEMPORAL_INSTANT_RECEIVER_DOES_NOT_HAVE_INITIALIZEDTEMPORALINSTANT),
};
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(heap));
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(message, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    fn emit_dtf_observe_value(
        &mut self,
        input: &ValueLocals,
        f: &mut Function,
    ) -> Result<ObservedDtfValue, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        value.copy_from(input, f);
        let kind = GcI32DomainLocal::new(schema, DateTimeValueKind::Legacy, f);
        let zoned = schema.reserve_i32_local(f);
        set_i32(zoned, 0, f);
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        kind.set_constant(DateTimeValueKind::PlainDate, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainYearMonthObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        kind.set_constant(DateTimeValueKind::PlainYearMonth, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainMonthDayObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        kind.set_constant(DateTimeValueKind::PlainMonthDay, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        kind.set_constant(DateTimeValueKind::PlainTime, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        kind.set_constant(DateTimeValueKind::PlainDateTime, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalInstantObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        kind.set_constant(DateTimeValueKind::Instant, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalZonedDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, f);
        kind.set_constant(DateTimeValueKind::Instant, f);
        set_i32(zoned, 1, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&kind, DateTimeValueKind::Legacy, f);
        self.open_frame(ControlFrameKind::If, f);
        let pending = schema.reserve_completion(f);
        pending.initialize(f);
        self.emit_value_to_number_payload(&value, &pending, f)?;
        self.emit_intl_number_adopt_completion(&pending, f);
        value.copy_from(pending.value(), f);
        pending.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(ObservedDtfValue { value, kind, zoned })
    }
    pub(super) fn emit_dtf_single_input(
        &mut self,
        formatter: &GcLocal<IntlDateTimeFormatObject>,
        f: &mut Function,
    ) -> Result<CompletedDtfInput, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &value, f);
        emit_tag_is(&value, WasmRuntimeValueTag::Undefined, f);
        self.open_frame(ControlFrameKind::If, f);
        let clock = self
            .functions
            .wall_clock_millis_import_function_index()
            .ok_or_else(|| {
                EmitError::unsupported("DateTimeFormat requires wall_clock_millis import")
            })?;
        f.instruction(&Instruction::Call(clock));
        f.instruction(&Instruction::I64ReinterpretF64);
        let bits = schema.reserve_i64_local(f);
        bits.store(f);
        value.set_number(bits, f);
        schema.release_i64_local(bits, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let observed = self.emit_dtf_observe_value(&value, f)?;
        let input = self.emit_dtf_input_record(formatter, &observed, f)?;
        observed.clear(schema, f);
        value.clear(f);
        Ok(input)
    }
    pub(super) fn emit_dtf_range_inputs(
        &mut self,
        formatter: &GcLocal<IntlDateTimeFormatObject>,
        f: &mut Function,
    ) -> Result<(CompletedDtfInput, CompletedDtfInput), EmitError> {
        let schema = self.runtime_schema();
        let left = schema.reserve_value_local(f);
        let right = schema.reserve_value_local(f);
        self.emit_builtin_arg_to_value(0, &left, f);
        self.emit_builtin_arg_to_value(1, &right, f);
        emit_tag_is(&left, WasmRuntimeValueTag::Undefined, f);
        emit_tag_is(&right, WasmRuntimeValueTag::Undefined, f);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(RuntimeErrorMessage::INTL_DATETIMEFORMAT_PROTOTYPE_FORMATRANGE_STARTDATE_AND_ENDDATE_MUST_BE_DEFINED,f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let left_observed = self.emit_dtf_observe_value(&left, f)?;
        let right_observed = self.emit_dtf_observe_value(&right, f)?;
        left_observed.kind.load(f);
        right_observed.kind.load(f);
        f.instruction(&Instruction::I32Ne);
        left_observed.zoned.load(f);
        right_observed.zoned.load(f);
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(RuntimeErrorMessage::INTL_DATETIMEFORMAT_PROTOTYPE_FORMATRANGE_STARTDATE_AND_ENDDATE_MUST_BE_THE_SAME_TYPE,f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let start = self.emit_dtf_input_record(formatter, &left_observed, f)?;
        let end = self.emit_dtf_input_record(formatter, &right_observed, f)?;
        right_observed.clear(schema, f);
        left_observed.clear(schema, f);
        right.clear(f);
        left.clear(f);
        Ok((start, end))
    }
    fn emit_dtf_input_record(
        &mut self,
        formatter: &GcLocal<IntlDateTimeFormatObject>,
        observed: &ObservedDtfValue,
        f: &mut Function,
    ) -> Result<CompletedDtfInput, EmitError> {
        let schema = self.runtime_schema();
        observed.zoned.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(
            RuntimeErrorMessage::INTL_DATETIMEFORMAT_DOES_NOT_SUPPORT_TEMPORAL_ZONEDDATETIME,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let input = CompletedDtfInput::new(schema, f);
        observed.kind.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        input.words[0].store(f);
        emit_domain_is(&observed.kind, DateTimeValueKind::Legacy, f);
        self.open_frame(ControlFrameKind::If, f);
        let integer = schema.reserve_i64_local(f);
        self.emit_date_time_clip(observed.value.scalar(), integer, f);
        integer.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        integer.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::F64Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_range_error(RuntimeErrorMessage::DATE_VALUE_IS_NOT_FINITE, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        integer.load(f);
        f.instruction(&Instruction::F64ReinterpretI64);
        f.instruction(&Instruction::I64TruncF64S);
        integer.store(f);
        integer.load(f);
        f.instruction(&Instruction::I64Const(1000));
        f.instruction(&Instruction::I64DivS);
        input.words[1].store(f);
        integer.load(f);
        f.instruction(&Instruction::I64Const(1000));
        f.instruction(&Instruction::I64RemS);
        f.instruction(&Instruction::I64Const(1_000_000));
        f.instruction(&Instruction::I64Mul);
        input.words[2].store(f);
        input.words[2].load(f);
        f.instruction(&Instruction::I64Const(0));
        f.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, f);
        input.words[1].load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        input.words[1].store(f);
        input.words[2].load(f);
        f.instruction(&Instruction::I64Const(1_000_000_000));
        f.instruction(&Instruction::I64Add);
        input.words[2].store(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i64_local(integer, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&observed.kind, DateTimeValueKind::Instant, f);
        self.open_frame(ControlFrameKind::If, f);
        let instant = schema.reserve_gc_local(f).initialize(
            observed
                .value
                .cast_reference::<TemporalInstantObject>(schema, f),
            f,
        );
        let epoch = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<TemporalInstantObject>()
                .field(TemporalInstantObjectSchema::EPOCH_NANOSECONDS)
                .read(&instant, schema, f)
                .reference(),
            f,
        );
        self.emit_dtf_epoch_pair(&epoch, input.words[1], input.words[2], f);
        epoch.clear(f);
        instant.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&observed.kind, DateTimeValueKind::PlainDate, f);
        self.open_frame(ControlFrameKind::If, f);
        let source = schema.reserve_gc_local(f).initialize(
            observed
                .value
                .cast_reference::<TemporalPlainDateObject>(schema, f),
            f,
        );
        let record = schema.struct_type::<TemporalPlainDateObject>();
        let field = schema.reserve_i32_local(f);
        let calendar = schema.reserve_gc_local(f).initialize(
            record
                .field(TemporalPlainDateObjectSchema::CALENDAR)
                .read(&source, schema, f)
                .reference(),
            f,
        );
        self.emit_dtf_calendar_compatibility(formatter, &calendar, true, f)?;
        calendar.clear(f);
        record
            .field(TemporalPlainDateObjectSchema::ISO_YEAR)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[3].store(f);
        record
            .field(TemporalPlainDateObjectSchema::ISO_MONTH)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[4].store(f);
        record
            .field(TemporalPlainDateObjectSchema::ISO_DAY)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[5].store(f);
        self.emit_dtf_require_available(formatter, DateTimeValueKind::PlainDate, f)?;
        f.instruction(&Instruction::I64Const(12));
        input.words[6].store(f);
        schema.release_i32_local(field, f);
        source.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&observed.kind, DateTimeValueKind::PlainYearMonth, f);
        self.open_frame(ControlFrameKind::If, f);
        let source = schema.reserve_gc_local(f).initialize(
            observed
                .value
                .cast_reference::<TemporalPlainYearMonthObject>(schema, f),
            f,
        );
        let record = schema.struct_type::<TemporalPlainYearMonthObject>();
        let field = schema.reserve_i32_local(f);
        let calendar = schema.reserve_gc_local(f).initialize(
            record
                .field(TemporalPlainYearMonthObjectSchema::CALENDAR)
                .read(&source, schema, f)
                .reference(),
            f,
        );
        self.emit_dtf_calendar_compatibility(formatter, &calendar, false, f)?;
        calendar.clear(f);
        record
            .field(TemporalPlainYearMonthObjectSchema::ISO_YEAR)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[3].store(f);
        record
            .field(TemporalPlainYearMonthObjectSchema::ISO_MONTH)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[4].store(f);
        record
            .field(TemporalPlainYearMonthObjectSchema::REFERENCE_ISO_DAY)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[5].store(f);
        self.emit_dtf_require_available(formatter, DateTimeValueKind::PlainYearMonth, f)?;
        f.instruction(&Instruction::I64Const(12));
        input.words[6].store(f);
        schema.release_i32_local(field, f);
        source.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&observed.kind, DateTimeValueKind::PlainMonthDay, f);
        self.open_frame(ControlFrameKind::If, f);
        let source = schema.reserve_gc_local(f).initialize(
            observed
                .value
                .cast_reference::<TemporalPlainMonthDayObject>(schema, f),
            f,
        );
        let record = schema.struct_type::<TemporalPlainMonthDayObject>();
        let field = schema.reserve_i32_local(f);
        let calendar = schema.reserve_gc_local(f).initialize(
            record
                .field(TemporalPlainMonthDayObjectSchema::CALENDAR)
                .read(&source, schema, f)
                .reference(),
            f,
        );
        self.emit_dtf_calendar_compatibility(formatter, &calendar, false, f)?;
        calendar.clear(f);
        record
            .field(TemporalPlainMonthDayObjectSchema::REFERENCE_ISO_YEAR)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[3].store(f);
        record
            .field(TemporalPlainMonthDayObjectSchema::ISO_MONTH)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[4].store(f);
        record
            .field(TemporalPlainMonthDayObjectSchema::ISO_DAY)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[5].store(f);
        self.emit_dtf_require_available(formatter, DateTimeValueKind::PlainMonthDay, f)?;
        f.instruction(&Instruction::I64Const(12));
        input.words[6].store(f);
        schema.release_i32_local(field, f);
        source.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&observed.kind, DateTimeValueKind::PlainTime, f);
        self.open_frame(ControlFrameKind::If, f);
        let source = schema.reserve_gc_local(f).initialize(
            observed
                .value
                .cast_reference::<TemporalPlainTimeObject>(schema, f),
            f,
        );
        let record = schema.struct_type::<TemporalPlainTimeObject>();
        let field = schema.reserve_i32_local(f);
        f.instruction(&Instruction::I64Const(1970));
        input.words[3].store(f);
        f.instruction(&Instruction::I64Const(1));
        input.words[4].store(f);
        f.instruction(&Instruction::I64Const(1));
        input.words[5].store(f);
        self.emit_dtf_require_available(formatter, DateTimeValueKind::PlainTime, f)?;
        record
            .field(TemporalPlainTimeObjectSchema::HOUR)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        input.words[6].store(f);
        record
            .field(TemporalPlainTimeObjectSchema::MINUTE)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        input.words[7].store(f);
        record
            .field(TemporalPlainTimeObjectSchema::SECOND)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        input.words[8].store(f);
        record
            .field(TemporalPlainTimeObjectSchema::MILLISECOND)
            .read(&source, schema, f)
            .store(field, f);
        input.words[9].load(f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(1000000));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        input.words[9].store(f);
        record
            .field(TemporalPlainTimeObjectSchema::MICROSECOND)
            .read(&source, schema, f)
            .store(field, f);
        input.words[9].load(f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(1000));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        input.words[9].store(f);
        record
            .field(TemporalPlainTimeObjectSchema::NANOSECOND)
            .read(&source, schema, f)
            .store(field, f);
        input.words[9].load(f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        input.words[9].store(f);
        schema.release_i32_local(field, f);
        source.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        emit_domain_is(&observed.kind, DateTimeValueKind::PlainDateTime, f);
        self.open_frame(ControlFrameKind::If, f);
        let source = schema.reserve_gc_local(f).initialize(
            observed
                .value
                .cast_reference::<TemporalPlainDateTimeObject>(schema, f),
            f,
        );
        let record = schema.struct_type::<TemporalPlainDateTimeObject>();
        let field = schema.reserve_i32_local(f);
        let calendar = schema.reserve_gc_local(f).initialize(
            record
                .field(TemporalPlainDateTimeObjectSchema::CALENDAR)
                .read(&source, schema, f)
                .reference(),
            f,
        );
        self.emit_dtf_calendar_compatibility(formatter, &calendar, true, f)?;
        calendar.clear(f);
        record
            .field(TemporalPlainDateTimeObjectSchema::ISO_YEAR)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[3].store(f);
        record
            .field(TemporalPlainDateTimeObjectSchema::ISO_MONTH)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[4].store(f);
        record
            .field(TemporalPlainDateTimeObjectSchema::ISO_DAY)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32S);
        input.words[5].store(f);
        self.emit_dtf_require_available(formatter, DateTimeValueKind::PlainDateTime, f)?;
        record
            .field(TemporalPlainDateTimeObjectSchema::HOUR)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        input.words[6].store(f);
        record
            .field(TemporalPlainDateTimeObjectSchema::MINUTE)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        input.words[7].store(f);
        record
            .field(TemporalPlainDateTimeObjectSchema::SECOND)
            .read(&source, schema, f)
            .store(field, f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        input.words[8].store(f);
        record
            .field(TemporalPlainDateTimeObjectSchema::MILLISECOND)
            .read(&source, schema, f)
            .store(field, f);
        input.words[9].load(f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(1000000));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        input.words[9].store(f);
        record
            .field(TemporalPlainDateTimeObjectSchema::MICROSECOND)
            .read(&source, schema, f)
            .store(field, f);
        input.words[9].load(f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(1000));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        input.words[9].store(f);
        record
            .field(TemporalPlainDateTimeObjectSchema::NANOSECOND)
            .read(&source, schema, f)
            .store(field, f);
        input.words[9].load(f);
        field.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Mul);
        f.instruction(&Instruction::I64Add);
        input.words[9].store(f);
        schema.release_i32_local(field, f);
        source.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(input)
    }
    pub(super) fn emit_dtf_calendar_compatibility(
        &mut self,
        formatter: &GcLocal<IntlDateTimeFormatObject>,
        calendar: &GcLocal<StringValue>,
        iso_allowed: bool,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let expected = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<IntlDateTimeFormatObject>()
                .field(IntlDateTimeFormatObjectSchema::CALENDAR)
                .read(formatter, schema, f)
                .reference(),
            f,
        );
        let compatible = schema.reserve_i32_local(f);
        self.emit_string_payload_equality_i32(calendar, &expected, f);
        compatible.store(f);
        if iso_allowed {
            let iso = schema
                .reserve_gc_local(f)
                .initialize(self.emit_interned_string_reference("iso8601", f)?, f);
            self.emit_string_payload_equality_i32(calendar, &iso, f);
            compatible.load(f);
            f.instruction(&Instruction::I32Or);
            compatible.store(f);
            iso.clear(f);
        }
        compatible.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_range_error(INTL_DTF_CALENDAR_MISMATCH, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(compatible, f);
        expected.clear(f);
        Ok(())
    }
    fn emit_dtf_require_available(
        &mut self,
        formatter: &GcLocal<IntlDateTimeFormatObject>,
        kind: DateTimeValueKind,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let available = GcI64DomainLocal::new(
            schema,
            DateTimeFormatAvailability::from_available_kinds([]),
            f,
        );
        schema
            .struct_type::<IntlDateTimeFormatObject>()
            .field(IntlDateTimeFormatObjectSchema::AVAILABLE_FORMATS)
            .read(formatter, schema, f)
            .store_i64_domain(&available, f);
        available.load(f);
        f.instruction(&Instruction::I64Const(
            DateTimeFormatAvailability::mask_for(kind) as i64,
        ));
        f.instruction(&Instruction::I64And);
        f.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(INTL_DTF_EMPTY_TEMPORAL_FORMAT, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        available.clear(schema, f);
        Ok(())
    }
    /// Divide canonical GC limbs as base-2^32 halfwords. The residue product is
    /// below 1e9*2^32; validated Temporal epoch magnitude keeps the quotient bounded.
    pub(super) fn emit_dtf_epoch_pair(
        &self,
        epoch: &GcLocal<BigIntValue>,
        seconds: I64Local,
        nano: I64Local,
        f: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let limbs = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<BigIntValue>()
                .field(BigIntValueSchema::LIMBS)
                .read(epoch, schema, f)
                .reference(),
            f,
        );
        let negative = schema.reserve_i32_local(f);
        schema
            .struct_type::<BigIntValue>()
            .field(BigIntValueSchema::NEGATIVE)
            .read(epoch, schema, f)
            .store(negative, f);
        let index = schema.reserve_i32_local(f);
        schema
            .array_type::<BigIntLimbArray>()
            .length(&limbs, schema, f);
        index.store(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(2));
        f.instruction(&Instruction::I32GtU);
        self.emit_dtf_input_corrupt_if(f);
        let limb = schema.reserve_i64_local(f);
        let combined = schema.reserve_i64_local(f);
        let quotient = schema.reserve_i64_local(f);
        for value in [seconds, nano] {
            f.instruction(&Instruction::I64Const(0));
            value.store(f);
        }
        f.instruction(&Instruction::Block(BlockType::Empty));
        f.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(f);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::BrIf(1));
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Sub);
        index.store(f);
        schema
            .array_type::<BigIntLimbArray>()
            .read(&limbs, index, schema, f)
            .store_i64(limb, f);
        for shift in [32, 0] {
            nano.load(f);
            f.instruction(&Instruction::I64Const(32));
            f.instruction(&Instruction::I64Shl);
            limb.load(f);
            f.instruction(&Instruction::I64Const(shift));
            f.instruction(&Instruction::I64ShrU);
            f.instruction(&Instruction::I64Const(0xffff_ffff));
            f.instruction(&Instruction::I64And);
            f.instruction(&Instruction::I64Or);
            combined.store(f);
            combined.load(f);
            f.instruction(&Instruction::I64Const(1_000_000_000));
            f.instruction(&Instruction::I64DivU);
            quotient.store(f);
            combined.load(f);
            f.instruction(&Instruction::I64Const(1_000_000_000));
            f.instruction(&Instruction::I64RemU);
            nano.store(f);
            seconds.load(f);
            f.instruction(&Instruction::I64Const(0xffff_ffff));
            f.instruction(&Instruction::I64GtU);
            self.emit_dtf_input_corrupt_if(f);
            seconds.load(f);
            f.instruction(&Instruction::I64Const(32));
            f.instruction(&Instruction::I64Shl);
            quotient.load(f);
            f.instruction(&Instruction::I64Or);
            seconds.store(f);
        }
        f.instruction(&Instruction::Br(0));
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        seconds.load(f);
        f.instruction(&Instruction::I64Const(8_640_000_000_000));
        f.instruction(&Instruction::I64GtU);
        self.emit_dtf_input_corrupt_if(f);
        seconds.load(f);
        f.instruction(&Instruction::I64Const(8_640_000_000_000));
        f.instruction(&Instruction::I64Eq);
        nano.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::I32And);
        self.emit_dtf_input_corrupt_if(f);
        negative.load(f);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::I64Const(0));
        seconds.load(f);
        f.instruction(&Instruction::I64Sub);
        seconds.store(f);
        nano.load(f);
        f.instruction(&Instruction::I64Eqz);
        f.instruction(&Instruction::I32Eqz);
        f.instruction(&Instruction::If(BlockType::Empty));
        seconds.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Sub);
        seconds.store(f);
        f.instruction(&Instruction::I64Const(1_000_000_000));
        nano.load(f);
        f.instruction(&Instruction::I64Sub);
        nano.store(f);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::End);
        schema.release_i64_local(quotient, f);
        schema.release_i64_local(combined, f);
        schema.release_i64_local(limb, f);
        schema.release_i32_local(index, f);
        schema.release_i32_local(negative, f);
        limbs.clear(f);
    }
    fn emit_dtf_input_corrupt_if(&self, f: &mut Function) {
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::Unreachable);
        f.instruction(&Instruction::End);
    }
}
