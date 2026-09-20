//! Compiled ECMA-402 observation and Realm semantics for DateTimeFormat.
//! The pinned provider selects patterns and formats primitive exact input records.

use super::super::*;
use crate::functions::NewTargetPrototypeFallback;
use crate::objects::TaggedLocals;
use lila_intl::{
    DateTimeCalendar, DateTimeHourCycle, DateTimeMonthWidth, DateTimeNumericWidth,
    DateTimePartKind, DateTimeRangeSource, DateTimeStyle, DateTimeTextWidth, DateTimeValueKind,
    FixedTimeZoneOffset, TimeZoneKind, TimeZoneNameStyle,
};

mod construction_lifecycle;
mod initialization;
mod provider_input;
mod provider_render;
mod provider_wire;
mod time_zone;
pub(crate) use initialization::IntlDateTimeFormatPurpose;
use provider_input::INTL_DTF_CALENDAR_MISMATCH;

struct IntlDtfOption {
    property: &'static str,
    slot_offset: u64,
    codes: &'static [(&'static str, i64)],
}

/// ECMA-402 11.5 Table 7, in table order. The constructor reads these after
/// `timeZone` and before `formatMatcher`, and the order here is what the
/// `constructor-options-order*.js` tests observe through getters.
///
/// `fractionalSecondDigits` is absent because it is a *number* option; the
/// constructor splices it in at its table position explicitly.
const INTL_DTF_COMPONENT_OPTIONS: &[IntlDtfOption] = &[
    IntlDtfOption {
        property: "weekday",
        slot_offset: HEAP_INTL_DTF_WEEKDAY_OFFSET,
        codes: DateTimeTextWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "era",
        slot_offset: HEAP_INTL_DTF_ERA_OFFSET,
        codes: DateTimeTextWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "year",
        slot_offset: HEAP_INTL_DTF_YEAR_OFFSET,
        codes: DateTimeNumericWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "month",
        slot_offset: HEAP_INTL_DTF_MONTH_OFFSET,
        codes: DateTimeMonthWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "day",
        slot_offset: HEAP_INTL_DTF_DAY_OFFSET,
        codes: DateTimeNumericWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "dayPeriod",
        slot_offset: HEAP_INTL_DTF_DAY_PERIOD_OFFSET,
        codes: DateTimeTextWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "hour",
        slot_offset: HEAP_INTL_DTF_HOUR_OFFSET,
        codes: DateTimeNumericWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "minute",
        slot_offset: HEAP_INTL_DTF_MINUTE_OFFSET,
        codes: DateTimeNumericWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "second",
        slot_offset: HEAP_INTL_DTF_SECOND_OFFSET,
        codes: DateTimeNumericWidth::OPTIONS,
    },
    IntlDtfOption {
        property: "timeZoneName",
        slot_offset: HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET,
        // Shared with [`TimeZoneNameStyle::code`]: the constructor's accepted
        // spellings and the renderer's understood codes are one list.
        codes: INTL_DTF_TIME_ZONE_NAME_CODES,
    },
];

/// The
/// `fractionalSecondDigits` number option is read immediately after `second`,
/// which is the entry before `timeZoneName`.
const INTL_DTF_FRACTIONAL_SECOND_DIGITS_AFTER: &str = "second";

const INTL_DTF_HOUR_CYCLE_OPTION: IntlDtfOption = IntlDtfOption {
    property: "hourCycle",
    slot_offset: HEAP_INTL_DTF_HOUR_CYCLE_OFFSET,
    codes: DateTimeHourCycle::OPTIONS,
};

const INTL_DTF_DATE_STYLE_OPTION: IntlDtfOption = IntlDtfOption {
    property: "dateStyle",
    slot_offset: HEAP_INTL_DTF_DATE_STYLE_OFFSET,
    codes: DateTimeStyle::OPTIONS,
};

const INTL_DTF_TIME_STYLE_OPTION: IntlDtfOption = IntlDtfOption {
    property: "timeStyle",
    slot_offset: HEAP_INTL_DTF_TIME_STYLE_OFFSET,
    codes: DateTimeStyle::OPTIONS,
};

