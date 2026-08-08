//! `Temporal.PlainDate` codegen.
//!
//! Temporal proposal 3: a calendar date with no time and no time zone. The
//! record is three plain `i64` ISO fields plus an interned calendar payload —
//! `RejectISODate` bounds every field, so nothing here needs the BigInt
//! machinery the epoch-nanosecond types carry.
//!
//! Two calendars exist in this backend, [`TemporalCalendarId::Iso8601`] and
//! [`TemporalCalendarId::Gregory`], and they share every piece of arithmetic:
//! `gregory` *is* the proleptic Gregorian calendar, which is what the ISO 8601
//! calendar computes, so `monthCode`, `day`, `daysInMonth`, `daysInYear`,
//! `monthsInYear`, `inLeapYear`, `dayOfWeek`, `dayOfYear`, `weekOfYear` and
//! `yearOfWeek` are bit-identical for the two and need no calendar dispatch at
//! all. They differ in exactly three observable ways, and each has one owner
//! here:
//!
//! * `era`/`eraYear` — [`FunctionBuilder::emit_temporal_calendar_era_field`].
//!   ISO 8601 has no eras; `gregory` has `ce`/`bce`.
//! * `calendarName: "auto"` prints `[u-ca=gregory]` and suppresses
//!   `[u-ca=iso8601]` —
//!   [`FunctionBuilder::emit_temporal_calendar_is_default_i32`], which also
//!   decides `TemporalYearMonthToString`'s reference day and
//!   `TemporalMonthDayToString`'s reference year.
//! * `until`/`since` refuse a calendar mismatch —
//!   [`FunctionBuilder::emit_temporal_require_same_calendar`].
//!
//! Not implemented, and deliberately: resolving a `gregory` property bag from
//! `{ era, eraYear }` instead of `{ year }`. That is a `CalendarResolveFields`
//! feature, not an identifier feature, and every test for it is under
//! `intl402/Temporal`. See [`TemporalCalendarId::Gregory`] for the five
//! currently-passing files that gap knowingly costs.
//!
//! # What the enum does *not* protect
//!
//! Be precise about the invariant, because it is easy to over-read. Exhaustive
//! matches over [`TemporalCalendarId`] exist in `canonical`, `spellings` and
//! [`FunctionBuilder::emit_temporal_calendar_era_field`] — and nowhere else.
//! Every arithmetic and field emitter above (`monthCode`, `daysInMonth`,
//! `daysInYear`, `monthsInYear`, `inLeapYear`, `dayOfWeek`, `weekOfYear`,
//! `yearOfWeek`) takes the ISO path with no calendar dispatch at all, and
//! `emit_temporal_calendar_is_default_i32` asks `== DEFAULT` rather than an
//! exhaustive question. That is *correct* for these two calendars, because
//! `gregory` really is the proleptic Gregorian calendar ISO 8601 computes.
//!
//! It is not correct for a third. Adding `TemporalCalendarId::Japanese`
//! compiles the moment `canonical`, `spellings` and the era field have arms,
//! and every one of those getters then returns a confidently wrong ISO answer.
//! A lane adding a calendar with different arithmetic must first give the enum
//! something the numeric-field emitters are forced to consume — a
//! `const fn arithmetic(self) -> …` whose exhaustive match they read — rather
//! than trusting this file's existing matches to stop it.

use super::super::*;

