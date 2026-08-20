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
//! Resolving a `gregory` property bag from `{ era, eraYear }` instead of
//! `{ year }` is the *fourth* difference, and it lives here too, in the three
//! emitters [`FunctionBuilder::emit_temporal_calendar_has_eras_i32`],
//! [`FunctionBuilder::emit_temporal_read_era_fields`] and
//! [`FunctionBuilder::emit_temporal_resolve_era_to_year`]. Between them they
//! are the only place any era rule is decided; the five `PrepareCalendarFields`
//! copies keep their own read order and share that one decision point. The
//! three-step type chain [`TemporalEraSlots`] -> [`TemporalEraLocals`] ->
//! [`TemporalResolvedYear`] is what makes "read a bag and forgot the era half"
//! a compile error rather than a wrong `TypeError` at run time.
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
    /// Both halves of the era feature are implemented: the accessors
    /// ([`FunctionBuilder::emit_temporal_gregorian_era_field`]) and the
    /// property-bag direction ([`FunctionBuilder::emit_temporal_resolve_era_to_year`]),
    /// which is what makes `{ era: "bce", eraYear: 1 }` a year source and an
    /// unknown era a RangeError.
    Gregory,
}

impl TemporalCalendarId {
    /// Every calendar, in the order `CanonicalizeCalendar` tests them. Order is
    /// not observable — the spellings are disjoint — but keeping the default
    /// first keeps the common case's compare first.
    ///
    /// **Adding a calendar with a leap month invalidates a shortcut elsewhere.**
    /// `emit_temporal_month_day_string_reference_year`
    /// (`temporal_plain_month_day.rs`) stores the literal
    /// `TEMPORAL_PLAIN_MONTH_DAY_REFERENCE_YEAR` (1972) unconditionally. That is
    /// only the `iso8601` branch's reference year; on the non-ISO branch the
    /// spec takes whatever `CalendarMonthDayFromFields` returns, and
    /// `intl402/Temporal/PlainMonthDay/from/reference-year-1972.js` pins that it
    /// is **not** always 1972 (`{monthCode:"M05L", day:1, calendar:"hebrew"}`
    /// asserts 1970). The shortcut is correct while this array holds only
    /// `Iso8601` and `Gregory`, because every gregory month-day exists in the
    /// leap year 1972. A lunisolar calendar (hebrew, chinese) added here must
    /// derive that year rather than inherit the constant — and no test can see
    /// the difference until such a calendar ships, which is exactly why this is
    /// written down at the array rather than only at the store.
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

    /// Every [`Era`] this calendar recognises, in no observable order.
    ///
    /// Exhaustive with no catch-all, and that is the whole point: era *codes*
    /// are not globally unique, so "is this era valid" is only answerable per
    /// calendar. Test262's `harness/temporalHelpers.js` `CalendarEras` table
    /// shows `japanese` reusing `bce`/`ce` alongside `meiji`..`reiwa`, and
    /// `roc`'s `broc` counting backwards from a different epoch entirely. A
    /// third calendar therefore cannot compile until it states its own set,
    /// and cannot silently inherit `gregory`'s answers.
    ///
    /// An empty set is load-bearing rather than a degenerate case: it is
    /// simultaneously what makes `era`/`eraYear` report `undefined` and what
    /// makes the two property-bag keys go *unread*. Those are the same fact
    /// derived from one predicate, which matters because
    /// `TemporalHelpers.propertyBagObserver` is a Proxy that logs every `get`
    /// — an unconditional `fields.era` read is observable and would break all
    /// 63 `built-ins/Temporal/**/order-of-operations.js` files.
    pub(crate) const fn eras(self) -> &'static [Era] {
        match self {
            Self::Iso8601 => &[],
            Self::Gregory => Era::GREGORY,
        }
    }

    /// What a `Temporal.PlainMonthDay` property bag does with a supplied
    /// `year`. See [`MonthDayYearUse`].
    pub(crate) const fn month_day_year_use(self) -> MonthDayYearUse {
        match self {
            Self::Iso8601 => MonthDayYearUse::OverflowOnly,
            Self::Gregory => MonthDayYearUse::RangeChecked,
        }
    }
}