/// Selected component slots in the provider wire order.
const INTL_DTF_FORMAT_COMPONENT_SLOTS: [u64; 11] = [
    HEAP_INTL_DTF_WEEKDAY_OFFSET,
    HEAP_INTL_DTF_ERA_OFFSET,
    HEAP_INTL_DTF_YEAR_OFFSET,
    HEAP_INTL_DTF_MONTH_OFFSET,
    HEAP_INTL_DTF_DAY_OFFSET,
    HEAP_INTL_DTF_DAY_PERIOD_OFFSET,
    HEAP_INTL_DTF_HOUR_OFFSET,
    HEAP_INTL_DTF_MINUTE_OFFSET,
    HEAP_INTL_DTF_SECOND_OFFSET,
    HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
    HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET,
];

struct IntlDtfWellFormedTypeValue {
    lowered_local: u32,
}
const INTL_DTF_TIME_ZONE_NAME_CODES: &[(&str, i64)] = &TimeZoneNameStyle::OPTIONS;
const INTL_DTF_OFFSET_SIGNS: [&str; 2] = ["+", "-"];
const INTL_DTF_RESOLVED_TIME_ZONE: &str = "UTC";
const INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE: &str = "Unsupported timeZone option";
const INTL_DTF_ZONED_DATE_TIME_UNSUPPORTED: &str =
    "Intl.DateTimeFormat does not support Temporal.ZonedDateTime";
const INTL_DTF_EMPTY_TEMPORAL_FORMAT: &str =
    "The requested format has no fields in common with this Temporal type";
const INTL_DTF_RANGE_DIFFERENT_TYPES_MESSAGE: &str =
    "Intl.DateTimeFormat.prototype.formatRange startDate and endDate must be the same type";
const INTL_DTF_RANGE_UNDEFINED_MESSAGE: &str =
    "Intl.DateTimeFormat.prototype.formatRange startDate and endDate must be defined";

#[derive(Clone, Copy)]
enum DtfFormatMode {
    String,
    Parts,
}

#[derive(Clone, Copy)]
pub(crate) enum DtfTemporalKind {
    PlainDate,
    PlainYearMonth,
    PlainMonthDay,
    PlainTime,
    PlainDateTime,
    Instant,
}
impl DtfTemporalKind {
    const fn code(self) -> i64 {
        match self {
            Self::PlainDate => 1,
            Self::PlainYearMonth => 2,
            Self::PlainMonthDay => 3,
            Self::PlainTime => 4,
            Self::PlainDateTime => 5,
            Self::Instant => 6,
        }
    }
    const fn brand(self) -> u64 {
        match self {
            Self::PlainDate => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE,
            Self::PlainYearMonth => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH,
            Self::PlainMonthDay => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY,
            Self::PlainTime => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_TIME,
            Self::PlainDateTime => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME,
            Self::Instant => OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT,
        }
    }
    const fn value_kind(self) -> DateTimeValueKind {
        match self {
            Self::PlainDate => DateTimeValueKind::PlainDate,
            Self::PlainYearMonth => DateTimeValueKind::PlainYearMonth,
            Self::PlainMonthDay => DateTimeValueKind::PlainMonthDay,
            Self::PlainTime => DateTimeValueKind::PlainTime,
            Self::PlainDateTime => DateTimeValueKind::PlainDateTime,
            Self::Instant => DateTimeValueKind::Instant,
        }
    }
    const fn type_name(self) -> &'static str {
        match self {
            Self::PlainDate => "Temporal.PlainDate",
            Self::PlainYearMonth => "Temporal.PlainYearMonth",
            Self::PlainMonthDay => "Temporal.PlainMonthDay",
            Self::PlainTime => "Temporal.PlainTime",
            Self::PlainDateTime => "Temporal.PlainDateTime",
            Self::Instant => "Temporal.Instant",
        }
    }
    const fn rejected_style(self) -> Option<(&'static str, u64)> {
        match self {
            Self::PlainDate | Self::PlainYearMonth | Self::PlainMonthDay => {
                Some(("timeStyle", HEAP_INTL_DTF_TIME_STYLE_OFFSET))
            }
            Self::PlainTime => Some(("dateStyle", HEAP_INTL_DTF_DATE_STYLE_OFFSET)),
            Self::PlainDateTime | Self::Instant => None,
        }
    }
}
const INTL_DTF_TEMPORAL_KINDS: &[DtfTemporalKind] = &[
    DtfTemporalKind::PlainDate,
    DtfTemporalKind::PlainYearMonth,
    DtfTemporalKind::PlainMonthDay,
    DtfTemporalKind::PlainTime,
    DtfTemporalKind::PlainDateTime,
    DtfTemporalKind::Instant,
];
#[derive(Clone, Copy)]
enum DtfBrandedKind {
    Temporal(DtfTemporalKind),
    ZonedDateTime,
}
impl DtfBrandedKind {
    fn all() -> impl Iterator<Item = Self> {
        INTL_DTF_TEMPORAL_KINDS
            .iter()
            .copied()
            .map(Self::Temporal)
            .chain([Self::ZonedDateTime])
    }
    const fn brand(self) -> u64 {
        match self {
            Self::Temporal(kind) => kind.brand(),
            Self::ZonedDateTime => OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME,
        }
    }
}
#[derive(Clone, Copy)]
enum DtfValueKind {
    Legacy,
    Branded(DtfBrandedKind),
}
impl DtfValueKind {
    const fn code(self) -> i64 {
        match self {
            Self::Legacy => 0,
            Self::Branded(DtfBrandedKind::Temporal(kind)) => kind.code(),
            Self::Branded(DtfBrandedKind::ZonedDateTime) => 7,
        }
    }
}