/// Every calendar identifier this backend answers to.
///
/// A closed enum rather than a `&str` compared at each site. `CanonicalizeCalendar`,
/// `era`, `eraYear`, `FormatCalendarAnnotation` and `CalendarEquals` each have
/// to decide something per calendar; an exhaustive `match` over this enum is
/// what turns "you added `japanese` and forgot `eraYear`" into a compile error.
/// There is deliberately **no** `_` arm anywhere this enum is matched — a
/// catch-all here is a silent wrong answer, not a missing feature.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalCalendarId {
    /// The proposal's default. No eras, and `calendarName: "auto"` suppresses
    /// its annotation.
    Iso8601,
    /// Proleptic Gregorian. Identical arithmetic to [`Self::Iso8601`]; it adds
    /// the `ce`/`bce` era pair and is always annotated under `auto`.
    ///
    /// # Known gap: `CalendarResolveFields` does not read `era`/`eraYear`
    ///
    /// The `era`/`eraYear` *accessors* are implemented (see
    /// [`FunctionBuilder::emit_temporal_gregorian_era_field`]), but nothing on
    /// the property-bag `from()` paths reads them back: the shared field
    /// resolvers take a `year`/`year_present` pair and know nothing about eras.
    /// So `{ era, eraYear }` is silently ignored as a year source, and the
    /// spec's `era`-half of `CalendarResolveFields` — unknown era is a
    /// `RangeError`, exactly one of the pair present is a `TypeError`, an
    /// `eraYear` disagreeing with an explicit `year` is a `RangeError` — is
    /// absent.
    ///
    /// Accepting `gregory` therefore knowingly turns five `intl402/Temporal`
    /// files from vacuous passes (they threw `RangeError` only because the
    /// identifier itself was rejected) into real failures:
    ///
    /// * `PlainDate/from/calendar-invalid-era-with-era-year.js`
    /// * `PlainDateTime/from/calendar-invalid-era-with-era-year.js`
    /// * `ZonedDateTime/from/calendar-invalid-era-with-era-year.js`
    ///   — all three now reach "fields require year" and give `TypeError`
    ///   where `RangeError` is asserted.
    /// * `PlainMonthDay/from/dont-calculate-month-info-for-out-of-range-year.js`
    ///   — `PlainMonthDay` deliberately neither range-checks nor stores a
    ///   supplied `year` (see `emit_temporal_month_day_resolve_fields`), so all
    ///   four `gregory` rows now succeed instead of throwing.
    /// * `PlainMonthDay/from/fields-overspecified.js`
    ///   — its `eraYear`/`year` disagreement case is simply a valid bag here.
    ///
    /// This is an accepted, recorded regression, not an oversight: the fix is
    /// the `era` half of `CalendarResolveFields` plus a `PlainMonthDay` year
    /// range check, which is its own lane. Do not "fix" it by making an
    /// unsupported `era` throw — that would be a right answer for a wrong
    /// reason, and would reject the conforming `{ era: "ce", eraYear: 2024 }`
    /// bag along with the malformed ones.
    Gregory,
}

impl TemporalCalendarId {
    /// Every calendar, in the order `CanonicalizeCalendar` tests them. Order is
    /// not observable — the spellings are disjoint — but keeping the default
    /// first keeps the common case's compare first.
    pub(crate) const ALL: [Self; 2] = [Self::Iso8601, Self::Gregory];

    /// `ToTemporalCalendarIdentifier(undefined)`.
    pub(crate) const DEFAULT: Self = Self::Iso8601;

    /// The single spelling every `[[Calendar]]` slot stores and every
    /// `calendarId` reports. Canonicalisation happens once, in
    /// [`FunctionBuilder::emit_temporal_canonicalize_calendar`], so no code
    /// downstream of a slot has to case-fold or alias again.
    ///
    /// Must agree with `INTL_DTF_ACCEPTED_CALENDARS` in
    /// `builtins/intl_datetimeformat.rs`; the integration note for this batch
    /// carries the `const` assertion that pins the two together.
    pub(crate) const fn canonical(self) -> &'static str {
        match self {
            Self::Iso8601 => "iso8601",
            Self::Gregory => "gregory",
        }
    }

    /// Every spelling `CanonicalizeCalendar` accepts, matched
    /// ASCII-case-insensitively. The canonical spelling is always one of them.
    pub(crate) const fn spellings(self) -> &'static [&'static str] {
        match self {
            Self::Iso8601 => &["iso8601"],
            // `gregorian` is the Unicode CLDR alias of `gregory`; CLDR's
            // alias table is normative for `CanonicalizeCalendar`, so both
            // spellings must resolve to the one canonical `gregory`.
            Self::Gregory => &["gregory", "gregorian"],
        }
    }
}

/// Which half of the `era`/`eraYear` accessor pair an emitter is producing.
///
/// One emitter serves both, so the pair cannot disagree about where the year-0
/// boundary falls; this names which answer the caller wants out of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalEraField {
    Era,
    EraYear,
}

/// The two eras of the proleptic Gregorian calendar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Era {
    Ce,
    Bce,
}

impl Era {
    /// The identifier `era` reports. Lowercase, per the CLDR era codes the
    /// proposal adopted (`"ce"`/`"bce"`, not `"AD"`/`"BC"`).
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::Ce => "ce",
            Self::Bce => "bce",
        }
    }

    /// Both eras, ordered by the sign of the ISO year that selects them:
    /// positive first.
    ///
    /// The ordering is what the type is for. `era` and `eraYear` are two
    /// accessors emitted under one `isoYear > 0` test, and each needs two arms.
    /// Choosing those arms per accessor is what would let `era` answer `ce` on
    /// the branch where `eraYear` counts backwards;
    /// [`FunctionBuilder::emit_temporal_gregorian_era_field`] instead
    /// destructures this one array for both and keys the arithmetic on the
    /// `Era` value rather than on branch position, so the pair cannot disagree
    /// about which side of year 0 it is on. The boundary is: ISO year 1 is
    /// `ce` 1, ISO year 0 is `bce` 1, ISO year -1 is `bce` 2.
    ///
    /// It is also the set the string pool derives the era codes from, so an
    /// era added here cannot be missing from the pool.
    ///
    /// `Intl.DateTimeFormat` encodes the same boundary independently (see the
    /// `display_year` computation in `builtins/intl_datetimeformat.rs`); the
    /// integration note records that duplication.
    pub(crate) const ALL: [Self; 2] = [Self::Ce, Self::Bce];
}

