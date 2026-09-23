use super::*;
use crate::builtins::temporal::TemporalEpochNanosecondsRecord;
use lila_intl::{DateTimeValueKind, DATE_TIME_INPUT_BYTES};

pub(super) const INTL_DTF_CALENDAR_MISMATCH: &str =
    "Temporal calendar does not match the formatter calendar";

struct DtfObservedValue {
    payload: u32,
    tag: u32,
    kind: u32,
}

impl DtfObservedValue {
    fn reserve(builder: &mut FunctionBuilder<'_>) -> Self {
        Self {
            payload: builder.reserve_temp_local(),
            tag: builder.reserve_temp_local(),
            kind: builder.reserve_temp_local(),
        }
    }
    fn release(self, builder: &mut FunctionBuilder<'_>) {
        builder.release_temp_local(self.kind);
        builder.release_temp_local(self.tag);
        builder.release_temp_local(self.payload);
    }
}

impl FunctionBuilder<'_> {
    /// ToDateTimeFormattable, without the later TimeClip or calendar checks.
    /// Range calls retain both results before either input record is created.
    fn emit_dtf_observe_value(
        &mut self,
        value: &DtfObservedValue,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand = self.reserve_temp_local();
        self.emit_dtf_set_const(value.kind, DtfValueKind::Legacy.code(), function);
        self.emit_dtf_if_code_eq(value.tag, ValueKind::Object.tag() as i64, function);
        self.load_i64_to_local_from_offset(
            value.payload,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand,
            function,
        );
        for kind in DtfBrandedKind::all() {
            self.emit_dtf_if_code_eq(brand, kind.brand() as i64, function);
            self.emit_dtf_set_const(value.kind, DtfValueKind::Branded(kind).code(), function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.emit_dtf_if_code_eq(value.kind, DtfValueKind::Legacy.code(), function);
        self.emit_value_to_number_payload(value.tag, value.payload, function)?;
        function.instruction(&Instruction::LocalSet(value.payload));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(brand);
        Ok(())
    }

    pub(super) fn emit_dtf_single_input(
        &mut self,
        formatter: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value = DtfObservedValue::reserve(self);
        self.emit_builtin_arg_to_locals(0, value.payload, value.tag, function);
        self.emit_dtf_if_code_eq(value.tag, ValueKind::Undefined.tag() as i64, function);
        let clock = self
            .functions
            .wall_clock_millis_import_function_index()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "Intl.DateTimeFormat format requires the lila_host.wall_clock_millis import",
                )
            })?;
        function.instruction(&Instruction::Call(clock));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(value.payload));
        self.emit_dtf_set_const(value.tag, ValueKind::Number.tag() as i64, function);
        function.instruction(&Instruction::End);
        self.emit_dtf_observe_value(&value, function)?;
        self.emit_dtf_input_record(formatter, &value, destination, function)?;
        value.release(self);
        Ok(())
    }

    pub(super) fn emit_dtf_range_inputs(
        &mut self,
        formatter: u32,
        start: u32,
        end: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let left = DtfObservedValue::reserve(self);
        let right = DtfObservedValue::reserve(self);
        self.emit_builtin_arg_to_locals(0, left.payload, left.tag, function);
        self.emit_builtin_arg_to_locals(1, right.payload, right.tag, function);
        for tag in [left.tag, right.tag] {
            function.instruction(&Instruction::LocalGet(tag));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
        }
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            INTL_DTF_RANGE_UNDEFINED_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_dtf_observe_value(&left, function)?;
        self.emit_dtf_observe_value(&right, function)?;
        function.instruction(&Instruction::LocalGet(left.kind));
        function.instruction(&Instruction::LocalGet(right.kind));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            INTL_DTF_RANGE_DIFFERENT_TYPES_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_dtf_input_record(formatter, &left, start, function)?;
        self.emit_dtf_input_record(formatter, &right, end, function)?;
        right.release(self);
        left.release(self);
        Ok(())
    }

    fn emit_dtf_input_record(
        &mut self,
        formatter: u32,
        value: &DtfObservedValue,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let integer = self.reserve_temp_local();
        let seconds = self.reserve_temp_local();
        let nanos = self.reserve_temp_local();
        self.emit_dtf_if_code_eq(
            value.kind,
            DtfValueKind::Branded(DtfBrandedKind::ZonedDateTime).code(),
            function,
        );
        self.emit_throw_current_function_realm_type_error(
            INTL_DTF_ZONED_DATE_TIME_UNSUPPORTED,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_const(DATE_TIME_INPUT_BYTES, function)?;
        function.instruction(&Instruction::LocalSet(destination));
        for offset in (0..DATE_TIME_INPUT_BYTES).step_by(8) {
            self.store_i64_const_at_offset(destination, offset, 0, function);
        }
        self.emit_dtf_if_code_eq(value.kind, DtfValueKind::Legacy.code(), function);
        self.emit_date_time_clip(value.payload, integer, function);
        function.instruction(&Instruction::LocalGet(integer));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(integer));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Date value is not finite",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(integer));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(integer));
        function.instruction(&Instruction::LocalGet(integer));
        function.instruction(&Instruction::I64Const(1000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(seconds));
        function.instruction(&Instruction::LocalGet(integer));
        function.instruction(&Instruction::I64Const(1000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(nanos));
        self.emit_dtf_store_exact_input(
            destination,
            DateTimeValueKind::Legacy,
            seconds,
            nanos,
            function,
        );
        function.instruction(&Instruction::End);
        for kind in INTL_DTF_TEMPORAL_KINDS {
            self.emit_dtf_if_code_eq(value.kind, kind.code(), function);
            self.load_i64_to_local_from_offset(
                value.payload,
                HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                record,
                function,
            );
            match kind {
                DtfTemporalKind::Instant => {
                    self.emit_temporal_epoch_nanoseconds_pair(
                        record,
                        TemporalEpochNanosecondsRecord::Instant,
                        seconds,
                        nanos,
                        function,
                    );
                    self.emit_dtf_store_exact_input(
                        destination,
                        DateTimeValueKind::Instant,
                        seconds,
                        nanos,
                        function,
                    );
                }
                DtfTemporalKind::PlainDate
                | DtfTemporalKind::PlainYearMonth
                | DtfTemporalKind::PlainMonthDay
                | DtfTemporalKind::PlainTime
                | DtfTemporalKind::PlainDateTime => {
                    self.emit_dtf_store_plain_input(
                        formatter,
                        record,
                        kind.value_kind(),
                        destination,
                        integer,
                        function,
                    )?;
                }
            }
            function.instruction(&Instruction::End);
        }
        for local in [nanos, seconds, integer, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_dtf_store_exact_input(
        &self,
        destination: u32,
        kind: DateTimeValueKind,
        seconds: u32,
        nanos: u32,
        function: &mut Function,
    ) {
        // The BigInt bridge returns a signed truncation pair. Normalize to
        // floor seconds before crossing the provider boundary, including -1ns.
        function.instruction(&Instruction::LocalGet(nanos));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seconds));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds));
        function.instruction(&Instruction::LocalGet(nanos));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(nanos));
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(destination, 0, kind.wire_code(), function);
        self.store_i64_local_at_offset(destination, 8, seconds, function);
        self.store_i64_local_at_offset(destination, 16, nanos, function);
    }

    fn emit_dtf_store_plain_input(
        &mut self,
        formatter: u32,
        source: u32,
        kind: DateTimeValueKind,
        destination: u32,
        scratch: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.store_i64_const_at_offset(destination, 0, kind.wire_code(), function);
        if kind != DateTimeValueKind::PlainTime {
            let calendar = self.reserve_temp_local();
            let expected = self.reserve_temp_local();
            let compatible = self.reserve_temp_local();
            let date_time = kind == DateTimeValueKind::PlainDateTime;
            self.load_i64_to_local_from_offset(
                source,
                if date_time {
                    HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET
                } else {
                    HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET
                },
                calendar,
                function,
            );
            self.load_i64_to_local_from_offset(
                formatter,
                HEAP_INTL_DTF_CALENDAR_OFFSET,
                expected,
                function,
            );
            self.emit_string_payload_equality_i32(calendar, expected, function);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(compatible));
            if matches!(
                kind,
                DateTimeValueKind::PlainDate | DateTimeValueKind::PlainDateTime
            ) {
                self.emit_dtf_set_string(expected, "iso8601", function);
                self.emit_string_payload_equality_i32(calendar, expected, function);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalGet(compatible));
                function.instruction(&Instruction::I64Or);
                function.instruction(&Instruction::LocalSet(compatible));
            }
            function.instruction(&Instruction::LocalGet(compatible));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_range_error(
                INTL_DTF_CALENDAR_MISMATCH,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
            for (from, to) in if date_time {
                [
                    (HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET, 24),
                    (HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET, 32),
                    (HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET, 40),
                ]
            } else {
                [
                    (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, 24),
                    (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, 32),
                    (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, 40),
                ]
            } {
                self.load_i64_to_local_from_offset(source, from, scratch, function);
                self.store_i64_local_at_offset(destination, to, scratch, function);
            }
            for local in [compatible, expected, calendar] {
                self.release_temp_local(local);
            }
        } else {
            for (offset, value) in [(24, 1970), (32, 1), (40, 1)] {
                self.store_i64_const_at_offset(destination, offset, value, function);
            }
        }
        // HandleDateTimeValue checks this operand's calendar and availability
        // before the next range operand can produce an error.
        self.load_i64_to_local_from_offset(
            formatter,
            HEAP_INTL_DTF_AVAILABLE_FORMATS_OFFSET,
            scratch,
            function,
        );
        function.instruction(&Instruction::LocalGet(scratch));
        function.instruction(&Instruction::I64Const(
            lila_intl::DateTimeFormatAvailability::mask_for(kind) as i64,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            INTL_DTF_EMPTY_TEMPORAL_FORMAT,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        if matches!(
            kind,
            DateTimeValueKind::PlainDate
                | DateTimeValueKind::PlainYearMonth
                | DateTimeValueKind::PlainMonthDay
        ) {
            self.store_i64_const_at_offset(destination, 48, 12, function);
        } else {
            let offsets = if kind == DateTimeValueKind::PlainDateTime {
                [
                    HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET,
                    HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET,
                    HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET,
                    HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET,
                    HEAP_TEMPORAL_PLAIN_DATE_TIME_MICROSECOND_OFFSET,
                    HEAP_TEMPORAL_PLAIN_DATE_TIME_NANOSECOND_OFFSET,
                ]
            } else {
                [
                    HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET,
                    HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET,
                    HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET,
                    HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET,
                    HEAP_TEMPORAL_PLAIN_TIME_MICROSECOND_OFFSET,
                    HEAP_TEMPORAL_PLAIN_TIME_NANOSECOND_OFFSET,
                ]
            };
            for (from, to) in offsets[..3].iter().copied().zip([48, 56, 64]) {
                self.load_i64_to_local_from_offset(source, from, scratch, function);
                self.store_i64_local_at_offset(destination, to, scratch, function);
            }
            self.emit_dtf_set_const(scratch, 0, function);
            for (offset, scale) in offsets[3..].iter().copied().zip([1_000_000, 1000, 1]) {
                function.instruction(&Instruction::LocalGet(scratch));
                function.instruction(&Instruction::LocalGet(source));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load(MemArg {
                    offset,
                    align: 3,
                    memory_index: 0,
                }));
                function.instruction(&Instruction::I64Const(scale));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(scratch));
            }
            self.store_i64_local_at_offset(destination, 72, scratch, function);
        }
        Ok(())
    }
}