/// Constant-time `&str` equality, because `str::eq` is not `const fn` and the
/// era tables below are checked at compile time.
const fn const_str_eq(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// [`Era::calendar`] must agree with [`TemporalCalendarId::eras`], so an era
/// filed under the wrong calendar is a build failure rather than a wrong
/// `RangeError` for a bag that named the other calendar.
///
/// Only one direction needs asserting because there is only one table.
/// [`TemporalCalendarId::eras`] is the single era list in the crate: the
/// resolver reads it, the accessors read it, and `data.rs` interns its
/// spellings by walking `TemporalCalendarId::ALL -> eras() -> spellings()`. An
/// earlier revision carried a second flat `Era::ALL` list for the string pool
/// and asserted membership both ways; that pair of assertions could not see
/// the case that actually breaks — a new calendar whose `eras()` is complete
/// but which nobody adds to the flat list — because a list that is never
/// consulted is trivially consistent with itself.
const _: () = {
    let mut calendar_index = 0;
    while calendar_index < TemporalCalendarId::ALL.len() {
        let calendar = TemporalCalendarId::ALL[calendar_index];
        let eras = calendar.eras();
        let mut era_index = 0;
        while era_index < eras.len() {
            assert!(
                eras[era_index].calendar() as u8 == calendar as u8,
                "an era listed in TemporalCalendarId::eras must report that calendar"
            );
            era_index += 1;
        }
        calendar_index += 1;
    }
};

/// No spelling may repeat inside one calendar: the resolver takes the first
/// match, so two eras sharing a spelling would make one of them unreachable
/// and the choice between them positional.
const _: () = {
    let mut calendar_index = 0;
    while calendar_index < TemporalCalendarId::ALL.len() {
        let eras = TemporalCalendarId::ALL[calendar_index].eras();
        let mut left = 0;
        while left < eras.len() {
            let left_spellings = eras[left].spellings();
            let mut left_spelling = 0;
            while left_spelling < left_spellings.len() {
                let mut right = 0;
                while right < eras.len() {
                    let right_spellings = eras[right].spellings();
                    let mut right_spelling = 0;
                    while right_spelling < right_spellings.len() {
                        assert!(
                            (left == right && left_spelling == right_spelling)
                                || !const_str_eq(
                                    left_spellings[left_spelling],
                                    right_spellings[right_spelling]
                                ),
                            "an era spelling is repeated inside one calendar"
                        );
                        right_spelling += 1;
                    }
                    right += 1;
                }
                left_spelling += 1;
            }
            left += 1;
        }
        calendar_index += 1;
    }
};

/// Which half of the `era`/`eraYear` accessor pair an emitter is producing.
///
/// One emitter serves both, so the pair cannot disagree about where the year-0
/// boundary falls; this names which answer the caller wants out of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalEraField {
    Era,
    EraYear,
}

/// Which way an era's year counts relative to the ISO year.
///
/// Both variants are *involutions*, so one function serves both directions of
/// the conversion: `isoYear -> eraYear` for the accessor and
/// `eraYear -> isoYear` for the resolver are literally the same code and cannot
/// drift apart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum EraDirection {
    /// The era year is the ISO year: `ce` 1 is ISO 1.
    Forward,
    /// The era year counts backwards from ISO year 1: proleptic year 0 is
    /// `bce` 1, ISO -1 is `bce` 2.
    Backward,
}

impl EraDirection {
    pub(crate) const ALL: [Self; 2] = [Self::Forward, Self::Backward];

    /// The involution, as a value. `Forward` is the identity; `Backward` is
    /// `y |-> 1 - y`.
    ///
    /// Non-positive era years are *remapped*, never rejected:
    /// `intl402/Temporal/PlainDate/from/era-boundary-gregory.js` pins `ce` 0 to
    /// ISO 0 (reported back as `bce` 1) and `ce` -1 to ISO -1 (`bce` 2).
    pub(crate) const fn convert(self, year: i64) -> i64 {
        match self {
            Self::Forward => year,
            Self::Backward => 1 - year,
        }
    }

    /// [`Self::convert`] as Wasm: leaves `convert(year_local)` on the stack.
    ///
    /// The two coefficients are *read out of* [`Self::convert`] rather than
    /// written again, so the emitted arithmetic cannot disagree with the model
    /// the `const` assertion below checks. `convert` is affine in `year` for
    /// every direction, which that assertion also pins.
    fn emit_convert(self, year_local: u32, function: &mut Function) {
        let constant = self.convert(0);
        let slope = self.convert(1) - self.convert(0);
        function.instruction(&Instruction::I64Const(constant));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::I64Const(slope));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
    }
}

/// The two properties [`EraDirection::emit_convert`] relies on: `convert` is
/// affine (so its two coefficients determine it exactly), and it is an
/// involution (so one function serves both directions).
const _: () = {
    let mut index = 0;
    while index < EraDirection::ALL.len() {
        let direction = EraDirection::ALL[index];
        let constant = direction.convert(0);
        let slope = direction.convert(1) - direction.convert(0);
        let mut year = -4_i64;
        while year <= 4 {
            assert!(
                direction.convert(year) == constant + year * slope,
                "EraDirection::convert must be affine for emit_convert to reproduce it"
            );
            assert!(
                direction.convert(direction.convert(year)) == year,
                "EraDirection::convert must be an involution"
            );
            year += 1;
        }
        index += 1;
    }
};