enum IntlDateTimeFormatReceiverOperation {
    ResolvedOptions,
    FormatGetter,
    FormatToParts,
    FormatRange,
    FormatRangeToParts,
}

impl IntlDateTimeFormatReceiverOperation {
    const ALL: [Self; 5] = [
        Self::ResolvedOptions,
        Self::FormatGetter,
        Self::FormatToParts,
        Self::FormatRange,
        Self::FormatRangeToParts,
    ];

    const fn full_message(&self) -> &'static str {
        match self {
            Self::ResolvedOptions => {
                "Intl.DateTimeFormat.prototype.resolvedOptions called on a non-Intl.DateTimeFormat object"
            }
            Self::FormatGetter => {
                "get Intl.DateTimeFormat.prototype.format called on a non-Intl.DateTimeFormat object"
            }
            Self::FormatToParts => {
                "Intl.DateTimeFormat.prototype.formatToParts called on a non-Intl.DateTimeFormat object"
            }
            Self::FormatRange => {
                "Intl.DateTimeFormat.prototype.formatRange called on a non-Intl.DateTimeFormat object"
            }
            Self::FormatRangeToParts => {
                "Intl.DateTimeFormat.prototype.formatRangeToParts called on a non-Intl.DateTimeFormat object"
            }
        }
    }
}

struct DtfCanonicalTimeZone {
    identifier_local: u32,
    fixed_seconds_local: u32,
    kind_local: u32,
}

impl DtfCanonicalTimeZone {
    fn reserve(builder: &mut FunctionBuilder<'_>) -> Self {
        Self {
            identifier_local: builder.reserve_temp_local(),
            fixed_seconds_local: builder.reserve_temp_local(),
            kind_local: builder.reserve_temp_local(),
        }
    }
}

struct DtfResolvedTimeZone(DtfCanonicalTimeZone);

impl DtfResolvedTimeZone {
    fn store(&self, builder: &FunctionBuilder<'_>, record_local: u32, function: &mut Function) {
        for (offset, local) in [
            (HEAP_INTL_DTF_TIME_ZONE_OFFSET, self.0.identifier_local),
            (
                HEAP_INTL_DTF_TIME_ZONE_FIXED_SECONDS_OFFSET,
                self.0.fixed_seconds_local,
            ),
            (HEAP_INTL_DTF_TIME_ZONE_KIND_OFFSET, self.0.kind_local),
        ] {
            builder.store_i64_local_at_offset(record_local, offset, local, function);
        }
    }

    fn release(self, builder: &mut FunctionBuilder<'_>) {
        builder.release_temp_local(self.0.kind_local);
        builder.release_temp_local(self.0.fixed_seconds_local);
        builder.release_temp_local(self.0.identifier_local);
    }
}

