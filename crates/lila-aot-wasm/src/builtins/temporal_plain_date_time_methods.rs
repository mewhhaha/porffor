//! `Temporal.PlainDateTime` statics and prototype methods.
//!
//! Split from `temporal_plain_date_time.rs` (record, constructor, accessors) so
//! the two halves stay readable; both are `impl FunctionBuilder` blocks.
//!
//! Almost everything here is composition rather than new arithmetic: the date
//! half reuses `Temporal.PlainDate`'s `RegulateISODate`/`CalendarResolveFields`
//! and its ISO date formatter, the time half reuses `Temporal.PlainTime`'s
//! `RegulateTime`, nanosecond-of-day scalar and time formatter, and every
//! option read goes through the `Temporal.Duration` option plumbing so the four
//! types agree on what `halfEven` and `smallestUnit` mean. Arithmetic here
//! includes the Gregorian add primitive's epoch-day round trip.
//! Calendar-dependent add/difference callers consume the retained calendar
//! through the shared arithmetic authority.

use super::super::*;
use super::temporal::TemporalTimeZoneStringGoal;
use super::temporal_options::{
    ShowCalendarName, TemporalConversionOverflowOptions, TemporalOverflow, TemporalRoundingMode,
    TemporalUnit, TemporalUnitOptionProperty, TemporalUnitSlot,
};
use super::temporal_plain_date::TemporalEraLocals;
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::temporal_plain_time_methods::TEMPORAL_PRECISION_AUTO;
use super::temporal_zone_provider::TemporalCalendarSlotLocals;
use super::temporal_zone_provider::TemporalZonedAllocationInput;
use crate::gc_types::*;
use crate::intrinsics::temporal::TemporalPrototypeSource;

/// Which `add` or `subtract` operation a plain Temporal builtin emits.
///
/// The four plain Temporal types share this domain so every standard-builtin
/// producer must name the direction and every arithmetic emitter must handle
/// both directions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TemporalPlainArithmeticOperation {
    Add,
    Subtract,
}

/// Which `until` or `since` operation a plain Temporal builtin emits.
///
/// The four plain Temporal types share this domain because the operation owns
/// both rounding-mode negation and final-result negation. Naming it once keeps
/// those two choices from being transposed independently.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TemporalPlainDifferenceOperation {
    Until,
    Since,
}

/// Receiver kind and direction jointly select the default unit and rounding
/// mode. Each entry reads options once before calling shared arithmetic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TemporalDateTimeDifferenceSettingsPlan {
    PlainUntil,
    PlainSince,
    ZonedUntil,
    ZonedSince,
    InstantUntil,
    InstantSince,
}

impl TemporalDateTimeDifferenceSettingsPlan {
    const fn fallback_largest_unit(self) -> TemporalUnit {
        match self {
            Self::PlainUntil | Self::PlainSince => TemporalUnit::Day,
            Self::ZonedUntil | Self::ZonedSince => TemporalUnit::Hour,
            Self::InstantUntil | Self::InstantSince => TemporalUnit::Second,
        }
    }

    const fn largest_allowed_unit(self) -> TemporalUnit {
        match self {
            Self::PlainUntil | Self::PlainSince | Self::ZonedUntil | Self::ZonedSince => {
                TemporalUnit::Year
            }
            Self::InstantUntil | Self::InstantSince => TemporalUnit::Hour,
        }
    }

    const fn invalid_unit_message(self) -> RuntimeErrorMessage {
        match self {
            Self::PlainUntil | Self::PlainSince | Self::ZonedUntil | Self::ZonedSince => {
                RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATETIME_UNIT_OPTION
            }
            Self::InstantUntil | Self::InstantSince => {
                RuntimeErrorMessage::INVALID_TEMPORAL_INSTANT_UNIT_OPTION
            }
        }
    }

    const fn negates_rounding_mode(self) -> bool {
        match self {
            Self::PlainUntil | Self::ZonedUntil | Self::InstantUntil => false,
            Self::PlainSince | Self::ZonedSince | Self::InstantSince => true,
        }
    }
}

/// The completed `GetDifferenceSettings` read. The four locals move together:
/// arithmetic may not consume a largest unit resolved under one fallback and a
/// rounding mode resolved under another operation.
///
/// The entry owns these locals until the arithmetic consumer has finished;
/// the user's option getters are never read a second time.
#[must_use = "resolved Temporal difference settings must be consumed"]
pub(super) struct ResolvedTemporalDateTimeDifferenceSettings {
    largest_unit_local: I64Local,
    smallest_unit_local: I64Local,
    increment_local: I64Local,
    mode_local: I64Local,
}

impl ResolvedTemporalDateTimeDifferenceSettings {
    pub(super) fn largest_unit(&self) -> I64Local {
        self.largest_unit_local
    }
    pub(super) fn smallest_unit(&self) -> I64Local {
        self.smallest_unit_local
    }
    pub(super) fn rounding_increment(&self) -> I64Local {
        self.increment_local
    }
    pub(super) fn rounding_mode(&self) -> I64Local {
        self.mode_local
    }
    pub(super) fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in [
            self.mode_local,
            self.increment_local,
            self.smallest_unit_local,
            self.largest_unit_local,
        ] {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}

/// One row of the combined `PrepareCalendarFields` / `ToTemporalTimeRecord`
/// sweep for a `Temporal.PlainDateTime` property bag, in the alphabetical order
/// the reads are observable in.
///
/// This replaces a `(&str, usize)` table whose `monthCode` row carried
/// `usize::MAX` as "not one of the nine numeric slots". Two more keys with no
/// slot of their own (`era`, `eraYear`, which fold into `year`) would have
/// meant two more magic values, each interpreted by an `if` at the consuming
/// site. A closed enum matched exhaustively puts that interpretation in one
/// place, and makes "you added a key and did not say how it is read" a compile
/// error.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TemporalDateTimeFieldKey {
    Day,
    EraPair,
    Hour,
    Microsecond,
    Millisecond,
    Minute,
    Month,
    MonthCode,
    Nanosecond,
    Offset,
    Second,
    Year,
}

/// How a [`TemporalDateTimeFieldKey`] is read, and where its value lands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TemporalDateTimeFieldRead {
    /// `ToPositiveIntegerWithTruncation` into `field_locals[index]`. Only the
    /// two calendar rows take it; `hour` .. `nanosecond` accept zero.
    PositiveInteger {
        property: &'static str,
        index: usize,
    },
    /// `ToIntegerWithTruncation` into `field_locals[index]`.
    Integer {
        property: &'static str,
        index: usize,
    },
    /// The `monthCode` string, which has no numeric slot.
    MonthCode,
    Offset,
    /// `era` and `eraYear` together. They are one row because the shared era
    /// emitter owns the order of the pair and the gate that decides whether
    /// either is read at all, and they fold into `year` in the resolver rather
    /// than occupying a slot.
    EraPair,
}

