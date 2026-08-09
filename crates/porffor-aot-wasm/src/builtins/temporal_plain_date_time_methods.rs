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
//! types agree on what `halfEven` and `smallestUnit` mean. The two genuinely
//! new pieces are `emit_temporal_add_iso_date` (calendar addition, which needs
//! the epoch-day round trip) and `emit_temporal_difference_iso_date`
//! (`CalendarDateUntil` for the ISO calendar).

use super::super::*;
use super::temporal_options::{
    ShowCalendarName, TemporalOverflow, TemporalRoundingMode, TemporalUnit, TemporalUnitSlot,
};
use super::temporal_plain_date::TemporalEraLocals;
use super::temporal_plain_time::NANOSECONDS_PER_TEMPORAL_DAY;
use super::temporal_plain_time_methods::{TEMPORAL_PRECISION_AUTO, TEMPORAL_PRECISION_MINUTE};

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
    /// `era` and `eraYear` together. They are one row because the shared era
    /// emitter owns the order of the pair and the gate that decides whether
    /// either is read at all, and they fold into `year` in the resolver rather
    /// than occupying a slot.
    EraPair,
}

impl TemporalDateTimeFieldKey {
    const ALL: [Self; 11] = [
        Self::Day,
        Self::EraPair,
        Self::Hour,
        Self::Microsecond,
        Self::Millisecond,
        Self::Minute,
        Self::Month,
        Self::MonthCode,
        Self::Nanosecond,
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
/// in `emit_temporal_plain_date_time_read_fields` total.
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
        options_payload_local: u32,
        options_tag_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_string_valued_option::<TemporalOverflow>(
            options_payload_local,
            options_tag_local,
            overflow_local,
            "Temporal.PlainDateTime options must be an object or undefined",
            "Invalid Temporal.PlainDateTime overflow option",
            function,
        )
    }