/// The two eras of the proleptic Gregorian calendar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum GregoryEra {
    Ce,
    Bce,
}

impl GregoryEra {
    /// Both eras, ordered by the sign of the ISO year that selects them:
    /// positive first.
    ///
    /// The ordering is what the type is for. `era` and `eraYear` are two
    /// accessors emitted under one `isoYear > 0` test, and each needs two arms.
    /// Choosing those arms per accessor is what would let `era` answer `ce` on
    /// the branch where `eraYear` counts backwards;
    /// [`FunctionBuilder::emit_temporal_gregorian_era_field`] instead
    /// destructures this one array for both and keys the arithmetic on the
    /// era value rather than on branch position, so the pair cannot disagree
    /// about which side of year 0 it is on. The boundary is: ISO year 1 is
    /// `ce` 1, ISO year 0 is `bce` 1, ISO year -1 is `bce` 2.
    ///
    /// `Intl.DateTimeFormat` encodes the same boundary independently (see the
    /// `display_year` computation in `builtins/intl_datetimeformat.rs`); the
    /// integration note records that duplication.
    pub(crate) const ALL: [Self; 2] = [Self::Ce, Self::Bce];

    const fn direction(self) -> EraDirection {
        match self {
            Self::Ce => EraDirection::Forward,
            Self::Bce => EraDirection::Backward,
        }
    }

    /// Every spelling `CalendarResolveFields` accepts for this era, canonical
    /// first. `ad`/`bc` are the CLDR aliases of `ce`/`bce`, and
    /// `intl402/Temporal/PlainDate/from/canonicalize-era-codes.js` and its two
    /// siblings pin both.
    const fn spellings(self) -> &'static [&'static str] {
        match self {
            Self::Ce => &["ce", "ad"],
            Self::Bce => &["bce", "bc"],
        }
    }
}

/// One era of one calendar.
///
/// The calendar is part of the value because era codes are *not* globally
/// unique — `japanese` also has `ce`/`bce`, and `roc`'s `broc` counts backwards
/// on a different epoch — so an era only means something once you know which
/// calendar asked. Wrapping the flat per-calendar enum is what keeps a future
/// calendar from reusing `gregory`'s answers by accident.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Era {
    Gregory(GregoryEra),
}

impl Era {
    /// The `gregory` era set, as [`TemporalCalendarId::eras`] hands it out.
    pub(crate) const GREGORY: &'static [Self] = &[
        Self::Gregory(GregoryEra::Ce),
        Self::Gregory(GregoryEra::Bce),
    ];

    /// The calendar this era belongs to. Pinned against
    /// [`TemporalCalendarId::eras`] by a `const` assertion above.
    pub(crate) const fn calendar(self) -> TemporalCalendarId {
        match self {
            Self::Gregory(_) => TemporalCalendarId::Gregory,
        }
    }

    /// Every accepted spelling, canonical first.
    pub(crate) const fn spellings(self) -> &'static [&'static str] {
        match self {
            Self::Gregory(era) => era.spellings(),
        }
    }

    /// The identifier the `era` accessor reports: the canonical spelling, by
    /// definition rather than by a second table. Adding an alias to
    /// [`Self::spellings`] therefore cannot change what `era` reports, and
    /// cannot fail to be accepted by the resolver or interned by `data.rs`.
    pub(crate) fn code(self) -> &'static str {
        self.spellings()[0]
    }

    pub(crate) const fn direction(self) -> EraDirection {
        match self {
            Self::Gregory(era) => era.direction(),
        }
    }
}

/// What a `Temporal.PlainMonthDay` property bag does with a supplied `year`.
///
/// A real fork, not a formality.
/// `built-ins/Temporal/PlainMonthDay/{from,prototype/with}/iso-year-used-only-for-overflow.js`
/// pin that `year: -999999` *succeeds* for `iso8601` — the year only decides
/// how 29 February constrains, and is never stored — while
/// `intl402/Temporal/PlainMonthDay/from/dont-calculate-month-info-for-out-of-range-year.js`
/// pins a RangeError for `gregory`. The predicate is "non-ISO", not "has eras":
/// `chinese` and `dangi` have no eras and still range-check, so a calendar
/// added later must re-decide this rather than inherit the exemption.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MonthDayYearUse {
    /// The year picks the overflow behaviour and is then discarded unchecked.
    OverflowOnly,
    /// The year must be inside `ISOYearMonthWithinLimits` before any month
    /// information is computed.
    RangeChecked,
}