pub(super) enum TemporalDateTimeFieldReadMode {
    Conversion,
    With,
    ZonedWith { offset_nanoseconds_local: I64Local },
}

pub(super) enum TemporalPlainDateTimeStringMode {
    ToString,
    ToJson,
}

pub(super) enum TemporalPlainDateTimeComponent {
    PlainDate,
    PlainTime,
}

impl TemporalDateTimeFieldKey {
    const ALL: [Self; 12] = [
        Self::Day,
        Self::EraPair,
        Self::Hour,
        Self::Microsecond,
        Self::Millisecond,
        Self::Minute,
        Self::Month,
        Self::MonthCode,
        Self::Nanosecond,
        Self::Offset,
        Self::Second,
        Self::Year,
    ];

    const fn read(self) -> TemporalDateTimeFieldRead {
        match self {
            Self::Day => TemporalDateTimeFieldRead::PositiveInteger {
                property: "day",
                index: 2,
            },
            Self::EraPair => TemporalDateTimeFieldRead::EraPair,
            Self::Hour => TemporalDateTimeFieldRead::Integer {
                property: "hour",
                index: 3,
            },
            Self::Microsecond => TemporalDateTimeFieldRead::Integer {
                property: "microsecond",
                index: 7,
            },
            Self::Millisecond => TemporalDateTimeFieldRead::Integer {
                property: "millisecond",
                index: 6,
            },
            Self::Minute => TemporalDateTimeFieldRead::Integer {
                property: "minute",
                index: 4,
            },
            Self::Month => TemporalDateTimeFieldRead::PositiveInteger {
                property: "month",
                index: 1,
            },
            Self::MonthCode => TemporalDateTimeFieldRead::MonthCode,
            Self::Nanosecond => TemporalDateTimeFieldRead::Integer {
                property: "nanosecond",
                index: 8,
            },
            Self::Offset => TemporalDateTimeFieldRead::Offset,
            Self::Second => TemporalDateTimeFieldRead::Integer {
                property: "second",
                index: 5,
            },
            Self::Year => TemporalDateTimeFieldRead::Integer {
                property: "year",
                index: 0,
            },
        }
    }

    /// The first and last property name this row reads. They differ only for
    /// [`Self::EraPair`], and only so the ordering assertion below can compare
    /// adjacent rows across a multi-key row.
    const fn read_name_bounds(self) -> (&'static str, &'static str) {
        match self.read() {
            TemporalDateTimeFieldRead::PositiveInteger { property, .. }
            | TemporalDateTimeFieldRead::Integer { property, .. } => (property, property),
            TemporalDateTimeFieldRead::MonthCode => ("monthCode", "monthCode"),
            TemporalDateTimeFieldRead::Offset => ("offset", "offset"),
            TemporalDateTimeFieldRead::EraPair => ("era", "eraYear"),
        }
    }
}

/// Byte-order `<` on `&str`, because `str::lt` is not `const fn`.
const fn const_str_lt(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    let mut index = 0;
    while index < left.len() && index < right.len() {
        if left[index] != right[index] {
            return left[index] < right[index];
        }
        index += 1;
    }
    left.len() < right.len()
}

/// The sweep order is observable — `TemporalHelpers.propertyBagObserver` logs
/// every `get`, and `built-ins/Temporal/PlainDateTime/from/order-of-operations.js`
/// compares the whole log — so the table must be strictly alphabetical. A key
/// inserted in the wrong place is a build failure rather than a diff in a
/// 20-minute Test262 node.
const _: () = {
    let mut index = 1;
    while index < TemporalDateTimeFieldKey::ALL.len() {
        let (_, previous_last) = TemporalDateTimeFieldKey::ALL[index - 1].read_name_bounds();
        let (current_first, _) = TemporalDateTimeFieldKey::ALL[index].read_name_bounds();
        assert!(
            const_str_lt(previous_last, current_first),
            "TemporalDateTimeFieldKey::ALL must be in strict alphabetical read order"
        );
        index += 1;
    }
};

/// Exactly one row reads the era pair, which is what makes the `Option` dance
/// in `emit_temporal_date_time_read_fields` total.
const _: () = {
    let mut count = 0;
    let mut index = 0;
    while index < TemporalDateTimeFieldKey::ALL.len() {
        if matches!(
            TemporalDateTimeFieldKey::ALL[index].read(),
            TemporalDateTimeFieldRead::EraPair
        ) {
            count += 1;
        }
        index += 1;
    }
    assert!(
        count == 1,
        "TemporalDateTimeFieldKey::ALL must read the era pair exactly once"
    );
};

