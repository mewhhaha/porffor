//! `Temporal.PlainMonthDay` codegen.
//!
//! Temporal proposal 10: a calendar month and day with a *reference* ISO year
//! that is not observable through any accessor. Its concrete Wasm-GC record
//! stores three ISO coordinates and a canonical calendar String. Calendar
//! projection and ISO arithmetic use the same owners as `Temporal.PlainDate`;
//! only the field set, the reference year and the string form differ.
//!
//! Non-ISO references preserve canonical month code/day and select the latest
//! eligible date through the shared completed partial-reference owner.

use crate::gc_types::*;

use super::super::*;
use super::temporal_options::{
    ShowCalendarName, TemporalConversionOverflowOptions, TemporalOverflow,
};
use super::temporal_plain_date::{TemporalMonthFieldContext, TemporalResolvedCalendarYear};
use super::temporal_plain_year_month::TemporalPartialDateType;
use super::temporal_zone_provider::TemporalCalendarSlotLocals;
use crate::intrinsics::temporal::{TemporalIntrinsicFamily, TemporalPrototypeSource};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalPlainMonthDayField {
    CalendarId,
    MonthCode,
    Day,
}

/// Formatting owns an options read only for the toString entry.
pub(super) enum TemporalPlainMonthDayStringMode {
    ToString,
    ToJson,
}

