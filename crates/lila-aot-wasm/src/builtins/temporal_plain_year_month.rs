//! `Temporal.PlainYearMonth` codegen.
//!
//! Temporal proposal 9: a calendar year and month with a *reference* ISO day
//! that is not observable through any accessor. The record is byte-identical to
//! `Temporal.PlainDate`'s — three `i64` ISO fields plus an interned calendar
//! payload — so every layout constant, the leap-year test, `ISODaysInMonth`
//! and `RegulateISODate` are shared with `temporal_plain_date.rs` rather than
//! duplicated. Only the brand, the range check (`ISOYearMonthWithinLimits`
//! instead of `ISODateWithinLimits`) and the field set differ.
//!
//! `monthsInYear` is always 12 and the reference day defaults to 1 for both of
//! this backend's calendars, because `gregory` is the same proleptic Gregorian
//! arithmetic as `iso8601`. `era`/`eraYear` are the one field pair that differs
//! and they go through `emit_temporal_calendar_era_field`; the reference day is
//! printed by `toString` exactly when the calendar is not `iso8601`, which is
//! `emit_temporal_calendar_is_default_i32`.

use super::super::*;
use super::temporal_plain_date::TemporalEraField;

/// The two Temporal types stored in the `Temporal.PlainDate` record shape.
///
/// The internal brand and the prototype are two halves of one decision, and
/// they used to be two independent arguments to
/// [`FunctionBuilder::emit_alloc_temporal_partial_date`]: pairing the month-day
/// brand with the year-month prototype compiled, and produced an object that is
/// a `Temporal.PlainMonthDay` to every brand check and a
/// `Temporal.PlainYearMonth` to every method lookup. Nothing throws on such an
/// object; it simply answers the wrong questions. Naming the type once, and
/// deriving both halves from it, makes that pairing unrepresentable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalPartialDateType {
    PlainYearMonth,
    PlainMonthDay,
}

impl TemporalPartialDateType {
    /// `[[InitializedTemporalYearMonth]]` / `[[InitializedTemporalMonthDay]]`.
    pub(crate) const fn brand(self) -> u64 {
        match self {
            TemporalPartialDateType::PlainYearMonth => {
                OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH
            }
            TemporalPartialDateType::PlainMonthDay => {
                OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY
            }
        }
    }

    /// The realm intrinsic `%Temporal.PlainYearMonth.prototype%` /
    /// `%Temporal.PlainMonthDay.prototype%`.
    pub(crate) const fn prototype_global_index(self) -> u32 {
        match self {
            TemporalPartialDateType::PlainYearMonth => {
                TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_GLOBAL_INDEX
            }
            TemporalPartialDateType::PlainMonthDay => {
                TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_GLOBAL_INDEX
            }
        }
    }
}

/// Where a partial-date object's prototype comes from. Both arms resolve
/// against the [`TemporalPartialDateType`] that also supplies the brand, so the
/// two cannot name different types.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalPartialDatePrototype {
    /// The realm intrinsic, for the `from`/`with`/`add` paths that never see a
    /// `new.target`.
    Intrinsic,
    /// `GetPrototypeFromConstructor(newTarget, ...)`, for the constructor.
    FromNewTarget,
}

/// `ISOYearMonthWithinLimits`. The year bound is one wider than
/// `ISODateWithinLimits` at each end because a whole month, not a single day,
/// has to fit.
const TEMPORAL_PLAIN_YEAR_MONTH_MINIMUM_YEAR: i64 = -271_821;
const TEMPORAL_PLAIN_YEAR_MONTH_MAXIMUM_YEAR: i64 = 275_760;
const TEMPORAL_PLAIN_YEAR_MONTH_MINIMUM_MONTH: i64 = 4;
const TEMPORAL_PLAIN_YEAR_MONTH_MAXIMUM_MONTH: i64 = 9;