impl<'a> FunctionBuilder<'a> {
    fn emit_temporal_plain_date_time_overflow_option(
        &mut self,
        options: &ValueLocals,
        overflow: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options,
            overflow,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATETIME_OVERFLOW_OPTION,
            function,
        )
    }

    /// Internal slots already contain a canonical String. The whole value is
    /// rooted through the concrete cast while the calendar proof is acquired.
    fn emit_temporal_plain_date_time_calendar_slot(
        &mut self,
        calendar: &ValueLocals,
        function: &mut Function,
    ) -> Result<TemporalCalendarSlotLocals, EmitError> {
        let schema = self.runtime_schema();
        let identifier = schema.reserve_gc_local(function).initialize(
            calendar.cast_reference::<StringValue>(schema, function),
            function,
        );
        let result = self.emit_temporal_calendar_slot_from_identifier(&identifier, function)?;
        identifier.clear(function);
        Ok(result)
    }

    /// `BalanceISOYearMonth`: fold a month outside 1..=12 into the year.
    pub(crate) fn emit_temporal_balance_iso_year_month(
        &mut self,
        year_local: I64Local,
        month_local: I64Local,
        function: &mut Function,
    ) {
        let carry_local = self.runtime_schema().reserve_i64_local(function);
        // Floor division by 12 on `month - 1`; `I64DivS` truncates, so the
        // negative side needs the `-11` bias.
        (month_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (carry_local).store(function);
        (carry_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (carry_local).load(function);
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        (carry_local).load(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64DivS);
        (carry_local).store(function);
        (month_local).load(function);
        (carry_local).load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (month_local).store(function);
        (year_local).load(function);
        (carry_local).load(function);
        function.instruction(&Instruction::I64Add);
        (year_local).store(function);
        self.runtime_schema()
            .release_i64_local(carry_local, function);
    }

    /// `AddISODate`. The year/month shift is calendar arithmetic with a day
    /// clamp; the week/day shift is a plain epoch-day round trip.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_add_iso_date(
        &mut self,
        year_local: I64Local,
        month_local: I64Local,
        day_local: I64Local,
        years_local: I64Local,
        months_local: I64Local,
        weeks_local: I64Local,
        days_local: I64Local,
        overflow_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let epoch_local = self.runtime_schema().reserve_i64_local(function);

        (year_local).load(function);
        (years_local).load(function);
        function.instruction(&Instruction::I64Add);
        (year_local).store(function);
        (month_local).load(function);
        (months_local).load(function);
        function.instruction(&Instruction::I64Add);
        (month_local).store(function);
        self.emit_temporal_balance_iso_year_month(year_local, month_local, function);
        self.emit_temporal_plain_date_regulate(
            year_local,
            month_local,
            day_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_date_epoch_days(
            year_local,
            month_local,
            day_local,
            epoch_local,
            function,
        );
        (epoch_local).load(function);
        (weeks_local).load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (days_local).load(function);
        function.instruction(&Instruction::I64Add);
        (epoch_local).store(function);
        self.emit_temporal_civil_from_days(
            epoch_local,
            year_local,
            month_local,
            day_local,
            function,
        );
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;

        self.runtime_schema()
            .release_i64_local(epoch_local, function);
        Ok(())
    }

    /// Leaves an `i64` in `output_local`: -1, 0 or 1 comparing the left ISO
    /// date against the right.
    pub(super) fn emit_temporal_compare_iso_date(
        &mut self,
        left: [I64Local; 3],
        right: [I64Local; 3],
        output_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        (output_local).store(function);
        for index in 0..3 {
            (output_local).load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            (left[index]).load(function);
            (right[index]).load(function);
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            (output_local).store(function);
            function.instruction(&Instruction::Else);
            (left[index]).load(function);
            (right[index]).load(function);
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            (output_local).store(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
    }

    /// Split a signed nanosecond count into a floored day count and the
    /// non-negative nanosecond-of-day remainder.
    fn emit_temporal_split_days_and_nanoseconds(
        &mut self,
        total_local: I64Local,
        days_local: I64Local,
        function: &mut Function,
    ) {
        (total_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64DivS);
        (days_local).store(function);
        (total_local).load(function);
        (days_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (total_local).store(function);
        (total_local).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        (total_local).load(function);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Add);
        (total_local).store(function);
        (days_local).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        (days_local).store(function);
        function.instruction(&Instruction::End);
    }

    /// `PrepareCalendarFields` over the date-and-time keys, in the
    /// alphabetical order the reads are observable in.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_temporal_date_time_read_fields(
        &mut self,
        argument: &ValueLocals,
        calendar: &TemporalCalendarSlotLocals,
        field_locals: &[I64Local; 9],
        present_locals: &[I64Local; 9],
        month_code: &ValueLocals,
        month_code_present: I64Local,
        any_present: I64Local,
        mode: TemporalDateTimeFieldReadMode,
        function: &mut Function,
    ) -> Result<TemporalEraLocals, EmitError> {
        let mut era_slots = Some(self.reserve_temporal_era_slots(function));
        let mut era = None;
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        let present = schema.reserve_i64_local(function);
        let parsed = schema.reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(0));
        any_present.store(function);
        for key in TemporalDateTimeFieldKey::ALL {
            let index = match key.read() {
                TemporalDateTimeFieldRead::MonthCode => {
                    self.emit_temporal_duration_option_get(
                        argument,
                        "monthCode",
                        &value,
                        function,
                    )?;
                    value.tag().load(function);
                    function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
                    function.instruction(&Instruction::I32Ne);
                    function.instruction(&Instruction::I64ExtendI32U);
                    month_code_present.store(function);
                    month_code_present.load(function);
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    self.open_frame(ControlFrameKind::If, function);
                    function.instruction(&Instruction::I64Const(1));
                    any_present.store(function);
                    self.emit_temporal_month_code_string(
                        &value,
                        RuntimeErrorMessage::TEMPORAL_PLAINDATE_MONTHCODE_MUST_BE_A_STRING,
                        RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATE_MONTHCODE,
                        function,
                    )?;
                    month_code.copy_from(&value, function);
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::End);
                    continue;
                }
                TemporalDateTimeFieldRead::Offset => {
                    match &mode {
                        TemporalDateTimeFieldReadMode::Conversion
                        | TemporalDateTimeFieldReadMode::With => {}
                        TemporalDateTimeFieldReadMode::ZonedWith {
                            offset_nanoseconds_local,
                        } => {
                            self.emit_temporal_duration_option_get(
                                argument, "offset", &value, function,
                            )?;
                            value.tag().load(function);
                            function.instruction(&Instruction::I32Const(
                                ValueKind::Undefined.tag() as i32,
                            ));
                            function.instruction(&Instruction::I32Ne);
                            self.open_frame(ControlFrameKind::If, function);
                            let primitive = schema.reserve_completion(function);
                            self.emit_tagged_to_primitive_locals(
                                ToPrimitiveHint::String,
                                &value,
                                &primitive,
                                ToPrimitiveAbruptRoute::ReturnCurrentFunction,
                                function,
                            )?;
                            value.copy_from(primitive.value(), function);
                            primitive.clear(function);
                            value.tag().load(function);
                            function.instruction(&Instruction::I32Const(
                                ValueKind::String.tag() as i32
                            ));
                            function.instruction(&Instruction::I32Ne);
                            self.open_frame(ControlFrameKind::If, function);
                            self.emit_temporal_error_and_return(
                                lila_ir::NativeErrorKind::TypeError,
                                RuntimeErrorMessage::TEMPORAL_ZONEDDATETIME_OFFSET_MUST_BE_A_STRING,
                                function,
                            )?;
                            self.pop_control(ControlFrameKind::If);
                            function.instruction(&Instruction::End);
                            let string = schema.reserve_gc_local(function).initialize(
                                value.cast_reference::<StringValue>(schema, function),
                                function,
                            );
                            self.emit_temporal_utc_offset_nanoseconds(
                                &string,
                                *offset_nanoseconds_local,
                                function,
                            )?;
                            string.clear(function);
                            function.instruction(&Instruction::I64Const(1));
                            any_present.store(function);
                            self.pop_control(ControlFrameKind::If);
                            function.instruction(&Instruction::End);
                        }
                    }
                    continue;
                }
                TemporalDateTimeFieldRead::EraPair => {
                    let slots = era_slots
                        .take()
                        .expect("the ordered field domain contains exactly one era pair");
                    let read = self.emit_temporal_read_era_fields(
                        slots,
                        argument,
                        calendar.calendar_id(),
                        function,
                    )?;
                    for local in read.present_locals() {
                        local.load(function);
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::I32Eqz);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::I64Const(1));
                        any_present.store(function);
                        function.instruction(&Instruction::End);
                    }
                    era = Some(read);
                    continue;
                }
                TemporalDateTimeFieldRead::PositiveInteger { property, index } => {
                    self.emit_temporal_property_bag_positive_integer(
                        argument,
                        property,
                        present,
                        parsed,
                        0,
                        RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_FIELDS_MUST_BE_FINITE,
                        RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_MONTH_AND_DAY_MUST_BE_POSITIVE,
                        function,
                    )?;
                    index
                }
                TemporalDateTimeFieldRead::Integer { property, index } => {
                    self.emit_temporal_property_bag_integer(
                        argument,
                        property,
                        present,
                        parsed,
                        0,
                        RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_FIELDS_MUST_BE_FINITE,
                        function,
                    )?;
                    index
                }
            };
            present.load(function);
            present_locals[index].store(function);
            present.load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            parsed.load(function);
            field_locals[index].store(function);
            function.instruction(&Instruction::I64Const(1));
            any_present.store(function);
            function.instruction(&Instruction::End);
        }
        value.clear(function);
        schema.release_i64_local(parsed, function);
        schema.release_i64_local(present, function);
        Ok(era.expect("the ordered field domain contains exactly one era pair"))
    }

    /// `ToTemporalDateTime`. Accepts a branded `Temporal.PlainDateTime`
    /// (cloned), a branded `Temporal.PlainDate` (at midnight), a branded
    /// `Temporal.ZonedDateTime` (projected through its retained zone), any other object
    /// (read as a property bag) or an ISO string.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_to_temporal_date_time(
        &mut self,
        argument: &ValueLocals,
        overflow_options: TemporalConversionOverflowOptions<'_>,
        fields: &[I64Local; 9],
        calendar_out: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let handled = schema.reserve_i64_local(function);
        let overflow = schema.reserve_i64_local(function);
        let fraction = schema.reserve_i64_local(function);
        let month_code = schema.reserve_value_local(function);
        let encoded_month_code = schema.reserve_i64_local(function);
        let month_code_present = schema.reserve_i64_local(function);
        let any_present = schema.reserve_i64_local(function);
        let present = self.reserve_temporal_plain_date_time_field_locals(function);
        function.instruction(&Instruction::I64Const(0));
        handled.store(function);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        overflow.store(function);
        month_code.set_undefined(function);
        for local in fields
            .iter()
            .chain(present.iter())
            .copied()
            .chain([month_code_present, encoded_month_code])
        {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        let iso_calendar = self.emit_temporal_iso_calendar_slot(function)?;
        calendar_out.set_reference(iso_calendar.identifier(), schema, function);
        iso_calendar.release(self, function);

        // Concrete record tests preserve the internal-slot fast paths without
        // object addresses or legacy heap-brand integer mirrors.
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<TemporalPlainDateTimeObject>(schema, function),
            function,
        );
        self.emit_temporal_plain_date_time_load_record(&record, fields, calendar_out, function);
        record.clear(function);
        if let TemporalConversionOverflowOptions::Read(options) = overflow_options {
            self.emit_temporal_plain_date_time_overflow_option(options, overflow, function)?;
        }
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalPlainDateObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let record = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<TemporalPlainDateObject>(schema, function),
            function,
        );
        self.emit_temporal_plain_date_load_record(
            &record,
            &[fields[0], fields[1], fields[2]],
            calendar_out,
            function,
        );
        record.clear(function);
        if let TemporalConversionOverflowOptions::Read(options) = overflow_options {
            self.emit_temporal_plain_date_time_overflow_option(options, overflow, function)?;
        }
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        argument.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<TemporalZonedDateTimeObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let branded = self.emit_temporal_branded_zoned_record_from_value(argument, function)?;
        let epoch = self.emit_temporal_normalized_instant_from_zoned_record(&branded, function)?;
        let zone = self.emit_temporal_zone_from_zoned_record(&branded, function)?;
        let snapshot = self.emit_temporal_zone_snapshot(&zone, &epoch, function)?;
        let iso = self.emit_temporal_zone_snapshot_iso_record(&snapshot, function)?;
        let calendar = self.emit_temporal_calendar_from_zoned_record(&branded, function)?;
        for (source, destination) in iso.fields().iter().zip(fields) {
            source.load(function);
            destination.store(function);
        }
        calendar_out.set_reference(calendar.identifier(), schema, function);
        calendar.release(self, function);
        iso.release(self, function);
        snapshot.release(self, function);
        zone.release(self, function);
        epoch.release(self, function);
        branded.release(function);
        if let TemporalConversionOverflowOptions::Read(options) = overflow_options {
            self.emit_temporal_plain_date_time_overflow_option(options, overflow, function)?;
        }
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        handled.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.emit_is_heap_object_like_tag_i32(argument.tag(), function);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let calendar_value = schema.reserve_value_local(function);
        self.emit_temporal_duration_option_get(argument, "calendar", &calendar_value, function)?;
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &calendar_value,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        calendar_out.set_reference(calendar.identifier(), schema, function);
        calendar_value.clear(function);
        let era = self.emit_temporal_date_time_read_fields(
            argument,
            &calendar,
            fields,
            &present,
            &month_code,
            month_code_present,
            any_present,
            TemporalDateTimeFieldReadMode::Conversion,
            function,
        )?;
        if let TemporalConversionOverflowOptions::Read(options) = overflow_options {
            self.emit_temporal_plain_date_time_overflow_option(options, overflow, function)?;
        }
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            fields[0],
            present[0],
            function,
        )?;
        self.emit_temporal_plain_date_resolve_fields(
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
        self.emit_temporal_regulate_time(
            &Self::temporal_plain_date_time_time_locals(fields),
            overflow,
            function,
        )?;
        calendar.release(self, function);
        function.instruction(&Instruction::I64Const(1));
        handled.store(function);
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
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_EXPECTS_A_STRING_A_PROPERTY_BAG_OR_A_TEMPORAL_PLAINDATETIME, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_parse_plain_date_time_string(
            &string,
            fields[0],
            fields[1],
            fields[2],
            fields[3],
            fields[4],
            fields[5],
            fraction,
            calendar_out,
            function,
        )?;
        string.clear(function);
        for index in [8_usize, 7, 6] {
            fraction.load(function);
            function.instruction(&Instruction::I64Const(1_000));
            function.instruction(&Instruction::I64RemS);
            fields[index].store(function);
            fraction.load(function);
            function.instruction(&Instruction::I64Const(1_000));
            function.instruction(&Instruction::I64DivS);
            fraction.store(function);
        }
        if let TemporalConversionOverflowOptions::Read(options) = overflow_options {
            self.emit_temporal_plain_date_time_overflow_option(options, overflow, function)?;
        }
        self.emit_temporal_reject_iso_date(fields[0], fields[1], fields[2], function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_reject_date_time_lower_bound(fields, function)?;
        self.release_temporal_plain_date_time_field_locals(present, function);
        month_code.clear(function);
        for local in [
            any_present,
            month_code_present,
            encoded_month_code,
            fraction,
            overflow,
            handled,
        ] {
            schema.release_i64_local(local, function);
        }
        Ok(())
    }

    /// Temporal proposal 5.2.2 `Temporal.PlainDateTime.from`.
    pub(crate) fn emit_temporal_plain_date_time_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_to_temporal_date_time(
            &argument,
            TemporalConversionOverflowOptions::Read(&options),
            &fields,
            &calendar_value,
            function,
        )?;
        let calendar =
            self.emit_temporal_plain_date_time_calendar_slot(&calendar_value, function)?;
        self.emit_alloc_temporal_plain_date_time(
            &fields,
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// `CompareISODateTime` over the nine fields.
    fn emit_temporal_plain_date_time_compare_fields(
        &mut self,
        left: &[I64Local; 9],
        right: &[I64Local; 9],
        comparison_local: I64Local,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        (comparison_local).store(function);
        for index in 0..9 {
            (comparison_local).load(function);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            (left[index]).load(function);
            (right[index]).load(function);
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            (comparison_local).store(function);
            function.instruction(&Instruction::Else);
            (left[index]).load(function);
            (right[index]).load(function);
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            (comparison_local).store(function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
    }

    /// Temporal proposal 5.2.3 `Temporal.PlainDateTime.compare`.
    pub(crate) fn emit_temporal_plain_date_time_compare(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let comparison = schema.reserve_i64_local(function);
        let left = self.reserve_temporal_plain_date_time_field_locals(function);
        let right = self.reserve_temporal_plain_date_time_field_locals(function);
        for (index, fields) in [(0_usize, &left), (1, &right)] {
            self.emit_builtin_arg_to_value(index, &argument, function);
            self.emit_to_temporal_date_time(
                &argument,
                TemporalConversionOverflowOptions::Omit,
                fields,
                &calendar_value,
                function,
            )?;
        }
        self.emit_temporal_plain_date_time_compare_fields(&left, &right, comparison, function);
        comparison.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        comparison.store(function);
        self.completion().value().set_number(comparison, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        schema.release_i64_local(comparison, function);
        self.release_temporal_plain_date_time_field_locals(right, function);
        self.release_temporal_plain_date_time_field_locals(left, function);
        calendar_value.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 5.3.x `equals`.
    pub(crate) fn emit_temporal_plain_date_time_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let other_calendar_value = schema.reserve_value_local(function);
        let comparison = schema.reserve_i64_local(function);
        let equal = schema.reserve_i32_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        let other = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &fields,
            &calendar_value,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_to_temporal_date_time(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &other,
            &other_calendar_value,
            function,
        )?;
        self.emit_temporal_plain_date_time_compare_fields(&fields, &other, comparison, function);
        comparison.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        let calendar = schema.reserve_gc_local(function).initialize(
            calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let other_calendar = schema.reserve_gc_local(function).initialize(
            other_calendar_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_string_payload_equality_i32(&calendar, &other_calendar, function);
        other_calendar.clear(function);
        calendar.clear(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        equal.store(function);
        self.completion().value().set_boolean(equal, function);
        self.completion()
            .set_normal(self.completion().value(), function);
        schema.release_i32_local(equal, function);
        schema.release_i64_local(comparison, function);
        self.release_temporal_plain_date_time_field_locals(other, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        other_calendar_value.clear(function);
        calendar_value.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 5.3.x `with`.
    pub(crate) fn emit_temporal_plain_date_time_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let acquired_month_code = schema.reserve_value_local(function);
        let receiver_month_code = schema.reserve_value_local(function);
        let value = schema.reserve_value_local(function);
        let overflow = schema.reserve_i64_local(function);
        let encoded_month_code = schema.reserve_i64_local(function);
        let month_code_present = schema.reserve_i64_local(function);
        let any_present = schema.reserve_i64_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        let present = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &fields,
            &calendar_value,
            function,
        )?;
        let calendar =
            self.emit_temporal_plain_date_time_calendar_slot(&calendar_value, function)?;
        let projected = self.emit_temporal_project_calendar_date(
            calendar.calendar_id(),
            [fields[0], fields[1], fields[2]],
            function,
        );
        for (source, destination) in projected
            .fields()
            .into_iter()
            .zip([fields[0], fields[1], fields[2]])
        {
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
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_PROTOTYPE_WITH_REQUIRES_AN_OBJECT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_reject_branded_partial_object(&argument,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_PROTOTYPE_WITH_DOES_NOT_ACCEPT_A_TEMPORAL_OBJECT, function)?;
        // Both observable Gets precede the ordered field sweep.
        for property in ["calendar", "timeZone"] {
            self.emit_temporal_duration_option_get(&argument, property, &value, function)?;
            value.tag().load(function);
            function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
            function.instruction(&Instruction::I32Ne);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
                RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_PROTOTYPE_WITH_DOES_NOT_ACCEPT_CALENDAR_OR_TIMEZONE, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        acquired_month_code.set_undefined(function);
        for local in present
            .iter()
            .copied()
            .chain([month_code_present, encoded_month_code])
        {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        let era = self.emit_temporal_date_time_read_fields(
            &argument,
            &calendar,
            &fields,
            &present,
            &acquired_month_code,
            month_code_present,
            any_present,
            TemporalDateTimeFieldReadMode::With,
            function,
        )?;
        any_present.load(function);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_PROTOTYPE_WITH_REQUIRES_AT_LEAST_ONE_DATE_OR_TIME_FIELD, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_plain_date_time_overflow_option(&options, overflow, function)?;
        // Resolve caller-supplied era/year before marking receiver defaults as
        // present, so an era-only bag cannot conflict with the receiver year.
        let resolved_year = self.emit_temporal_resolve_era_to_calendar_year(
            era,
            calendar.calendar_id(),
            fields[0],
            present[0],
            function,
        )?;
        present[1].load(function);
        month_code_present.load(function);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        acquired_month_code.copy_from(&receiver_month_code, function);
        function.instruction(&Instruction::I64Const(1));
        month_code_present.store(function);
        function.instruction(&Instruction::End);
        for index in [0_usize, 2] {
            function.instruction(&Instruction::I64Const(1));
            present[index].store(function);
        }
        self.emit_temporal_plain_date_resolve_fields(
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
        self.emit_temporal_regulate_time(
            &Self::temporal_plain_date_time_time_locals(&fields),
            overflow,
            function,
        )?;
        self.emit_alloc_temporal_plain_date_time(
            &fields,
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        self.release_temporal_plain_date_time_field_locals(present, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        for local in [
            any_present,
            month_code_present,
            encoded_month_code,
            overflow,
        ] {
            schema.release_i64_local(local, function);
        }
        value.clear(function);
        receiver_month_code.clear(function);
        acquired_month_code.clear(function);
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 5.3.x `withPlainTime`. An absent argument means
    /// midnight, not "keep the current time".
    pub(crate) fn emit_temporal_plain_date_time_with_plain_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        let time = self.reserve_temporal_plain_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &fields,
            &calendar_value,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        for local in time {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_to_temporal_time(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &time,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for (source, destination) in time.iter().zip(fields[3..].iter()) {
            source.load(function);
            destination.store(function);
        }
        let calendar =
            self.emit_temporal_plain_date_time_calendar_slot(&calendar_value, function)?;
        self.emit_alloc_temporal_plain_date_time(
            &fields,
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        self.release_temporal_plain_time_field_locals(time, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        calendar_value.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 5.3.x `withCalendar`.
    pub(crate) fn emit_temporal_plain_date_time_with_calendar(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let existing_calendar = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &fields,
            &existing_calendar,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let calendar = self.emit_temporal_to_temporal_calendar_identifier(
            &argument,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_CALENDAR_MUST_BE_A_STRING,
            function,
        )?;
        self.emit_alloc_temporal_plain_date_time(
            &fields,
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        existing_calendar.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// Temporal proposal 5.3.x `toPlainDate` and `toPlainTime`.
    pub(super) fn emit_temporal_plain_date_time_to_component(
        &mut self,
        component: TemporalPlainDateTimeComponent,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar_value = self.runtime_schema().reserve_value_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &fields,
            &calendar_value,
            function,
        )?;
        match component {
            TemporalPlainDateTimeComponent::PlainDate => {
                let calendar =
                    self.emit_temporal_plain_date_time_calendar_slot(&calendar_value, function)?;
                self.emit_alloc_temporal_plain_date(
                    fields[0],
                    fields[1],
                    fields[2],
                    &calendar,
                    TemporalPrototypeSource::Intrinsic,
                    function,
                )?;
                calendar.release(self, function);
            }
            TemporalPlainDateTimeComponent::PlainTime => self.emit_alloc_temporal_plain_time(
                &Self::temporal_plain_date_time_time_locals(&fields),
                TemporalPrototypeSource::Intrinsic,
                function,
            )?,
        }
        self.release_temporal_plain_date_time_field_locals(fields, function);
        calendar_value.clear(function);
        Ok(())
    }

    /// Temporal proposal 5.3.x `add` and `subtract`, both through
    /// `AddDurationToDateTime`.
    pub(super) fn emit_temporal_plain_date_time_add_or_subtract(
        &mut self,
        operation: TemporalPlainArithmeticOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let overflow_local = schema.reserve_i64_local(function);
        let seconds_local = schema.reserve_i64_local(function);
        let subsecond_local = schema.reserve_i64_local(function);
        let total_local = schema.reserve_i64_local(function);
        let day_delta_local = schema.reserve_i64_local(function);
        let field_locals = self.reserve_temporal_plain_date_time_field_locals(function);
        let duration_locals = self.reserve_temporal_duration_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            &calendar_value,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_to_temporal_duration(&argument, &duration_locals, function)?;
        self.emit_temporal_plain_date_time_overflow_option(&options, overflow_local, function)?;
        match operation {
            TemporalPlainArithmeticOperation::Add => {}
            TemporalPlainArithmeticOperation::Subtract => {
                self.emit_temporal_duration_negate_fields(&duration_locals, function);
            }
        }
        let date_fields =
            self.reserve_temporal_duration_date_field_locals(&duration_locals, function);

        // Hours and below fold into a nanosecond offset; the whole days that
        // fall out of it join the duration's own day count.
        self.emit_temporal_duration_normalize_seconds(
            &duration_locals,
            TemporalUnit::Hour,
            seconds_local,
            subsecond_local,
            function,
        );
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        (day_delta_local).store(function);
        (seconds_local).load(function);
        (day_delta_local).load(function);
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        (seconds_local).store(function);
        let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
        self.emit_temporal_plain_time_total_nanoseconds(&time_locals, total_local, function);
        (total_local).load(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (subsecond_local).load(function);
        function.instruction(&Instruction::I64Add);
        (total_local).store(function);
        self.emit_temporal_split_days_and_nanoseconds(total_local, seconds_local, function);
        (day_delta_local).load(function);
        (seconds_local).load(function);
        function.instruction(&Instruction::I64Add);
        (date_fields[3]).load(function);
        function.instruction(&Instruction::I64Add);
        (day_delta_local).store(function);
        self.emit_temporal_plain_time_from_nanoseconds(total_local, &time_locals, function);
        for index in 0..6 {
            (time_locals[index]).load(function);
            (field_locals[index + 3]).store(function);
        }
        let calendar =
            self.emit_temporal_plain_date_time_calendar_slot(&calendar_value, function)?;
        self.emit_temporal_add_calendar_date(
            &calendar,
            field_locals[0],
            field_locals[1],
            field_locals[2],
            date_fields[0],
            date_fields[1],
            date_fields[2],
            day_delta_local,
            overflow_local,
            function,
        )?;

        self.emit_temporal_reject_date_time_lower_bound(&field_locals, function)?;
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;

        calendar.release(self, function);
        for local in date_fields.into_iter().rev() {
            self.runtime_schema().release_i64_local(local, function);
        }
        self.release_temporal_duration_field_locals(duration_locals, function);
        self.release_temporal_plain_date_time_field_locals(field_locals, function);
        for local in [
            day_delta_local,
            total_local,
            subsecond_local,
            seconds_local,
            overflow_local,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// `RoundISODateTime`: round the selected wall-clock field and smaller
    /// fields, then carry any whole day into the date.
    fn emit_temporal_round_iso_date_time(
        &mut self,
        field_locals: &[I64Local; 9],
        unit_local: I64Local,
        quantum_local: I64Local,
        mode_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let total_local = self.runtime_schema().reserve_i64_local(function);
        let day_delta_local = self.runtime_schema().reserve_i64_local(function);
        let epoch_local = self.runtime_schema().reserve_i64_local(function);
        let time_locals = Self::temporal_plain_date_time_time_locals(field_locals);

        self.emit_temporal_plain_time_total_nanoseconds(&time_locals, total_local, function);
        self.emit_temporal_round_time_nanoseconds(
            total_local,
            unit_local,
            quantum_local,
            mode_local,
            function,
        );
        self.emit_temporal_split_days_and_nanoseconds(total_local, day_delta_local, function);
        self.emit_temporal_plain_time_from_nanoseconds(total_local, &time_locals, function);
        for index in 0..6 {
            (time_locals[index]).load(function);
            (field_locals[index + 3]).store(function);
        }
        (day_delta_local).load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_plain_date_epoch_days(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            epoch_local,
            function,
        );
        (epoch_local).load(function);
        (day_delta_local).load(function);
        function.instruction(&Instruction::I64Add);
        (epoch_local).store(function);
        self.emit_temporal_civil_from_days(
            epoch_local,
            field_locals[0],
            field_locals[1],
            field_locals[2],
            function,
        );
        self.emit_temporal_reject_iso_date(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Outside the day-carry branch: rounding down to the minimum day's
        // midnight leaves the date untouched but still leaves the range, and
        // `toString` reads the rounded fields without going through
        // `CreateTemporalDateTime`.
        self.emit_temporal_reject_date_time_lower_bound(field_locals, function)?;

        for local in [epoch_local, day_delta_local, total_local] {
            self.runtime_schema().release_i64_local(local, function);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `round`. `smallestUnit` runs from `day` down to
    /// `nanosecond`; a day increment must be exactly 1.
    pub(crate) fn emit_temporal_plain_date_time_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let unit = schema.reserve_i64_local(function);
        let increment = schema.reserve_i64_local(function);
        let mode = schema.reserve_i64_local(function);
        let quantum = schema.reserve_i64_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &fields,
            &calendar_value,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_PROTOTYPE_ROUND_REQUIRES_A_ROUNDTO_ARGUMENT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        increment.store(function);
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        mode.store(function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::String.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let string = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_temporal_plain_time_unit_from_string(&string, unit, function)?;
        string.clear(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(&argument, function)?;
        self.emit_temporal_duration_rounding_increment_option(&argument, increment, function)?;
        self.emit_temporal_duration_rounding_mode_option(
            &argument,
            TemporalRoundingMode::HalfExpand,
            mode,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            &argument,
            TemporalUnitOptionProperty::SmallestUnit,
            unit,
            function,
        )?;
        unit.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_PROTOTYPE_ROUND_REQUIRES_SMALLESTUNIT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            unit,
            TemporalUnit::Day,
            TemporalUnit::Nanosecond,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATETIME_UNIT_OPTION,
            function,
        )?;
        unit.load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        increment.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(
            lila_ir::NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATETIME_ROUNDING_INCREMENT,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        quantum.store(function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_plain_time_validate_increment(unit, increment, function)?;
        self.emit_temporal_plain_time_rounding_quantum(unit, increment, quantum, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_round_iso_date_time(&fields, unit, quantum, mode, function)?;
        let calendar =
            self.emit_temporal_plain_date_time_calendar_slot(&calendar_value, function)?;
        self.emit_alloc_temporal_plain_date_time(
            &fields,
            &calendar,
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        calendar.release(self, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        for local in [quantum, mode, increment, unit] {
            schema.release_i64_local(local, function);
        }
        calendar_value.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// The shared `GetDifferenceSettings` boundary for PlainDateTime,
    /// ZonedDateTime and Instant differences. `plan` is compile-time policy:
    /// no raw unit or direction flag can cross this boundary independently of
    /// the consumer that owns it.
    pub(super) fn emit_temporal_date_time_difference_settings(
        &mut self,
        options: &ValueLocals,
        plan: TemporalDateTimeDifferenceSettingsPlan,
        function: &mut Function,
    ) -> Result<ResolvedTemporalDateTimeDifferenceSettings, EmitError> {
        let largest_unit_local = self.runtime_schema().reserve_i64_local(function);
        let smallest_unit_local = self.runtime_schema().reserve_i64_local(function);
        let increment_local = self.runtime_schema().reserve_i64_local(function);
        let mode_local = self.runtime_schema().reserve_i64_local(function);

        // `GetDifferenceSettings` reads largestUnit, then the two rounding
        // options, then smallestUnit - the order is observable.
        self.emit_temporal_duration_options_object(options, function)?;
        self.emit_temporal_duration_unit_option(
            options,
            TemporalUnitOptionProperty::LargestUnit,
            largest_unit_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(options, increment_local, function)?;
        self.emit_temporal_duration_rounding_mode_option(
            options,
            TemporalRoundingMode::Trunc,
            mode_local,
            function,
        )?;
        if plan.negates_rounding_mode() {
            let original_mode_local = self.runtime_schema().reserve_i64_local(function);
            // `NegateRoundingMode`: ceil and floor swap, as do halfCeil and
            // halfFloor; the sign-symmetric modes are unchanged.
            (mode_local).load(function);
            (original_mode_local).store(function);
            for mode in TemporalRoundingMode::ALL {
                if mode.negated() == mode {
                    continue;
                }
                (original_mode_local).load(function);
                function.instruction(&Instruction::I64Const(mode.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(mode.negated().code()));
                (mode_local).store(function);
                function.instruction(&Instruction::End);
            }
            self.runtime_schema()
                .release_i64_local(original_mode_local, function);
        }
        self.emit_temporal_duration_unit_option(
            options,
            TemporalUnitOptionProperty::SmallestUnit,
            smallest_unit_local,
            function,
        )?;
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        (smallest_unit_local).store(function);
        function.instruction(&Instruction::End);
        // Recognized units are checked against the receiver's category only
        // after all four options have been read and independently validated.
        self.emit_temporal_require_unit_range(
            smallest_unit_local,
            plan.largest_allowed_unit(),
            TemporalUnit::Nanosecond,
            plan.invalid_unit_message(),
            function,
        )?;

        // An unset or `"auto"` largestUnit falls back to the larger of the
        // consumer's closed fallback and the resolved smallest unit.
        let fallback_largest_unit = plan.fallback_largest_unit();
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        (largest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(fallback_largest_unit.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(fallback_largest_unit.code()));
        function.instruction(&Instruction::End);
        (largest_unit_local).store(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            largest_unit_local,
            plan.largest_allowed_unit(),
            TemporalUnit::Nanosecond,
            plan.invalid_unit_message(),
            function,
        )?;
        self.emit_temporal_require_largest_not_smaller(
            largest_unit_local,
            smallest_unit_local,
            function,
        )?;
        (smallest_unit_local).load(function);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_plain_time_validate_increment(
            smallest_unit_local,
            increment_local,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        Ok(ResolvedTemporalDateTimeDifferenceSettings {
            largest_unit_local,
            smallest_unit_local,
            increment_local,
            mode_local,
        })
    }

    /// Temporal proposal 5.3.x `until` and `since`, both through
    /// `DifferencePlainDateTimeWithRounding`.
    pub(super) fn emit_temporal_plain_date_time_until_or_since(
        &mut self,
        operation: TemporalPlainDifferenceOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let other_calendar_value = schema.reserve_value_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        let other = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &fields,
            &calendar_value,
            function,
        )?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_builtin_arg_to_value(1, &options, function);
        self.emit_to_temporal_date_time(
            &argument,
            TemporalConversionOverflowOptions::Omit,
            &other,
            &other_calendar_value,
            function,
        )?;
        let calendar =
            self.emit_temporal_plain_date_time_calendar_slot(&calendar_value, function)?;
        let other_calendar =
            self.emit_temporal_plain_date_time_calendar_slot(&other_calendar_value, function)?;
        // Calendar agreement precedes every observable options Get.
        self.emit_temporal_require_same_calendar(
            calendar.calendar_id(),
            other_calendar.calendar_id(),
            TemporalDifferenceGuard::PlainDateTimeSameCalendar,
            function,
        )?;
        let plan = match operation {
            TemporalPlainDifferenceOperation::Until => {
                TemporalDateTimeDifferenceSettingsPlan::PlainUntil
            }
            TemporalPlainDifferenceOperation::Since => {
                TemporalDateTimeDifferenceSettingsPlan::PlainSince
            }
        };
        let settings =
            self.emit_temporal_date_time_difference_settings(&options, plan, function)?;
        self.emit_temporal_difference_date_time(
            &calendar, &fields, &other, &settings, operation, function,
        )?;
        settings.release(self, function);
        other_calendar.release(self, function);
        calendar.release(self, function);
        self.release_temporal_plain_date_time_field_locals(other, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        other_calendar_value.clear(function);
        calendar_value.clear(function);
        options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// `Temporal.PlainDateTime.prototype.toLocaleString`.
    ///
    /// `new Intl.DateTimeFormat(locales, options).format(this)`. This is the
    /// one plain type with no rejected style — it has both date and time
    /// fields, so `dateStyle` and `timeStyle` are each meaningful.
    pub(crate) fn emit_temporal_plain_date_time_to_locale_string(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar = self.runtime_schema().reserve_value_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(&fields, &calendar, function)?;
        self.release_temporal_plain_date_time_field_locals(fields, function);
        calendar.clear(function);
        self.emit_intl_dtf_temporal_to_locale_string(
            super::intl_datetimeformat::DtfTemporalKind::PlainDateTime,
            function,
        )
    }

    /// `TemporalDateTimeToString`. The closed mode selects whether the option bag is
    /// read: `toString` reads it and `toJSON` is fixed at `auto` precision and
    /// `auto` calendar name.
    pub(crate) fn emit_temporal_plain_date_time_to_string(
        &mut self,
        mode: TemporalPlainDateTimeStringMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let options = schema.reserve_value_local(function);
        let calendar_value = schema.reserve_value_local(function);
        let output_value = schema.reserve_value_local(function);
        let time_value = schema.reserve_value_local(function);
        let show_calendar = schema.reserve_i64_local(function);
        let digits = schema.reserve_i64_local(function);
        let unit = schema.reserve_i64_local(function);
        let rounding_mode = schema.reserve_i64_local(function);
        let precision = schema.reserve_i64_local(function);
        let increment = schema.reserve_i64_local(function);
        let quantum = schema.reserve_i64_local(function);
        let fields = self.reserve_temporal_plain_date_time_field_locals(function);
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &fields,
            &calendar_value,
            function,
        )?;
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        precision.store(function);
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        show_calendar.store(function);
        match mode {
            TemporalPlainDateTimeStringMode::ToString => {
                self.emit_builtin_arg_to_value(0, &options, function);
                self.emit_temporal_duration_options_object(&options, function)?;
                self.emit_temporal_string_valued_option::<ShowCalendarName>(
                    &options, show_calendar,
                    RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
                    RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATETIME_CALENDARNAME_OPTION, function)?;
                self.emit_temporal_plain_time_fractional_digits_option(&options, digits, function)?;
                self.emit_temporal_duration_rounding_mode_option(
                    &options,
                    TemporalRoundingMode::Trunc,
                    rounding_mode,
                    function,
                )?;
                self.emit_temporal_duration_unit_option(
                    &options,
                    TemporalUnitOptionProperty::SmallestUnit,
                    unit,
                    function,
                )?;
                self.emit_temporal_seconds_string_precision(
                    digits,
                    unit,
                    precision,
                    increment,
                    RuntimeErrorMessage::INVALID_TEMPORAL_PLAINDATETIME_UNIT_OPTION,
                    function,
                )?;
                self.emit_temporal_plain_time_rounding_quantum(unit, increment, quantum, function);
                self.emit_temporal_round_iso_date_time(
                    &fields,
                    unit,
                    quantum,
                    rounding_mode,
                    function,
                )?;
            }
            TemporalPlainDateTimeStringMode::ToJson => {}
        }
        self.emit_temporal_iso_date_string(
            fields[0],
            fields[1],
            fields[2],
            &output_value,
            function,
        )?;
        let output = schema.reserve_gc_local(function).initialize(
            output_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        let separator = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("T", function)?,
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(&output, &separator, function),
            function,
        );
        separator.clear(function);
        self.emit_temporal_plain_time_record_to_string(
            &Self::temporal_plain_date_time_time_locals(&fields),
            precision,
            &time_value,
            function,
        )?;
        let time = schema.reserve_gc_local(function).initialize(
            time_value.cast_reference::<StringValue>(schema, function),
            function,
        );
        output.replace(
            self.emit_concat_gc_strings(&output, &time, function),
            function,
        );
        time.clear(function);
        output_value.set_reference(&output, schema, function);
        output.clear(function);
        let calendar =
            self.emit_temporal_plain_date_time_calendar_slot(&calendar_value, function)?;
        self.emit_temporal_append_calendar_annotation(
            &calendar,
            show_calendar,
            &output_value,
            function,
        )?;
        calendar.release(self, function);
        self.completion().set_normal(&output_value, function);
        self.release_temporal_plain_date_time_field_locals(fields, function);
        for local in [
            quantum,
            increment,
            precision,
            rounding_mode,
            unit,
            digits,
            show_calendar,
        ] {
            schema.release_i64_local(local, function);
        }
        time_value.clear(function);
        output_value.clear(function);
        calendar_value.clear(function);
        options.clear(function);
        Ok(())
    }

    /// Resolve the zone before options, retain disambiguation through inverse
    /// selection, and preserve the receiver's actual calendar slot.
    pub(crate) fn emit_temporal_plain_date_time_to_zoned_date_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let time_zone = schema.reserve_value_local(function);
        let options = schema.reserve_value_local(function);
        let (iso_record, calendar) =
            self.emit_temporal_iso_record_from_plain_date_time_receiver(function)?;
        self.emit_builtin_arg_to_value(0, &time_zone, function);
        time_zone.tag().load(function);
        function.instruction(&Instruction::I32Const(ValueKind::Undefined.tag() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_temporal_error_and_return(lila_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::TEMPORAL_PLAINDATETIME_PROTOTYPE_TOZONEDDATETIME_REQUIRES_A_TIME_ZONE, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let zone = self.emit_temporal_zoned_date_time_time_zone(
            &time_zone,
            TemporalTimeZoneStringGoal::Object,
            function,
        )?;
        // Zone conversion precedes the sole options/disambiguation read.
        self.emit_builtin_arg_to_value(1, &options, function);
        let disambiguation =
            self.emit_temporal_plain_date_time_zoned_disambiguation(&options, function)?;
        let local = self.emit_temporal_local_coordinate_from_iso_record(&iso_record, function)?;
        let instant =
            self.emit_temporal_get_epoch_nanoseconds_for(&zone, &local, &disambiguation, function)?;
        self.emit_alloc_temporal_zoned_date_time(
            TemporalZonedAllocationInput::new(&instant, &zone, &calendar),
            TemporalPrototypeSource::Intrinsic,
            function,
        )?;
        instant.release(self, function);
        local.release(self, function);
        disambiguation.release(self, function);
        zone.release(self, function);
        calendar.release(self, function);
        iso_record.release(self, function);
        options.clear(function);
        time_zone.clear(function);
        Ok(())
    }
}