/// Four reserved-but-unwritten locals for an `era`/`eraYear` pair.
///
/// The first link of a three-step chain that exists so a bag path cannot skip
/// era resolution and silently answer "fields require year":
///
/// 1. [`FunctionBuilder::reserve_temporal_era_slots`] mints this,
/// 2. [`FunctionBuilder::emit_temporal_read_era_fields`] consumes it *by value*
///    and returns [`TemporalEraLocals`],
/// 3. [`FunctionBuilder::emit_temporal_resolve_era_to_year`] consumes that by
///    value and returns [`TemporalResolvedYear`],
/// 4. every `*_resolve_fields` emitter takes a [`TemporalResolvedYear`] instead
///    of a bare `(year, year-present)` pair.
///
/// Nothing else accepts either type and neither is `Copy`, so a step skipped in
/// the middle is a type error at the *next* step, not a wrong answer at run
/// time.
///
/// What the chain does **not** catch is a bag that is simply *dropped*:
/// `#[must_use]` fires only on an unused expression, so
/// `let _ = self.reserve_temporal_era_slots();` compiles, and the four slots
/// leak from the strict LIFO stack `release_temp_local` asserts on. The symptom
/// is then an `assert_eq!` panic inside an unrelated later emitter.
///
/// A `Drop` impl would move that panic to the leak site, and the obvious
/// objection — a type that implements `Drop` cannot be destructured by move,
/// which is how all three steps consume their input — is soluble: a private
/// zero-sized guard *field* keeps the destructuring legal, at the cost of a
/// `std::mem::forget` at each step. The reason not to do it is the other one.
/// Every caller reserves these slots and then emits through `?`
/// (`emit_object_read`, `emit_temporal_to_temporal_calendar_identifier`,
/// `emit_temporal_property_bag_integer`, …), so a bag alive across an
/// `EmitError` is *routine*, not a bug: `EmitError::unsupported` is a recovered
/// outcome that reports one case as NotImplemented. A panicking `Drop` would
/// convert every one of those into a compiler crash. Making it fire only on the
/// leak needs the failure path to defuse the guard explicitly, which is a real
/// change and wants a lane that can run the temporal suites. Until then the
/// ordering is hand-checked across the ten emitters that reserve slots.
///
/// The split into two types is deliberate: reserving and reading are
/// separate because the locals must be reserved before the caller's scratch
/// locals (`reserve_temp_local` is a strict LIFO stack) while the reads must be
/// emitted in the middle of the alphabetical sweep.
#[must_use]
pub(crate) struct TemporalEraSlots {
    era_payload_local: u32,
    era_present_local: u32,
    era_year_local: u32,
    era_year_present_local: u32,
}

/// An `era`/`eraYear` pair that has been read from a property bag.
///
/// See [`TemporalEraSlots`] for the chain this sits in. Consumed by value by
/// [`FunctionBuilder::emit_temporal_resolve_era_to_year`], which also releases
/// the four locals, so reading without resolving cannot compile and resolving
/// twice cannot either.
#[must_use]
pub(crate) struct TemporalEraLocals {
    era_payload_local: u32,
    era_present_local: u32,
    era_year_local: u32,
    era_year_present_local: u32,
}

impl TemporalEraLocals {
    /// The two `present` flags, for the `with` paths whose "requires at least
    /// one field" TypeError has to count `era`/`eraYear` as fields before the
    /// resolver consumes the bag.
    pub(crate) const fn present_locals(&self) -> [u32; 2] {
        [self.era_present_local, self.era_year_present_local]
    }
}

/// A `(year, year-present)` local pair that has been through
/// [`FunctionBuilder::emit_temporal_resolve_era_to_year`].
///
/// The fields are private and there is deliberately no second constructor, so
/// the resolver is the only thing in the crate that can mint one. Every
/// `*_resolve_fields` emitter takes this instead of two bare `u32`s, which is
/// what makes "read a bag and forgot the era half" fail to typecheck.
#[must_use]
pub(crate) struct TemporalResolvedYear {
    year_local: u32,
    year_present_local: u32,
}

impl TemporalResolvedYear {
    pub(crate) const fn year_local(&self) -> u32 {
        self.year_local
    }