impl FunctionBuilder<'_> {
    fn emit_dtf_ascii_number_string(
        &mut self,
        number_local: u32,
        width: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_dtf_set_string(dest_local, "", function);
        self.emit_date_append_padded_decimal(dest_local, number_local, width, function)
    }
    fn emit_dtf_set_const(&self, local: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::LocalSet(local));
    }

    fn emit_dtf_set_string(&mut self, local: u32, value: &str, function: &mut Function) {
        let payload = self.strings.payload(value);
        function.instruction(&Instruction::I64Const(payload));
        function.instruction(&Instruction::LocalSet(local));
    }

    fn emit_intl_dtf_record_from_receiver(
        &mut self,
        record_local: u32,
        operation: &IntlDateTimeFormatReceiverOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: Intl.DateTimeFormat method without receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: Intl.DateTimeFormat method without receiver tag",
            )
        })?;
        let brand_local = self.reserve_temp_local();
        let message = operation.full_message();

        self.emit_dtf_set_const(record_local, 0, function);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_INTL_DATE_TIME_FORMAT as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(record_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(brand_local);
        Ok(())
    }

    fn emit_intl_dtf_string_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        option: &IntlDtfOption,
        dest_local: u32,
        present_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();
        let recognized_local = self.reserve_temp_local();
        let message = format!("Invalid {} option", option.property);

        self.emit_dtf_set_const(dest_local, 0, function);
        if let Some(present_local) = present_local {
            self.emit_dtf_set_const(present_local, 0, function);
        }
        self.emit_dtf_set_string(key_local, option.property, function);
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(present_local) = present_local {
            self.emit_dtf_set_const(present_local, 1, function);
        }
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_dtf_set_const(recognized_local, 0, function);
        for (spelling, code) in option.codes {
            self.emit_dtf_set_string(expected_local, spelling, function);
            self.emit_string_payload_equality_i32(value_payload_local, expected_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_const(recognized_local, 1, function);
            self.emit_dtf_set_const(dest_local, *code, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(recognized_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            &message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            recognized_local,
            expected_local,
            value_tag_local,
            value_payload_local,
            key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_intl_dtf_type_nonterminal_guard(
        &mut self,
        value_payload_local: u32,
        ok_local: u32,
        lowered_local: u32,
        range_message: &str,
        function: &mut Function,
    ) -> Result<IntlDtfWellFormedTypeValue, EmitError> {
        self.emit_intl_dtf_is_unicode_type_i32(value_payload_local, ok_local, function);
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            range_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_intl_dtf_ascii_lowercase(value_payload_local, lowered_local, function)?;
        Ok(IntlDtfWellFormedTypeValue { lowered_local })
    }

    fn emit_intl_dtf_is_unicode_type_i32(
        &mut self,
        payload_local: u32,
        ok_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let run_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(payload_local, offset_local, length_local, function);
        self.emit_dtf_set_const(ok_local, 1, function);
        self.emit_dtf_set_const(index_local, 0, function);
        self.emit_dtf_set_const(run_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // A separator closes a run, which must have been 3..=8 long.
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        self.emit_dtf_set_const(run_local, 0, function);
        function.instruction(&Instruction::Else);
        self.emit_intl_dtf_is_alphanum_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(ok_local, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(run_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        // The final run has no separator to close it.
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::LocalGet(run_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(ok_local, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            byte_local,
            run_local,
            index_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
    }

    fn emit_intl_dtf_is_alphanum_i32(&self, byte_local: u32, function: &mut Function) {
        for (low, high) in [('0', '9'), ('A', 'Z'), ('a', 'z')] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(low as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(high as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
        }
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_intl_dtf_fractional_second_digits_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        dest_local: u32,
        present_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_dtf_set_const(dest_local, 0, function);
        self.emit_dtf_set_const(present_local, 0, function);
        self.emit_dtf_set_string(key_local, "fractionalSecondDigits", function);
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(present_local, 1, function);
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        // NaN, or outside 1..=3 after truncation, is a RangeError.
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(3.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "fractionalSecondDigits must be between 1 and 3",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::I64TruncF64S);
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::End);

        for local in [value_tag_local, value_payload_local, key_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_intl_dtf_time_zone_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        zone: DtfCanonicalTimeZone,
        function: &mut Function,
    ) -> Result<DtfResolvedTimeZone, EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let parsed_local = self.reserve_temp_local();
        let minutes_local = self.reserve_temp_local();

        self.emit_dtf_set_string(zone.identifier_local, INTL_DTF_RESOLVED_TIME_ZONE, function);
        self.emit_dtf_set_const(zone.fixed_seconds_local, 0, function);
        self.emit_dtf_set_const(zone.kind_local, TimeZoneKind::Named.code(), function);
        self.emit_dtf_set_string(key_local, "timeZone", function);
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_intl_dtf_parse_utc_offset(
            value_payload_local,
            minutes_local,
            parsed_local,
            function,
        );
        self.emit_dtf_if_nonzero(parsed_local, function);
        self.emit_dtf_set_const(zone.kind_local, TimeZoneKind::FixedOffset.code(), function);
        function.instruction(&Instruction::LocalGet(minutes_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(zone.fixed_seconds_local));
        self.emit_intl_dtf_format_offset_identifier(
            minutes_local,
            zone.identifier_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(zone.identifier_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_dtf_if_code_eq(zone.kind_local, TimeZoneKind::Named.code(), function);
        self.emit_intl_dtf_lookup_named_time_zone(zone.identifier_local, function)?;
        function.instruction(&Instruction::End);
        for local in [
            minutes_local,
            parsed_local,
            value_tag_local,
            value_payload_local,
            key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(DtfResolvedTimeZone(zone))
    }

    fn emit_intl_dtf_parse_utc_offset(
        &mut self,
        payload_local: u32,
        minutes_local: u32,
        ok_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let minute_start_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(payload_local, offset_local, length_local, function);
        self.emit_dtf_set_const(ok_local, 0, function);
        self.emit_dtf_set_const(minutes_local, 0, function);
        self.emit_dtf_set_const(hour_local, 0, function);
        self.emit_dtf_set_const(minute_local, 0, function);
        self.emit_dtf_set_const(minute_start_local, 0, function);

        // Every rejection is a `Br(0)` out of this block, so the accepting tail
        // is the only path that reaches `ok = 1`.
        function.instruction(&Instruction::Block(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));

        self.emit_dtf_set_const(index_local, 0, function);
        self.emit_load_string_byte(offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(sign_local, -1, function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_set_const(sign_local, 1, function);
        function.instruction(&Instruction::End);

        for digit_index in [1_i64, 2] {
            self.emit_dtf_set_const(index_local, digit_index, function);
            self.emit_load_string_byte(offset_local, index_local, byte_local, function);
            self.emit_intl_dtf_reject_unless_ascii_digit(byte_local, 0, function);
            function.instruction(&Instruction::LocalGet(hour_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const('0' as i64));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(hour_local));
        }
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(FixedTimeZoneOffset::MAX_HOUR));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(0));

        // Where the two minute digits begin: 3 for `+HHMM`, 4 for `+HH:MM`, and
        // 0 for `+HH`, which has none. A six-byte string whose byte 3 is not
        // `':'` — `'-10.50'`, `'+13234'` — leaves it 0 and is rejected below.
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(5));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(minute_start_local, 3, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(index_local, 3, function);
        self.emit_load_string_byte(offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(minute_start_local, 4, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(minute_start_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::BrIf(0));

        self.emit_dtf_if_nonzero(minute_start_local, function);
        for digit_index in [0_i64, 1] {
            function.instruction(&Instruction::LocalGet(minute_start_local));
            function.instruction(&Instruction::I64Const(digit_index));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index_local));
            self.emit_load_string_byte(offset_local, index_local, byte_local, function);
            self.emit_intl_dtf_reject_unless_ascii_digit(byte_local, 1, function);
            function.instruction(&Instruction::LocalGet(minute_local));
            function.instruction(&Instruction::I64Const(10));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const('0' as i64));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(minute_local));
        }
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Const(FixedTimeZoneOffset::MAX_MINUTE));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(minute_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(minutes_local));
        self.emit_dtf_set_const(ok_local, 1, function);
        function.instruction(&Instruction::End);

        for local in [
            minute_start_local,
            minute_local,
            hour_local,
            sign_local,
            byte_local,
            index_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
    }

    fn emit_intl_dtf_reject_unless_ascii_digit(
        &self,
        byte_local: u32,
        depth: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('0' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(depth));
    }

    fn emit_intl_dtf_format_offset_identifier(
        &mut self,
        minutes_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let magnitude_local = self.reserve_temp_local();
        let piece_local = self.reserve_temp_local();
        let field_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(minutes_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_string(dest_local, INTL_DTF_OFFSET_SIGNS[1], function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalGet(minutes_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(magnitude_local));
        function.instruction(&Instruction::Else);
        self.emit_dtf_set_string(dest_local, INTL_DTF_OFFSET_SIGNS[0], function);
        function.instruction(&Instruction::LocalGet(minutes_local));
        function.instruction(&Instruction::LocalSet(magnitude_local));
        function.instruction(&Instruction::End);

        for (index, separator) in [(0_usize, None), (1, Some(":"))] {
            if let Some(separator) = separator {
                self.emit_dtf_set_string(piece_local, separator, function);
                self.emit_concat_string_payloads_local(dest_local, piece_local, function)?;
                function.instruction(&Instruction::LocalSet(dest_local));
            }
            function.instruction(&Instruction::LocalGet(magnitude_local));
            function.instruction(&Instruction::I64Const(60));
            if index == 0 {
                function.instruction(&Instruction::I64DivU);
            } else {
                function.instruction(&Instruction::I64RemU);
            }
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(field_local));
            self.emit_dtf_ascii_number_string(field_local, 2, piece_local, function)?;
            self.emit_concat_string_payloads_local(dest_local, piece_local, function)?;
            function.instruction(&Instruction::LocalSet(dest_local));
        }

        for local in [field_local, piece_local, magnitude_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_intl_dtf_ascii_lowercase(
        &mut self,
        payload_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(payload_local, offset_local, length_local, function);
        self.emit_heap_alloc_from_local(length_local, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.emit_dtf_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('A' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const('Z' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(buffer_local, length_local, function);
        function.instruction(&Instruction::LocalSet(dest_local));

        for local in [
            byte_local,
            index_local,
            buffer_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_intl_dtf_hour12_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_dtf_set_const(dest_local, 0, function);
        self.emit_dtf_set_string(key_local, "hour12", function);
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_to_boolean_payload_from_tagged_locals(
            value_tag_local,
            value_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::End);

        for local in [value_tag_local, value_payload_local, key_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn emit_intl_date_time_format_resolved_options(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let code_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();

        self.emit_intl_dtf_record_from_receiver(
            record_local,
            &IntlDateTimeFormatReceiverOperation::ResolvedOptions,
            function,
        )?;
        self.emit_dtf_result_object(function)?;
        function.instruction(&Instruction::LocalSet(object_local));

        for (name, offset) in [
            ("locale", HEAP_INTL_DTF_LOCALE_OFFSET),
            ("calendar", HEAP_INTL_DTF_CALENDAR_OFFSET),
            ("numberingSystem", HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET),
            ("timeZone", HEAP_INTL_DTF_TIME_ZONE_OFFSET),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, payload_local, function);
            self.emit_dtf_set_string(key_local, name, function);
            self.emit_dtf_set_const(tag_local, ValueKind::String.tag() as i64, function);
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                true,
                true,
                true,
                function,
            )?;
        }

        // `hourCycle` and `hour12` exist only when the resolved pattern has an
        // hour field: an explicit `hour`, or any `timeStyle`.
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_HOUR_OFFSET,
            hour_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_TIME_STYLE_OFFSET,
            code_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(hour_local));
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_HOUR_CYCLE_OFFSET,
            code_local,
            function,
        );
        self.emit_intl_dtf_code_to_string(
            &INTL_DTF_HOUR_CYCLE_OPTION,
            code_local,
            payload_local,
            function,
        );
        self.emit_dtf_set_string(key_local, "hourCycle", function);
        self.emit_dtf_set_const(tag_local, ValueKind::String.tag() as i64, function);
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            true,
            true,
            function,
        )?;
        // hour12 is true exactly for the h11 and h12 cycles.
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_dtf_set_string(key_local, "hour12", function);
        self.emit_dtf_set_const(tag_local, ValueKind::Boolean.tag() as i64, function);
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            true,
            true,
            true,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_DATE_STYLE_OFFSET,
            code_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_TIME_STYLE_OFFSET,
            hour_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Table 7 components, each present only when its code is nonzero;
        // `fractionalSecondDigits` is spliced in at its table position.
        for option in INTL_DTF_COMPONENT_OPTIONS {
            self.load_i64_to_local_from_offset(
                record_local,
                option.slot_offset,
                code_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(code_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_intl_dtf_code_to_string(option, code_local, payload_local, function);
            self.emit_dtf_set_string(key_local, option.property, function);
            self.emit_dtf_set_const(tag_local, ValueKind::String.tag() as i64, function);
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                true,
                true,
                true,
                function,
            )?;
            function.instruction(&Instruction::End);
            if option.property == INTL_DTF_FRACTIONAL_SECOND_DIGITS_AFTER {
                self.load_i64_to_local_from_offset(
                    record_local,
                    HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
                    code_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(code_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(code_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(payload_local));
                self.emit_dtf_set_string(key_local, "fractionalSecondDigits", function);
                self.emit_dtf_set_const(tag_local, ValueKind::Number.tag() as i64, function);
                self.emit_object_append_data_property_with_flags(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    true,
                    true,
                    true,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
        }

        function.instruction(&Instruction::End);

        for option in [&INTL_DTF_DATE_STYLE_OPTION, &INTL_DTF_TIME_STYLE_OPTION] {
            self.load_i64_to_local_from_offset(
                record_local,
                option.slot_offset,
                code_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(code_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_intl_dtf_code_to_string(option, code_local, payload_local, function);
            self.emit_dtf_set_string(key_local, option.property, function);
            self.emit_dtf_set_const(tag_local, ValueKind::String.tag() as i64, function);
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                true,
                true,
                true,
                function,
            )?;
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            hour_local,
            code_local,
            tag_local,
            payload_local,
            key_local,
            object_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_intl_dtf_code_to_string(
        &mut self,
        option: &IntlDtfOption,
        code_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        self.emit_dtf_set_const(dest_local, 0, function);
        for (spelling, code) in option.codes {
            function.instruction(&Instruction::LocalGet(code_local));
            function.instruction(&Instruction::I64Const(*code));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_string(dest_local, spelling, function);
            function.instruction(&Instruction::End);
        }
    }

    pub(crate) fn emit_intl_date_time_format_format_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let bound_local = self.reserve_temp_local();
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: format getter without receiver",
            )
        })?;

        self.emit_intl_dtf_record_from_receiver(
            record_local,
            &IntlDateTimeFormatReceiverOperation::FormatGetter,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_BOUND_FORMAT_OFFSET,
            bound_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(bound_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let meta = self
            .functions
            .get(&StandardBuiltinId::IntlDateTimeFormatBoundFormat.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Intl.DateTimeFormat Format Function`",
                )
            })?;
        self.emit_current_builtin_realm_closure_value(
            &meta,
            this_payload_local,
            bound_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_BOUND_FORMAT_OFFSET,
            bound_local,
            function,
        );
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(bound_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(bound_local);
        self.release_temp_local(record_local);
        Ok(())
    }

    fn emit_dtf_if_code_eq(&self, local: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
    }

    fn emit_dtf_if_nonzero(&self, local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
    }

    pub(crate) fn emit_intl_dtf_temporal_to_locale_string(
        &mut self,
        brand: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = INTL_DTF_TEMPORAL_KINDS
            .iter()
            .find(|kind| kind.brand() == brand)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: Intl.DateTimeFormat has no field set for this Temporal brand",
                )
            })?;
        let this_payload_local = self.reserve_temp_local();
        let this_tag_local = self.reserve_temp_local();
        let dtf_payload_local = self.reserve_temp_local();
        let dtf_tag_local = self.reserve_temp_local();
        let format_payload_local = self.reserve_temp_local();
        let format_tag_local = self.reserve_temp_local();

        self.compile_this_to_locals(this_payload_local, this_tag_local, function)?;

        self.emit_intl_create_date_time_format(
            IntlDateTimeFormatPurpose::TemporalPlain(*kind),
            function,
        )?;
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(dtf_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(dtf_tag_local));

        let format_getter_meta = self
            .functions
            .get(&StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `get Intl.DateTimeFormat.prototype.format`",
                )
            })?;
        self.emit_direct_js_call(
            &format_getter_meta,
            Some((dtf_payload_local, Some(dtf_tag_local))),
            &[],
            format_payload_local,
            format_tag_local,
            function,
        )?;
        self.emit_function_handle_call(
            format_payload_local,
            format_tag_local,
            None,
            &[(this_payload_local, this_tag_local)],
            self.result_local,
            self.result_tag_local,
            function,
        )?;

        for local in [
            format_tag_local,
            format_payload_local,
            dtf_tag_local,
            dtf_payload_local,
            this_tag_local,
            this_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}

fn intl_dtf_temporal_style_message(type_name: &str, property: &str) -> String {
    format!("{type_name}.prototype.toLocaleString does not support the {property} option")
}

pub(crate) fn intl_date_time_format_pool_strings() -> Vec<String> {
    let mut values = Vec::new();
    for value in [
        "Intl.DateTimeFormat",
        "DateTimeFormat",
        "supportedLocalesOf",
        "resolvedOptions",
        "format",
        "formatToParts",
        "formatRange",
        "formatRangeToParts",
        "type",
        "value",
        "source",
        "locale",
        "hour12",
        "fractionalSecondDigits",
        "localeMatcher",
        "formatMatcher",
        "lookup",
        "best fit",
        "basic",
        "timeZone",
        "numberingSystem",
        "calendar",
        "",
        ":",
        "0",
        "Intl.DateTimeFormat constructor requires new",
        "dateStyle and timeStyle may not be used with explicit date-time components",
        "fractionalSecondDigits must be between 1 and 3",
        "Date value is not finite",
        "Date.prototype.toLocaleDateString does not support the timeStyle option",
        "Date.prototype.toLocaleTimeString does not support the dateStyle option",
        "Invalid language tag",
        INTL_DTF_RESOLVED_TIME_ZONE,
        INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE,
        INTL_DTF_RANGE_UNDEFINED_MESSAGE,
        INTL_DTF_RANGE_DIFFERENT_TYPES_MESSAGE,
        INTL_DTF_ZONED_DATE_TIME_UNSUPPORTED,
        INTL_DTF_EMPTY_TEMPORAL_FORMAT,
        INTL_DTF_CALENDAR_MISMATCH,
    ] {
        values.push(value.to_owned());
    }
    for calendar in DateTimeCalendar::ALL {
        values.push(calendar.as_str().to_owned());
    }
    for part in DateTimePartKind::ALL {
        values.push(part.as_str().to_owned());
    }
    for source in DateTimeRangeSource::ALL {
        values.push(source.as_str().to_owned());
    }
    for option in INTL_DTF_COMPONENT_OPTIONS.iter().chain([
        &INTL_DTF_HOUR_CYCLE_OPTION,
        &INTL_DTF_DATE_STYLE_OPTION,
        &INTL_DTF_TIME_STYLE_OPTION,
    ]) {
        values.push(option.property.to_owned());
        values.push(format!("Invalid {} option", option.property));
        values.extend(option.codes.iter().map(|(name, _)| (*name).to_owned()));
    }
    for property in [
        "calendar",
        "numberingSystem",
        "localeMatcher",
        "formatMatcher",
    ] {
        values.push(format!("Invalid {property} option"));
    }
    for sign in INTL_DTF_OFFSET_SIGNS {
        values.push(sign.to_owned());
    }
    for operation in IntlDateTimeFormatReceiverOperation::ALL {
        values.push(operation.full_message().to_owned());
    }
    for kind in INTL_DTF_TEMPORAL_KINDS {
        if let Some((property, _)) = kind.rejected_style() {
            values.push(intl_dtf_temporal_style_message(kind.type_name(), property));
        }
    }
    values
}