/// The `[[Year]]` half of a `ParseTemporalMonthDayString` result: the parsed ISO
/// year, plus whether the source actually carried one.
///
/// `ToTemporalMonthDay`'s string branch asks two questions of that pair and then
/// throws the year away (steps named, not lettered — see
/// [`FunctionBuilder::emit_temporal_month_day_string_reference_year`] for why):
///
/// * **year-empty rejection** — `result.[[Year]] is empty` and the calendar is
///   not `iso8601` is a RangeError, which is what rejects
///   `"11-18[u-ca=gregory]"`;
/// * **`ISODateWithinLimits`** — a non-ISO calendar bounds the *parsed* date,
///   which is what rejects `"±999999-01-01[u-ca=gregory]"`.
///
/// The sole consumer takes this pair by value, performs both source checks,
/// then replaces the parsed date with the calendar's completed reference ISO
/// carrier. Gregorian references use 1972; Indian chooses the latest matching
/// date on or before 1972-12-31. The parsed-year fact remains distinct from the
/// completed partial-reference owner consumed at publication.
///
/// Fields are private and only the actual month-day parser mints this proof.
/// `#[must_use]` warns when the parser result is discarded; the conversion's
/// completed reference is separately required by the actual partial allocator.
#[must_use]
struct TemporalParsedMonthDayYear {
    year_local: I64Local,
    year_present_local: I64Local,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_temporal_month_day_overflow_option(
        &mut self,
        options: &ValueLocals,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options,
            overflow,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINMONTHDAY_OVERFLOW_OPTION,
            function,
        )
    }

    /// Recover the retained canonical calendar String as a concrete slot proof.
    fn emit_temporal_month_day_calendar_slot(
        &mut self,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<TemporalCalendarSlotLocals, EmitError> {
        let schema = self.runtime_schema();
        let identifier = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let calendar = self.emit_temporal_calendar_slot_from_identifier(&identifier, function)?;
        identifier.clear(function);
        Ok(calendar)
    }

    /// Temporal proposal 10.1.1:
    /// `Temporal.PlainMonthDay(isoMonth, isoDay [, calendar [, referenceISOYear]])`.
    pub(crate) fn emit_temporal_plain_month_day_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_require_construct_call(
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_CONSTRUCTOR_REQUIRES_NEW,
            function,
        )?;
        let schema = self.runtime_schema();
        let input = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        self.emit_builtin_arg_to_value(0, &input, function);
        self.emit_temporal_to_integer_with_truncation(
            &input,
            fields[1],
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_MONTH_MUST_BE_AN_INTEGER,
            function,
        )?;
        self.emit_builtin_arg_to_value(1, &input, function);
        self.emit_temporal_to_integer_with_truncation(
            &input,
            fields[2],
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_DAY_MUST_BE_AN_INTEGER,
            function,
        )?;
        self.emit_builtin_arg_to_value(2, &input, function);
        let calendar = self.emit_temporal_plain_date_calendar(&input, function)?;
        function.instruction(&Instruction::I64Const(1972));
        fields[0].store(function);
        self.emit_builtin_arg_to_value(3, &input, function);
        input.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_to_integer_with_truncation(
            &input,
            fields[0],
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_REFERENCE_YEAR_MUST_BE_AN_INTEGER,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let reference = self.emit_temporal_complete_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainMonthDay,
            fields,
            function,
        )?;
        let prototype = self.emit_temporal_constructor_prototype(
            TemporalIntrinsicFamily::PlainMonthDay,
            function,
        )?;
        self.emit_alloc_temporal_partial_reference(
            &reference,
            TemporalPrototypeSource::Constructor(&prototype),
            function,
        )?;
        prototype.release(function);
        reference.release(self, function);
        calendar.release(self, function);
        input.clear(function);
        for local in fields.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        Ok(())
    }

    /// The `[[InitializedTemporalMonthDay]]` brand check.
    pub(crate) fn emit_temporal_plain_month_day_record_from_receiver(
        &mut self,
        function: &mut Function,
    ) -> Result<GcLocal<TemporalPlainMonthDayObject>, EmitError> {
        self.emit_temporal_record_from_receiver(function)
    }

    pub(crate) fn emit_temporal_plain_month_day_load_record(
        &self,
        record: &GcLocal<TemporalPlainMonthDayObject>,
        fields: &[I64Local; 3],
        calendar: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let scalar = schema.reserve_i32_local(function);
        for (field, output) in [
            TemporalPlainMonthDayObjectSchema::REFERENCE_ISO_YEAR,
            TemporalPlainMonthDayObjectSchema::ISO_MONTH,
            TemporalPlainMonthDayObjectSchema::ISO_DAY,
        ]
        .into_iter()
        .zip(fields)
        {
            schema
                .struct_type::<TemporalPlainMonthDayObject>()
                .field(field)
                .read(record, schema, function)
                .store(scalar, function);
            scalar.load(function);
            function.instruction(&Instruction::I64ExtendI32S);
            output.store(function);
        }
        let text = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<TemporalPlainMonthDayObject>()
                .field(TemporalPlainMonthDayObjectSchema::CALENDAR)
                .read(record, schema, function)
                .reference(),
            function,
        );
        calendar.set_reference(&text, schema, function);
        text.clear(function);
        schema.release_i32_local(scalar, function);
    }

    /// The three `Temporal.PlainMonthDay.prototype` accessors.
    pub(crate) fn emit_temporal_plain_month_day_field(
        &mut self,
        field: TemporalPlainMonthDayField,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record =
            self.emit_temporal_record_from_receiver::<TemporalPlainMonthDayObject>(function)?;
        let fields = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        let output = schema.reserve_value_local(function);
        self.emit_temporal_plain_month_day_load_record(&record, &fields, &calendar_value, function);
        let identifier = schema.reserve_gc_local(function).initialize(
            calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let calendar = self.emit_temporal_calendar_slot_from_identifier(&identifier, function)?;
        let date_field = match field {
            TemporalPlainMonthDayField::CalendarId => {
                super::temporal_plain_date::TemporalPlainDateField::CalendarId
            }
            TemporalPlainMonthDayField::MonthCode => {
                super::temporal_plain_date::TemporalPlainDateField::MonthCode
            }
            TemporalPlainMonthDayField::Day => {
                super::temporal_plain_date::TemporalPlainDateField::Day
            }
        };
        self.emit_temporal_date_field_value(date_field, &calendar, fields, &output, function)?;
        self.completion().set_normal(&output, function);
        calendar.release(self, function);
        identifier.clear(function);
        output.clear(function);
        calendar_value.clear(function);
        for local in fields.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        record.clear(function);
        Ok(())
    }

    /// `RegulateISODate` without the `ISODateWithinLimits` tail. A month-day
    /// uses this primitive only in the Gregorian MonthDay branch. A supplied
    /// ISO `year` decides how 29 February constrains - Test262's
    /// `from/iso-year-used-only-for-overflow.js` pins that an out-of-range year
    /// must not throw.
    pub(crate) fn emit_temporal_month_day_regulate(
        &mut self,
        year_local: I64Local,
        month_local: I64Local,
        day_local: I64Local,
        overflow_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        overflow_local.load(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        month_local.load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        day_local.load(function);
        maximum_day_local.load(function);
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_IS_NOT_A_VALID_ISO_DATE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        month_local.load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(12));
        month_local.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        day_local.load(function);
        maximum_day_local.load(function);
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        maximum_day_local.load(function);
        day_local.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(maximum_day_local, function);
        Ok(())
    }

    /// `CalendarMonthDayToISOReferenceDate`. The supplied year, if any, decides
    /// overflow regulation. The stored ISO carrier is the calendar's completed
    /// reference, which may vary by month/day for Indian.
    ///
    /// For a calendar whose [`super::temporal_plain_date::TemporalCalendarId::month_day_year_use`] is
    /// [`super::temporal_plain_date::MonthDayYearUse::RangeChecked`] the year is *also* bounded, before any
    /// month information is computed. That fork is real: `iso8601` accepts
    /// `year: -999999` and `gregory` rejects it. See [`super::temporal_plain_date::MonthDayYearUse`].
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_month_day_resolve_fields(
        &mut self,
        resolved_year: TemporalResolvedCalendarYear,
        month: I64Local,
        month_present: I64Local,
        acquired_month_code: &ValueLocals,
        encoded_month_code: I64Local,
        month_code_present: I64Local,
        day: I64Local,
        day_present: I64Local,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar_id = resolved_year.calendar_id();
        let year = resolved_year.year_local();
        let year_present = resolved_year.year_present_local();
        day_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_FIELDS_REQUIRE_DAY,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        month_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        month_code_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_FIELDS_REQUIRE_MONTH_OR_MONTHCODE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // A non-ISO numeric month needs its native year to determine the ordinal.
        self.emit_temporal_calendar_is_default_i32(calendar_id, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        month_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        year_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_MONTH_REQUIRES_YEAR_FOR_A_NON_ISO_CALENDAR,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let resolved_month = self.emit_temporal_resolve_calendar_month(
            resolved_year,
            month,
            month_present,
            acquired_month_code,
            encoded_month_code,
            month_code_present,
            TemporalMonthFieldContext::PlainMonthDay,
            function,
        )?;
        month.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        month_present.load(function);
        year_present.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        day.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_MONTH_AND_DAY_MUST_BE_POSITIVE,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let reference = self.emit_temporal_month_day_reference_from_fields(
            resolved_month,
            day,
            overflow,
            function,
        )?;
        for (source, destination) in reference.fields().into_iter().zip([year, month, day]) {
            source.load(function);
            destination.store(function);
        }
        reference.release(self, function);
        Ok(())
    }

    /// `ToTemporalMonthDay`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_to_temporal_month_day(
        &mut self,
        argument: &ValueLocals,
        overflow_options: TemporalConversionOverflowOptions<'_>,
        fields: &[I64Local; 3],
        calendar_out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let overflow = schema.reserve_i64_local(function);
        let present: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let month_code = schema.reserve_value_local(function);
        let encoded_month_code = schema.reserve_i64_local(function);
        let month_code_present = schema.reserve_i64_local(function);
        let any_present = schema.reserve_i64_local(function);
        let handled = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        handled.store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        overflow.store(function);
        month_code.set_undefined(function);
        for local in fields
            .iter()
            .chain(present.iter())
            .copied()
            .chain([encoded_month_code, month_code_present])
        {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        self.open_frame(ControlFrameKind::If, function);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainMonthDayObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<TemporalPlainMonthDayObject>(schema, function),
            function,
        );
        self.emit_temporal_plain_month_day_load_record(&record, fields, calendar_out, function);
        record.clear(function);
        match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => {
                self.emit_temporal_month_day_overflow_option(options, overflow, function)?
            }
            TemporalConversionOverflowOptions::Omit => {}
        }
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let calendar_value = schema.reserve_value_local(function);
        self.emit_temporal_duration_option_get(argument, "calendar", &calendar_value, function)?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATE_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        calendar_out.set_reference(calendar.identifier(), schema, function);
        calendar_value.clear(function);
        let era = self.emit_temporal_plain_date_read_fields(
            argument,
            &calendar,
            fields,
            &present,
            &month_code,
            month_code_present,
            any_present,
            function,
        )?;
        match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => {
                self.emit_temporal_month_day_overflow_option(options, overflow, function)?
            }
            TemporalConversionOverflowOptions::Omit => {}
        }
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            fields[0],
            present[0],
            function,
        )?;
        self.emit_temporal_month_day_resolve_fields(
            resolved_year,
            fields[1],
            present[1],
            &month_code,
            encoded_month_code,
            month_code_present,
            fields[2],
            present[2],
            overflow,
            function,
        )?;
        calendar.release(self, function);
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_EXPECTS_A_STRING_A_PROPERTY_BAG_OR_A_TEMPORAL_PLAINMONTHDAY, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        let parsed = self.emit_temporal_parse_month_day_string(
            &string,
            fields[0],
            present[0],
            fields[1],
            fields[2],
            calendar_out,
            function,
        )?;
        string.clear(function);
        // GetTemporalOverflowOption follows parsing and precedes the source
        // year-empty and source-date range checks, including on equals' path.
        match overflow_options {
            TemporalConversionOverflowOptions::Read(options) => {
                self.emit_temporal_month_day_overflow_option(options, overflow, function)?
            }
            TemporalConversionOverflowOptions::Omit => {}
        }
        let calendar = self.emit_temporal_month_day_calendar_slot(calendar_out, function)?;
        self.emit_temporal_month_day_string_reference_year(
            parsed, &calendar, fields[1], fields[2], function,
        )?;
        calendar.release(self, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        schema.release_i64_local(handled, function);
        schema.release_i64_local(any_present, function);
        schema.release_i64_local(month_code_present, function);
        schema.release_i64_local(encoded_month_code, function);
        month_code.clear(function);
        for local in present.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        schema.release_i64_local(overflow, function);
        Ok(())
    }

    /// Temporal proposal 10.2.2 `Temporal.PlainMonthDay.from`.
    pub(crate) fn emit_temporal_plain_month_day_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_temporal_to_temporal_month_day(
            &argument,
            TemporalConversionOverflowOptions::Read(&options),
            &fields,
            &calendar_value,
            function,
        )?;
        let calendar = self.emit_temporal_month_day_calendar_slot(&calendar_value, function)?;
        let reference = self.emit_temporal_complete_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainMonthDay,
            fields,
            function,
        )?;
        self.emit_alloc_temporal_partial_reference(
            &reference,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        reference.release(self, function);
        calendar.release(self, function);
        for local in fields.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// `ParseTemporalMonthDayString`, via the shared bare-form rewrite.
    ///
    /// The rewrite prepends `1972` to the four year-less spellings, so after it
    /// nothing downstream can tell `"--10-01"` from `"1972-10-01"`. That is why
    /// the rewrite reports `result.[[Year]] is empty` into a slot here rather
    /// than releasing it, and why this returns a
    /// [`TemporalParsedMonthDayYear`] instead of `()`.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_parse_month_day_string(
        &mut self,
        string: &GcLocal<StringValue>,
        year: I64Local,
        year_present: I64Local,
        month: I64Local,
        day: I64Local,
        calendar: &ValueLocals,
        function: &mut Function,
    ) -> Result<TemporalParsedMonthDayYear, EmitError> {
        let schema = self.runtime_schema();
        let rewritten = schema.reserve_value_local(function);
        let year_empty = schema.reserve_i64_local(function);
        self.emit_temporal_month_day_rewrite_string(
            string,
            &rewritten,
            Some(year_empty),
            function,
        )?;
        // Convert the parser's empty flag into the field readers' presence flag.
        year_empty.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        year_present.store(function);
        let text = schema.reserve_gc_local(function).initialize(
            rewritten.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_parse_plain_date_string(&text, year, month, day, calendar, function)?;
        text.clear(function);
        schema.release_i64_local(year_empty, function);
        rewritten.clear(function);
        Ok(TemporalParsedMonthDayYear {
            year_local: year,
            year_present_local: year_present,
        })
    }

    /// The three things `ToTemporalMonthDay`'s string branch does with the
    /// parsed `[[Year]]` before discarding it.
    ///
    /// Steps are named rather than lettered. An earlier version of this comment
    /// lettered them (g)/(k)/(l) and the letters were shifted; spec step letters
    /// in this area have moved between proposal revisions, and a stale letter
    /// reads as authority it does not have. The operation names do not drift.
    ///
    /// Emitted in spec order:
    ///
    /// * **year-empty rejection** — `result.[[Year]] is empty` with a
    ///   non-`iso8601` calendar is a RangeError. `"11-18[u-ca=gregory]"` is the
    ///   case; `"11-18"` and `"--10-01"` are not, because the `iso8601` branch
    ///   returns before this is reached, and that ISO gate is why
    ///   `plainMonthDayStringsValid()`'s bare forms keep working.
    /// * **`ISODateWithinLimits`** — a non-`iso8601` calendar bounds the parsed
    ///   date. `"±999999-01-01[u-ca=gregory]"` is the case;
    ///   `"±999999-10-01[u-ca=iso8601]"` is explicitly *valid* and is the proof
    ///   that this bound must stay behind the same ISO gate.
    /// * **calendar reference conversion** — the complete parsed ISO date
    ///   projects to the calendar month/day, then the completed reference
    ///   factory chooses its deterministic ISO carrier. Gregorian calendars
    ///   use 1972; Indian uses the latest matching date before 1973.
    ///
    /// Both checks are outside the caller's overflow-options match, because
    /// `equals` reaches `ToTemporalMonthDay` with no options at all and
    /// `prototype/equals/argument-string-invalid.js` still requires both
    /// RangeErrors. That the overflow read is emitted *before* them follows the
    /// spec (GetTemporalOverflowOption precedes both). The checks stay after
    /// that read even when the source's syntax has already been accepted.
    ///
    /// Taking the [`TemporalParsedMonthDayYear`] by value is the point: this is
    /// also the string path's completed reference conversion, so a future path
    /// cannot publish the parser's supplied year as its reference carrier.
    fn emit_temporal_month_day_string_reference_year(
        &mut self,
        parsed: TemporalParsedMonthDayYear,
        calendar: &TemporalCalendarSlotLocals,
        month: I64Local,
        day: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let TemporalParsedMonthDayYear {
            year_local: year,
            year_present_local: year_present,
        } = parsed;
        self.emit_temporal_calendar_is_default_i32(calendar.calendar_id(), function);
        function.instruction(&Instruction::I32Eqz);
        year_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_MONTH_DAY_STRING_WITH_A_NON_ISO_CALENDAR_REQUIRES_A_YEAR, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_calendar_is_default_i32(calendar.calendar_id(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let days = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_iso_date_within_limits(
            year,
            month,
            day,
            days,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_IS_OUTSIDE_THE_SUPPORTED_DATE_RANGE,
            function,
        )?;
        self.runtime_schema().release_i64_local(days, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let reference = self.emit_temporal_calendar_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainMonthDay,
            [year, month, day],
            function,
        )?;
        for (source, destination) in reference.fields().into_iter().zip([year, month, day]) {
            source.load(function);
            destination.store(function);
        }
        reference.release(self, function);
        Ok(())
    }

    /// Temporal proposal 10.3.x `equals`.
    pub(crate) fn emit_temporal_plain_month_day_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_temporal_plain_month_day_record_from_receiver(function)?;
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let other: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        let other_calendar_value = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let equal = schema.reserve_i32_local(function);
        self.emit_temporal_plain_month_day_load_record(&record, &fields, &calendar_value, function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_temporal_to_temporal_month_day(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &other,
            &other_calendar_value,
            function,
        )?;
        function.instruction(&Instruction::I32Const(0));
        equal.store(function);
        fields[0].load(function);
        other[0].load(function);
        function.instruction(&Instruction::I64Eq);
        fields[1].load(function);
        other[1].load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        fields[2].load(function);
        other[2].load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let calendar = schema.reserve_gc_local(function).initialize(
            calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let other_calendar = schema.reserve_gc_local(function).initialize(
            other_calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_string_payload_equality_i32(&calendar, &other_calendar, function);
        equal.store(function);
        other_calendar.clear(function);
        calendar.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let output = schema.reserve_value_local(function);
        output.set_boolean(equal, function);
        self.completion().set_normal(&output, function);
        output.clear(function);
        schema.release_i32_local(equal, function);
        argument.clear(function);
        other_calendar_value.clear(function);
        calendar_value.clear(function);
        for local in other.into_iter().rev().chain(fields.into_iter().rev()) {
            schema.release_i64_local(local, function);
        }
        record.clear(function);
        Ok(())
    }

    /// Temporal proposal 10.3.x `with`.
    pub(crate) fn emit_temporal_plain_month_day_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_temporal_plain_month_day_record_from_receiver(function)?;
        let receiver: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let present: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let probe = schema.reserve_value_local(function);
        let receiver_month_code = schema.reserve_value_local(function);
        let acquired_month_code = schema.reserve_value_local(function);
        let overflow = schema.reserve_i64_local(function);
        let encoded_month_code = schema.reserve_i64_local(function);
        let month_code_present = schema.reserve_i64_local(function);
        let any_present = schema.reserve_i64_local(function);
        self.emit_temporal_plain_month_day_load_record(
            &record,
            &receiver,
            &calendar_value,
            function,
        );
        let calendar = self.emit_temporal_month_day_calendar_slot(&calendar_value, function)?;
        let projected =
            self.emit_temporal_project_calendar_date(calendar.calendar_id(), receiver, function);
        for (source, destination) in projected.fields().into_iter().zip(receiver) {
            source.load(function);
            destination.store(function);
        }
        self.emit_temporal_calendar_month_code_payload(&projected, &receiver_month_code, function)?;
        projected.release(self, function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_PROTOTYPE_WITH_REQUIRES_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_reject_branded_partial_object(&argument,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_PROTOTYPE_WITH_DOES_NOT_ACCEPT_A_TEMPORAL_OBJECT, function)?;
        // These ordinary Gets precede the ordered native date-field sweep.
        for property in ["calendar", "timeZone"] {
            self.emit_temporal_duration_option_get(&argument, property, &probe, function)?;
            probe.tag().load(function);
            function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
                RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_PROTOTYPE_WITH_DOES_NOT_ACCEPT_CALENDAR_OR_TIMEZONE, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        acquired_month_code.set_undefined(function);
        for local in fields
            .iter()
            .chain(present.iter())
            .copied()
            .chain([encoded_month_code, month_code_present])
        {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        let era = self.emit_temporal_plain_date_read_fields(
            &argument,
            &calendar,
            &fields,
            &present,
            &acquired_month_code,
            month_code_present,
            any_present,
            function,
        )?;
        any_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_PROTOTYPE_WITH_REQUIRES_AT_LEAST_ONE_FIELD,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_month_day_overflow_option(&options, overflow, function)?;
        // Resolve only the caller's year/era pair; a MonthDay has no year default.
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            fields[0],
            present[0],
            function,
        )?;
        present[2].load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        receiver[2].load(function);
        fields[2].store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        present[2].store(function);
        present[1].load(function);
        month_code_present.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        acquired_month_code.copy_from(&receiver_month_code, function);
        function.instruction(&Instruction::I64Const(1));
        month_code_present.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_month_day_resolve_fields(
            resolved_year,
            fields[1],
            present[1],
            &acquired_month_code,
            encoded_month_code,
            month_code_present,
            fields[2],
            present[2],
            overflow,
            function,
        )?;
        let reference = self.emit_temporal_complete_partial_reference(
            calendar.calendar_id(),
            TemporalPartialDateType::PlainMonthDay,
            fields,
            function,
        )?;
        self.emit_alloc_temporal_partial_reference(
            &reference,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        reference.release(self, function);
        calendar.release(self, function);
        for local in [
            any_present,
            month_code_present,
            encoded_month_code,
            overflow,
        ] {
            schema.release_i64_local(local, function);
        }
        acquired_month_code.clear(function);
        receiver_month_code.clear(function);
        probe.clear(function);
        options.clear(function);
        argument.clear(function);
        calendar_value.clear(function);
        for local in present
            .into_iter()
            .rev()
            .chain(fields.into_iter().rev())
            .chain(receiver.into_iter().rev())
        {
            schema.release_i64_local(local, function);
        }
        record.clear(function);
        Ok(())
    }

    /// `Temporal.PlainMonthDay.prototype.toLocaleString`.
    ///
    /// `new Intl.DateTimeFormat(locales, options).format(this)`. The reference
    /// year is masked away by the month-day field set, so the year the record
    /// carries never reaches the output.
    pub(crate) fn emit_temporal_plain_month_day_to_locale_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.emit_temporal_plain_month_day_record_from_receiver(function)?;
        record.clear(function);
        self.emit_intl_dtf_temporal_to_locale_string(
            super::intl_datetimeformat::DtfTemporalKind::PlainMonthDay,
            function,
        )
    }

    /// `TemporalMonthDayToString`. A non-ISO calendar always retains the
    /// reference ISO year, including when its annotation is hidden.
    pub(crate) fn emit_temporal_plain_month_day_to_string(
        &mut self,
        mode: TemporalPlainMonthDayStringMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_temporal_plain_month_day_record_from_receiver(function)?;
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let output = schema.reserve_value_local(function);
        let show_calendar = schema.reserve_i64_local(function);
        self.emit_temporal_plain_month_day_load_record(&record, &fields, &calendar_value, function);
        let calendar = self.emit_temporal_month_day_calendar_slot(&calendar_value, function)?;
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        show_calendar.store(function);
        match mode {
            TemporalPlainMonthDayStringMode::ToString => {
                self.emit_builtin_arg_to_value(0, &options, function);
                self.emit_temporal_string_valued_option::<ShowCalendarName>(
                    &options, show_calendar,
                    RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
                    RuntimeErrorMessage::INVALID_TEMPORAL_PLAINMONTHDAY_CALENDARNAME_OPTION, function)?;
            }
            TemporalPlainMonthDayStringMode::ToJson => {}
        }
        self.emit_temporal_show_partial_reference_i32(
            show_calendar,
            calendar.calendar_id(),
            function,
        );
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_pad_iso_year(fields[0], &output, function)?;
        self.emit_temporal_append_separated_two_digits(fields[1], "-", &output, function)?;
        function.instruction(&Instruction::Else);
        let empty = schema
            .reserve_gc_local(function)
            .initialize(self.emit_interned_string_reference("", function)?, function);
        output.set_reference(&empty, schema, function);
        empty.clear(function);
        self.emit_temporal_append_separated_two_digits(fields[1], "", &output, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_append_separated_two_digits(fields[2], "-", &output, function)?;
        self.emit_temporal_append_calendar_annotation(&calendar, show_calendar, &output, function)?;
        self.completion().set_normal(&output, function);
        calendar.release(self, function);
        schema.release_i64_local(show_calendar, function);
        output.clear(function);
        options.clear(function);
        calendar_value.clear(function);
        for local in fields.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        record.clear(function);
        Ok(())
    }

    /// Temporal proposal 10.3.x `toPlainDate ( item )`: the receiver's month
    /// and day plus a `year` read from `item`.
    pub(crate) fn emit_temporal_plain_month_day_to_plain_date(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let record = self.emit_temporal_plain_month_day_record_from_receiver(function)?;
        let fields: [I64Local; 3] = std::array::from_fn(|_| schema.reserve_i64_local(function));
        let calendar_value = schema.reserve_value_local(function);
        let argument = schema.reserve_value_local(function);
        let acquired_month_code = schema.reserve_value_local(function);
        let year_present = schema.reserve_i64_local(function);
        let overflow = schema.reserve_i64_local(function);
        let encoded_month_code = schema.reserve_i64_local(function);
        let month_code_present = schema.reserve_i64_local(function);
        let month_present = schema.reserve_i64_local(function);
        self.emit_temporal_plain_month_day_load_record(&record, &fields, &calendar_value, function);
        let calendar = self.emit_temporal_month_day_calendar_slot(&calendar_value, function)?;
        let projected =
            self.emit_temporal_project_calendar_date(calendar.calendar_id(), fields, function);
        for (source, destination) in projected.fields().into_iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        self.emit_temporal_calendar_month_code_payload(&projected, &acquired_month_code, function)?;
        projected.release(self, function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_PROTOTYPE_TOPLAINDATE_REQUIRES_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // era/eraYear precede year and are read only for calendars with eras.
        let era_slots = self.reserve_temporal_era_slots(function);
        let era = self.emit_temporal_read_era_fields(
            era_slots,
            &argument,
            calendar.calendar_id(),
            function,
        )?;
        self.emit_temporal_property_bag_integer(
            &argument,
            "year",
            year_present,
            fields[0],
            0,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_YEAR_MUST_BE_FINITE,
            function,
        )?;
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            fields[0],
            year_present,
            function,
        )?;
        resolved_year.year_present_local().load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINMONTHDAY_PROTOTYPE_TOPLAINDATE_REQUIRES_A_YEAR,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        overflow.store(function);
        function.instruction(&Instruction::I64Const(1));
        month_code_present.store(function);
        function.instruction(&Instruction::I64Const(0));
        month_present.store(function);
        function.instruction(&Instruction::I64Const(0));
        encoded_month_code.store(function);
        let resolved_month = self.emit_temporal_resolve_calendar_month(
            resolved_year,
            fields[1],
            month_present,
            &acquired_month_code,
            encoded_month_code,
            month_code_present,
            TemporalMonthFieldContext::PlainMonthDayToPlainDate,
            function,
        )?;
        self.emit_temporal_calendar_date_to_iso(resolved_month, fields[2], overflow, function)?;
        self.emit_temporal_reject_iso_date(fields[0], fields[1], fields[2], function)?;
        self.emit_alloc_temporal_plain_date(
            fields[0],
            fields[1],
            fields[2],
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        for local in [
            month_present,
            month_code_present,
            encoded_month_code,
            overflow,
            year_present,
        ] {
            schema.release_i64_local(local, function);
        }
        acquired_month_code.clear(function);
        argument.clear(function);
        calendar_value.clear(function);
        for local in fields.into_iter().rev() {
            schema.release_i64_local(local, function);
        }
        record.clear(function);
        Ok(())
    }
}