/// The five branded Temporal types the specification gives a `[[Calendar]]`
/// internal slot, each paired with where that slot lives in its boxed record.
///
/// The pairing is the point. `emit_temporal_calendar_slot_fast_path` used to
/// take a substitute payload and write it back for every brand, so it answered
/// `iso8601` for everything; that was correct only while `iso8601` was the sole
/// calendar, and it would have kept compiling — and kept answering `iso8601` —
/// after `gregory` landed. Reading the real slot needs a per-brand offset, and
/// a brand with no offset now fails to build.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalCalendarCarrier {
    PlainDate,
    PlainDateTime,
    PlainMonthDay,
    PlainYearMonth,
    ZonedDateTime,
}

impl TemporalCalendarCarrier {
    /// `Temporal.Instant`, `Temporal.Duration` and `Temporal.PlainTime` are
    /// deliberately absent: they carry no `[[Calendar]]`, so a value of those
    /// brands must fall through to the caller's TypeError.
    pub(crate) const ALL: [Self; 5] = [
        Self::PlainDate,
        Self::PlainDateTime,
        Self::PlainMonthDay,
        Self::PlainYearMonth,
        Self::ZonedDateTime,
    ];

    pub(crate) const fn brand(self) -> u64 {
        match self {
            Self::PlainDate => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE,
            Self::PlainDateTime => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME,
            Self::PlainMonthDay => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY,
            Self::PlainYearMonth => OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH,
            Self::ZonedDateTime => OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME,
        }
    }

    /// Byte offset of the interned calendar payload inside the boxed record.
    pub(crate) const fn calendar_payload_offset(self) -> u64 {
        match self {
            // `PlainMonthDay` and `PlainYearMonth` are stored in the
            // `PlainDate` record shape under a different brand, so all three
            // share one offset.
            Self::PlainDate | Self::PlainMonthDay | Self::PlainYearMonth => {
                HEAP_TEMPORAL_PLAIN_DATE_CALENDAR_PAYLOAD_OFFSET
            }
            Self::PlainDateTime => HEAP_TEMPORAL_PLAIN_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
            Self::ZonedDateTime => HEAP_TEMPORAL_ZONED_DATE_TIME_CALENDAR_PAYLOAD_OFFSET,
        }
    }
}

/// `ISODateToEpochDays` bounds from `ISODateWithinLimits`: noon on the day must
/// stay inside `nsMinInstant - nsPerDay` .. `nsMaxInstant + nsPerDay`, which
/// works out to one more day below the epoch-day limit than above it.
/// `-271821-04-19` and `+275760-09-13` are the exact endpoints Test262's
/// `PlainDate/limits.js` pins.
const TEMPORAL_PLAIN_DATE_MINIMUM_EPOCH_DAY: i64 = -100_000_001;
const TEMPORAL_PLAIN_DATE_MAXIMUM_EPOCH_DAY: i64 = 100_000_000;

