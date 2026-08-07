//! `Intl.DateTimeFormat` — ECMA-402 11.
//!
//! Scope, stated honestly. This implements `CreateDateTimeFormat` (11.1.2) in
//! full — every option is read, in the observable order, with the spec's
//! validation — and `resolvedOptions` (11.4.4), `supportedLocalesOf` (11.2.2),
//! `format` (11.4.3) and `formatToParts` (11.4.5) over a **single locale**,
//! `en-US`, a single calendar, `gregory`, a single numbering system, `latn`,
//! and a single time zone, `UTC`.
//!
//! Locale negotiation therefore always resolves to `"en-US"`: `ResolveLocale`
//! with `AvailableLocales = « "en-US" »` falls back to the default locale for
//! every request. That is a real answer for `en`/`en-US` and an honest
//! fallback elsewhere, which is what an implementation with no CLDR data can
//! say. Non-`UTC` time zones and non-`gregory` calendars are **rejected**
//! (`RangeError`) rather than accepted and mis-formatted.
//!
//! # The table is the single source of truth
//!
//! Every string-valued option appears exactly once, as an [`IntlDtfOption`]
//! with its property name, its record slot and its `(spelling, code)` list.
//! The constructor reads options by walking that table and `resolvedOptions`
//! writes them back by walking the same table, so a spelling can never be
//! accepted by one and unknown to the other, and a slot can never be written
//! by one and read from a different offset by the other. Adding a value is a
//! one-line change in one place.
//!
//! # Formatting is emitted once
//!
//! `format` and `formatToParts` are required to agree — `reduce(parts) ===
//! format(x)` is itself a Test262 assertion. Both are emitted from
//! [`FunctionBuilder::emit_intl_dtf_build_format`] with a
//! [`DtfFormatMode`] discriminator, so the field order, the literals and the
//! numeral rendering come from one body of Rust and cannot drift apart.

use super::super::*;

/// Where a component's code lives and what spellings map to it.
///
/// `codes` is ordered as the spec's Values column. Code 0 is reserved: it
/// always means "option absent", so `resolvedOptions` can decide between
/// emitting a property and omitting it by testing against zero alone.
pub(crate) struct IntlDtfOption {
    pub(crate) property: &'static str,
    pub(crate) slot_offset: u64,
    pub(crate) codes: &'static [(&'static str, i64)],
}

/// ECMA-402 11.5 Table 7, in table order. The constructor reads these after
/// `timeZone` and before `formatMatcher`, and the order here is what the
/// `constructor-options-order*.js` tests observe through getters.
///
/// `fractionalSecondDigits` is absent because it is a *number* option; the
/// constructor splices it in at its table position explicitly.
pub(crate) const INTL_DTF_COMPONENT_OPTIONS: &[IntlDtfOption] = &[
    IntlDtfOption {
        property: "weekday",
        slot_offset: HEAP_INTL_DTF_WEEKDAY_OFFSET,
        codes: &[("narrow", 1), ("short", 2), ("long", 3)],
    },
    IntlDtfOption {
        property: "era",
        slot_offset: HEAP_INTL_DTF_ERA_OFFSET,
        codes: &[("narrow", 1), ("short", 2), ("long", 3)],
    },
    IntlDtfOption {
        property: "year",
        slot_offset: HEAP_INTL_DTF_YEAR_OFFSET,
        codes: &[("2-digit", 1), ("numeric", 2)],
    },
    IntlDtfOption {
        property: "month",
        slot_offset: HEAP_INTL_DTF_MONTH_OFFSET,
        codes: &[
            ("2-digit", 1),
            ("numeric", 2),
            ("narrow", 3),
            ("short", 4),
            ("long", 5),
        ],
    },
    IntlDtfOption {
        property: "day",
        slot_offset: HEAP_INTL_DTF_DAY_OFFSET,
        codes: &[("2-digit", 1), ("numeric", 2)],
    },
    IntlDtfOption {
        property: "dayPeriod",
        slot_offset: HEAP_INTL_DTF_DAY_PERIOD_OFFSET,
        codes: &[("narrow", 1), ("short", 2), ("long", 3)],
    },
    IntlDtfOption {
        property: "hour",
        slot_offset: HEAP_INTL_DTF_HOUR_OFFSET,
        codes: &[("2-digit", 1), ("numeric", 2)],
    },
    IntlDtfOption {
        property: "minute",
        slot_offset: HEAP_INTL_DTF_MINUTE_OFFSET,
        codes: &[("2-digit", 1), ("numeric", 2)],
    },
    IntlDtfOption {
        property: "second",
        slot_offset: HEAP_INTL_DTF_SECOND_OFFSET,
        codes: &[("2-digit", 1), ("numeric", 2)],
    },
    IntlDtfOption {
        property: "timeZoneName",
        slot_offset: HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET,
        codes: &[
            ("short", 1),
            ("long", 2),
            ("shortOffset", 3),
            ("longOffset", 4),
            ("shortGeneric", 5),
            ("longGeneric", 6),
        ],
    },
];

/// Index of `dayPeriod` in [`INTL_DTF_COMPONENT_OPTIONS`]. The
/// `fractionalSecondDigits` number option is read immediately after `second`,
/// which is the entry before `timeZoneName`.
const INTL_DTF_FRACTIONAL_SECOND_DIGITS_AFTER: &str = "second";

pub(crate) const INTL_DTF_HOUR_CYCLE_OPTION: IntlDtfOption = IntlDtfOption {
    property: "hourCycle",
    slot_offset: HEAP_INTL_DTF_HOUR_CYCLE_OFFSET,
    codes: &[("h11", 1), ("h12", 2), ("h23", 3), ("h24", 4)],
};

pub(crate) const INTL_DTF_DATE_STYLE_OPTION: IntlDtfOption = IntlDtfOption {
    property: "dateStyle",
    slot_offset: HEAP_INTL_DTF_DATE_STYLE_OFFSET,
    codes: &[("full", 1), ("long", 2), ("medium", 3), ("short", 4)],
};

pub(crate) const INTL_DTF_TIME_STYLE_OPTION: IntlDtfOption = IntlDtfOption {
    property: "timeStyle",
    slot_offset: HEAP_INTL_DTF_TIME_STYLE_OFFSET,
    codes: &[("full", 1), ("long", 2), ("medium", 3), ("short", 4)],
};

/// The one locale this implementation has data for. `ResolveLocale` returns it
/// for every request, so `resolvedOptions().locale` is always this string.
const INTL_DTF_RESOLVED_LOCALE: &str = "en-US";
const INTL_DTF_RESOLVED_CALENDAR: &str = "gregory";
const INTL_DTF_RESOLVED_NUMBERING_SYSTEM: &str = "latn";
const INTL_DTF_RESOLVED_TIME_ZONE: &str = "UTC";

/// `en` month names, index 0 = January.
const INTL_DTF_MONTHS_LONG: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const INTL_DTF_MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const INTL_DTF_MONTHS_NARROW: [&str; 12] =
    ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"];
/// `en` weekday names, index 0 = Sunday (matching `WeekDay(t)`).
const INTL_DTF_WEEKDAYS_LONG: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const INTL_DTF_WEEKDAYS_SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const INTL_DTF_WEEKDAYS_NARROW: [&str; 7] = ["S", "M", "T", "W", "T", "F", "S"];

/// Which artefact [`FunctionBuilder::emit_intl_dtf_build_format`] produces.
///
/// Both arms run the same field walk; only the accumulator differs. Keeping
/// them one function is what makes `reduce(formatToParts(x)) === format(x)`
/// true by construction instead of by review.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DtfFormatMode {
    /// Concatenate into a single string payload.
    String,
    /// Append `{ type, value }` objects to an array.
    Parts,
}

/// The accumulator locals threaded through the field walk.
struct DtfFormatSink {
    mode: DtfFormatMode,
    /// String mode: the running output. Parts mode: unused.
    text_local: u32,
    /// Parts mode: the array payload and its element buffer plus a length.
    array_local: u32,
    buffer_local: u32,
    length_local: u32,
    /// A pending literal to emit before the next real field, or 0.
    pending_literal_local: u32,
    /// 1 once at least one non-literal field has been emitted.
    emitted_local: u32,
    scratch_local: u32,
}