impl<'a> FunctionBuilder<'a> {
    /// A brand check that any of the partial-date types can share: throws a
    /// TypeError unless `this` is an object carrying `brand`, then leaves the
    /// boxed record pointer in `record_local`.
    pub(crate) fn emit_temporal_branded_record_from_receiver(
        &mut self,
        brand: u64,
        message: &str,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(brand as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    /// Loads the four ISO slots out of a partial-date record. The layout is the
    /// `Temporal.PlainDate` one for all three types.
    pub(crate) fn emit_temporal_partial_date_load_record(
        &mut self,
        record_local: u32,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        function: &mut Function,
    ) {
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
    }

    /// Allocates a partial-date object: the `Temporal.PlainDate` record shape
    /// under `kind`'s brand and `kind`'s prototype. Both come from the same
    /// argument, so the object cannot be branded as one type and shaped as the
    /// other.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_alloc_temporal_partial_date(
        &mut self,
        kind: TemporalPartialDateType,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        prototype: TemporalPartialDatePrototype,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        match prototype {
            TemporalPartialDatePrototype::Intrinsic => {
                function.instruction(&Instruction::GlobalGet(kind.prototype_global_index()));
                function.instruction(&Instruction::LocalSet(prototype_payload_local));
            }
            TemporalPartialDatePrototype::FromNewTarget => {
                self.emit_error_new_target_prototype_to_local(
                    kind.prototype_global_index(),
                    None,
                    prototype_payload_local,
                    function,
                )?;
            }
        }
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.emit_heap_alloc_const(HEAP_TEMPORAL_PLAIN_DATE_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        for (offset, local) in [
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET, year_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET, month_local),
            (HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET, day_local),
            (
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET,
                calendar_payload_local,
            ),
        ] {
            self.store_i64_local_at_offset(record_local, offset, local, function);
        }
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            kind.brand(),
            function,
        );
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(record_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    /// `IsValidISODate` followed by `ISOYearMonthWithinLimits`. Both failures
    /// are RangeErrors.
    pub(crate) fn emit_temporal_reject_iso_year_month(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();

        self.emit_temporal_iso_days_in_month(year_local, month_local, maximum_day_local, function);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(day_local));
        function.instruction(&Instruction::LocalGet(maximum_day_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainYearMonth is not a valid ISO date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_year_month_within_limits_check(year_local, month_local, function)?;

        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    /// `ISOYearMonthWithinLimits` on its own, for callers that have already
    /// validated the day.
    pub(crate) fn emit_temporal_year_month_within_limits_check(
        &mut self,
        year_local: u32,
        month_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_YEAR_MONTH_MINIMUM_YEAR,
        ));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_YEAR_MONTH_MAXIMUM_YEAR,
        ));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_YEAR_MONTH_MINIMUM_YEAR,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_YEAR_MONTH_MINIMUM_MONTH,
        ));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_YEAR_MONTH_MAXIMUM_YEAR,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_YEAR_MONTH_MAXIMUM_MONTH,
        ));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainYearMonth is outside the supported range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Temporal proposal 9.1.1:
    /// `Temporal.PlainYearMonth(isoYear, isoMonth [, calendar [, referenceISODay]])`.
    pub(crate) fn emit_temporal_plain_year_month_constructor(
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
            "Temporal.PlainYearMonth constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for (index, output_local, message) in [
            (
                0,
                year_local,
                "Temporal.PlainYearMonth year must be an integer",
            ),
            (
                1,
                month_local,
                "Temporal.PlainYearMonth month must be an integer",
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
        // `referenceISODay` defaults to 1, which is also the day every
        // calendar-derived `Temporal.PlainYearMonth` carries.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(day_local));
        self.emit_builtin_arg_to_locals(3, argument_payload_local, argument_tag_local, function);
        function.instruction(&Instruction::LocalGet(argument_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_to_integer_with_truncation(
            argument_payload_local,
            argument_tag_local,
            day_local,
            "Temporal.PlainYearMonth reference day must be an integer",
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_temporal_reject_iso_year_month(year_local, month_local, day_local, function)?;
        self.emit_alloc_temporal_partial_date(
            TemporalPartialDateType::PlainYearMonth,
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

    /// The `[[InitializedTemporalYearMonth]]` brand check.
    pub(crate) fn emit_temporal_plain_year_month_record_from_receiver(
        &mut self,
        record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_branded_record_from_receiver(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH,
            "Temporal.PlainYearMonth receiver does not have [[InitializedTemporalYearMonth]]",
            record_local,
            function,
        )
    }

    /// Every `Temporal.PlainYearMonth.prototype` accessor.
    pub(crate) fn emit_temporal_plain_year_month_field(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_year_month_record_from_receiver(record_local, function)?;
        self.emit_temporal_partial_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );

        match builtin {
            StandardBuiltinId::TemporalPlainYearMonthPrototypeCalendarIdGetter => {
                function.instruction(&Instruction::LocalGet(calendar_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthCodeGetter => {
                self.emit_temporal_month_code_payload(month_local, function);
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainYearMonthPrototypeEraGetter => {
                self.emit_temporal_calendar_era_field(
                    calendar_payload_local,
                    year_local,
                    TemporalEraField::Era,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainYearMonthPrototypeEraYearGetter => {
                self.emit_temporal_calendar_era_field(
                    calendar_payload_local,
                    year_local,
                    TemporalEraField::EraYear,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainYearMonthPrototypeInLeapYearGetter => {
                self.emit_temporal_iso_year_is_leap_i32(year_local, function);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            _ => {
                let value_local = self.reserve_temp_local();
                match builtin {
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeYearGetter => {
                        function.instruction(&Instruction::LocalGet(year_local));
                        function.instruction(&Instruction::LocalSet(value_local));
                    }
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthGetter => {
                        function.instruction(&Instruction::LocalGet(month_local));
                        function.instruction(&Instruction::LocalSet(value_local));
                    }
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthsInYearGetter => {
                        function.instruction(&Instruction::I64Const(12));
                        function.instruction(&Instruction::LocalSet(value_local));
                    }
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInMonthGetter => {
                        self.emit_temporal_iso_days_in_month(
                            year_local,
                            month_local,
                            value_local,
                            function,
                        );
                    }
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInYearGetter => {
                        self.emit_temporal_iso_year_is_leap_i32(year_local, function);
                        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                        function.instruction(&Instruction::I64Const(366));
                        function.instruction(&Instruction::Else);
                        function.instruction(&Instruction::I64Const(365));
                        function.instruction(&Instruction::End);
                        function.instruction(&Instruction::LocalSet(value_local));
                    }
                    _ => unreachable!("non-numeric Temporal.PlainYearMonth accessor"),
                }
                function.instruction(&Instruction::LocalGet(value_local));
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(value_local);
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

    /// `ToMonthCode`: `ToPrimitive` with a string hint, then the result must
    /// *be* a String - `{ toString: () => 5 }` primitivises to the Number 5 and
    /// is a TypeError, not the string `"5"`. The syntax check (`M` + two digits
    /// + an optional leap marker `L`) runs at *read* time, before any later
    /// field is fetched, because Test262's `from/monthcode-invalid.js` pins
    /// that `{ monthCode: "L99M", year: Symbol() }` is a RangeError while
    /// `{ monthCode: "M99L", year: Symbol() }` is a TypeError - syntax first,
    /// suitability last.
    pub(crate) fn emit_temporal_month_code_string(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        type_error_message: &str,
        range_error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let valid_local = self.reserve_temp_local();
        let primitive_payload_local = self.reserve_temp_local();
        let primitive_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::String,
            value_payload_local,
            value_tag_local,
            primitive_payload_local,
            primitive_tag_local,
            // A user hook may throw an arbitrary value. Return it before the
            // normal-result String check below can replace it with a TypeError.
            ToPrimitiveAbruptRoute::ReturnCurrentFunction,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(primitive_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::LocalGet(primitive_tag_local));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            type_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_unpack_string_payload(value_payload_local, offset_local, length_local, function);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'M' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(valid_local));
        for index in 1_i64..=2 {
            function.instruction(&Instruction::I64Const(index));
            function.instruction(&Instruction::LocalSet(index_local));
            self.emit_load_string_byte(offset_local, index_local, byte_local, function);
            function.instruction(&Instruction::LocalGet(valid_local));
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'0' as i64));
            function.instruction(&Instruction::I64GeU);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(b'9' as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(valid_local));
        }
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'L' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(valid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(valid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            range_error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            primitive_tag_local,
            primitive_payload_local,
            valid_local,
            index_local,
            byte_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `M01`..`M12` into `self.result_local`. The ISO calendar has no leap
    /// months, so the twelve interned strings cover every case.
    pub(crate) fn emit_temporal_month_code_payload(
        &mut self,
        month_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(self.strings.payload("M01")));
        function.instruction(&Instruction::LocalSet(self.result_local));
        for month in 2_i64..=12 {
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(
                self.strings.payload(&format!("M{month:02}")),
            ));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::End);
        }
    }

    /// Temporal deliberately forbids implicit comparison, so `valueOf` always
    /// throws on every Temporal type.
    pub(crate) fn emit_temporal_partial_date_value_of(
        &mut self,
        type_name: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let message =
            format!("{type_name} does not support implicit conversion; use compare() or equals()");
        self.emit_throw_current_function_realm_type_error(
            &message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