impl<'a> FunctionBuilder<'a> {
    /// `ToIntegerWithTruncation`: `ToNumber`, reject NaN and the infinities with
    /// a RangeError, then truncate toward zero.
    pub(crate) fn emit_temporal_to_integer_with_truncation(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        output_local: u32,
        error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_value_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::MAX)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            error_message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(output_local));
        Ok(())
    }

    /// `ToTemporalCalendarIdentifier` step 1: an object that already carries a
    /// `[[Calendar]]` internal slot resolves to *that slot*, without any
    /// observable property access — no `calendar` / `calendarId` getter runs.
    ///
    /// It reads the object's own slot and takes no substitute payload on
    /// purpose. The previous shape accepted an `iso8601` payload and wrote it
    /// back for every matched brand; that was right only while `iso8601` was
    /// the only calendar, and it would have kept compiling — and kept
    /// answering `iso8601` for a `gregory` receiver — once a second calendar
    /// existed. Dropping the parameter is what turns "a caller assumed
    /// `iso8601`" into a build error instead of a wrong `calendarId`.
    ///
    /// Rewrites `calendar_*_local` in place to the `String`-tagged slot when
    /// the fast path applies, and leaves them untouched otherwise.
    pub(crate) fn emit_temporal_calendar_slot_fast_path(
        &mut self,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        function: &mut Function,
    ) {
        let object_payload_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // The receiver pointer has to survive the brand walk: the caller's
        // `calendar_payload_local` is both the object coming in and the slot
        // payload going out, so the first matching brand would otherwise
        // clobber the pointer the remaining brands still read.
        function.instruction(&Instruction::LocalGet(calendar_payload_local));
        function.instruction(&Instruction::LocalSet(object_payload_local));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        // Exactly one brand can match, so these are independent `if`s rather
        // than a chain; a value carrying none of them leaves the locals alone
        // and falls through to the caller's TypeError.
        for carrier in TemporalCalendarCarrier::ALL {
            function.instruction(&Instruction::LocalGet(brand_local));
            function.instruction(&Instruction::I64Const(carrier.brand() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                object_payload_local,
                HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                record_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                record_local,
                carrier.calendar_payload_offset(),
                calendar_payload_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(calendar_tag_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.release_temp_local(record_local);
        self.release_temp_local(brand_local);
        self.release_temp_local(object_payload_local);
    }

    /// `CanonicalizeCalendar`, shared by every constructor that takes a
    /// calendar argument.
    ///
    /// `undefined` defaults to [`TemporalCalendarId::DEFAULT`]; an object with
    /// a `[[Calendar]]` slot resolves to that slot; any other non-string is a
    /// TypeError; a string that is no spelling of any [`TemporalCalendarId`] is
    /// a RangeError. The RangeError must stay *after* the TypeError and must
    /// keep firing for `""`, `"notacal"`, `"11111111"`, `"1111-11-11"` and
    /// `"1997-12-04[u-ca=iso8601]"` — the five rows of
    /// `PlainDate/calendar-invalid-iso-string.js` and its seven siblings. This
    /// operation is *not* `ParseTemporalCalendarString`: an ISO date string is
    /// rejected here and accepted by
    /// [`Self::emit_temporal_to_temporal_calendar_identifier`].
    ///
    /// Exactly one place canonicalises, so every `[[Calendar]]` slot in the
    /// heap holds a pooled [`TemporalCalendarId::canonical`] payload and no
    /// reader downstream has to case-fold or resolve an alias again.
    pub(crate) fn emit_temporal_canonicalize_calendar(
        &mut self,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        type_error_message: &str,
        range_error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let expected_payload_local = self.reserve_temp_local();
        let canonical_payload_local = self.reserve_temp_local();
        let matched_local = self.reserve_temp_local();
        let case_fold_local = self.reserve_temp_local();

        self.emit_temporal_calendar_slot_fast_path(
            calendar_payload_local,
            calendar_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings
                .payload(TemporalCalendarId::DEFAULT.canonical()),
        ));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
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
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(matched_local));
        for calendar in TemporalCalendarId::ALL {
            let canonical_payload = self.strings.payload(calendar.canonical());
            for &spelling in calendar.spellings() {
                function.instruction(&Instruction::I64Const(self.strings.payload(spelling)));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(case_fold_local));
                self.emit_string_payload_equality_i32_with_ascii_case_folding(
                    calendar_payload_local,
                    expected_payload_local,
                    Some(case_fold_local),
                    function,
                );
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(canonical_payload));
                function.instruction(&Instruction::LocalSet(canonical_payload_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(matched_local));
                function.instruction(&Instruction::End);
            }
        }
        function.instruction(&Instruction::LocalGet(matched_local));
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
        function.instruction(&Instruction::LocalGet(canonical_payload_local));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(case_fold_local);
        self.release_temp_local(matched_local);
        self.release_temp_local(canonical_payload_local);
        self.release_temp_local(expected_payload_local);
        Ok(())
    }

    /// `CanonicalizeCalendar` with the `Temporal.PlainDate` family's messages.
    /// Used by the `PlainDate`, `PlainDateTime`, `PlainYearMonth` and
    /// `PlainMonthDay` constructors, which all report the same two.
    pub(crate) fn emit_temporal_plain_date_calendar(
        &mut self,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_temporal_canonicalize_calendar(
            calendar_payload_local,
            calendar_tag_local,
            "Temporal.PlainDate calendar must be a string",
            "Invalid Temporal.PlainDate calendar",
            function,
        )
    }

    /// Leaves an `i32` on the stack: 1 when the calendar payload is
    /// [`TemporalCalendarId::DEFAULT`].
    ///
    /// `FormatCalendarAnnotation` step 2, `TemporalYearMonthToString` step 4
    /// and `TemporalMonthDayToString` step 2 all ask exactly this question, so
    /// they all ask it here.
    pub(crate) fn emit_temporal_calendar_is_default_i32(
        &mut self,
        calendar_payload_local: u32,
        function: &mut Function,
    ) {
        let default_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .payload(TemporalCalendarId::DEFAULT.canonical()),
        ));
        function.instruction(&Instruction::LocalSet(default_payload_local));
        self.emit_string_payload_equality_i32(
            calendar_payload_local,
            default_payload_local,
            function,
        );
        self.release_temp_local(default_payload_local);
    }

    /// `CalendarEquals` as the difference operations use it: `until` and
    /// `since` throw a RangeError when the two receivers name different
    /// calendars, before any option is read.
    ///
    /// With one calendar this could never fire; with two it is the difference
    /// between `PlainDateTime/prototype/{until,since}/different-calendars-throws.js`
    /// passing because the feature works and passing because
    /// `new Temporal.PlainDateTime(..., "gregory")` threw first.
    pub(crate) fn emit_temporal_require_same_calendar(
        &mut self,
        calendar_payload_local: u32,
        other_calendar_payload_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_string_payload_equality_i32(
            calendar_payload_local,
            other_calendar_payload_local,
            function,
        );
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `era` and `eraYear` for any calendar, from one emitter.
    ///
    /// The dispatch is an exhaustive `match` over [`TemporalCalendarId`] with
    /// no catch-all, so a calendar added to that enum has to state here what
    /// its eras are before this compiles. The default answer is `undefined`,
    /// which is what a calendar with no eras reports.
    pub(crate) fn emit_temporal_calendar_era_field(
        &mut self,
        calendar_payload_local: u32,
        iso_year_local: u32,
        field: TemporalEraField,
        function: &mut Function,
    ) {
        let expected_payload_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        for calendar in TemporalCalendarId::ALL {
            match calendar {
                // The ISO 8601 calendar has no eras, so both slots stay
                // `undefined` rather than borrowing a fabricated `ce`/`bce`.
                TemporalCalendarId::Iso8601 => {}
                TemporalCalendarId::Gregory => {
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
                    self.emit_temporal_gregorian_era_field(iso_year_local, field, function);
                    function.instruction(&Instruction::End);
                }
            }
        }
        self.release_temp_local(expected_payload_local);
    }

    /// Proleptic Gregorian `era` / `eraYear` into the result pair.
    ///
    /// One `isoYear > 0` test, and both of its arms are filled from
    /// [`Era::ALL`] in its declared order. Neither accessor decides for itself
    /// which branch is `ce`: `emit_temporal_gregorian_era_arm` is handed the
    /// [`Era`] and derives *both* the era code and the era-year arithmetic from
    /// it, so the `era` an accessor reports and the `eraYear` beside it cannot
    /// disagree about year 0 being `bce` 1 — and a third era would fail to
    /// build there rather than silently reuse the `bce` formula.
    fn emit_temporal_gregorian_era_field(
        &mut self,
        iso_year_local: u32,
        field: TemporalEraField,
        function: &mut Function,
    ) {
        let [positive_year_era, non_positive_year_era] = Era::ALL;
        function.instruction(&Instruction::LocalGet(iso_year_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        self.emit_temporal_gregorian_era_arm(iso_year_local, positive_year_era, field, function);
        function.instruction(&Instruction::Else);
        self.emit_temporal_gregorian_era_arm(
            iso_year_local,
            non_positive_year_era,
            field,
            function,
        );
        function.instruction(&Instruction::End);
        match field {
            TemporalEraField::Era => {
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            TemporalEraField::EraYear => {
                function.instruction(&Instruction::F64ConvertI64S);
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
        }
    }

    /// One arm of [`Self::emit_temporal_gregorian_era_field`]: leaves either the
    /// era's interned code or its era-year on the stack as an `i64`.
    ///
    /// The era-year arithmetic is keyed on the [`Era`], not on which branch
    /// this is, which is what makes swapping the two arms swap both answers
    /// together instead of producing `ce` counting backwards.
    fn emit_temporal_gregorian_era_arm(
        &self,
        iso_year_local: u32,
        era: Era,
        field: TemporalEraField,
        function: &mut Function,
    ) {
        match field {
            TemporalEraField::Era => {
                function.instruction(&Instruction::I64Const(self.strings.payload(era.code())));
            }
            TemporalEraField::EraYear => match era {
                // `ce` 1 is ISO year 1, and it counts up with it.
                Era::Ce => {
                    function.instruction(&Instruction::LocalGet(iso_year_local));
                }
                // Proleptic year 0 is 1 bce, so the era year counts backwards
                // from 1: `1 - isoYear`.
                Era::Bce => {
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalGet(iso_year_local));
                    function.instruction(&Instruction::I64Sub);
                }
            },
        }
    }

    /// `ToTemporalCalendarIdentifier` — the property-bag / `withCalendar` form,
    /// which runs `ParseTemporalCalendarString` on a string argument. A string
    /// is first tried as a `TemporalDateString` / `TemporalYearMonthString` /
    /// `TemporalMonthDayString` / `TemporalTimeString`, and only the bare
    /// `AnnotationValue` spelling falls through to `CanonicalizeCalendar`. So
    /// `{ calendar: "2020-01-01" }` resolves to `iso8601` rather than throwing.
    ///
    /// This is NOT the constructor form: `new Temporal.PlainDate(y, m, d, cal)`
    /// calls `CanonicalizeCalendar` directly, so `"1111-11-11"` and
    /// `"11111111"` must stay a RangeError there
    /// (`PlainDate/calendar-invalid-iso-string.js` and its four siblings).
    /// Keep using [`Self::emit_temporal_plain_date_calendar`] for the five
    /// constructors — switching them here is a net regression.
    ///
    /// The prefix (slot fast path, `undefined` default, non-string TypeError)
    /// is identical to `emit_temporal_plain_date_calendar`; only the string
    /// resolution differs, and that is outlined into the shared
    /// `ToTemporalCalendarIdentifier` helper so the ISO parser is inlined once
    /// per module rather than once per call site.
    pub(crate) fn emit_temporal_to_temporal_calendar_identifier(
        &mut self,
        calendar_payload_local: u32,
        calendar_tag_local: u32,
        type_error_message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper_index = self
            .temporal_calendar_identifier_helper_function_index()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: \
                     ToTemporalCalendarIdentifier helper without heap",
                )
            })?;
        let expected_payload_local = self.reserve_temp_local();
        let saved_result_local = self.reserve_temp_local();
        let saved_result_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .payload(TemporalCalendarId::DEFAULT.canonical()),
        ));
        function.instruction(&Instruction::LocalSet(expected_payload_local));
        self.emit_temporal_calendar_slot_fast_path(
            calendar_payload_local,
            calendar_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(expected_payload_local));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(calendar_tag_local));
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

        // The helper reports its own throw through the completion tuple, so the
        // caller's in-flight statement result has to survive a successful call.
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(saved_result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(saved_result_tag_local));

        function.instruction(&Instruction::LocalGet(calendar_payload_local));
        for _ in 0..5 {
            function.instruction(&Instruction::I64Const(0));
        }
        // Only created-realm standard builtins use a self-backed environment
        // that the shared helper may interpret as realm metadata. User
        // functions can have nonzero lexical environments with a different
        // layout, so pass zero for every non-standard caller.
        if self
            .function_id
            .as_ref()
            .and_then(|function_id| StandardBuiltinId::from_function_id(function_id))
            .is_some()
        {
            function.instruction(&Instruction::LocalGet(self.current_env_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Call(helper_index));
        self.store_call_results_to(
            self.result_local,
            self.result_tag_local,
            self.completion_local,
            self.completion_aux_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(calendar_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(calendar_tag_local));
        function.instruction(&Instruction::LocalGet(saved_result_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(saved_result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(saved_result_tag_local);
        self.release_temp_local(saved_result_local);
        self.release_temp_local(expected_payload_local);
        Ok(())
    }

    /// Leaves an `i32` on the stack: 1 when the ISO year is a leap year.
    pub(crate) fn emit_temporal_iso_year_is_leap_i32(
        &mut self,
        year_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(100));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(400));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Or);
    }

    /// `ISODaysInMonth` into `output_local`.
    pub(crate) fn emit_temporal_iso_days_in_month(
        &mut self,
        year_local: u32,
        month_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::LocalSet(output_local));
        for month in [4_i64, 6, 9, 11] {
            function.instruction(&Instruction::LocalGet(month_local));
            function.instruction(&Instruction::I64Const(month));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(30));
            function.instruction(&Instruction::LocalSet(output_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(month_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_temporal_iso_year_is_leap_i32(year_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(28));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
    }

    /// `ISODateToEpochDays` into `days_local`, reserving and releasing the
    /// scratch locals `emit_temporal_days_from_civil` needs.
    pub(crate) fn emit_temporal_plain_date_epoch_days(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        days_local: u32,
        function: &mut Function,
    ) {
        let adjusted_year_local = self.reserve_temp_local();
        let era_local = self.reserve_temp_local();
        let month_index_local = self.reserve_temp_local();
        self.emit_temporal_days_from_civil(
            year_local,
            month_local,
            day_local,
            adjusted_year_local,
            era_local,
            month_index_local,
            days_local,
            function,
        );
        self.release_temp_local(month_index_local);
        self.release_temp_local(era_local);
        self.release_temp_local(adjusted_year_local);
    }

    /// `RejectISODate` followed by the `ISODateWithinLimits` check that
    /// `CreateTemporalDate` performs. Both failures are RangeErrors, so the
    /// two are fused into one guard.
    pub(crate) fn emit_temporal_reject_iso_date(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let maximum_day_local = self.reserve_temp_local();
        let days_local = self.reserve_temp_local();

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
            "Temporal.PlainDate is not a valid ISO date",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_temporal_plain_date_epoch_days(
            year_local,
            month_local,
            day_local,
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_DATE_MINIMUM_EPOCH_DAY,
        ));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(
            TEMPORAL_PLAIN_DATE_MAXIMUM_EPOCH_DAY,
        ));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal.PlainDate is outside the supported date range",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(days_local);
        self.release_temp_local(maximum_day_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_temporal_plain_date(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        calendar_payload_local: u32,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_payload_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
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
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE,
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
        self.release_temp_local(record_local);
        self.release_temp_local(object_payload_local);
        Ok(())
    }

    /// The `[[InitializedTemporalDate]]` brand check. On failure it throws and
    /// returns, so callers may assume `record_local` is a live record after it.
    pub(crate) fn emit_temporal_plain_date_record_from_receiver(
        &mut self,
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
            "Temporal.PlainDate receiver does not have [[InitializedTemporalDate]]",
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
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal.PlainDate receiver does not have [[InitializedTemporalDate]]",
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

    fn emit_temporal_plain_date_load_record(
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

    /// Temporal proposal 3.1: `Temporal.PlainDate(isoYear, isoMonth, isoDay [, calendar])`.
    pub(crate) fn emit_temporal_plain_date_constructor(
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
        let prototype_payload_local = self.reserve_temp_local();
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
            "Temporal.PlainDate constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        for (index, output_local, message) in [
            (0, year_local, "Temporal.PlainDate year must be an integer"),
            (
                1,
                month_local,
                "Temporal.PlainDate month must be an integer",
            ),
            (2, day_local, "Temporal.PlainDate day must be an integer"),
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
        self.emit_builtin_arg_to_locals(3, calendar_payload_local, calendar_tag_local, function);
        self.emit_temporal_plain_date_calendar(
            calendar_payload_local,
            calendar_tag_local,
            function,
        )?;
        self.emit_temporal_reject_iso_date(year_local, month_local, day_local, function)?;
        self.emit_error_new_target_prototype_to_local(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
            None,
            prototype_payload_local,
            function,
        )?;
        self.emit_alloc_temporal_plain_date(
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            prototype_payload_local,
            function,
        )?;

        for local in [
            new_target_tag_local,
            new_target_payload_local,
            prototype_payload_local,
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

    /// Every `Temporal.PlainDate.prototype` accessor. They all start from the
    /// same three ISO fields, so one emitter serves the family the way
    /// `emit_temporal_zoned_date_time_iso_field` does for ZonedDateTime.
    pub(crate) fn emit_temporal_plain_date_field(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let calendar_payload_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_record_from_receiver(record_local, function)?;
        self.emit_temporal_plain_date_load_record(
            record_local,
            year_local,
            month_local,
            day_local,
            calendar_payload_local,
            function,
        );

        match builtin {
            StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter => {
                function.instruction(&Instruction::LocalGet(calendar_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter => {
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
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeEraGetter => {
                self.emit_temporal_calendar_era_field(
                    calendar_payload_local,
                    year_local,
                    TemporalEraField::Era,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter => {
                self.emit_temporal_calendar_era_field(
                    calendar_payload_local,
                    year_local,
                    TemporalEraField::EraYear,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter => {
                self.emit_temporal_iso_year_is_leap_i32(year_local, function);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
            }
            _ => {
                let value_local = self.reserve_temp_local();
                self.emit_temporal_plain_date_numeric_field(
                    builtin,
                    year_local,
                    month_local,
                    day_local,
                    value_local,
                    function,
                );
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

    /// The purely-numeric accessors. Split out of `emit_temporal_plain_date_field`
    /// so the calendar-arithmetic locals are only reserved when a caller needs
    /// them.
    pub(crate) fn emit_temporal_plain_date_numeric_field(
        &mut self,
        builtin: StandardBuiltinId,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        match builtin {
            StandardBuiltinId::TemporalPlainDatePrototypeYearGetter => {
                function.instruction(&Instruction::LocalGet(year_local));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter => {
                function.instruction(&Instruction::LocalGet(month_local));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDayGetter => {
                function.instruction(&Instruction::LocalGet(day_local));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter => {
                function.instruction(&Instruction::I64Const(7));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter => {
                function.instruction(&Instruction::I64Const(12));
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter => {
                self.emit_temporal_iso_days_in_month(
                    year_local,
                    month_local,
                    output_local,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter => {
                self.emit_temporal_iso_year_is_leap_i32(year_local, function);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(366));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(365));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(output_local));
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter => {
                self.emit_temporal_plain_date_day_of_week(
                    year_local,
                    month_local,
                    day_local,
                    output_local,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter => {
                self.emit_temporal_plain_date_day_of_year(
                    year_local,
                    month_local,
                    day_local,
                    output_local,
                    function,
                );
            }
            StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter => {
                let week_local = self.reserve_temp_local();
                let year_of_week_local = self.reserve_temp_local();
                self.emit_temporal_plain_date_iso_week(
                    year_local,
                    month_local,
                    day_local,
                    week_local,
                    year_of_week_local,
                    function,
                );
                let selected = if matches!(
                    builtin,
                    StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
                ) {
                    week_local
                } else {
                    year_of_week_local
                };
                function.instruction(&Instruction::LocalGet(selected));
                function.instruction(&Instruction::LocalSet(output_local));
                self.release_temp_local(year_of_week_local);
                self.release_temp_local(week_local);
            }
            _ => unreachable!("non-numeric Temporal.PlainDate accessor"),
        }
    }

    /// ISO weekday, 1 = Monday .. 7 = Sunday. Epoch day 0 is a Thursday, so the
    /// `+3` shift lands Monday on 0 before the floor-mod.
    fn emit_temporal_plain_date_day_of_week(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let days_local = self.reserve_temp_local();
        self.emit_temporal_plain_date_epoch_days(
            year_local,
            month_local,
            day_local,
            days_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(days_local));
        // `I64RemS` truncates toward zero, so a pre-epoch date would yield a
        // negative remainder; the `+7 % 7` restores floor semantics.
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_local));
        self.release_temp_local(days_local);
    }

    fn emit_temporal_plain_date_day_of_year(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        output_local: u32,
        function: &mut Function,
    ) {
        let days_local = self.reserve_temp_local();
        let start_local = self.reserve_temp_local();
        let one_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(one_local));
        self.emit_temporal_plain_date_epoch_days(
            year_local,
            month_local,
            day_local,
            days_local,
            function,
        );
        self.emit_temporal_plain_date_epoch_days(
            year_local,
            one_local,
            one_local,
            start_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(days_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(output_local));
        self.release_temp_local(one_local);
        self.release_temp_local(start_local);
        self.release_temp_local(days_local);
    }

    /// Leaves an `i32` on the stack: 1 when ISO year `year_local` has 53 weeks.
    /// A year is long when 1 January is a Thursday, or when it is a leap year
    /// starting on a Wednesday.
    fn emit_temporal_plain_date_year_is_long_i32(
        &mut self,
        year_local: u32,
        function: &mut Function,
    ) {
        let january_first_weekday_local = self.reserve_temp_local();
        let one_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(one_local));
        self.emit_temporal_plain_date_day_of_week(
            year_local,
            one_local,
            one_local,
            january_first_weekday_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(january_first_weekday_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(january_first_weekday_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Eq);
        self.emit_temporal_iso_year_is_leap_i32(year_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Or);
        self.release_temp_local(one_local);
        self.release_temp_local(january_first_weekday_local);
    }

    /// ISO 8601 week-of-year and its week-numbering year.
    fn emit_temporal_plain_date_iso_week(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        week_local: u32,
        year_of_week_local: u32,
        function: &mut Function,
    ) {
        let day_of_year_local = self.reserve_temp_local();
        let day_of_week_local = self.reserve_temp_local();
        let adjacent_year_local = self.reserve_temp_local();
        let weeks_in_year_local = self.reserve_temp_local();

        self.emit_temporal_plain_date_day_of_year(
            year_local,
            month_local,
            day_local,
            day_of_year_local,
            function,
        );
        self.emit_temporal_plain_date_day_of_week(
            year_local,
            month_local,
            day_local,
            day_of_week_local,
            function,
        );
        // `dayOfYear >= 1` and `dayOfWeek <= 7`, so the dividend is at least 4
        // and the truncating `I64DivS` already floors.
        function.instruction(&Instruction::LocalGet(day_of_year_local));
        function.instruction(&Instruction::LocalGet(day_of_week_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::LocalSet(week_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalSet(year_of_week_local));

        function.instruction(&Instruction::LocalGet(week_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(adjacent_year_local));
        function.instruction(&Instruction::LocalGet(adjacent_year_local));
        function.instruction(&Instruction::LocalSet(year_of_week_local));
        self.emit_temporal_plain_date_year_is_long_i32(adjacent_year_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(53));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(week_local));
        function.instruction(&Instruction::Else);
        self.emit_temporal_plain_date_year_is_long_i32(year_local, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(53));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(52));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(weeks_in_year_local));
        function.instruction(&Instruction::LocalGet(week_local));
        function.instruction(&Instruction::LocalGet(weeks_in_year_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(week_local));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(year_of_week_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(weeks_in_year_local);
        self.release_temp_local(adjacent_year_local);
        self.release_temp_local(day_of_week_local);
        self.release_temp_local(day_of_year_local);
    }
}