/// Upper bound on emitted parts: eleven fields with a literal between each,
/// plus the era and fractional-second extras. Rounded up so the array never
/// needs to grow.
const INTL_DTF_MAX_PARTS: i64 = 48;

impl<'a> FunctionBuilder<'a> {
    fn emit_dtf_set_const(&self, local: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::LocalSet(local));
    }

    fn emit_dtf_set_string(&mut self, local: u32, value: &str, function: &mut Function) {
        let payload = self.strings.payload(value);
        function.instruction(&Instruction::I64Const(payload));
        function.instruction(&Instruction::LocalSet(local));
    }

    /// `record = O.[[InitializedDateTimeFormat]]`, throwing a `TypeError` when
    /// the receiver does not carry the brand.
    ///
    /// ECMA-402 11.4.3/11.4.4/11.4.5 all begin with this check, and the
    /// "legacy unwrap" of `Intl.DateTimeFormat.call(obj)` is not implemented,
    /// so the brand is read straight off the receiver.
    fn emit_intl_dtf_record_from_receiver(
        &mut self,
        record_local: u32,
        method: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: Intl.DateTimeFormat method without receiver",
            )
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: Intl.DateTimeFormat method without receiver tag",
            )
        })?;
        let brand_local = self.reserve_temp_local();
        let message = format!("{method} called on a non-Intl.DateTimeFormat object");

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
            &message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(brand_local);
        Ok(())
    }

    /// `GetOption(options, prop, string, values, undefined)` writing the
    /// matched code (or 0) into `dest_local`.
    ///
    /// `present_local`, when given, is set to 1 exactly when the property was
    /// not `undefined` — the constructor needs that to detect explicit format
    /// components independently of which value was chosen.
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

    /// `GetOption(options, prop, string, values, default)` where the value is
    /// only validated, never stored — `localeMatcher` and `formatMatcher`.
    fn emit_intl_dtf_validate_only_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        property: &str,
        allowed: &[&str],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();
        let recognized_local = self.reserve_temp_local();
        let message = format!("Invalid {property} option");

        self.emit_dtf_set_string(key_local, property, function);
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
        self.emit_dtf_set_const(recognized_local, 0, function);
        for spelling in allowed {
            self.emit_dtf_set_string(expected_local, spelling, function);
            self.emit_string_payload_equality_i32(value_payload_local, expected_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_const(recognized_local, 1, function);
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

    /// `GetOption(options, prop, string, empty, undefined)` followed by the
    /// `-u-` key-type well-formedness check of ECMA-402 11.1.2 steps 7 and 10.
    ///
    /// A Unicode extension type is one or more `alphanum{3,8}` subtags joined
    /// by `-`; anything else is a `RangeError`. The value is otherwise
    /// discarded because only `gregory`/`latn` have data, and a request for
    /// anything else must not silently resolve to them.
    fn emit_intl_dtf_unicode_type_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        property: &str,
        accepted: &[&str],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();
        let ok_local = self.reserve_temp_local();
        let range_message = format!("Invalid {property} option");
        let unsupported_message = format!("Unsupported {property} option");

        self.emit_dtf_set_string(key_local, property, function);
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
        self.emit_intl_dtf_is_unicode_type_i32(value_payload_local, ok_local, function);
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            &range_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        // Well formed but not one this implementation has data for.
        self.emit_dtf_set_const(ok_local, 0, function);
        for spelling in accepted {
            self.emit_dtf_set_string(expected_local, spelling, function);
            self.emit_string_payload_equality_i32(value_payload_local, expected_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_const(ok_local, 1, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            &unsupported_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            ok_local,
            expected_local,
            value_tag_local,
            value_payload_local,
            key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `ok_local = 1` when the string is `alphanum{3,8}(-alphanum{3,8})*`.
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

    /// i32 on the stack: byte is `[0-9A-Za-z]`.
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

    /// `GetNumberOption(options, "fractionalSecondDigits", 1, 3, undefined)`.
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

    /// `Get(options, "timeZone")` plus `IsValidTimeZoneName`.
    ///
    /// Only `UTC` has data, so every spelling that canonicalises to `UTC` is
    /// accepted case-insensitively and everything else — including offset
    /// strings, which would need a real zone database to be meaningful — is a
    /// `RangeError` rather than a silent substitution.
    fn emit_intl_dtf_time_zone_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let lowered_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();
        let ok_local = self.reserve_temp_local();

        self.emit_dtf_set_string(dest_local, INTL_DTF_RESOLVED_TIME_ZONE, function);
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
        self.emit_intl_dtf_ascii_lowercase(value_payload_local, lowered_local, function)?;
        self.emit_dtf_set_const(ok_local, 0, function);
        for spelling in [
            "utc",
            "etc/utc",
            "etc/gmt",
            "etc/universal",
            "etc/zulu",
            "gmt",
        ] {
            self.emit_dtf_set_string(expected_local, spelling, function);
            self.emit_string_payload_equality_i32(lowered_local, expected_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_const(ok_local, 1, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Unsupported timeZone option",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            ok_local,
            expected_local,
            lowered_local,
            value_tag_local,
            value_payload_local,
            key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Copies an ASCII string payload with `A-Z` folded to `a-z`.
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

    /// `CreateDateTimeFormat(newTarget, locales, options, any, date)` —
    /// ECMA-402 11.1.2.
    ///
    /// The option reads below are in the exact order the specification
    /// prescribes, which is observable through accessor properties on the
    /// options bag; do not reorder them.
    pub(crate) fn emit_intl_date_time_format_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let locales_payload_local = self.reserve_temp_local();
        let locales_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let locale_local = self.reserve_temp_local();
        let matched_tag_local = self.reserve_temp_local();
        let extension_hour_cycle_local = self.reserve_temp_local();
        let scratch_suffix_local = self.reserve_temp_local();
        let hour12_local = self.reserve_temp_local();
        let hour_cycle_local = self.reserve_temp_local();
        let time_zone_local = self.reserve_temp_local();
        let explicit_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        let date_style_local = self.reserve_temp_local();
        let time_style_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let component_locals: Vec<u32> = INTL_DTF_COMPONENT_OPTIONS
            .iter()
            .map(|_| self.reserve_temp_local())
            .collect();
        let fractional_local = self.reserve_temp_local();

        // ECMA-402 11.1.1 step 1: a plain call substitutes the active function
        // object for NewTarget, so `Intl.DateTimeFormat()` builds an instance
        // rather than throwing. `ChainDateTimeFormat`'s legacy
        // %IntlLegacyConstructedSymbol% brand is **not** installed, so the
        // `Intl.DateTimeFormat.call(existingInstance)` re-initialisation path
        // is absent rather than half-implemented.
        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;

        // Step 2: CanonicalizeLocaleList. Every tag is validated even though
        // negotiation always lands on `en-US`, because an invalid tag is a
        // RangeError the caller can observe.
        self.emit_builtin_arg_to_locals(0, locales_payload_local, locales_tag_local, function);
        self.emit_intl_dtf_canonicalize_locale_list(
            locales_payload_local,
            locales_tag_local,
            locale_local,
            matched_tag_local,
            function,
        )?;

        // Step 3: CoerceOptionsToObject.
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(options_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(options_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_object_locals(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        // Steps 5, 7, 10: localeMatcher, calendar, numberingSystem.
        self.emit_intl_dtf_validate_only_option(
            options_payload_local,
            options_tag_local,
            "localeMatcher",
            &["lookup", "best fit"],
            function,
        )?;
        self.emit_intl_dtf_unicode_type_option(
            options_payload_local,
            options_tag_local,
            "calendar",
            &[INTL_DTF_RESOLVED_CALENDAR, "gregorian"],
            function,
        )?;
        self.emit_intl_dtf_unicode_type_option(
            options_payload_local,
            options_tag_local,
            "numberingSystem",
            &[INTL_DTF_RESOLVED_NUMBERING_SYSTEM],
            function,
        )?;

        // Steps 13-14: hour12 then hourCycle. Reading hour12 first is
        // observable; a present hour12 discards hourCycle entirely.
        self.emit_intl_dtf_hour12_option(
            options_payload_local,
            options_tag_local,
            hour12_local,
            function,
        )?;
        self.emit_intl_dtf_string_option(
            options_payload_local,
            options_tag_local,
            &INTL_DTF_HOUR_CYCLE_OPTION,
            hour_cycle_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(hour12_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(hour_cycle_local, 0, function);
        function.instruction(&Instruction::End);

        // ResolveLocale: the `hc` keyword of the negotiated locale is used only
        // when neither `hourCycle` nor `hour12` asked for something, and when
        // it is used the resolved locale carries it, per 9.2.7 step 12.
        self.emit_intl_dtf_extension_hour_cycle(
            matched_tag_local,
            extension_hour_cycle_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(hour_cycle_local));
        function.instruction(&Instruction::LocalGet(hour12_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(extension_hour_cycle_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(extension_hour_cycle_local));
        function.instruction(&Instruction::LocalSet(hour_cycle_local));
        for (spelling, code) in INTL_DTF_HOUR_CYCLE_OPTION.codes {
            self.emit_dtf_if_code_eq(extension_hour_cycle_local, *code, function);
            let suffix = self.strings.payload(&format!("-u-hc-{spelling}"));
            function.instruction(&Instruction::I64Const(suffix));
            function.instruction(&Instruction::LocalSet(scratch_suffix_local));
            self.emit_concat_string_payloads_local(locale_local, scratch_suffix_local, function)?;
            function.instruction(&Instruction::LocalSet(locale_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        // Step 29: timeZone.
        self.emit_intl_dtf_time_zone_option(
            options_payload_local,
            options_tag_local,
            time_zone_local,
            function,
        )?;

        // Step 36: Table 7, in table order, with fractionalSecondDigits
        // spliced in after `second`.
        self.emit_dtf_set_const(explicit_local, 0, function);
        for (option, dest_local) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&component_locals) {
            self.emit_intl_dtf_string_option(
                options_payload_local,
                options_tag_local,
                option,
                *dest_local,
                Some(present_local),
                function,
            )?;
            function.instruction(&Instruction::LocalGet(explicit_local));
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(explicit_local));
            if option.property == INTL_DTF_FRACTIONAL_SECOND_DIGITS_AFTER {
                self.emit_intl_dtf_fractional_second_digits_option(
                    options_payload_local,
                    options_tag_local,
                    fractional_local,
                    present_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(explicit_local));
                function.instruction(&Instruction::LocalGet(present_local));
                function.instruction(&Instruction::I64Or);
                function.instruction(&Instruction::LocalSet(explicit_local));
            }
        }

        // Step 37: formatMatcher, validated and discarded.
        self.emit_intl_dtf_validate_only_option(
            options_payload_local,
            options_tag_local,
            "formatMatcher",
            &["basic", "best fit"],
            function,
        )?;

        // Steps 38-40: dateStyle then timeStyle.
        self.emit_intl_dtf_string_option(
            options_payload_local,
            options_tag_local,
            &INTL_DTF_DATE_STYLE_OPTION,
            date_style_local,
            None,
            function,
        )?;
        self.emit_intl_dtf_string_option(
            options_payload_local,
            options_tag_local,
            &INTL_DTF_TIME_STYLE_OPTION,
            time_style_local,
            None,
            function,
        )?;

        // Step 42: a style and an explicit component cannot be combined.
        function.instruction(&Instruction::LocalGet(date_style_local));
        function.instruction(&Instruction::LocalGet(time_style_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(explicit_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "dateStyle and timeStyle may not be used with explicit date-time components",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Step 44: with neither a style nor an explicit component, `defaults`
        // is `date`, so year, month and day become "numeric".
        function.instruction(&Instruction::LocalGet(date_style_local));
        function.instruction(&Instruction::LocalGet(time_style_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(explicit_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (option, dest_local) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&component_locals) {
            if matches!(option.property, "year" | "month" | "day") {
                self.emit_dtf_set_const(*dest_local, 2, function);
            }
        }
        function.instruction(&Instruction::End);

        self.emit_intl_dtf_resolve_hour_cycle(hour12_local, hour_cycle_local, function);

        self.emit_error_new_target_prototype_to_local(
            INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.emit_heap_alloc_const(HEAP_INTL_DATE_TIME_FORMAT_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));

        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_LOCALE_OFFSET,
            locale_local,
            function,
        );
        for (name, offset) in [
            (INTL_DTF_RESOLVED_CALENDAR, HEAP_INTL_DTF_CALENDAR_OFFSET),
            (
                INTL_DTF_RESOLVED_NUMBERING_SYSTEM,
                HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET,
            ),
        ] {
            let payload = self.strings.payload(name);
            self.store_i64_const_at_offset(record_local, offset, payload as u64, function);
        }
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_TIME_ZONE_OFFSET,
            time_zone_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_HOUR_CYCLE_OFFSET,
            hour_cycle_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_HOUR12_OFFSET,
            hour12_local,
            function,
        );
        for (option, dest_local) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&component_locals) {
            self.store_i64_local_at_offset(record_local, option.slot_offset, *dest_local, function);
        }
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
            fractional_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_DATE_STYLE_OFFSET,
            date_style_local,
            function,
        );
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_TIME_STYLE_OFFSET,
            time_style_local,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_INTL_DTF_BOUND_FORMAT_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_INTL_DATE_TIME_FORMAT,
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

        self.release_temp_local(fractional_local);
        for local in component_locals.into_iter().rev() {
            self.release_temp_local(local);
        }
        for local in [
            object_payload_local,
            prototype_payload_local,
            record_local,
            time_style_local,
            date_style_local,
            present_local,
            explicit_local,
            time_zone_local,
            hour_cycle_local,
            hour12_local,
            scratch_suffix_local,
            extension_hour_cycle_local,
            matched_tag_local,
            locale_local,
            options_tag_local,
            options_payload_local,
            locales_tag_local,
            locales_payload_local,
            new_target_tag_local,
            new_target_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `GetOption(options, "hour12", boolean, empty, undefined)`; 0 absent,
    /// 1 false, 2 true.
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

    /// ECMA-402 11.1.2 steps 32-35 for `en`, whose default hour cycle is
    /// `h12`: an explicit `hour12` overrides the requested cycle, mapping true
    /// to `h12` and false to `h23`.
    fn emit_intl_dtf_resolve_hour_cycle(
        &mut self,
        hour12_local: u32,
        hour_cycle_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(hour12_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(hour_cycle_local, 2, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(hour12_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(hour_cycle_local, 3, function);
        function.instruction(&Instruction::End);
        // No request at all: `en` defaults to h12.
        function.instruction(&Instruction::LocalGet(hour_cycle_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(hour_cycle_local, 2, function);
        function.instruction(&Instruction::End);
    }

    /// `CanonicalizeLocaleList` (ECMA-402 9.2.1) followed by `LookupMatcher`
    /// over `AvailableLocales = « "en", "en-US" »`.
    ///
    /// The whole list is walked even though only the first match can win,
    /// because an invalid tag anywhere in it is a `RangeError` the caller can
    /// observe. `resolved_local` receives the negotiated locale: the requested
    /// base name when it is one of the two available ones, the truncation
    /// `"en"` for any other `en-*` request, and the default `"en-US"`
    /// otherwise.
    fn emit_intl_dtf_canonicalize_locale_list(
        &mut self,
        locales_payload_local: u32,
        locales_tag_local: u32,
        resolved_local: u32,
        matched_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let tag_payload_local = self.reserve_temp_local();
        let language_local = self.reserve_temp_local();
        let script_local = self.reserve_temp_local();
        let region_local = self.reserve_temp_local();
        let base_name_local = self.reserve_temp_local();
        let ok_local = self.reserve_temp_local();
        let matched_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();

        self.emit_dtf_set_string(resolved_local, INTL_DTF_RESOLVED_LOCALE, function);
        self.emit_dtf_set_const(matched_local, 0, function);
        self.emit_dtf_set_const(matched_tag_local, 0, function);
        function.instruction(&Instruction::LocalGet(locales_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(locales_payload_local));
        function.instruction(&Instruction::LocalSet(input_payload_local));
        self.emit_intl_canonicalize_locale_tag(
            input_payload_local,
            tag_payload_local,
            language_local,
            script_local,
            region_local,
            base_name_local,
            ok_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid language tag",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_intl_dtf_record_lookup_match(
            language_local,
            base_name_local,
            tag_payload_local,
            matched_local,
            expected_local,
            resolved_local,
            matched_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(locales_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_like_snapshot_payload(
            locales_payload_local,
            locales_tag_local,
            source_payload_local,
            "Intl.DateTimeFormat locales must be an object",
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            source_payload_local,
            HEAP_LEN_OFFSET,
            source_len_local,
            function,
        );
        self.emit_dtf_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            source_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_intl_locale_argument_to_string_payload(
            element_payload_local,
            element_tag_local,
            input_payload_local,
            "Intl.DateTimeFormat locale must be a string or an object",
            function,
        )?;
        self.emit_intl_canonicalize_locale_tag(
            input_payload_local,
            tag_payload_local,
            language_local,
            script_local,
            region_local,
            base_name_local,
            ok_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid language tag",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_intl_dtf_record_lookup_match(
            language_local,
            base_name_local,
            tag_payload_local,
            matched_local,
            expected_local,
            resolved_local,
            matched_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            expected_local,
            matched_local,
            ok_local,
            base_name_local,
            region_local,
            script_local,
            language_local,
            tag_payload_local,
            input_payload_local,
            index_local,
            source_len_local,
            source_payload_local,
            element_tag_local,
            element_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `LookupMatcher` for one requested tag: the first `en`-language request
    /// wins, resolving to its own base name when that base name is available
    /// and to the `"en"` truncation otherwise.
    fn emit_intl_dtf_record_lookup_match(
        &mut self,
        language_local: u32,
        base_name_local: u32,
        tag_local: u32,
        matched_local: u32,
        expected_local: u32,
        resolved_local: u32,
        matched_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(matched_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_string(expected_local, "en", function);
        self.emit_string_payload_equality_i32(language_local, expected_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(matched_local, 1, function);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(matched_tag_local));
        self.emit_dtf_set_string(resolved_local, "en", function);
        for available in ["en", INTL_DTF_RESOLVED_LOCALE] {
            self.emit_dtf_set_string(expected_local, available, function);
            self.emit_string_payload_equality_i32(base_name_local, expected_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_string(resolved_local, available, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    /// `dest_local = 1` when `needle` occurs as a byte substring of the
    /// canonicalised tag.
    ///
    /// Canonicalisation has already lowercased the tag and normalised its
    /// separators, so an exact `-<key>-<type>` needle is a sound test for a
    /// Unicode extension keyword: the only other place those bytes could occur
    /// is a private-use sequence, which no `Intl` option consults.
    fn emit_intl_dtf_tag_contains(
        &mut self,
        tag_local: u32,
        needle: &str,
        dest_local: u32,
        function: &mut Function,
    ) {
        let offset_local = self.reserve_temp_local();
        let length_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let inner_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let matched_local = self.reserve_temp_local();
        let needle_bytes: Vec<i64> = needle.bytes().map(|byte| byte as i64).collect();
        let needle_len = needle_bytes.len() as i64;

        self.emit_unpack_string_payload(tag_local, offset_local, length_local, function);
        self.emit_dtf_set_const(dest_local, 0, function);
        self.emit_dtf_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(needle_len));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(length_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_dtf_set_const(matched_local, 1, function);
        for (position, expected) in needle_bytes.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(position as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(inner_local));
            self.emit_load_string_byte(offset_local, inner_local, byte_local, function);
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(*expected));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_const(matched_local, 0, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(matched_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(dest_local, 1, function);
        // Block / Loop / If: depth 2 is the enclosing Block.
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            matched_local,
            byte_local,
            inner_local,
            index_local,
            length_local,
            offset_local,
        ] {
            self.release_temp_local(local);
        }
    }

    /// The `hc` Unicode extension keyword of the negotiated locale, or 0.
    ///
    /// ECMA-402 9.2.7 `ResolveLocale` only honours a relevant-extension-key
    /// whose value is one this implementation supports; every other spelling
    /// is ignored, which falls out of testing only the four legal ones.
    fn emit_intl_dtf_extension_hour_cycle(
        &mut self,
        tag_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        let found_local = self.reserve_temp_local();
        self.emit_dtf_set_const(dest_local, 0, function);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (spelling, code) in INTL_DTF_HOUR_CYCLE_OPTION.codes {
            self.emit_intl_dtf_tag_contains(
                tag_local,
                &format!("-hc-{spelling}"),
                found_local,
                function,
            );
            self.emit_dtf_if_nonzero(found_local, function);
            self.emit_dtf_set_const(dest_local, *code, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.release_temp_local(found_local);
    }
}

impl<'a> FunctionBuilder<'a> {
    /// `Intl.DateTimeFormat.prototype.resolvedOptions` — ECMA-402 11.4.4.
    ///
    /// The property order below is Table 8's order and is observable through
    /// `Object.getOwnPropertyNames`; do not reorder it. Every component is
    /// written from [`INTL_DTF_COMPONENT_OPTIONS`], the same table the
    /// constructor read it with, so a code can never be spelled two ways.
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
            "Intl.DateTimeFormat.prototype.resolvedOptions",
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
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

    /// Inverse of the option reader: maps a stored code back to the spelling
    /// the same [`IntlDtfOption`] accepted.
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

    /// `Intl.DateTimeFormat.supportedLocalesOf` — ECMA-402 11.2.2.
    ///
    /// `LookupSupportedLocales` over `AvailableLocales = « "en-US" »`: a
    /// requested tag is supported when its language subtag is `en`.
    pub(crate) fn emit_intl_date_time_format_supported_locales_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let locales_payload_local = self.reserve_temp_local();
        let locales_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let has_single_local = self.reserve_temp_local();
        let single_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let input_payload_local = self.reserve_temp_local();
        let tag_payload_local = self.reserve_temp_local();
        let language_local = self.reserve_temp_local();
        let script_local = self.reserve_temp_local();
        let region_local = self.reserve_temp_local();
        let base_name_local = self.reserve_temp_local();
        let ok_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_buffer_local = self.reserve_temp_local();
        let result_len_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, locales_payload_local, locales_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);
        // Step 2: GetOptionsObject then the localeMatcher validation.
        function.instruction(&Instruction::LocalGet(options_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(options_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Intl.DateTimeFormat.supportedLocalesOf options must be an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_intl_dtf_validate_only_option(
            options_payload_local,
            options_tag_local,
            "localeMatcher",
            &["lookup", "best fit"],
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_dtf_set_const(has_single_local, 0, function);
        self.emit_dtf_set_const(single_payload_local, 0, function);
        self.emit_dtf_set_const(source_len_local, 0, function);
        self.emit_dtf_set_const(source_payload_local, 0, function);
        function.instruction(&Instruction::LocalGet(locales_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(locales_payload_local));
        function.instruction(&Instruction::LocalSet(single_payload_local));
        self.emit_dtf_set_const(has_single_local, 1, function);
        self.emit_dtf_set_const(source_len_local, 1, function);
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(locales_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_like_snapshot_payload(
            locales_payload_local,
            locales_tag_local,
            source_payload_local,
            "Intl.DateTimeFormat.supportedLocalesOf locales must be an object",
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            source_payload_local,
            HEAP_LEN_OFFSET,
            source_len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(
            source_len_local,
            result_payload_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            result_payload_local,
            HEAP_PTR_OFFSET,
            result_buffer_local,
            function,
        );
        self.emit_dtf_set_const(result_len_local, 0, function);
        self.emit_dtf_set_const(index_local, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(source_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(has_single_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            source_payload_local,
            index_local,
            element_payload_local,
            element_tag_local,
            function,
        );
        self.emit_intl_locale_argument_to_string_payload(
            element_payload_local,
            element_tag_local,
            input_payload_local,
            "Intl.DateTimeFormat.supportedLocalesOf locale must be a string or an object",
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(single_payload_local));
        function.instruction(&Instruction::LocalSet(input_payload_local));
        function.instruction(&Instruction::End);

        self.emit_intl_canonicalize_locale_tag(
            input_payload_local,
            tag_payload_local,
            language_local,
            script_local,
            region_local,
            base_name_local,
            ok_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid language tag",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_dtf_set_string(expected_local, "en", function);
        self.emit_string_payload_equality_i32(language_local, expected_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(result_buffer_local));
        function.instruction(&Instruction::LocalGet(result_len_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_TAG_OFFSET,
            ValueKind::String.tag() as u64,
            function,
        );
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            tag_payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(result_len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            result_payload_local,
            HEAP_LEN_OFFSET,
            result_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            entry_local,
            result_len_local,
            result_buffer_local,
            result_payload_local,
            expected_local,
            ok_local,
            base_name_local,
            region_local,
            script_local,
            language_local,
            tag_payload_local,
            input_payload_local,
            element_tag_local,
            element_payload_local,
            index_local,
            single_payload_local,
            has_single_local,
            source_len_local,
            source_payload_local,
            options_tag_local,
            options_payload_local,
            locales_tag_local,
            locales_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `get Intl.DateTimeFormat.prototype.format` — ECMA-402 11.4.3.
    ///
    /// The bound function is created once and memoised in the record, so
    /// `dtf.format === dtf.format` holds as the specification requires.
    pub(crate) fn emit_intl_date_time_format_format_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let bound_local = self.reserve_temp_local();
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: format getter without receiver",
            )
        })?;

        self.emit_intl_dtf_record_from_receiver(
            record_local,
            "get Intl.DateTimeFormat.prototype.format",
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Intl.DateTimeFormat Format Function`",
                )
            })?;
        self.emit_function_value_payload(&meta, function)?;
        function.instruction(&Instruction::LocalSet(bound_local));
        // The format function reaches its DateTimeFormat through the function
        // object's environment handle, the same channel a promise resolving
        // function uses for its capability record.
        self.store_i64_local_at_offset(
            bound_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            this_payload_local,
            function,
        );
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
}

impl<'a> FunctionBuilder<'a> {
    /// `if <float local> == <constant> { ... }` — opens a wasm `If`.
    fn emit_dtf_if_float_eq(&self, local: u32, value: f64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(value)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
    }

    /// `if <integer local> == <constant> { ... }` — opens a wasm `If`.
    fn emit_dtf_if_code_eq(&self, local: u32, value: i64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
    }

    /// `if <integer local> != 0 { ... }` — opens a wasm `If`.
    fn emit_dtf_if_nonzero(&self, local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
    }

    /// `dest = ""` then the decimal rendering of `number`, left-padded with
    /// zeroes to `width`.
    fn emit_dtf_number_string(
        &mut self,
        number_local: u32,
        width: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_dtf_set_string(dest_local, "", function);
        self.emit_date_append_padded_decimal(dest_local, number_local, width, function)
    }

    /// Selects one of `names` by a zero-based float index into `dest_local`.
    fn emit_dtf_name_from_index(
        &mut self,
        index_local: u32,
        names: &[&'static str],
        dest_local: u32,
        function: &mut Function,
    ) {
        self.emit_dtf_set_string(dest_local, names[0], function);
        for (index, name) in names.iter().enumerate().skip(1) {
            self.emit_dtf_if_float_eq(index_local, index as f64, function);
            self.emit_dtf_set_string(dest_local, name, function);
            function.instruction(&Instruction::End);
        }
    }

    /// Flushes any pending literal, then appends one field to the sink.
    fn emit_dtf_push(
        &mut self,
        sink: &DtfFormatSink,
        part_type: &'static str,
        value_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_dtf_if_nonzero(sink.pending_literal_local, function);
        self.emit_dtf_append(sink, "literal", sink.pending_literal_local, function)?;
        self.emit_dtf_set_const(sink.pending_literal_local, 0, function);
        function.instruction(&Instruction::End);
        self.emit_dtf_append(sink, part_type, value_local, function)?;
        self.emit_dtf_set_const(sink.emitted_local, 1, function);
        Ok(())
    }

    /// The one place a part reaches the output. In `String` mode it is
    /// concatenated; in `Parts` mode a `{ type, value }` object is appended.
    fn emit_dtf_append(
        &mut self,
        sink: &DtfFormatSink,
        part_type: &'static str,
        value_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match sink.mode {
            DtfFormatMode::String => {
                self.emit_concat_string_payloads_local(sink.text_local, value_local, function)?;
                function.instruction(&Instruction::LocalSet(sink.text_local));
            }
            DtfFormatMode::Parts => {
                let object_local = self.reserve_temp_local();
                let key_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                let entry_local = self.reserve_temp_local();

                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(object_local));
                self.emit_dtf_set_const(tag_local, ValueKind::String.tag() as i64, function);
                self.emit_dtf_set_string(key_local, "type", function);
                self.emit_dtf_set_string(sink.scratch_local, part_type, function);
                self.emit_object_append_data_property_with_flags(
                    object_local,
                    key_local,
                    sink.scratch_local,
                    tag_local,
                    true,
                    true,
                    true,
                    function,
                )?;
                self.emit_dtf_set_string(key_local, "value", function);
                self.emit_object_append_data_property_with_flags(
                    object_local,
                    key_local,
                    value_local,
                    tag_local,
                    true,
                    true,
                    true,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(sink.buffer_local));
                function.instruction(&Instruction::LocalGet(sink.length_local));
                function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(entry_local));
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_ARRAY_TAG_OFFSET,
                    ValueKind::Object.tag() as u64,
                    function,
                );
                self.store_i64_local_at_offset(
                    entry_local,
                    HEAP_ARRAY_PAYLOAD_OFFSET,
                    object_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    entry_local,
                    HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                    ARRAY_DESCRIPTOR_NORMAL_DATA,
                    function,
                );
                function.instruction(&Instruction::LocalGet(sink.length_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(sink.length_local));

                for local in [entry_local, tag_local, key_local, object_local] {
                    self.release_temp_local(local);
                }
            }
        }
        Ok(())
    }

    /// Sets the literal to emit before the next field.
    fn emit_dtf_pending(&mut self, sink: &DtfFormatSink, text: &str, function: &mut Function) {
        self.emit_dtf_set_string(sink.pending_literal_local, text, function);
    }
}

impl<'a> FunctionBuilder<'a> {
    /// `PartitionDateTimePattern` (ECMA-402 11.5.6) for `en-US`/`gregory`/
    /// `latn`/`UTC`, in the one shape `format` and `formatToParts` share.
    ///
    /// `time_local` is a finite time value in milliseconds; the caller has
    /// already run `ToNumber` and `TimeClip`.
    pub(crate) fn emit_intl_dtf_build_format(
        &mut self,
        record_local: u32,
        time_local: u32,
        mode: DtfFormatMode,
        out_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let e_weekday = self.reserve_temp_local();
        let e_era = self.reserve_temp_local();
        let e_year = self.reserve_temp_local();
        let e_month = self.reserve_temp_local();
        let e_day = self.reserve_temp_local();
        let e_day_period = self.reserve_temp_local();
        let e_hour = self.reserve_temp_local();
        let e_minute = self.reserve_temp_local();
        let e_second = self.reserve_temp_local();
        let e_fractional = self.reserve_temp_local();
        let e_time_zone_name = self.reserve_temp_local();
        let hour_cycle_local = self.reserve_temp_local();
        let join_at_local = self.reserve_temp_local();
        let style_local = self.reserve_temp_local();

        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let second_local = self.reserve_temp_local();
        let ms_local = self.reserve_temp_local();
        let weekday_index_local = self.reserve_temp_local();
        let display_year_local = self.reserve_temp_local();
        let scratch_number_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();

        let body_last_local = self.reserve_temp_local();
        let time_started_local = self.reserve_temp_local();
        let has_time_local = self.reserve_temp_local();

        let sink = DtfFormatSink {
            mode,
            text_local: self.reserve_temp_local(),
            array_local: self.reserve_temp_local(),
            buffer_local: self.reserve_temp_local(),
            length_local: self.reserve_temp_local(),
            pending_literal_local: self.reserve_temp_local(),
            emitted_local: self.reserve_temp_local(),
            scratch_local: self.reserve_temp_local(),
        };

        // --- effective components -------------------------------------------
        for (offset, local) in [
            (HEAP_INTL_DTF_WEEKDAY_OFFSET, e_weekday),
            (HEAP_INTL_DTF_ERA_OFFSET, e_era),
            (HEAP_INTL_DTF_YEAR_OFFSET, e_year),
            (HEAP_INTL_DTF_MONTH_OFFSET, e_month),
            (HEAP_INTL_DTF_DAY_OFFSET, e_day),
            (HEAP_INTL_DTF_DAY_PERIOD_OFFSET, e_day_period),
            (HEAP_INTL_DTF_HOUR_OFFSET, e_hour),
            (HEAP_INTL_DTF_MINUTE_OFFSET, e_minute),
            (HEAP_INTL_DTF_SECOND_OFFSET, e_second),
            (HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET, e_fractional),
            (HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET, e_time_zone_name),
            (HEAP_INTL_DTF_HOUR_CYCLE_OFFSET, hour_cycle_local),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_dtf_set_const(join_at_local, 0, function);

        // `dateStyle` expands to the `en-US` date skeleton for that width.
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_DATE_STYLE_OFFSET,
            style_local,
            function,
        );
        for (code, weekday, month, day, year) in [
            (1_i64, 3_i64, 5_i64, 2_i64, 2_i64),
            (2, 0, 5, 2, 2),
            (3, 0, 4, 2, 2),
            (4, 0, 2, 2, 1),
        ] {
            self.emit_dtf_if_code_eq(style_local, code, function);
            self.emit_dtf_set_const(e_weekday, weekday, function);
            self.emit_dtf_set_const(e_month, month, function);
            self.emit_dtf_set_const(e_day, day, function);
            self.emit_dtf_set_const(e_year, year, function);
            if code == 1 || code == 2 {
                self.emit_dtf_set_const(join_at_local, 1, function);
            }
            function.instruction(&Instruction::End);
        }

        // `timeStyle` likewise. The connector only matters when both are set.
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_TIME_STYLE_OFFSET,
            style_local,
            function,
        );
        for (code, hour, minute, second, zone) in [
            (1_i64, 2_i64, 1_i64, 1_i64, 2_i64),
            (2, 2, 1, 1, 1),
            (3, 2, 1, 1, 0),
            (4, 2, 1, 0, 0),
        ] {
            self.emit_dtf_if_code_eq(style_local, code, function);
            self.emit_dtf_set_const(e_hour, hour, function);
            self.emit_dtf_set_const(e_minute, minute, function);
            self.emit_dtf_set_const(e_second, second, function);
            self.emit_dtf_set_const(e_time_zone_name, zone, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(style_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(join_at_local, 0, function);
        function.instruction(&Instruction::End);

        // --- date components ------------------------------------------------
        self.emit_date_components_from_time(
            time_local,
            year_local,
            month_local,
            day_local,
            hour_local,
            minute_local,
            second_local,
            ms_local,
            function,
        );
        self.emit_date_day_from_time(time_local, weekday_index_local, function);
        function.instruction(&Instruction::LocalGet(weekday_index_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(weekday_index_local));
        self.emit_date_positive_mod(weekday_index_local, 7.0, function);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(weekday_index_local));

        // Proleptic Gregorian year 0 is 1 BC, so the displayed year is the era
        // year, never a zero or a negative.
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalSet(display_year_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(display_year_local));
        function.instruction(&Instruction::End);

        // --- sink -----------------------------------------------------------
        self.emit_dtf_set_string(sink.text_local, "", function);
        self.emit_dtf_set_const(sink.pending_literal_local, 0, function);
        self.emit_dtf_set_const(sink.emitted_local, 0, function);
        self.emit_dtf_set_const(sink.length_local, 0, function);
        if mode == DtfFormatMode::Parts {
            self.emit_dtf_set_const(sink.scratch_local, INTL_DTF_MAX_PARTS, function);
            self.emit_alloc_array_payload_with_length(
                sink.scratch_local,
                sink.array_local,
                function,
            )?;
            self.load_i64_to_local_from_offset(
                sink.array_local,
                HEAP_PTR_OFFSET,
                sink.buffer_local,
                function,
            );
        }
        self.emit_dtf_set_const(body_last_local, 0, function);

        // --- weekday --------------------------------------------------------
        self.emit_dtf_if_nonzero(e_weekday, function);
        for (code, names) in [
            (1_i64, &INTL_DTF_WEEKDAYS_NARROW),
            (2, &INTL_DTF_WEEKDAYS_SHORT),
            (3, &INTL_DTF_WEEKDAYS_LONG),
        ] {
            self.emit_dtf_if_code_eq(e_weekday, code, function);
            self.emit_dtf_name_from_index(weekday_index_local, names, value_local, function);
            function.instruction(&Instruction::End);
        }
        self.emit_dtf_push(&sink, "weekday", value_local, function)?;
        function.instruction(&Instruction::End);

        // --- date body ------------------------------------------------------
        // `M/d/y` when the month is numeric or absent, otherwise the textual
        // `MMMM d, y` shape; both are the `en-US` orderings.
        function.instruction(&Instruction::LocalGet(e_month));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_dtf_if_nonzero(e_month, function);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::End);
        self.emit_dtf_month_number(month_local, scratch_number_local, function);
        self.emit_dtf_two_digit_width(e_month, function);
        self.emit_dtf_number_string(scratch_number_local, 2, value_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_dtf_number_string(scratch_number_local, 1, value_local, function)?;
        function.instruction(&Instruction::End);
        self.emit_dtf_push(&sink, "month", value_local, function)?;
        self.emit_dtf_set_const(body_last_local, 1, function);
        function.instruction(&Instruction::End);

        self.emit_dtf_if_nonzero(e_day, function);
        self.emit_dtf_if_nonzero(body_last_local, function);
        self.emit_dtf_pending(&sink, "/", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_dtf_two_digit_width(e_day, function);
        self.emit_dtf_number_string(day_local, 2, value_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_dtf_number_string(day_local, 1, value_local, function)?;
        function.instruction(&Instruction::End);
        self.emit_dtf_push(&sink, "day", value_local, function)?;
        self.emit_dtf_set_const(body_last_local, 2, function);
        function.instruction(&Instruction::End);

        self.emit_dtf_if_nonzero(e_year, function);
        self.emit_dtf_if_nonzero(body_last_local, function);
        self.emit_dtf_pending(&sink, "/", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_dtf_year_value(e_year, display_year_local, value_local, function)?;
        self.emit_dtf_push(&sink, "year", value_local, function)?;
        self.emit_dtf_set_const(body_last_local, 3, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Else);

        self.emit_dtf_if_nonzero(e_month, function);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::End);
        for (code, names) in [
            (3_i64, &INTL_DTF_MONTHS_NARROW),
            (4, &INTL_DTF_MONTHS_SHORT),
            (5, &INTL_DTF_MONTHS_LONG),
        ] {
            self.emit_dtf_if_code_eq(e_month, code, function);
            self.emit_dtf_name_from_index(month_local, names, value_local, function);
            function.instruction(&Instruction::End);
        }
        self.emit_dtf_push(&sink, "month", value_local, function)?;
        self.emit_dtf_set_const(body_last_local, 1, function);
        function.instruction(&Instruction::End);

        self.emit_dtf_if_nonzero(e_day, function);
        self.emit_dtf_if_nonzero(body_last_local, function);
        self.emit_dtf_pending(&sink, " ", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_dtf_two_digit_width(e_day, function);
        self.emit_dtf_number_string(day_local, 2, value_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_dtf_number_string(day_local, 1, value_local, function)?;
        function.instruction(&Instruction::End);
        self.emit_dtf_push(&sink, "day", value_local, function)?;
        self.emit_dtf_set_const(body_last_local, 2, function);
        function.instruction(&Instruction::End);

        self.emit_dtf_if_nonzero(e_year, function);
        self.emit_dtf_if_code_eq(body_last_local, 2, function);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_if_nonzero(body_last_local, function);
        self.emit_dtf_pending(&sink, " ", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_dtf_year_value(e_year, display_year_local, value_local, function)?;
        self.emit_dtf_push(&sink, "year", value_local, function)?;
        self.emit_dtf_set_const(body_last_local, 3, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        // --- era ------------------------------------------------------------
        self.emit_dtf_if_nonzero(e_era, function);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_pending(&sink, " ", function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (code, name) in [(1_i64, "A"), (2, "AD"), (3, "Anno Domini")] {
            self.emit_dtf_if_code_eq(e_era, code, function);
            self.emit_dtf_set_string(value_local, name, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Else);
        for (code, name) in [(1_i64, "B"), (2, "BC"), (3, "Before Christ")] {
            self.emit_dtf_if_code_eq(e_era, code, function);
            self.emit_dtf_set_string(value_local, name, function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.emit_dtf_push(&sink, "era", value_local, function)?;
        function.instruction(&Instruction::End);

        // --- time -----------------------------------------------------------
        function.instruction(&Instruction::LocalGet(e_hour));
        function.instruction(&Instruction::LocalGet(e_minute));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(e_second));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(e_fractional));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(e_day_period));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(has_time_local));
        self.emit_dtf_set_const(time_started_local, 0, function);

        self.emit_dtf_if_nonzero(has_time_local, function);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_if_nonzero(join_at_local, function);
        self.emit_dtf_pending(&sink, " at ", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_dtf_if_nonzero(e_hour, function);
        self.emit_dtf_hour_value(
            e_hour,
            hour_cycle_local,
            hour_local,
            scratch_number_local,
            value_local,
            function,
        )?;
        self.emit_dtf_push(&sink, "hour", value_local, function)?;
        self.emit_dtf_set_const(time_started_local, 1, function);
        function.instruction(&Instruction::End);

        // `en` writes `mm` and `ss` whenever the minute or second shares the
        // pattern with another time field, and only a lone field keeps the
        // width the option asked for.
        for (code_local, component_local, part_type, companions) in [
            (
                e_minute,
                minute_local,
                "minute",
                [e_hour, e_second, e_fractional],
            ),
            (
                e_second,
                second_local,
                "second",
                [e_hour, e_minute, e_second],
            ),
        ] {
            self.emit_dtf_if_nonzero(code_local, function);
            self.emit_dtf_if_nonzero(time_started_local, function);
            self.emit_dtf_pending(&sink, ":", function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::LocalGet(code_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
            for companion in companions {
                if companion == code_local {
                    continue;
                }
                function.instruction(&Instruction::LocalGet(companion));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I32Or);
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_number_string(component_local, 2, value_local, function)?;
            function.instruction(&Instruction::Else);
            self.emit_dtf_number_string(component_local, 1, value_local, function)?;
            function.instruction(&Instruction::End);
            self.emit_dtf_push(&sink, part_type, value_local, function)?;
            self.emit_dtf_set_const(time_started_local, 1, function);
            function.instruction(&Instruction::End);
        }

        self.emit_dtf_if_nonzero(e_fractional, function);
        self.emit_dtf_pending(&sink, ".", function);
        for (digits, divisor) in [(1_i64, 100.0_f64), (2, 10.0), (3, 1.0)] {
            self.emit_dtf_if_code_eq(e_fractional, digits, function);
            function.instruction(&Instruction::LocalGet(ms_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(divisor)));
            function.instruction(&Instruction::F64Div);
            function.instruction(&Instruction::F64Floor);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(scratch_number_local));
            self.emit_dtf_number_string(
                scratch_number_local,
                digits as u32,
                value_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        self.emit_dtf_push(&sink, "fractionalSecond", value_local, function)?;
        self.emit_dtf_set_const(time_started_local, 1, function);
        function.instruction(&Instruction::End);

        // The `dayPeriod` option replaces the `a` marker of a 12-hour pattern.
        self.emit_dtf_if_nonzero(e_day_period, function);
        self.emit_dtf_if_nonzero(time_started_local, function);
        self.emit_dtf_pending(&sink, " ", function);
        function.instruction(&Instruction::End);
        self.emit_dtf_day_period_value(
            e_day_period,
            hour_local,
            minute_local,
            second_local,
            ms_local,
            value_local,
            function,
        );
        self.emit_dtf_push(&sink, "dayPeriod", value_local, function)?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(e_hour));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(hour_cycle_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_pending(&sink, " ", function);
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_string(value_local, "AM", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_set_string(value_local, "PM", function);
        function.instruction(&Instruction::End);
        self.emit_dtf_push(&sink, "dayPeriod", value_local, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // --- time zone name --------------------------------------------------
        self.emit_dtf_if_nonzero(e_time_zone_name, function);
        self.emit_dtf_if_nonzero(time_started_local, function);
        self.emit_dtf_pending(&sink, " ", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_if_nonzero(sink.emitted_local, function);
        self.emit_dtf_pending(&sink, ", ", function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for (code, name) in [
            (1_i64, "UTC"),
            (2, "Coordinated Universal Time"),
            (3, "GMT"),
            (4, "GMT+00:00"),
            (5, "UTC"),
            (6, "UTC"),
        ] {
            self.emit_dtf_if_code_eq(e_time_zone_name, code, function);
            self.emit_dtf_set_string(value_local, name, function);
            function.instruction(&Instruction::End);
        }
        self.emit_dtf_push(&sink, "timeZoneName", value_local, function)?;
        function.instruction(&Instruction::End);

        match mode {
            DtfFormatMode::String => {
                function.instruction(&Instruction::LocalGet(sink.text_local));
                function.instruction(&Instruction::LocalSet(out_local));
            }
            DtfFormatMode::Parts => {
                self.store_i64_local_at_offset(
                    sink.array_local,
                    HEAP_LEN_OFFSET,
                    sink.length_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(sink.array_local));
                function.instruction(&Instruction::LocalSet(out_local));
            }
        }

        for local in [
            sink.scratch_local,
            sink.emitted_local,
            sink.pending_literal_local,
            sink.length_local,
            sink.buffer_local,
            sink.array_local,
            sink.text_local,
            has_time_local,
            time_started_local,
            body_last_local,
            value_local,
            scratch_number_local,
            display_year_local,
            weekday_index_local,
            ms_local,
            second_local,
            minute_local,
            hour_local,
            day_local,
            month_local,
            year_local,
            style_local,
            join_at_local,
            hour_cycle_local,
            e_time_zone_name,
            e_fractional,
            e_second,
            e_minute,
            e_hour,
            e_day_period,
            e_day,
            e_month,
            e_year,
            e_era,
            e_weekday,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `dest = month + 1`, turning the zero-based `MonthFromTime` into the
    /// one-based numeral a pattern prints.
    fn emit_dtf_month_number(&self, month_local: u32, dest_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_local));
    }

    /// Opens `if code == 1 { <2-digit> } else { <numeric> }`; the caller emits
    /// both arms and the closing `End`.
    fn emit_dtf_two_digit_width(&self, code_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
    }

    /// The year numeral: the era year in full, or its last two digits for the
    /// `2-digit` width.
    fn emit_dtf_year_value(
        &mut self,
        code_local: u32,
        display_year_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let scratch_local = self.reserve_temp_local();
        self.emit_dtf_two_digit_width(code_local, function);
        function.instruction(&Instruction::LocalGet(display_year_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(display_year_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(100.0)));
        function.instruction(&Instruction::F64Div);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Const(Ieee64::from(100.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scratch_local));
        self.emit_dtf_number_string(scratch_local, 2, dest_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_dtf_number_string(display_year_local, 1, dest_local, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(scratch_local);
        Ok(())
    }

    /// The hour numeral for the resolved cycle: `h11` wraps to 0-11, `h12` to
    /// 1-12, `h24` to 1-24, `h23` is the raw hour. The 24-hour cycles pad to
    /// two digits, matching the `HH` of the `en` patterns.
    fn emit_dtf_hour_value(
        &mut self,
        code_local: u32,
        hour_cycle_local: u32,
        hour_local: u32,
        scratch_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::LocalSet(scratch_local));
        function.instruction(&Instruction::LocalGet(hour_cycle_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(hour_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scratch_local));
        function.instruction(&Instruction::End);
        self.emit_dtf_if_code_eq(hour_cycle_local, 2, function);
        self.emit_dtf_if_float_eq(scratch_local, 0.0, function);
        function.instruction(&Instruction::F64Const(Ieee64::from(12.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scratch_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_dtf_if_code_eq(hour_cycle_local, 4, function);
        self.emit_dtf_if_float_eq(scratch_local, 0.0, function);
        function.instruction(&Instruction::F64Const(Ieee64::from(24.0)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scratch_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(hour_cycle_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_number_string(scratch_local, 2, dest_local, function)?;
        function.instruction(&Instruction::Else);
        self.emit_dtf_number_string(scratch_local, 1, dest_local, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// The `en` day-period name, following CLDR's rules: `noon` at exactly
    /// 12:00:00.000, morning 06-12, afternoon 12-18, evening 18-21 and night
    /// otherwise.
    fn emit_dtf_day_period_value(
        &mut self,
        code_local: u32,
        hour_local: u32,
        minute_local: u32,
        second_local: u32,
        ms_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        self.emit_dtf_set_string(dest_local, "at night", function);
        for (low, high, name) in [
            (6.0_f64, 12.0_f64, "in the morning"),
            (12.0, 18.0, "in the afternoon"),
            (18.0, 21.0, "in the evening"),
        ] {
            function.instruction(&Instruction::LocalGet(hour_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(low)));
            function.instruction(&Instruction::F64Ge);
            function.instruction(&Instruction::LocalGet(hour_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(high)));
            function.instruction(&Instruction::F64Lt);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_string(dest_local, name, function);
            function.instruction(&Instruction::End);
        }
        self.emit_dtf_if_float_eq(hour_local, 12.0, function);
        self.emit_dtf_if_float_eq(minute_local, 0.0, function);
        self.emit_dtf_if_float_eq(second_local, 0.0, function);
        self.emit_dtf_if_float_eq(ms_local, 0.0, function);
        self.emit_dtf_if_code_eq(code_local, 1, function);
        self.emit_dtf_set_string(dest_local, "n", function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_set_string(dest_local, "noon", function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }
}

impl<'a> FunctionBuilder<'a> {
    /// `HandleDateTimeValue` (ECMA-402 11.5.4) for the non-Temporal path:
    /// `undefined` means "now", anything else goes through `ToNumber` and
    /// `TimeClip`, and a non-finite result is a `RangeError`.
    fn emit_intl_dtf_argument_time(
        &mut self,
        argument_index: usize,
        time_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(
            argument_index,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        let wall_clock_millis_import_function_index = self
            .functions
            .wall_clock_millis_import_function_index()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "Intl.DateTimeFormat format requires the porf_host.wall_clock_millis import",
                )
            })?;
        function.instruction(&Instruction::Call(wall_clock_millis_import_function_index));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(time_local));
        function.instruction(&Instruction::Else);
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_date_time_clip(value_payload_local, time_local, function);
        function.instruction(&Instruction::LocalGet(time_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(time_local));
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
        function.instruction(&Instruction::End);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        Ok(())
    }

    /// The DateTime Format Function (ECMA-402 11.1.5): a nullary-named
    /// closure over the `Intl.DateTimeFormat` that produced it, reached
    /// through the function object's environment handle.
    pub(crate) fn emit_intl_date_time_format_bound_format(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let time_local = self.reserve_temp_local();
        let out_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(object_local));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_intl_dtf_argument_time(0, time_local, function)?;
        self.emit_intl_dtf_build_format(
            record_local,
            time_local,
            DtfFormatMode::String,
            out_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [out_local, time_local, record_local, object_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `Intl.DateTimeFormat.prototype.formatToParts` — ECMA-402 11.4.5.
    pub(crate) fn emit_intl_date_time_format_format_to_parts(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let time_local = self.reserve_temp_local();
        let out_local = self.reserve_temp_local();

        self.emit_intl_dtf_record_from_receiver(
            record_local,
            "Intl.DateTimeFormat.prototype.formatToParts",
            function,
        )?;
        self.emit_intl_dtf_argument_time(0, time_local, function)?;
        self.emit_intl_dtf_build_format(
            record_local,
            time_local,
            DtfFormatMode::Parts,
            out_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [out_local, time_local, record_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}

/// Every string literal the `Intl.DateTimeFormat` emitters ask the pool for.
///
/// Derived from the option tables and the name arrays rather than repeated by
/// hand: adding a spelling to [`INTL_DTF_COMPONENT_OPTIONS`] or a month name
/// puts it in the pool automatically, so the emitter can never reference a
/// string the data section does not contain.
pub(crate) fn intl_date_time_format_pool_strings() -> Vec<String> {
    let mut values: Vec<String> = Vec::new();

    for value in [
        "Intl.DateTimeFormat",
        "DateTimeFormat",
        "supportedLocalesOf",
        "resolvedOptions",
        "format",
        "formatToParts",
        "type",
        "value",
        "literal",
        "fractionalSecond",
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
        INTL_DTF_RESOLVED_LOCALE,
        INTL_DTF_RESOLVED_CALENDAR,
        "gregorian",
        INTL_DTF_RESOLVED_NUMBERING_SYSTEM,
        INTL_DTF_RESOLVED_TIME_ZONE,
        "en",
        "utc",
        "etc/utc",
        "etc/gmt",
        "etc/universal",
        "etc/zulu",
        "gmt",
        "Coordinated Universal Time",
        "GMT",
        "GMT+00:00",
        "AM",
        "PM",
        "A",
        "AD",
        "Anno Domini",
        "B",
        "BC",
        "Before Christ",
        "at night",
        "in the morning",
        "in the afternoon",
        "in the evening",
        "noon",
        "n",
        "",
        " ",
        ", ",
        "/",
        ":",
        ".",
        " at ",
        "0",
    ] {
        values.push(value.to_string());
    }
    for names in [
        &INTL_DTF_MONTHS_LONG[..],
        &INTL_DTF_MONTHS_SHORT[..],
        &INTL_DTF_MONTHS_NARROW[..],
        &INTL_DTF_WEEKDAYS_LONG[..],
        &INTL_DTF_WEEKDAYS_SHORT[..],
        &INTL_DTF_WEEKDAYS_NARROW[..],
    ] {
        for name in names {
            values.push((*name).to_string());
        }
    }
    for (spelling, _) in INTL_DTF_HOUR_CYCLE_OPTION.codes {
        values.push(format!("-hc-{spelling}"));
        values.push(format!("-u-hc-{spelling}"));
    }
    for (spelling, _) in INTL_DTF_HOUR_CYCLE_OPTION.codes {
        values.push(format!("-hc-{spelling}"));
        values.push(format!("-u-hc-{spelling}"));
    }
    for option in INTL_DTF_COMPONENT_OPTIONS.iter().chain([
        &INTL_DTF_HOUR_CYCLE_OPTION,
        &INTL_DTF_DATE_STYLE_OPTION,
        &INTL_DTF_TIME_STYLE_OPTION,
    ]) {
        values.push(option.property.to_string());
        values.push(format!("Invalid {} option", option.property));
        for (spelling, _) in option.codes {
            values.push((*spelling).to_string());
        }
    }
    for property in [
        "localeMatcher",
        "formatMatcher",
        "calendar",
        "numberingSystem",
    ] {
        values.push(format!("Invalid {property} option"));
        values.push(format!("Unsupported {property} option"));
    }
    values.push("Unsupported timeZone option".to_string());
    for method in [
        "Intl.DateTimeFormat.prototype.resolvedOptions",
        "get Intl.DateTimeFormat.prototype.format",
        "Intl.DateTimeFormat.prototype.formatToParts",
    ] {
        values.push(format!(
            "{method} called on a non-Intl.DateTimeFormat object"
        ));
    }
    for value in [
        "Intl.DateTimeFormat constructor requires new",
        "Intl.DateTimeFormat locales must be an object",
        "Intl.DateTimeFormat locale must be a string or an object",
        "Intl.DateTimeFormat.supportedLocalesOf options must be an object",
        "Intl.DateTimeFormat.supportedLocalesOf locales must be an object",
        "Intl.DateTimeFormat.supportedLocalesOf locale must be a string or an object",
        "dateStyle and timeStyle may not be used with explicit date-time components",
        "fractionalSecondDigits must be between 1 and 3",
        "Date value is not finite",
        "Invalid language tag",
    ] {
        values.push(value.to_string());
    }
    values
}