    pub(crate) const fn year_present_local(&self) -> u32 {
        self.year_present_local
    }
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

/// The `DifferenceTemporal*` guard messages, as one closed domain.
///
/// `until`/`since` reject a calendar mismatch in all four families, and a
/// time-zone mismatch in `ZonedDateTime`, with a RangeError whose message is a
/// **pool string**: `StringPool::payload` looks the text up in a map built
/// before emission and *panics* — ``string `..` must exist in pool`` — rather
/// than degrading when it was never interned.
///
/// Batch 6 shipped exactly that. [`FunctionBuilder::emit_temporal_require_same_calendar`]
/// took its message as a bare `&str`, the `Temporal.ZonedDateTime` arithmetic
/// lane spelled two new literals at its call site, `data.rs` grew no matching
/// intern entry, and `cargo test -p lila-aot-wasm --lib` went **24 red** —
/// 24 rather than 2, because every test that emits a full bootstrap takes the
/// panic whatever that test is about. Nothing in the type system could see it:
/// a `&str` parameter and a runtime map lookup have nothing to disagree about
/// at compile time.
///
/// So a guard message is no longer spellable at a call site. A site names a
/// variant, [`Self::message`] is the only source of the text, and `data.rs`
/// interns by walking [`Self::ALL`] and asking each variant which builtins emit
/// it ([`Self::emitting_builtins`]) — the same shape as the
/// `TemporalCalendarId::ALL -> eras() -> spellings()` walk beside it. Both
/// matches are exhaustive with no `_` arm, so a fifth difference family cannot
/// compile without stating its message and its gate, and the pool then picks it
/// up with no edit in `data.rs` at all.
///
/// **What this does not enforce**, stated so it is not over-read: [`Self::ALL`]
/// is a hand-written array. The const assertion below rejects a duplicate, a
/// reordering, and a variant dropped from the middle, but a variant *appended*
/// to the enum and left off the end of `ALL` still compiles. Keep `ALL`
/// adjacent to the variant list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalDifferenceGuard {
    /// `CalendarEquals` in `DifferenceTemporalPlainDate`.
    PlainDateSameCalendar,
    /// `CalendarEquals` in `DifferenceTemporalPlainDateTime`. Reached by
    /// `Temporal.ZonedDateTime.prototype.{until,since}` too, which delegate
    /// their arithmetic to this body — but through a runtime call, so the
    /// message it throws is this one and the ZonedDateTime guards below fire
    /// first, in the caller.
    PlainDateTimeSameCalendar,
    /// `CalendarEquals` in `DifferenceTemporalPlainYearMonth`.
    PlainYearMonthSameCalendar,
    /// `CalendarEquals` in `DifferenceTemporalZonedDateTime`.
    ZonedDateTimeSameCalendar,
    /// `TimeZoneEquals` in `DifferenceTemporalZonedDateTime`. Deliberately
    /// applied unconditionally rather than only for date `largestUnit`s; the
    /// emitter's own comment carries that choice and its cost.
    ZonedDateTimeSameTimeZone,
}

impl TemporalDifferenceGuard {
    /// Every guard, in declaration order. `data.rs` walks this to build the
    /// pool, so a variant reachable from an emitter but missing here is the
    /// `string must exist in pool` compiler panic again.
    pub(crate) const ALL: [Self; 5] = [
        Self::PlainDateSameCalendar,
        Self::PlainDateTimeSameCalendar,
        Self::PlainYearMonthSameCalendar,
        Self::ZonedDateTimeSameCalendar,
        Self::ZonedDateTimeSameTimeZone,
    ];

    /// The RangeError text, and the only place it is spelled.
    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::PlainDateSameCalendar => {
                "Temporal.PlainDate until and since require the same calendar"
            }
            Self::PlainDateTimeSameCalendar => {
                "Temporal.PlainDateTime until and since require the same calendar"
            }
            Self::PlainYearMonthSameCalendar => {
                "Temporal.PlainYearMonth until and since require the same calendar"
            }
            Self::ZonedDateTimeSameCalendar => {
                "Temporal.ZonedDateTime until and since require the same calendar"
            }
            Self::ZonedDateTimeSameTimeZone => {
                "Temporal.ZonedDateTime until and since require the same time zone"
            }
        }
    }

    /// The builtins whose compiled body reads [`Self::message`] back out of the
    /// pool. `data.rs` gates the interning on exactly this set, which is what
    /// keeps a program that touches no `until`/`since` from carrying the text.
    pub(crate) const fn emitting_builtins(self) -> &'static [StandardBuiltinId] {
        match self {
            Self::PlainDateSameCalendar => &[
                StandardBuiltinId::TemporalPlainDatePrototypeUntil,
                StandardBuiltinId::TemporalPlainDatePrototypeSince,
            ],
            Self::PlainDateTimeSameCalendar => &[
                StandardBuiltinId::TemporalPlainDateTimePrototypeUntil,
                StandardBuiltinId::TemporalPlainDateTimePrototypeSince,
            ],
            Self::PlainYearMonthSameCalendar => &[
                StandardBuiltinId::TemporalPlainYearMonthPrototypeUntil,
                StandardBuiltinId::TemporalPlainYearMonthPrototypeSince,
            ],
            Self::ZonedDateTimeSameCalendar | Self::ZonedDateTimeSameTimeZone => &[
                StandardBuiltinId::TemporalZonedDateTimePrototypeUntil,
                StandardBuiltinId::TemporalZonedDateTimePrototypeSince,
            ],
        }
    }

    /// Position in [`Self::ALL`]. Exists only for the const assertion below,
    /// which is what makes `ALL` a checked list rather than a hopeful one.
    const fn index(self) -> usize {
        match self {
            Self::PlainDateSameCalendar => 0,
            Self::PlainDateTimeSameCalendar => 1,
            Self::PlainYearMonthSameCalendar => 2,
            Self::ZonedDateTimeSameCalendar => 3,
            Self::ZonedDateTimeSameTimeZone => 4,
        }
    }
}

