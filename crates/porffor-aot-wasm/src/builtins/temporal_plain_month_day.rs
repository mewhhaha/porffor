//! `Temporal.PlainMonthDay` codegen.
//!
//! Temporal proposal 10: a calendar month and day with a *reference* ISO year
//! that is not observable through any accessor. The record is the
//! `Temporal.PlainDate` one under a different brand, so every layout constant
//! and every ISO helper is shared rather than duplicated; only the field set,
//! the reference year and the string form differ.
//!
//! The reference year is 1972 - a leap year, so `--02-29` is representable.

use super::super::*;
use super::temporal_options::{ShowCalendarName, TemporalOverflow};
use super::temporal_plain_date::{MonthDayYearUse, TemporalCalendarId, TemporalResolvedYear};
use super::temporal_plain_year_month::{TemporalPartialDatePrototype, TemporalPartialDateType};

/// `ISO_REFERENCE_YEAR` from the proposal.
const TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR: i64 = 1972;

/// The `ISOYearMonthWithinLimits` year bounds, which
/// [`MonthDayYearUse::RangeChecked`] applies to a supplied `year`.
///
/// `temporal_plain_year_month.rs` names the same two values privately for its
/// own limit check. They are restated rather than shared because this file must
/// not edit a sibling module — the same reason
/// `TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR` is restated in
/// `temporal_plain_date_methods.rs`. A lane that owns both files should collapse
/// the three copies.
const TEMPORAL_MONTH_DAY_MINIMUM_YEAR: i64 = -271_821;
const TEMPORAL_MONTH_DAY_MAXIMUM_YEAR: i64 = 275_760;

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
/// Only then is the year replaced with
/// `TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR`. Both facts were previously
/// unrecoverable by the time the reference year was stored, and both checks were
/// simply absent; naming the pair is what stops them being droppable again.
///
/// The fields are private and there is no other constructor, so
/// `emit_temporal_parse_month_day_string` is the only thing outside this
/// module's own body that can produce one — the `ToTemporalCalendarIdentifier`
/// probe in `temporal.rs` shares the rewrite but cannot mint this. Be precise
/// about the limit: Rust privacy is per module, so a *third* function added to
/// this file could still write the literal. What the type buys is that the only
/// consumer, `emit_temporal_month_day_string_reference_year`, takes it by value
/// and is also the only thing that stores the reference year. A string path that
/// skips the checks therefore also skips the reference year and produces a
/// visibly wrong `[[ISOYear]]`, rather than a correct-looking `PlainMonthDay`
/// built from a string the spec rejects.
///
/// **Be precise about the enforcement level: this is a warning, not a compile
/// error, and the enforcing witness is a test.** `#[must_use]` fires on a
/// *discarded expression*; it does not fire on
/// `let parsed = self.emit_temporal_parse_month_day_string(...)?;` followed by
/// nothing. `cargo xc` is `check --workspace --all-targets` (`.cargo/config.toml`
/// `[alias]`) with no `-D warnings`, and this crate sets no `deny`, so a future
/// string path that reserves this struct and never feeds it to the consumer
/// compiles clean.
///
/// What actually catches that omission is
/// `built-ins/Temporal/PlainMonthDay/from/fields-string.js`: it calls
/// `TemporalHelpers.assertPlainMonthDay(plainMonthDay, "M10", 1, ...)` whose
/// fifth parameter defaults to `1972` and is asserted as
/// `Number(monthDay.toString({calendarName:"always"}).split("-")[0])`
/// (`harness/temporalHelpers.js:302-308`). A missing reference-year store fails
/// there, on the **ISO** path, i.e. in this lane's own target test — not only
/// through `equals`.
///
/// The compile-error version, if one is wanted later: make
/// `emit_alloc_temporal_partial_date` take a year *token* that only this
/// checker can mint, instead of a bare `u32` local. That moves the obligation to
/// the allocation site, which every path must reach.
#[must_use]
pub(crate) struct TemporalParsedMonthDayYear {
    year_local: u32,
    year_present_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    fn emit_temporal_month_day_overflow_option(
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
            "Temporal.PlainMonthDay options must be an object or undefined",
            "Invalid Temporal.PlainMonthDay overflow option",
            function,
        )
    }

    /// Temporal proposal 10.1.1:
    /// `Temporal.PlainMonthDay(isoMonth, isoDay [, calendar [, referenceISOYear]])`.
    pub(crate) fn emit_temporal_plain_month_day_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
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
            "Temporal.PlainMonthDay constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for (index, output_local, message) in [
            (
                0,
                month_local,
                "Temporal.PlainMonthDay month must be an integer",
            ),
            (
                1,
                day_local,
                "Temporal.PlainMonthDay day must be an integer",
            ),
        ] {
            self.emit_builtin_arg_to_locals(
                index,
                argument_payload_local,
                argument_tag_local,
                function,
            );
            self.emit_temporal_to_integer_with_truncation(
                argument_payload_local,
                argument_tag_local,
                output_local,
                message,
                function,
            )?;
        }
        self.emit_builtin_arg_to_locals(2, calendar_payload_local, calendar_tag_local, function);
        self.emit_temporal_plain_date_calendar(
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        ));
        function.instruction(&Instruction::LocalSet(year_local));
        self.emit_builtin_arg_to_locals(3, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_to_integer_with_truncation(
            argument_payload_local,
            argument_tag_local,
            year_local,
            "Temporal.PlainMonthDay reference year must be an integer",
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainMonthDay,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::FromNewTarget,
            function,
        )?;

        for local in [
            new_target_tag_local,
            new_target_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The `[[InitializedTemporalMonthDay]]` brand check.
    pub(crate) fn emit_temporal_plain_month_day_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_branded_record_from_receiver(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY,
            "Temporal.PlainMonthDay receiver does not have [[InitializedTemporalMonthDay]]",
            record_local,
            function,
        )
    }

    /// The three `Temporal.PlainMonthDay.prototype` accessors.
    pub(crate) fn emit_temporal_plain_month_day_field(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );

        match builtin {
            StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter => {
                function.instruction(&Instruction::LocalGet(calendar_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter => {
                self.emit_temporal_month_code_payload(month_local, function);
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            _ => {
                function.instruction(&Instruction::LocalGet(day_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }

        for local in [
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `RegulateISODate` without the `ISODateWithinLimits` tail. A month-day
    /// stores the reference year 1972, so a caller-supplied `year` only ever
    /// decides how 29 February constrains - Test262's
    /// `from/iso-year-used-only-for-overflow.js` pins that an out-of-range year
    /// must not throw.
    fn emit_temporal_month_day_regulate(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        function.instruction(&Instruction::LocalGet(overflow_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Reject.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainMonthDay is not a valid ISO date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);
        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::LocalSet(day_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    /// `CalendarMonthDayToISOReferenceDate`. The supplied year, if any, decides
    /// how 29 February is constrained; the stored year is always 1972.
    ///
    /// For a calendar whose [`TemporalCalendarId::month_day_year_use`] is
    /// [`MonthDayYearUse::RangeChecked`] the year is *also* bounded, before any
    /// month information is computed. That fork is real: `iso8601` accepts
    /// `year: -999999` and `gregory` rejects it. See [`MonthDayYearUse`].
    #[allow(clippy::too_many_arguments)]
    fn emit_temporal_month_day_resolve_fields(
        &mut self,
        resolved_year: &TemporalResolvedYear,
        calendar_payload_local: u32,
        month_local: u32,
        month_present_local: u32,
        month_code_payload_local: u32,
        month_code_present_local: u32,
        day_local: u32,
        day_present_local: u32,
        overflow_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let year_local = resolved_year.year_local();
        let year_present_local = resolved_year.year_present_local();
        let month_from_code_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay fields require day",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay fields require month or monthCode",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // The non-ISO year bound, before any month information is computed:
        // `intl402/.../PlainMonthDay/from/dont-calculate-month-info-for-out-of-range-year.js`.
        // Exhaustive over the calendars, so the ISO exemption cannot be
        // inherited by a calendar added later — `chinese` and `dangi` have no
        // eras at all and still range-check, so "has eras" would be the wrong
        // predicate.
        for calendar in TemporalCalendarId::ALL {
            match calendar.month_day_year_use() {
                MonthDayYearUse::OverflowOnly => {}
                MonthDayYearUse::RangeChecked => {
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(calendar.canonical()),
                    ));
                    function.instruction(&Instruction::LocalSet(expected_payload_local));
                    self.emit_string_payload_equality_i32(
                        calendar_payload_local,
                        expected_payload_local,
                        function,
                    );
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(year_present_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(year_local));
                    function.instruction(&Instruction::I64Const(TEMPORAL_MONTH_DAY_MINIMUM_YEAR));
                    function.instruction(&Instruction::I64LtS);
                    function.instruction(&Instruction::LocalGet(year_local));
                    function.instruction(&Instruction::I64Const(TEMPORAL_MONTH_DAY_MAXIMUM_YEAR));
                    function.instruction(&Instruction::I64GtS);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_throw_current_function_realm_range_error(
                        "Temporal.PlainMonthDay year is outside the supported range",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);
                }
            }
        }

        // The ISO reference year stands in when the bag carries no `year`. A
        // supplied year is only ever used to pick how 29 February constrains -
        // under `OverflowOnly` it is deliberately not range-checked, and it is
        // never stored under either.
        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        ));
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(month_from_code_local));
        for month in 1_i64..=12 {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&format!("M{month:02}")),
            ));
            function.instruction(&Instruction::LocalSet(expected_payload_local));
            self.emit_string_payload_equality_i32(
                month_code_payload_local,
                expected_payload_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::LocalSet(month_from_code_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal.PlainMonthDay monthCode",
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
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainMonthDay month and monthCode must agree",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(month_from_code_local));
        function.instruction(&Instruction::LocalSet(month_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainMonthDay month and day must be positive",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_month_day_regulate(
            year_local,
            month_local,
            day_local,
            overflow_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        ));
        function.instruction(&Instruction::LocalSet(year_local));

        self.release_temp_local(expected_payload_local);
        self.release_temp_local(month_from_code_local);
        Ok(())
    }

    /// `ToTemporalMonthDay`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_temporal_to_temporal_month_day(
        &mut self,
        argument_payload_local: u32,
        argument_tag_local: u32,
        options_payload_local: u32,
        options_tag_local: u32,
        read_options: bool,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let calendar_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let year_present_local = self.reserve_temp_local();
        let month_present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let day_present_local = self.reserve_temp_local();
        let handled_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(handled_local));
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));

        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
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
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            argument_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        if read_options {
            self.emit_temporal_month_day_overflow_option(
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

        function.instruction(&Instruction::LocalGet(handled_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let era = self.emit_temporal_plain_date_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            year_local,
            year_present_local,
            month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            day_local,
            day_present_local,
            true,
            true,
            function,
        )?;
        if read_options {
            self.emit_temporal_month_day_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        let resolved_year = self.emit_temporal_resolve_era_to_year(
            era,
            calendar_payload_local,
            year_local,
            year_present_local,
            function,
        )?;
        self.emit_temporal_month_day_resolve_fields(
            &resolved_year,
            calendar_payload_local,
            month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            day_local,
            day_present_local,
            overflow_local,
            function,
        )?;
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
            "Temporal.PlainMonthDay expects a string, a property bag, or a Temporal.PlainMonthDay",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        let parsed = self.emit_temporal_parse_month_day_string(
            argument_payload_local,
            year_local,
            year_present_local,
            month_local,
            day_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        // Step (f). `from/observable-get-overflow-argument-string-invalid.js`
        // pins that a string the parser rejects reads no option at all
        // (`from("13-34", observer)` must leave the observer log empty), so this
        // stays after the parse. That is the only string-path ordering
        // constraint the corpus actually observes.
        //
        // Its placement *before* the year-empty and ISODateWithinLimits throws
        // follows the spec — GetTemporalOverflowOption precedes both — and is
        // **unobserved**: no test in the 286-case corpus passes an options
        // observer together with an input that reaches those throws. The only
        // inputs that reach them are the three `[u-ca=gregory]` strings, and
        // neither red test supplies an observer. Moving this read would
        // therefore turn nothing red; do not read a green suite as confirmation
        // that the order is right.
        //
        // `from/options-read-before-algorithmic-validation.js` does NOT pin
        // this, despite what an earlier version of this comment said: its only
        // call is `Temporal.PlainMonthDay.from({ monthCode: "M08L", day: 14 },
        // options)`, a property bag, which `emit_temporal_to_temporal_month_day`
        // routes through the `handled_local == 0` bag branch and which never
        // enters this string branch at all.
        //
        // `equals` arrives with `read_options: false` and still owes both
        // RangeErrors, which is why they are not inside here.
        if read_options {
            self.emit_temporal_month_day_overflow_option(
                options_payload_local,
                options_tag_local,
                overflow_local,
                function,
            )?;
        }
        self.emit_temporal_month_day_string_reference_year(
            parsed,
            calendar_payload_local,
            month_local,
            day_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        for local in [
            handled_local,
            day_present_local,
            month_code_present_local,
            month_code_payload_local,
            month_present_local,
            year_present_local,
            record_local,
            brand_local,
            overflow_local,
            calendar_tag_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 10.2.2 `Temporal.PlainMonthDay.from`.
    pub(crate) fn emit_temporal_plain_month_day_from(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_temporal_to_temporal_month_day(
            argument_payload_local,
            argument_tag_local,
            options_payload_local,
            options_tag_local,
            true,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainMonthDay,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::Intrinsic,
            function,
        )?;

        for local in [
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
        ] {
            self.release_temp_local(local);
        }
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
    pub(crate) fn emit_temporal_parse_month_day_string(
        &mut self,
        string_payload_local: u32,
        year_local: u32,
        year_present_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        function: &mut Function,
    ) -> Result<TemporalParsedMonthDayYear, EmitError> {
        let rewritten_local = self.reserve_temp_local();
        let year_empty_local = self.reserve_temp_local();

        self.emit_temporal_month_day_rewrite_string(
            string_payload_local,
            rewritten_local,
            Some(year_empty_local),
            function,
        )?;
        // The rewrite answers "the year is empty"; every consumer downstream
        // asks "is the year present", the same polarity as the property-bag
        // `*_present_local` flags. Invert once, here, so no consumer has to
        // remember which way round the parser reports it.
        function.instruction(&Instruction::LocalGet(year_empty_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(year_present_local));
        self.emit_temporal_parse_plain_date_string(
            rewritten_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;

        self.release_temp_local(year_empty_local);
        self.release_temp_local(rewritten_local);
        Ok(TemporalParsedMonthDayYear {
            year_local,
            year_present_local,
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
    /// * **reference-year store** — the stored `[[ISOYear]]` becomes
    ///   [`TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR`].
    ///
    /// **The unconditional 1972 store is a shortcut, valid only while
    /// [`TemporalCalendarId::ALL`] contains no calendar with a leap month.** The
    /// literal 1972 is the `iso8601` branch's reference year; on the non-ISO
    /// branch the reference year is whatever `CalendarMonthDayFromFields`
    /// returns, and the spec does not promise 1972 there.
    /// `intl402/Temporal/PlainMonthDay/from/reference-year-1972.js` pins exactly
    /// that: `result4` (`{monthCode:"M05L", day:1, calendar:"hebrew"}`) asserts
    /// **1970** and `result7` asserts 1971, checked through
    /// `TemporalHelpers.assertPlainMonthDay`'s fifth parameter
    /// (`harness/temporalHelpers.js:302-308`, which reads the year back out of
    /// `toString({calendarName:"always"})`). `ALL` is `[Iso8601, Gregory]` and
    /// every gregory month-day exists in the leap year 1972, so the shortcut is
    /// correct today and only today. A lunisolar calendar added to `ALL` must
    /// derive this year from `CalendarMonthDayFromFields` instead — see the note
    /// on `ALL` itself.
    ///
    /// Both checks are outside the caller's `if read_options` block, because
    /// `equals` reaches `ToTemporalMonthDay` with no options at all and
    /// `prototype/equals/argument-string-invalid.js` still requires both
    /// RangeErrors. That the overflow read is emitted *before* them follows the
    /// spec (GetTemporalOverflowOption precedes both) but is **unobserved** by
    /// the corpus — see the comment at the caller's option-read site.
    ///
    /// Taking the [`TemporalParsedMonthDayYear`] by value is the point: this is
    /// also the only place the reference year is stored, so a future string path
    /// that forgets to call this fails loudly on `[[ISOYear]]` instead of
    /// quietly accepting a string the spec rejects.
    fn emit_temporal_month_day_string_reference_year(
        &mut self,
        parsed: TemporalParsedMonthDayYear,
        calendar_payload_local: u32,
        month_local: u32,
        day_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let TemporalParsedMonthDayYear {
            year_local,
            year_present_local,
        } = parsed;

        // Year-empty rejection — non-ISO calendar and no year in the source.
        self.emit_temporal_calendar_is_default_i32(calendar_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainMonthDay month-day string with a non-ISO calendar requires a year",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // `ISODateWithinLimits` — non-ISO calendar: the parsed date, not the
        // reference date, is
        // what has to be representable. `ISODateWithinLimits`, not
        // `ISOYearMonthWithinLimits`: the latter is the bag path's bound and
        // disagrees on the two boundary days.
        self.emit_temporal_calendar_is_default_i32(calendar_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let days_local = self.reserve_temp_local();
        self.emit_temporal_iso_date_within_limits(
            year_local,
            month_local,
            day_local,
            days_local,
            "Temporal.PlainMonthDay is outside the supported date range",
            function,
        )?;
        self.release_temp_local(days_local);
        function.instruction(&Instruction::End);

        // Reference-year store — the parsed year is never stored. See the doc
        // comment: the unconditional literal is a shortcut that holds only while
        // no calendar in `TemporalCalendarId::ALL` has a leap month.
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR,
        ));
        function.instruction(&Instruction::LocalSet(year_local));
        Ok(())
    }

    /// Temporal proposal 10.3.x `equals`.
    pub(crate) fn emit_temporal_plain_month_day_equals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let undefined_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let other_year_local = self.reserve_temp_local();
        let other_month_local = self.reserve_temp_local();
        let other_day_local = self.reserve_temp_local();
        let other_calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_temporal_to_temporal_month_day(
            argument_payload_local,
            argument_tag_local,
            undefined_local,
            undefined_tag_local,
            false,
            other_year_local,
            other_month_local,
            other_day_local,
            other_calendar_payload_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(other_year_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::LocalGet(other_month_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(other_day_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
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

        for local in [
            other_calendar_payload_local,
            other_day_local,
            other_month_local,
            other_year_local,
            undefined_tag_local,
            undefined_local,
            argument_tag_local,
            argument_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 10.3.x `with`.
    pub(crate) fn emit_temporal_plain_month_day_with(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let calendar_tag_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let new_year_local = self.reserve_temp_local();
        let year_present_local = self.reserve_temp_local();
        let new_month_local = self.reserve_temp_local();
        let month_present_local = self.reserve_temp_local();
        let month_code_payload_local = self.reserve_temp_local();
        let month_code_present_local = self.reserve_temp_local();
        let new_day_local = self.reserve_temp_local();
        let day_present_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay.prototype.with requires an object",
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
            "Temporal.PlainMonthDay.prototype.with does not accept a Temporal object",
            function,
        )?;

        // `RejectTemporalLikeObject` reads both keys with `Get`, not with a
        // `HasProperty` probe, and Test262's `with/order-of-operations.js`
        // observes the two reads.
        for property in ["calendar", "timeZone"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(property)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                argument_payload_local,
                argument_tag_local,
                argument_payload_local,
                argument_tag_local,
                key_local,
                present_local,
                calendar_tag_local,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::LocalGet(calendar_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_current_function_realm_type_error(
                "Temporal.PlainMonthDay.prototype.with does not accept calendar or timeZone",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        let era = self.emit_temporal_plain_date_read_fields(
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            calendar_tag_local,
            new_year_local,
            year_present_local,
            new_month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            new_day_local,
            day_present_local,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Or);
        for local in era.present_locals() {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Or);
        }
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay.prototype.with requires at least one field",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_month_day_overflow_option(
            options_payload_local,
            options_tag_local,
            overflow_local,
            function,
        )?;

        // No receiver-year merge here: a `Temporal.PlainMonthDay` has no
        // observable year, so the reference year stands in when the bag
        // supplies neither `year` nor an era pair.
        let resolved_year = self.emit_temporal_resolve_era_to_year(
            era,
            calendar_payload_local,
            new_year_local,
            year_present_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(day_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalSet(new_day_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(day_present_local));
        function.instruction(&Instruction::LocalGet(month_present_local));
        function.instruction(&Instruction::LocalGet(month_code_present_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_month_code_payload(month_local, function);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(month_code_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(month_code_present_local));
        function.instruction(&Instruction::End);

        self.emit_temporal_month_day_resolve_fields(
            &resolved_year,
            calendar_payload_local,
            new_month_local,
            month_present_local,
            month_code_payload_local,
            month_code_present_local,
            new_day_local,
            day_present_local,
            overflow_local,
            function,
        )?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainMonthDay,
            new_year_local,
            new_month_local,
            new_day_local,
            calendar_payload_local,
            TemporalPartialDatePrototype::Intrinsic,
            function,
        )?;

        for local in [
            day_present_local,
            new_day_local,
            month_code_present_local,
            month_code_payload_local,
            month_present_local,
            new_month_local,
            year_present_local,
            new_year_local,
            present_local,
            key_local,
            overflow_local,
            options_tag_local,
            options_payload_local,
            argument_tag_local,
            argument_payload_local,
            calendar_tag_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
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
        let record_local = self.reserve_temp_local();
        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.release_temp_local(record_local);
        self.emit_intl_dtf_temporal_to_locale_string(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY,
            function,
        )
    }

    /// `TemporalMonthDayToString`. The reference year is prefixed only when the
    /// calendar annotation is shown, which is the only way a round-trip could
    /// otherwise lose it.
    pub(crate) fn emit_temporal_plain_month_day_to_string(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let show_calendar_local = self.reserve_temp_local();
        let output_payload_local = self.reserve_temp_local();
        let piece_payload_local = self.reserve_temp_local();
        let number_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ShowCalendarName::Auto.code()));
        function.instruction(&Instruction::LocalSet(show_calendar_local));
        if matches!(
            builtin,
            StandardBuiltinId::TemporalPlainMonthDayPrototypeToString
        ) {
            self.emit_builtin_arg_to_locals(0, options_payload_local, options_tag_local, function);
            self.emit_temporal_string_valued_option::<ShowCalendarName>(
                options_payload_local,
                options_tag_local,
                show_calendar_local,
                "Temporal.PlainMonthDay options must be an object or undefined",
                "Invalid Temporal.PlainMonthDay calendarName option",
                function,
            )?;
        }

        // `TemporalMonthDayToString` step 2: the reference year is printed
        // under exactly the condition that prints the calendar annotation, so
        // `--01-05[u-ca=gregory]` is never emitted without its `1972-`.
        self.emit_temporal_show_calendar_annotation_i32(
            show_calendar_local,
            calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_pad_iso_year(
            year_local,
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        self.emit_temporal_append_separated_two_digits(
            month_local,
            "-",
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_payload_local));
        self.emit_temporal_append_separated_two_digits(
            month_local,
            "",
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_temporal_append_separated_two_digits(
            day_local,
            "-",
            output_payload_local,
            piece_payload_local,
            number_payload_local,
            function,
        )?;
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

        for local in [
            number_payload_local,
            piece_payload_local,
            output_payload_local,
            show_calendar_local,
            options_tag_local,
            options_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Temporal proposal 10.3.x `toPlainDate ( item )`: the receiver's month
    /// and day plus a `year` read from `item`.
    pub(crate) fn emit_temporal_plain_month_day_to_plain_date(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let overflow_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_month_day_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(argument_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainMonthDay.prototype.toPlainDate requires an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        // `PrepareCalendarFields(calendar, item, « year », «», « year »)` picks
        // up `era`/`eraYear` for a calendar that has them, and they sort before
        // `year`. `intl402/.../PlainMonthDay/prototype/toPlainDate/infinity-throws-rangeerror.js`
        // asserts the call log is exactly the `eraYear` coercion.
        let era_slots = self.reserve_temporal_era_slots();
        let era = self.emit_temporal_read_era_fields(
            era_slots,
            argument_payload_local,
            argument_tag_local,
            calendar_payload_local,
            function,
        )?;
        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "year",
            key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            year_local,
            0,
            "Temporal.PlainMonthDay year must be finite",
            function,
        )?;
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
            "Temporal.PlainMonthDay.prototype.toPlainDate requires a year",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(TemporalOverflow::Constrain.code()));
        function.instruction(&Instruction::LocalSet(overflow_local));
        self.emit_temporal_plain_date_regulate(
            year_local,
            month_local,
            day_local,
            overflow_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.emit_alloc_temporal_plain_date(
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            prototype_payload_local,
            overflow_local,
            present_local,
            value_tag_local,
            value_payload_local,
            key_local,
            argument_tag_local,
            argument_payload_local,
            calendar_payload_local,
            day_local,
            month_local,
            year_local,
            record_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