    /// `min(day, ISODaysInMonth(year, month))`, the day clamp both
    /// `RegulateISODate` and `CalendarDateUntil` need.
    fn emit_temporal_iso_date_clamp_day(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        function: &mut Function,
    ) {
        let maximum_local = self.reserve_temp_local();
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_local, function);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(maximum_local));
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(maximum_local);
    }

    /// `BalanceISOYearMonth`: fold a month outside 1..=12 into the year.
    pub(crate) fn emit_temporal_balance_iso_year_month(
        &mut self,
        year_local: u32,
        month_local: u32,
        function: &mut Function,
    ) {
        let carry_local = self.reserve_temp_local();
        // Floor division by 12 on `month - 1`; `I64DivS` truncates, so the
        // negative side needs the `-11` bias.
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(carry_local));
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(carry_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(carry_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(year_local));
        self.release_temp_local(carry_local);
    }

    /// `AddISODate`. The year/month shift is calendar arithmetic with a day
    /// clamp; the week/day shift is a plain epoch-day round trip.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_add_iso_date(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        years_local: u32,
        months_local: u32,
        weeks_local: u32,
        days_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let epoch_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(years_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(months_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(month_local));
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
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::LocalGet(weeks_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(epoch_local));
        self.emit_temporal_civil_from_days(
            epoch_local,
            year_local,
            month_local,
            day_local,
            function,
        );
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;

        self.release_temp_local(epoch_local);
        Ok(())
    }

    /// Leaves an `i64` in `output_local`: -1, 0 or 1 comparing the left ISO
    /// date against the right.
    fn emit_temporal_compare_iso_date(
        &mut self,
        left: [u32; 3],
        right: [u32; 3],
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(output_local));
        for index in 0..3 {
            function.instruction(&Instruction::LocalGet(output_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(left[index]));
            function.instruction(&Instruction::LocalGet(right[index]));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(left[index]));
            function.instruction(&Instruction::LocalGet(right[index]));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }
    }

    /// `CalendarDateUntil` for the ISO calendar. `largest_unit_local` picks
    /// between the year/month form (calendar arithmetic with a day clamp) and
    /// the week/day form (a plain epoch-day subtraction).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_difference_iso_date(
        &mut self,
        left: [u32; 3],
        right: [u32; 3],
        largest_unit_local: u32,
        years_local: u32,
        months_local: u32,
        weeks_local: u32,
        days_local: u32,
        function: &mut Function,
    ) {
        let sign_local = self.reserve_temp_local();
        let mid_sign_local = self.reserve_temp_local();
        let mid_year_local = self.reserve_temp_local();
        let mid_month_local = self.reserve_temp_local();
        let mid_day_local = self.reserve_temp_local();
        let left_epoch_local = self.reserve_temp_local();
        let right_epoch_local = self.reserve_temp_local();
        let done_local = self.reserve_temp_local();

        for local in [years_local, months_local, weeks_local, days_local] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        self.emit_temporal_plain_date_epoch_days(
            left[0],
            left[1],
            left[2],
            left_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            right[0],
            right[1],
            right[2],
            right_epoch_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(right_epoch_local));
        function.instruction(&Instruction::LocalGet(left_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Week.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(weeks_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(weeks_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        self.emit_temporal_compare_iso_date(right, left, sign_local, function);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(done_local));

        function.instruction(&Instruction::LocalGet(right[0]));
        function.instruction(&Instruction::LocalGet(left[0]));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(years_local));
        self.emit_temporal_difference_iso_date_midpoint(
            left,
            years_local,
            months_local,
            mid_year_local,
            mid_month_local,
            mid_day_local,
            function,
        );
        self.emit_temporal_compare_iso_date(
            right,
            [mid_year_local, mid_month_local, mid_day_local],
            mid_sign_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mid_sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(done_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(done_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(right[1]));
        function.instruction(&Instruction::LocalGet(left[1]));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(months_local));
        function.instruction(&Instruction::LocalGet(mid_sign_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(years_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(years_local));
        function.instruction(&Instruction::LocalGet(months_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(months_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_difference_iso_date_midpoint(
            left,
            years_local,
            months_local,
            mid_year_local,
            mid_month_local,
            mid_day_local,
            function,
        );
        self.emit_temporal_compare_iso_date(
            right,
            [mid_year_local, mid_month_local, mid_day_local],
            mid_sign_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(mid_sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(mid_sign_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(months_local));
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(months_local));
        self.emit_temporal_difference_iso_date_midpoint(
            left,
            years_local,
            months_local,
            mid_year_local,
            mid_month_local,
            mid_day_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_temporal_plain_date_epoch_days(
            mid_year_local,
            mid_month_local,
            mid_day_local,
            left_epoch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(right_epoch_local));
        function.instruction(&Instruction::LocalGet(left_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Month.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(months_local));
        function.instruction(&Instruction::LocalGet(years_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(months_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(years_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            done_local,
            right_epoch_local,
            left_epoch_local,
            mid_day_local,
            mid_month_local,
            mid_year_local,
            mid_sign_local,
            sign_local,
        ] {
            self.release_temp_local(local);
        }
    }

    /// `left + (years, months)` with the day clamped, the intermediate every
    /// step of `CalendarDateUntil` compares against.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_difference_iso_date_midpoint(
        &mut self,
        left: [u32; 3],
        years_local: u32,
        months_local: u32,
        mid_year_local: u32,
        mid_month_local: u32,
        mid_day_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(left[0]));
        function.instruction(&Instruction::LocalGet(years_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mid_year_local));
        function.instruction(&Instruction::LocalGet(left[1]));
        function.instruction(&Instruction::LocalGet(months_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(mid_month_local));
        self.emit_temporal_balance_iso_year_month(mid_year_local, mid_month_local, function);
        function.instruction(&Instruction::LocalGet(left[2]));
        function.instruction(&Instruction::LocalSet(mid_day_local));
        self.emit_temporal_iso_date_clamp_day(
            mid_year_local,
            mid_month_local,
            mid_day_local,
            function,
        );
    }

    /// Split a signed nanosecond count into a floored day count and the
    /// non-negative nanosecond-of-day remainder.
    fn emit_temporal_split_days_and_nanoseconds(
        &mut self,
        total_local: u32,
        days_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::End);
    }

    /// `PrepareCalendarFields` over the eleven date-and-time keys, in the
    /// alphabetical order the reads are observable in.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_plain_date_time_read_fields(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        field_locals: &[u32; 9],
        present_locals: &[u32; 9],
        month_code_payload_local: u32,
        month_code_present_local: u32,
        any_present_local: u32,
        read_calendar: bool,
        function: &mut Function,
    ) -> Result<TemporalEraLocals, EmitError> {
        let mut era_slots = Some(self.reserve_temporal_era_slots());
        let mut era = None;
        let property_key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let parsed_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(any_present_local));
        if read_calendar {
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
            self.emit_temporal_to_temporal_calendar_identifier(
                calendar_payload_local,
                calendar_tag_local,
                "Temporal.PlainDateTime calendar must be a string",
                function,
            )?;
        }

        for key in TemporalDateTimeFieldKey::ALL {
            let index = match key.read() {
                TemporalDateTimeFieldRead::MonthCode => {
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
                    function.instruction(&Instruction::LocalGet(month_code_present_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(any_present_local));
                    function.instruction(&Instruction::End);
                    self.emit_temporal_property_bag_string(
                        value_payload_local,
                        value_tag_local,
                        "Temporal.PlainDate monthCode must be a string",
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(value_payload_local));
                    function.instruction(&Instruction::LocalSet(month_code_payload_local));
                    continue;
                }
                TemporalDateTimeFieldRead::EraPair => {
                    let slots = era_slots
                        .take()
                        .expect("TemporalDateTimeFieldKey::ALL reads the era pair exactly once");
                    let read = self.emit_temporal_read_era_fields(
                        slots,
                        argument_payload_local,
                        argument_tag_local,
                        calendar_payload_local,
                        function,
                    )?;
                    // `with` needs a supplied `era`/`eraYear` to count as a
                    // field, or `instance.with({ era: "bce", eraYear: 1 })`
                    // dies on "requires at least one date or time field"
                    // before `CalendarResolveFields` ever sees the pair.
                    for local in read.present_locals() {
                        function.instruction(&Instruction::LocalGet(local));
                        function.instruction(&Instruction::I64Eqz);
                        function.instruction(&Instruction::I32Eqz);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        function.instruction(&Instruction::I64Const(1));
                        function.instruction(&Instruction::LocalSet(any_present_local));
                        function.instruction(&Instruction::End);
                    }
                    era = Some(read);
                    continue;
                }
                TemporalDateTimeFieldRead::PositiveInteger { property, index } => {
                    self.emit_temporal_property_bag_positive_integer(
                        argument_payload_local,
                        argument_tag_local,
                        property,
                        property_key_local,
                        value_payload_local,
                        value_tag_local,
                        present_local,
                        parsed_local,
                        0,
                        "Temporal.PlainDateTime fields must be finite",
                        "Temporal.PlainDateTime month and day must be positive",
                        function,
                    )?;
                    index
                }
                TemporalDateTimeFieldRead::Integer { property, index } => {
                    self.emit_temporal_property_bag_integer(
                        argument_payload_local,
                        argument_tag_local,
                        property,
                        property_key_local,
                        value_payload_local,
                        value_tag_local,
                        present_local,
                        parsed_local,
                        0,
                        "Temporal.PlainDateTime fields must be finite",
                        function,
                    )?;
                    index
                }
            };
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::LocalSet(present_locals[index]));
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(parsed_local));
            function.instruction(&Instruction::LocalSet(field_locals[index]));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(any_present_local));
            function.instruction(&Instruction::End);
        }

        for local in [
            parsed_local,
            present_local,
            value_tag_local,
            value_payload_local,
            property_key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(era.expect("TemporalDateTimeFieldKey::ALL reads the era pair exactly once"))
    }

    /// `ToTemporalDateTime`. Accepts a branded `Temporal.PlainDateTime`
    /// (cloned), a branded `Temporal.PlainDate` (at midnight), any other object
    /// (read as a property bag) or an ISO string.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_to_temporal_date_time(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        options_payload_local: u32,
        options_tag_local: u32,
        read_options: bool,
        field_locals: &[u32; 9],
        calendar_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let any_present_local = self.reserve_temp_local();
        let present_locals = self.reserve_temporal_plain_date_time_field_locals();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("iso8601")));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        for local in field_locals.iter() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(*local));
        }
        for local in present_locals.iter() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(*local));
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(month_code_present_local));

        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_date_time_brand_check_i32(
            argument_payload_local,
            argument_tag_local,
            brand_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_temporal_plain_date_time_load_record(
            record_local,
            field_locals,
            calendar_payload_local,
            function,
        );
        if read_options {
            self.emit_temporal_plain_date_time_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, field_locals[0]),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, field_locals[1]),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, field_locals[2]),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        if read_options {
            self.emit_temporal_plain_date_time_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let era = self.emit_temporal_plain_date_time_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            field_locals,
            &present_locals,
            month_code_payload_local,
            month_code_present_local,
            any_present_local,
            true,
            function,
        )?;
        if read_options {
            self.emit_temporal_plain_date_time_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        let resolved_year = self.emit_temporal_resolve_era_to_year(
            era,
            calendar_payload_local,
            field_locals[0],
            present_locals[0],
            function,
        )?;
        self.emit_temporal_plain_date_resolve_fields(
            &resolved_year,
            field_locals[1],
            present_locals[1],
            month_code_payload_local,
            month_code_present_local,
            field_locals[2],
            present_locals[2],
            overflow_local,
            function,
        )?;
        let time_locals = Self::temporal_plain_date_time_time_locals(field_locals);
        self.emit_temporal_regulate_time(&time_locals, overflow_local, function)?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDateTime expects a string, a property bag, or a Temporal.PlainDateTime",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_parse_plain_date_time_string(
            argument_payload_local,
            field_locals[0],
            field_locals[1],
            field_locals[2],
            field_locals[3],
            field_locals[4],
            field_locals[5],
            record_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        // The parser hands back one nanosecond count for the whole fraction.
        for (index, divisor) in [(8_usize, 1_000_i64), (7, 1_000), (6, 1_000)] {
            function.instruction(&Instruction::LocalGet(record_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64RemS);
            function.instruction(&Instruction::LocalSet(field_locals[index]));
            function.instruction(&Instruction::LocalGet(record_local));
            function.instruction(&Instruction::I64Const(divisor));
            function.instruction(&Instruction::I64DivS);
            function.instruction(&Instruction::LocalSet(record_local));
        }
        if read_options {
            self.emit_temporal_plain_date_time_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        self.emit_temporal_reject_iso_date(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            function,
        )?;
        function.instruction(&Instruction::End);

        // Applies to every branch above — property bag, `PlainDate`,
        // `PlainDateTime` and string all land here with the ISO fields resolved.
        self.emit_temporal_reject_date_time_lower_bound(field_locals, function)?;

        self.release_temporal_plain_date_time_field_locals(present_locals);
        for local in [
            any_present_local,
            month_code_present_local,
            month_code_payload_local,
            handled_local,
            record_local,
            brand_local,
            overflow_local,
            calendar_tag_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.2.2 `Temporal.PlainDateTime.from`.
    pub(crate) fn emit_temporal_plain_date_time_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_to_temporal_date_time(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            true,
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            None,
            function,
        )?;

        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            calendar_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `CompareISODateTime` over the nine fields.
    fn emit_temporal_plain_date_time_compare_fields(
        &mut self,
        left: &[u32; 9],
        right: &[u32; 9],
        comparison_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(comparison_local));
        for index in 0..9 {
            function.instruction(&Instruction::LocalGet(comparison_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(left[index]));
            function.instruction(&Instruction::LocalGet(right[index]));
            function.instruction(&Instruction::I64LtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(-1));
            function.instruction(&Instruction::LocalSet(comparison_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(left[index]));
            function.instruction(&Instruction::LocalGet(right[index]));
            function.instruction(&Instruction::I64GtS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(comparison_local));
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
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let comparison_local = self.reserve_temp_local();
        let left_locals = self.reserve_temporal_plain_date_time_field_locals();
        let right_locals = self.reserve_temporal_plain_date_time_field_locals();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        for (index, locals) in [(0_usize, &left_locals), (1, &right_locals)] {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            self.emit_to_temporal_date_time(
                argument_payload_local,
                argument_tag_local,
                undefined_payload_local,
                undefined_tag_local,
                false,
                locals,
                calendar_payload_local,
                function,
            )?;
        }
        self.emit_temporal_plain_date_time_compare_fields(
            &left_locals,
            &right_locals,
            comparison_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(comparison_local));
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_plain_date_time_field_locals(right_locals);
        self.release_temporal_plain_date_time_field_locals(left_locals);
        for local in [
            comparison_local,
            calendar_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `equals`.
    pub(crate) fn emit_temporal_plain_date_time_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();
        let comparison_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        let other_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_to_temporal_date_time(
            argument_payload_local,
            argument_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            false,
            &other_locals,
            other_calendar_payload_local,
            function,
        )?;
        self.emit_temporal_plain_date_time_compare_fields(
            &field_locals,
            &other_locals,
            comparison_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(comparison_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.emit_string_payload_equality_i32(
            calendar_payload_local,
            other_calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_plain_date_time_field_locals(other_locals);
        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            comparison_local,
            other_calendar_payload_local,
            calendar_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `with`.
    pub(crate) fn emit_temporal_plain_date_time_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let any_present_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        let present_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDateTime.prototype.with requires an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // `IsPartialTemporalObject` step 2 runs before the two `Get`s below.
        self.emit_temporal_reject_branded_partial_object(
            argument_payload_local,
            argument_tag_local,
            "Temporal.PlainDateTime.prototype.with does not accept a Temporal object",
            function,
        )?;

        // `RejectTemporalLikeObject`: a bag that names a calendar or a time
        // zone is a caller mistake, not a partial date-time.
        for property in ["calendar", "timeZone"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(property)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_own_property_present(
                argument_payload_local,
                argument_tag_local,
                key_local,
                present_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Temporal.PlainDateTime.prototype.with does not accept calendar or timeZone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        for local in present_locals.iter() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(*local));
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(month_code_present_local));
        let era = self.emit_temporal_plain_date_time_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            &field_locals,
            &present_locals,
            month_code_payload_local,
            month_code_present_local,
            any_present_local,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(any_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDateTime.prototype.with requires at least one date or time field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_plain_date_time_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
            function,
        )?;

        // Era resolution before the merge below: `{ era, eraYear }` excludes
        // the receiver's `year`, which is still sitting untouched in
        // `field_locals[0]` because `read_fields` only overwrites a slot the
        // bag actually supplied. `present_locals[0]` is therefore still 0 for
        // an era-only bag, so the era/year agreement check cannot fire against
        // a year the caller never wrote.
        let resolved_year = self.emit_temporal_resolve_era_to_year(
            era,
            calendar_payload_local,
            field_locals[0],
            present_locals[0],
            function,
        )?;

        // `CalendarMergeFields` drops the receiver's `monthCode` as soon as the
        // argument supplies either `month` or `monthCode`, so a lone `month` is
        // never a conflict; every other absent key keeps the receiver's value,
        // which `emit_temporal_plain_date_time_read_fields` already left in
        // place.
        function.instruction(&Instruction::LocalGet(present_locals[1]));
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(present_locals[1]));
        function.instruction(&Instruction::End);
        for index in [0_usize, 2] {
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(present_locals[index]));
        }

        self.emit_temporal_plain_date_resolve_fields(
            &resolved_year,
            field_locals[1],
            present_locals[1],
            month_code_payload_local,
            month_code_present_local,
            field_locals[2],
            present_locals[2],
            overflow_local,
            function,
        )?;
        let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
        self.emit_temporal_regulate_time(&time_locals, overflow_local, function)?;
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            None,
            function,
        )?;

        self.release_temporal_plain_date_time_field_locals(present_locals);
        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            any_present_local,
            month_code_present_local,
            month_code_payload_local,
            present_local,
            key_local,
            overflow_local,
            calendar_tag_local,
            calendar_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `withPlainTime`. An absent argument means
    /// midnight, not "keep the current time".
    pub(crate) fn emit_temporal_plain_date_time_with_plain_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        let time_locals = self.reserve_temporal_plain_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        for local in time_locals.iter() {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(*local));
        }
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_to_temporal_time(
            argument_payload_local,
            argument_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            false,
            &time_locals,
            function,
        )?;
        function.instruction(&Instruction::End);
        for index in 0..6 {
            function.instruction(&Instruction::LocalGet(time_locals[index]));
            function.instruction(&Instruction::LocalSet(field_locals[index + 3]));
        }
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            None,
            function,
        )?;

        self.release_temporal_plain_time_field_locals(time_locals);
        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            calendar_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `withCalendar`.
    pub(crate) fn emit_temporal_plain_date_time_with_calendar(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let existing_calendar_payload_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            existing_calendar_payload_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, calendar_payload_local, calendar_tag_local, function);
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDateTime calendar must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_temporal_to_temporal_calendar_identifier(
            calendar_payload_local,
            calendar_tag_local,
            "Temporal.PlainDateTime calendar must be a string",
            function,
        )?;
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            None,
            function,
        )?;

        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            calendar_tag_local,
            calendar_payload_local,
            existing_calendar_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `toPlainDate` and `toPlainTime`.
    pub(crate) fn emit_temporal_plain_date_time_to_component(
        &mut self,
        time: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar_payload_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        if time {
            let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
            self.emit_alloc_temporal_plain_time(&time_locals, None, function)?;
        } else {
            function.instruction(&Instruction::GlobalGet(
                TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            self.emit_alloc_temporal_plain_date(
                field_locals[0],
                field_locals[1],
                field_locals[2],
                calendar_payload_local,
                prototype_payload_local,
                function,
            )?;
        }

        self.release_temporal_plain_date_time_field_locals(field_locals);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(calendar_payload_local);
        Ok(())
    }

    /// Temporal proposal 5.3.x `add` and `subtract`, both through
    /// `AddDurationToDateTime`.
    pub(crate) fn emit_temporal_plain_date_time_add_or_subtract(
        &mut self,
        subtract: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let total_local = self.reserve_temp_local();
        let day_delta_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_to_temporal_duration(
            argument_payload_local,
            argument_tag_local,
            &duration_locals,
            function,
        )?;
        self.emit_temporal_plain_date_time_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
            function,
        )?;
        if subtract {
            for local in duration_locals.iter() {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(*local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(*local));
            }
        }

        // Hours and below fold into a nanosecond offset; the whole days that
        // fall out of it join the duration's own day count.
        self.emit_temporal_duration_normalize_seconds(
            &duration_locals,
            TemporalUnit::Hour,
            seconds_local,
            subsecond_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(day_delta_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::LocalGet(day_delta_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
        self.emit_temporal_plain_time_total_nanoseconds(&time_locals, total_local, function);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(subsecond_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_local));
        self.emit_temporal_split_days_and_nanoseconds(total_local, seconds_local, function);
        function.instruction(&Instruction::LocalGet(day_delta_local));
        function.instruction(&Instruction::LocalGet(seconds_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(duration_locals[3]));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(day_delta_local));
        self.emit_temporal_plain_time_from_nanoseconds(total_local, &time_locals, function);
        for index in 0..6 {
            function.instruction(&Instruction::LocalGet(time_locals[index]));
            function.instruction(&Instruction::LocalSet(field_locals[index + 3]));
        }
        self.emit_temporal_add_iso_date(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            duration_locals[0],
            duration_locals[1],
            duration_locals[2],
            day_delta_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_reject_date_time_lower_bound(&field_locals, function)?;
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            None,
            function,
        )?;

        self.release_temporal_duration_field_locals(duration_locals);
        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            day_delta_local,
            total_local,
            subsecond_local,
            seconds_local,
            calendar_payload_local,
            overflow_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `RoundRelativeDuration` for a calendar `smallestUnit` (`year`, `month`
    /// or `week`) in `until`/`since`.
    ///
    /// A calendar unit has no fixed nanosecond length, so the difference cannot
    /// be rounded by dividing. The proposal instead dates both candidates: the
    /// truncated duration `r1` already in `years/months/weeks`, and `r2`, one
    /// `increment` of `smallestUnit` further in the direction of travel. Adding
    /// each to the receiver gives two instants that bracket the real end
    /// instant, and the rounding mode is applied to where the end falls between
    /// them. `PlainDateTime` is its own `relativeTo`, so no anchor argument is
    /// needed.
    ///
    /// The fraction is measured in nanoseconds relative to `r1`'s instant
    /// rather than in absolute epoch nanoseconds: an absolute count at the ends
    /// of the representable range overflows `i64`, while the bracket is at most
    /// a few years wide.
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_plain_date_time_nudge_calendar_unit(
        &mut self,
        field_locals: &[u32; 9],
        other_locals: &[u32; 9],
        smallest_unit_local: u32,
        increment_local: u32,
        mode_local: u32,
        years_local: u32,
        months_local: u32,
        weeks_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let overflow_local = self.reserve_temp_local();
        let sign_local = self.reserve_temp_local();
        let step_local = self.reserve_temp_local();
        let receiver_time_local = self.reserve_temp_local();
        let other_time_local = self.reserve_temp_local();
        let receiver_epoch_local = self.reserve_temp_local();
        let other_epoch_local = self.reserve_temp_local();
        let start_epoch_local = self.reserve_temp_local();
        let end_epoch_local = self.reserve_temp_local();
        let numerator_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let start_year_local = self.reserve_temp_local();
        let start_month_local = self.reserve_temp_local();
        let start_day_local = self.reserve_temp_local();
        let end_year_local = self.reserve_temp_local();
        let end_month_local = self.reserve_temp_local();
        let end_day_local = self.reserve_temp_local();
        let nudge_years_local = self.reserve_temp_local();
        let nudge_months_local = self.reserve_temp_local();
        let nudge_weeks_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));

        let receiver_time_locals = Self::temporal_plain_date_time_time_locals(field_locals);
        let other_time_locals = Self::temporal_plain_date_time_time_locals(other_locals);
        self.emit_temporal_plain_time_total_nanoseconds(
            &receiver_time_locals,
            receiver_time_local,
            function,
        );
        self.emit_temporal_plain_time_total_nanoseconds(
            &other_time_locals,
            other_time_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            receiver_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            other_locals[0],
            other_locals[1],
            other_locals[2],
            other_epoch_local,
            function,
        );

        // `DurationSign` of the untruncated difference: compare the dates
        // first, then the wall-clock times when the dates match.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::LocalGet(other_epoch_local));
        function.instruction(&Instruction::LocalGet(receiver_epoch_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(other_time_local));
        function.instruction(&Instruction::LocalGet(receiver_time_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(other_time_local));
        function.instruction(&Instruction::LocalGet(receiver_time_local));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(other_epoch_local));
        function.instruction(&Instruction::LocalGet(receiver_epoch_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(sign_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(step_local));
        for (source, destination) in [
            (years_local, nudge_years_local),
            (months_local, nudge_months_local),
            (weeks_local, nudge_weeks_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        for (unit, local) in [
            (TemporalUnit::Year.code(), nudge_years_local),
            (TemporalUnit::Month.code(), nudge_months_local),
            (TemporalUnit::Week.code(), nudge_weeks_local),
        ] {
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(unit));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::LocalGet(step_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
        }

        for (year, month, day) in [
            (start_year_local, start_month_local, start_day_local),
            (end_year_local, end_month_local, end_day_local),
        ] {
            for (source, destination) in [
                (field_locals[0], year),
                (field_locals[1], month),
                (field_locals[2], day),
            ] {
                function.instruction(&Instruction::LocalGet(source));
                function.instruction(&Instruction::LocalSet(destination));
            }
        }
        self.emit_temporal_add_iso_date(
            start_year_local,
            start_month_local,
            start_day_local,
            years_local,
            months_local,
            weeks_local,
            zero_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_add_iso_date(
            end_year_local,
            end_month_local,
            end_day_local,
            nudge_years_local,
            nudge_months_local,
            nudge_weeks_local,
            zero_local,
            overflow_local,
            function,
        )?;
        self.emit_temporal_plain_date_epoch_days(
            start_year_local,
            start_month_local,
            start_day_local,
            start_epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            end_year_local,
            end_month_local,
            end_day_local,
            end_epoch_local,
            function,
        );

        // numerator: end instant minus `r1`'s instant. quantum: the width of
        // the bracket, always a whole number of days because both candidates
        // keep the receiver's wall-clock time.
        function.instruction(&Instruction::LocalGet(other_epoch_local));
        function.instruction(&Instruction::LocalGet(start_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(other_time_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(receiver_time_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(numerator_local));
        function.instruction(&Instruction::LocalGet(end_epoch_local));
        function.instruction(&Instruction::LocalGet(start_epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(sign_local));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(quantum_local));

        function.instruction(&Instruction::LocalGet(quantum_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_round_nanoseconds(
            numerator_local,
            quantum_local,
            mode_local,
            function,
        );
        // The end instant lies inside the bracket, so the rounded value is
        // either zero (keep `r1`) or the whole bracket (take `r2`).
        function.instruction(&Instruction::LocalGet(numerator_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (source, destination) in [
            (nudge_years_local, years_local),
            (nudge_months_local, months_local),
            (nudge_weeks_local, weeks_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            zero_local,
            nudge_weeks_local,
            nudge_months_local,
            nudge_years_local,
            end_day_local,
            end_month_local,
            end_year_local,
            start_day_local,
            start_month_local,
            start_year_local,
            quantum_local,
            numerator_local,
            end_epoch_local,
            start_epoch_local,
            other_epoch_local,
            receiver_epoch_local,
            other_time_local,
            receiver_time_local,
            step_local,
            sign_local,
            overflow_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `RoundISODateTime`: round the wall-clock time to a whole number of
    /// `quantum_local` nanoseconds and carry any whole day into the date.
    fn emit_temporal_round_iso_date_time(
        &mut self,
        field_locals: &[u32; 9],
        quantum_local: u32,
        mode_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let total_local = self.reserve_temp_local();
        let day_delta_local = self.reserve_temp_local();
        let epoch_local = self.reserve_temp_local();
        let time_locals = Self::temporal_plain_date_time_time_locals(field_locals);

        self.emit_temporal_plain_time_total_nanoseconds(&time_locals, total_local, function);
        self.emit_temporal_plain_time_round_nanoseconds(
            total_local,
            quantum_local,
            mode_local,
            function,
        );
        self.emit_temporal_split_days_and_nanoseconds(total_local, day_delta_local, function);
        self.emit_temporal_plain_time_from_nanoseconds(total_local, &time_locals, function);
        for index in 0..6 {
            function.instruction(&Instruction::LocalGet(time_locals[index]));
            function.instruction(&Instruction::LocalSet(field_locals[index + 3]));
        }
        function.instruction(&Instruction::LocalGet(day_delta_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_date_epoch_days(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            epoch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::LocalGet(day_delta_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(epoch_local));
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
        function.instruction(&Instruction::End);

        // Outside the day-carry branch: rounding down to the minimum day's
        // midnight leaves the date untouched but still leaves the range, and
        // `toString` reads the rounded fields without going through
        // `CreateTemporalDateTime`.
        self.emit_temporal_reject_date_time_lower_bound(field_locals, function)?;

        for local in [epoch_local, day_delta_local, total_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `round`. `smallestUnit` runs from `day` down to
    /// `nanosecond`; a day increment must be exactly 1.
    pub(crate) fn emit_temporal_plain_date_time_round(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDateTime.prototype.round requires a roundTo argument",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(increment_local));
        function.instruction(&Instruction::I64Const(
            TemporalRoundingMode::HalfExpand.code(),
        ));
        function.instruction(&Instruction::LocalSet(mode_local));
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_unit_from_payload(
            argument_payload_local,
            unit_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_temporal_duration_options_object(
            argument_payload_local,
            argument_tag_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(
            argument_payload_local,
            argument_tag_local,
            increment_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            argument_payload_local,
            argument_tag_local,
            TemporalRoundingMode::HalfExpand,
            mode_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            argument_payload_local,
            argument_tag_local,
            "smallestUnit",
            false,
            unit_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDateTime.prototype.round requires smallestUnit",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_temporal_require_unit_range(
            unit_local,
            TemporalUnit::Day,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.PlainDateTime unit option",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainDateTime rounding increment",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::LocalSet(quantum_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_plain_time_validate_increment(unit_local, increment_local, function)?;
        self.emit_temporal_plain_time_rounding_quantum(
            unit_local,
            increment_local,
            quantum_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_temporal_round_iso_date_time(&field_locals, quantum_local, mode_local, function)?;
        self.emit_alloc_temporal_plain_date_time(
            &field_locals,
            calendar_payload_local,
            None,
            function,
        )?;

        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            calendar_payload_local,
            quantum_local,
            mode_local,
            increment_local,
            unit_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `until` and `since`, both through
    /// `DifferencePlainDateTimeWithRounding`.
    pub(crate) fn emit_temporal_plain_date_time_until_or_since(
        &mut self,
        since: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();
        let largest_unit_local = self.reserve_temp_local();
        let smallest_unit_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let original_mode_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let total_local = self.reserve_temp_local();
        let other_total_local = self.reserve_temp_local();
        let date_sign_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let years_local = self.reserve_temp_local();
        let months_local = self.reserve_temp_local();
        let weeks_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let adjusted_year_local = self.reserve_temp_local();
        let adjusted_month_local = self.reserve_temp_local();
        let adjusted_day_local = self.reserve_temp_local();
        let epoch_local = self.reserve_temp_local();
        let time_largest_unit_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        let other_locals = self.reserve_temporal_plain_date_time_field_locals();
        let duration_locals = self.reserve_temporal_duration_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_to_temporal_date_time(
            argument_payload_local,
            argument_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            false,
            &other_locals,
            other_calendar_payload_local,
            function,
        )?;
        // `DifferenceTemporalPlainDateTime` step 2: `CalendarEquals` runs
        // between `ToTemporalDateTime` and `GetOptionsObject`, which is what
        // `since/different-calendars-throws.js` and its `until` twin pin.
        self.emit_temporal_require_same_calendar(
            calendar_payload_local,
            other_calendar_payload_local,
            "Temporal.PlainDateTime until and since require the same calendar",
            function,
        )?;

        // `GetDifferenceSettings` reads largestUnit, then the two rounding
        // options, then smallestUnit - the order is observable.
        self.emit_temporal_duration_options_object(
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_temporal_duration_unit_option(
            options_payload_local,
            options_tag_local,
            "largestUnit",
            true,
            largest_unit_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_increment_option(
            options_payload_local,
            options_tag_local,
            increment_local,
            function,
        )?;
        self.emit_temporal_duration_rounding_mode_option(
            options_payload_local,
            options_tag_local,
            TemporalRoundingMode::Trunc,
            mode_local,
            function,
        )?;
        if since {
            // `NegateRoundingMode`: ceil and floor swap, as do halfCeil and
            // halfFloor; the sign-symmetric modes are unchanged.
            function.instruction(&Instruction::LocalGet(mode_local));
            function.instruction(&Instruction::LocalSet(original_mode_local));
            for mode in TemporalRoundingMode::ALL {
                if mode.negated() == mode {
                    continue;
                }
                function.instruction(&Instruction::LocalGet(original_mode_local));
                function.instruction(&Instruction::I64Const(mode.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(mode.negated().code()));
                function.instruction(&Instruction::LocalSet(mode_local));
                function.instruction(&Instruction::End);
            }
        }
        self.emit_temporal_duration_unit_option(
            options_payload_local,
            options_tag_local,
            "smallestUnit",
            false,
            smallest_unit_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
        function.instruction(&Instruction::LocalSet(smallest_unit_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            smallest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.PlainDateTime unit option",
            function,
        )?;
        // An unset or `"auto"` largestUnit falls back to the larger of day and
        // the smallest unit.
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnitSlot::Auto.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(largest_unit_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_require_unit_range(
            largest_unit_local,
            TemporalUnit::Year,
            TemporalUnit::Nanosecond,
            "Invalid Temporal.PlainDateTime unit option",
            function,
        )?;
        self.emit_temporal_require_largest_not_smaller(
            largest_unit_local,
            smallest_unit_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_validate_increment(
            smallest_unit_local,
            increment_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
        let other_time_locals = Self::temporal_plain_date_time_time_locals(&other_locals);
        self.emit_temporal_plain_time_total_nanoseconds(&time_locals, total_local, function);
        self.emit_temporal_plain_time_total_nanoseconds(
            &other_time_locals,
            other_total_local,
            function,
        );

        for local in [years_local, months_local, weeks_local, days_local] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }

        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Hour and below: the whole difference is a nanosecond count.
        self.emit_temporal_plain_date_epoch_days(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            epoch_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            other_locals[0],
            other_locals[1],
            other_locals[2],
            adjusted_day_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(adjusted_day_local));
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(other_total_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::LocalSet(time_largest_unit_local));
        function.instruction(&Instruction::Else);
        // Day and above: borrow a day when the time-of-day difference runs
        // against the date difference, then take a calendar difference.
        self.emit_temporal_compare_iso_date(
            [other_locals[0], other_locals[1], other_locals[2]],
            [field_locals[0], field_locals[1], field_locals[2]],
            date_sign_local,
            function,
        );
        for (source, destination) in [
            (other_locals[0], adjusted_year_local),
            (other_locals[1], adjusted_month_local),
            (other_locals[2], adjusted_day_local),
        ] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(destination));
        }
        function.instruction(&Instruction::LocalGet(other_total_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_date_epoch_days(
            adjusted_year_local,
            adjusted_month_local,
            adjusted_day_local,
            epoch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(epoch_local));
        self.emit_temporal_civil_from_days(
            epoch_local,
            adjusted_year_local,
            adjusted_month_local,
            adjusted_day_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(date_sign_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_difference_iso_date(
            [field_locals[0], field_locals[1], field_locals[2]],
            [
                adjusted_year_local,
                adjusted_month_local,
                adjusted_day_local,
            ],
            largest_unit_local,
            years_local,
            months_local,
            weeks_local,
            days_local,
            function,
        );
        function.instruction(&Instruction::I64Const(TemporalUnit::Hour.code()));
        function.instruction(&Instruction::LocalSet(time_largest_unit_local));
        function.instruction(&Instruction::End);

        // Rounding. A time smallestUnit rounds the nanosecond tail and carries
        // a whole day into the day count; a `day` smallestUnit rounds the day
        // count itself; a calendar smallestUnit truncates, which is what an
        // unrounded difference already is.
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_plain_time_rounding_quantum(
            smallest_unit_local,
            increment_local,
            quantum_local,
            function,
        );
        self.emit_temporal_plain_time_round_nanoseconds(
            total_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(largest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(epoch_local));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::LocalGet(epoch_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(smallest_unit_local));
        function.instruction(&Instruction::I64Const(TemporalUnit::Day.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(total_local));
        function.instruction(&Instruction::LocalGet(increment_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(quantum_local));
        self.emit_temporal_plain_time_round_nanoseconds(
            total_local,
            quantum_local,
            mode_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(NANOSECONDS_PER_TEMPORAL_DAY));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(days_local));
        function.instruction(&Instruction::End);
        // A calendar smallestUnit drops everything below it, which is the
        // `trunc` answer and the lower of the two candidates every other mode
        // picks between.
        for (limit, local) in [
            (TemporalUnit::Week.code(), days_local),
            (TemporalUnit::Month.code(), weeks_local),
            (TemporalUnit::Year.code(), months_local),
        ] {
            function.instruction(&Instruction::LocalGet(smallest_unit_local));
            function.instruction(&Instruction::I64Const(limit));
            function.instruction(&Instruction::I64LeS);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(total_local));
        self.emit_temporal_plain_date_time_nudge_calendar_unit(
            &field_locals,
            &other_locals,
            smallest_unit_local,
            increment_local,
            mode_local,
            years_local,
            months_local,
            weeks_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        if since {
            for local in [
                years_local,
                months_local,
                weeks_local,
                days_local,
                total_local,
            ] {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalGet(local));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalSet(local));
            }
        }
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(total_local));
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::LocalSet(subsecond_local));
        self.emit_temporal_duration_balance(
            seconds_local,
            subsecond_local,
            time_largest_unit_local,
            &duration_locals,
            function,
        )?;
        for (source, index) in [(years_local, 0_usize), (months_local, 1), (weeks_local, 2)] {
            function.instruction(&Instruction::LocalGet(source));
            function.instruction(&Instruction::LocalSet(duration_locals[index]));
        }
        function.instruction(&Instruction::LocalGet(duration_locals[3]));
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(duration_locals[3]));
        self.emit_create_temporal_duration(&duration_locals, function)?;

        self.release_temporal_duration_field_locals(duration_locals);
        self.release_temporal_plain_date_time_field_locals(other_locals);
        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            time_largest_unit_local,
            epoch_local,
            adjusted_day_local,
            adjusted_month_local,
            adjusted_year_local,
            days_local,
            weeks_local,
            months_local,
            years_local,
            subsecond_local,
            seconds_local,
            date_sign_local,
            other_total_local,
            total_local,
            quantum_local,
            original_mode_local,
            mode_local,
            increment_local,
            smallest_unit_local,
            largest_unit_local,
            other_calendar_payload_local,
            calendar_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
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
        let calendar_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();
        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        self.release_temporal_plain_date_time_field_locals(field_locals);
        self.release_temp_local(calendar_payload_local);
        self.emit_intl_dtf_temporal_to_locale_string(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME,
            function,
        )
    }

    /// `TemporalDateTimeToString`. `builtin` selects whether the option bag is
    /// read: `toString` reads it and `toJSON` is fixed at `auto` precision and
    /// `auto` calendar name.
    pub(crate) fn emit_temporal_plain_date_time_to_string(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let show_calendar_local = self.reserve_temp_local();
        let digits_local = self.reserve_temp_local();
        let unit_local = self.reserve_temp_local();
        let mode_local = self.reserve_temp_local();
        let precision_local = self.reserve_temp_local();
        let increment_local = self.reserve_temp_local();
        let quantum_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(TEMPORAL_PRECISION_AUTO));
        function.instruction(&Instruction::LocalSet(precision_local));
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        function.instruction(&Instruction::LocalSet(show_calendar_local));
        if matches!(
            builtin,
            StandardBuiltinId::TemporalPlainDateTimePrototypeToString
        ) {
            self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
            self.emit_temporal_duration_options_object(
                options_payload_local,
                options_tag_local,
                function,
            )?;
            self.emit_temporal_string_valued_option::<ShowCalendarName>(
                options_payload_local,
                options_tag_local,
                show_calendar_local,
                "Temporal.PlainDateTime options must be an object or undefined",
                "Invalid Temporal.PlainDateTime calendarName option",
                function,
            )?;
            self.emit_temporal_plain_time_fractional_digits_option(
                options_payload_local,
                options_tag_local,
                digits_local,
                function,
            )?;
            self.emit_temporal_duration_rounding_mode_option(
                options_payload_local,
                options_tag_local,
                TemporalRoundingMode::Trunc,
                mode_local,
                function,
            )?;
            self.emit_temporal_duration_unit_option(
                options_payload_local,
                options_tag_local,
                "smallestUnit",
                false,
                unit_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(unit_local));
            function.instruction(&Instruction::I64Const(TemporalUnitSlot::Unset.code()));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_temporal_require_unit_range(
                unit_local,
                TemporalUnit::Minute,
                TemporalUnit::Nanosecond,
                "Invalid Temporal.PlainDateTime unit option",
                function,
            )?;
            for (unit, precision) in [
                (TemporalUnit::Minute, TEMPORAL_PRECISION_MINUTE),
                (TemporalUnit::Second, 0),
                (TemporalUnit::Millisecond, 3),
                (TemporalUnit::Microsecond, 6),
                (TemporalUnit::Nanosecond, 9),
            ] {
                function.instruction(&Instruction::LocalGet(unit_local));
                function.instruction(&Instruction::I64Const(unit.code()));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(precision));
                function.instruction(&Instruction::LocalSet(precision_local));
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(increment_local));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(digits_local));
            function.instruction(&Instruction::LocalSet(precision_local));
            function.instruction(&Instruction::I64Const(TemporalUnit::Nanosecond.code()));
            function.instruction(&Instruction::LocalSet(unit_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(increment_local));
            for (low, high, unit, scale) in [
                (0_i64, 0_i64, TemporalUnit::Second, 0_i64),
                (1, 3, TemporalUnit::Millisecond, 3),
                (4, 6, TemporalUnit::Microsecond, 6),
                (7, 9, TemporalUnit::Nanosecond, 9),
            ] {
                function.instruction(&Instruction::LocalGet(digits_local));
                function.instruction(&Instruction::I64Const(low));
                function.instruction(&Instruction::I64GeS);
                function.instruction(&Instruction::LocalGet(digits_local));
                function.instruction(&Instruction::I64Const(high));
                function.instruction(&Instruction::I64LeS);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(unit.code()));
                function.instruction(&Instruction::LocalSet(unit_local));
                for digits in low..=high {
                    function.instruction(&Instruction::LocalGet(digits_local));
                    function.instruction(&Instruction::I64Const(digits));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function
                        .instruction(&Instruction::I64Const(10_i64.pow((scale - digits) as u32)));
                    function.instruction(&Instruction::LocalSet(increment_local));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);

            self.emit_temporal_plain_time_rounding_quantum(
                unit_local,
                increment_local,
                quantum_local,
                function,
            );
            self.emit_temporal_round_iso_date_time(
                &field_locals,
                quantum_local,
                mode_local,
                function,
            )?;
        }

        self.emit_temporal_iso_date_string(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("T")));
        function.instruction(&Instruction::LocalSet(piece_payload_local));
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));
        let time_locals = Self::temporal_plain_date_time_time_locals(&field_locals);
        self.emit_temporal_plain_time_record_to_string(
            &time_locals,
            precision_local,
            piece_payload_local,
            function,
        )?;
        self.emit_concat_string_payloads_local(
            output_payload_local,
            piece_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(output_payload_local));

        // `FormatCalendarAnnotation`, shared with the three date-only types so
        // the `auto` suppression rule is decided in one place.
        self.emit_temporal_append_calendar_annotation(
            show_calendar_local,
            calendar_payload_local,
            output_payload_local,
            piece_payload_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(output_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            number_payload_local,
            piece_payload_local,
            output_payload_local,
            calendar_payload_local,
            quantum_local,
            increment_local,
            precision_local,
            mode_local,
            unit_local,
            digits_local,
            show_calendar_local,
            options_tag_local,
            options_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 5.3.x `toZonedDateTime`. Only `UTC` and fixed numeric
    /// offsets resolve in this backend, which is the same limit
    /// `Temporal.ZonedDateTime` itself carries.
    pub(crate) fn emit_temporal_plain_date_time_to_zoned_date_time(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let time_zone_payload_local = self.reserve_temp_local();
        let time_zone_tag_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let offset_seconds_local = self.reserve_temp_local();
        let seconds_local = self.reserve_temp_local();
        let subsecond_local = self.reserve_temp_local();
        let epoch_payload_local = self.reserve_temp_local();
        let epoch_tag_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let field_locals = self.reserve_temporal_plain_date_time_field_locals();

        self.emit_temporal_plain_date_time_fields_from_receiver(
            &field_locals,
            calendar_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
        self.emit_builtin_arg_to_locals(0, time_zone_payload_local, time_zone_tag_local, function);
        function.instruction(&Instruction::LocalGet(time_zone_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDateTime.prototype.toZonedDateTime requires a time zone",
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
        self.emit_temporal_fixed_time_zone_offset_seconds(
            time_zone_payload_local,
            offset_seconds_local,
            function,
        )?;

        self.emit_temporal_plain_date_epoch_days(
            field_locals[0],
            field_locals[1],
            field_locals[2],
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(86_400));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(field_locals[3]));
        function.instruction(&Instruction::I64Const(3_600));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(field_locals[4]));
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(field_locals[5]));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(offset_seconds_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(seconds_local));
        function.instruction(&Instruction::LocalGet(field_locals[6]));
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalGet(field_locals[7]));
        function.instruction(&Instruction::I64Const(1_000));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(field_locals[8]));
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

        self.release_temporal_plain_date_time_field_locals(field_locals);
        for local in [
            prototype_payload_local,
            days_local,
            epoch_tag_local,
            epoch_payload_local,
            subsecond_local,
            seconds_local,
            offset_seconds_local,
            calendar_tag_local,
            calendar_payload_local,
            time_zone_tag_local,
            time_zone_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