const _: () = {
    let mut position = 0;
    while position < TemporalDifferenceGuard::ALL.len() {
        assert!(
            TemporalDifferenceGuard::ALL[position].index() == position,
            "TemporalDifferenceGuard::ALL must list every variant once, in declaration order"
        );
        position += 1;
    }
};

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
    ///
    /// The message arrives as a [`TemporalDifferenceGuard`] rather than as a
    /// `&str` because it is a pool string and an uninterned pool string is a
    /// compile-time panic in every full bootstrap, not a wrong answer in one
    /// case. That enum's doc carries the incident.
    pub(crate) fn emit_temporal_require_same_calendar(
        &mut self,
        calendar_payload_local: u32,
        other_calendar_payload_local: u32,
        guard: TemporalDifferenceGuard,
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
            guard.message(),
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
    /// [`GregoryEra::ALL`] in its declared order. Neither accessor decides for itself
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
        let [positive_year_era, non_positive_year_era] = GregoryEra::ALL;
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
    /// The era-year arithmetic is keyed on the era value, not on which branch
    /// this is, which is what makes swapping the two arms swap both answers
    /// together instead of producing `ce` counting backwards. It goes through
    /// [`EraDirection::emit_convert`] — the same emitter
    /// [`Self::emit_temporal_resolve_era_to_year`] uses for the opposite
    /// direction, which is exactly why the accessor and the resolver cannot
    /// disagree about where the year-0 boundary falls.
    fn emit_temporal_gregorian_era_arm(
        &self,
        iso_year_local: u32,
        era: GregoryEra,
        field: TemporalEraField,
        function: &mut Function,
    ) {
        let era = Era::Gregory(era);
        match field {
            TemporalEraField::Era => {
                function.instruction(&Instruction::I64Const(self.strings.payload(era.code())));
            }
            TemporalEraField::EraYear => {
                era.direction().emit_convert(iso_year_local, function);
            }
        }
    }

    /// Leaves an `i32` on the stack: 1 when the calendar payload names a
    /// calendar with a non-empty [`TemporalCalendarId::eras`].
    ///
    /// A plain payload compare is enough with no case folding: every
    /// `[[Calendar]]` slot and every property-bag `calendar` value has already
    /// been through [`Self::emit_temporal_canonicalize_calendar`] exactly once,
    /// so only the canonical spelling can reach here.
    pub(crate) fn emit_temporal_calendar_has_eras_i32(
        &mut self,
        calendar_payload_local: u32,
        function: &mut Function,
    ) {
        let expected_payload_local = self.reserve_temp_local();
        let matched_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(matched_local));
        for calendar in TemporalCalendarId::ALL {
            if calendar.eras().is_empty() {
                continue;
            }
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
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(matched_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(matched_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.release_temp_local(matched_local);
        self.release_temp_local(expected_payload_local);
    }

    /// Step 1 of the era chain: four locals, nothing emitted.
    ///
    /// Separate from the read because `reserve_temp_local` is a strict LIFO
    /// stack: a `PrepareCalendarFields` sweep reserves its own scratch locals
    /// first and releases them at the end, so an era bag reserved *inside* the
    /// sweep could not outlive it. Callers therefore reserve here, before their
    /// scratch, and read in the middle of the sweep.
    pub(crate) fn reserve_temporal_era_slots(&mut self) -> TemporalEraSlots {
        TemporalEraSlots {
            era_payload_local: self.reserve_temp_local(),
            era_present_local: self.reserve_temp_local(),
            era_year_local: self.reserve_temp_local(),
            era_year_present_local: self.reserve_temp_local(),
        }
    }

    /// Step 2: `PrepareCalendarFields`' `era` and `eraYear` rows, in that
    /// (alphabetical) order, and only for a calendar that has eras.
    ///
    /// The gate is load-bearing rather than an optimisation. The reads
    /// themselves are observable — `TemporalHelpers.propertyBagObserver` is a
    /// Proxy that logs every `get` — so an unconditional `fields.era` breaks
    /// all 63 `built-ins/Temporal/**/order-of-operations.js` files, and
    /// `built-ins/Temporal/PlainDate/prototype/with/time-units-ignored.js`
    /// hands `{ day: 30, era: 'BC' }` to an `iso8601` receiver and requires it
    /// ignored.
    ///
    /// `era` is `ToString`; `eraYear` is `ToIntegerWithTruncation`, which must
    /// accept `0` and negatives (they are remapped, not rejected) and must
    /// RangeError on the infinities after fetching the primitive — the call log
    /// `["get eraYear.valueOf", "call eraYear.valueOf"]` that the 13
    /// `infinity-throws-rangeerror.js` targets assert.
    pub(crate) fn emit_temporal_read_era_fields(
        &mut self,
        slots: TemporalEraSlots,
        argument_payload_local: u32,
        argument_tag_local: u32,
        calendar_payload_local: u32,
        function: &mut Function,
    ) -> Result<TemporalEraLocals, EmitError> {
        let TemporalEraSlots {
            era_payload_local,
            era_present_local,
            era_year_local,
            era_year_present_local,
        } = slots;
        let property_key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();

        for local in [
            era_payload_local,
            era_present_local,
            era_year_local,
            era_year_present_local,
        ] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }

        self.emit_temporal_calendar_has_eras_i32(calendar_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(self.strings.payload("era")));
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
        function.instruction(&Instruction::LocalSet(era_present_local));
        self.emit_temporal_property_bag_string(
            value_payload_local,
            value_tag_local,
            "Temporal era must be a string",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(era_payload_local));

        self.emit_temporal_property_bag_integer(
            argument_payload_local,
            argument_tag_local,
            "eraYear",
            property_key_local,
            value_payload_local,
            value_tag_local,
            present_local,
            era_year_local,
            0,
            "Temporal eraYear must be finite",
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::LocalSet(era_year_present_local));

        function.instruction(&Instruction::End);

        for local in [
            present_local,
            value_tag_local,
            value_payload_local,
            property_key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(TemporalEraLocals {
            era_payload_local,
            era_present_local,
            era_year_local,
            era_year_present_local,
        })
    }

    /// Step 3: the `era` half of `CalendarResolveFields`, and the only place in
    /// the backend where any era rule is decided.
    ///
    /// In order:
    ///
    /// * a calendar with no eras does nothing at all — the two locals are still
    ///   zero because the read above was skipped;
    /// * exactly one of the pair present is a **TypeError**
    ///   (`.../from/one-of-era-erayear-undefined.js` and the `with`
    ///   `mutually-exclusive-fields-gregory.js` files), and it must beat every
    ///   RangeError below, which is what
    ///   `PlainDate/prototype/with/calendarresolvefields-error-ordering-gregory.js`
    ///   asserts;
    /// * an era matching no spelling of any era *of this calendar* is a
    ///   **RangeError**, whether or not `year` is also present —
    ///   `PlainDate/from/calendar-invalid-era.js` supplies `year: 2025` beside
    ///   `era: "xyz"` and still wants one, while the three
    ///   `calendar-invalid-era-with-era-year.js` files omit `year` entirely;
    /// * the ISO year is `direction().convert(eraYear)`, and disagreeing with
    ///   an explicit `year` is a **RangeError**
    ///   (`PlainMonthDay/from/fields-overspecified.js`).
    ///
    /// Callers must place this *after* their overflow-option read:
    /// `built-ins/Temporal/PlainDate/from/options-read-before-algorithmic-validation.js`
    /// pins that every option is read and cast before any algorithmic
    /// validation throws.
    pub(crate) fn emit_temporal_resolve_era_to_year(
        &mut self,
        era: TemporalEraLocals,
        calendar_payload_local: u32,
        year_local: u32,
        year_present_local: u32,
        function: &mut Function,
    ) -> Result<TemporalResolvedYear, EmitError> {
        let TemporalEraLocals {
            era_payload_local,
            era_present_local,
            era_year_local,
            era_year_present_local,
        } = era;
        let expected_payload_local = self.reserve_temp_local();
        let iso_year_local = self.reserve_temp_local();
        let matched_local = self.reserve_temp_local();

        self.emit_temporal_calendar_has_eras_i32(calendar_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(era_present_local));
        function.instruction(&Instruction::LocalGet(era_year_present_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Temporal era and eraYear must be provided together",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(era_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(matched_local));
        for calendar in TemporalCalendarId::ALL {
            let eras = calendar.eras();
            if eras.is_empty() {
                // No compare at all for a calendar with no eras, so `iso8601`
                // costs nothing here.
                continue;
            }
            // The calendar gate is not redundant with `has_eras`: era codes are
            // only unique *within* a calendar, so `bce` may not be allowed to
            // select `gregory`'s arithmetic for some future calendar that
            // spells the same code differently.
            //
            // Emitted once per *calendar*, wrapping all of that calendar's
            // eras, rather than once per era. Same answer either way — the era
            // arms are independent of each other — but the cost of the compare
            // scaled as calendars x eras, replicated across the six emitters
            // that reach this resolver. The one function that has already
            // crossed Cranelift's limit in this backend is an `Intl` body, so
            // the general rule stands: a compare that does not depend on the
            // loop variable does not belong inside the loop.
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
            for &candidate in eras {
                for &spelling in candidate.spellings() {
                    function.instruction(&Instruction::I64Const(self.strings.payload(spelling)));
                    function.instruction(&Instruction::LocalSet(expected_payload_local));
                    self.emit_string_payload_equality_i32(
                        era_payload_local,
                        expected_payload_local,
                        function,
                    );
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(matched_local));
                    candidate.direction().emit_convert(era_year_local, function);
                    function.instruction(&Instruction::LocalSet(iso_year_local));
                    function.instruction(&Instruction::End);
                }
            }
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(matched_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Invalid Temporal era for this calendar",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(year_present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(year_local));
        function.instruction(&Instruction::LocalGet(iso_year_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            "Temporal era and year must agree",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(iso_year_local));
        function.instruction(&Instruction::LocalSet(year_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(year_present_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        for local in [matched_local, iso_year_local, expected_payload_local] {
            self.release_temp_local(local);
        }
        for local in [
            era_year_present_local,
            era_year_local,
            era_present_local,
            era_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(TemporalResolvedYear {
            year_local,
            year_present_local,
        })
    }

    /// `CalendarMergeFields` for the year slot on the three `with` paths: when
    /// the bag resolved no year of its own, the receiver's ISO year stands in,
    /// and the merged bag always has one.
    ///
    /// This runs *after* [`Self::emit_temporal_resolve_era_to_year`] on
    /// purpose. `{ era, eraYear, year }` is one mutually-exclusive group in
    /// `NonISOFieldKeysToIgnore`, so a bag supplying the era pair must exclude
    /// the receiver's year rather than be checked against it — the "era and
    /// eraYear together exclude year" row of the three
    /// `with/mutually-exclusive-fields-gregory.js` files.
    pub(crate) fn emit_temporal_resolved_year_default_to(
        &mut self,
        resolved: &TemporalResolvedYear,
        receiver_year_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(resolved.year_present_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(receiver_year_local));
        function.instruction(&Instruction::LocalSet(resolved.year_local()));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(resolved.year_present_local()));
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
                    "unsupported in lila wasm-aot first slice: \
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

    /// `ISODateWithinLimits`, on its own, as a RangeError carrying `message`.
    ///
    /// Extracted from [`Self::emit_temporal_reject_iso_date`] rather than
    /// written again because `ToTemporalMonthDay` step (k) applies exactly this
    /// bound to a *parsed* year that the month-day record will never store, and
    /// a second copy of an epoch-day limit is a copy that drifts. It is
    /// deliberately not `ISOYearMonthWithinLimits`: that bound is a pair of year
    /// constants, and it answers wrongly on the two boundary days
    /// `-271821-04-19` and `+275760-09-13`, which are inside the date range and
    /// outside no year.
    ///
    /// `days_local` is a caller-owned scratch slot rather than one reserved
    /// here, so that [`Self::emit_temporal_reject_iso_date`] can keep reserving
    /// it in its original position. `reserve_temp_local` hands out
    /// `base + depth`, so a reservation moved across the `RejectISODate` half
    /// would renumber every local that half's nested emitters use — the
    /// RangeError's own prototype slot among them — and the extraction would
    /// stop being byte-identical for `Temporal.PlainDate`.
    pub(crate) fn emit_temporal_iso_date_within_limits(
        &mut self,
        year_local: u32,
        month_local: u32,
        day_local: u32,
        days_local: u32,
        message: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
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
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `RejectISODate` followed by the `ISODateWithinLimits` check that
    /// `CreateTemporalDate` performs. Both failures are RangeErrors, so the
    /// two are fused into one guard.
    ///
    /// The second half is [`Self::emit_temporal_iso_date_within_limits`], which
    /// `ToTemporalMonthDay` step (k) reaches without the `RejectISODate` half.
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

        self.emit_temporal_iso_date_within_limits(
            year_local,
            month_local,
            day_local,
            days_local,
            "Temporal.PlainDate is outside the supported date range",
            function,
        )?;

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
