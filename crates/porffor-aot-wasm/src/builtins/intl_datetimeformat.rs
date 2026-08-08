//! `Intl.DateTimeFormat` — ECMA-402 11.
//!
//! Scope, stated honestly. This implements `CreateDateTimeFormat` (11.1.2) in
//! full — every option is read, in the observable order, with the spec's
//! validation — and `resolvedOptions` (11.4.4), `supportedLocalesOf` (11.2.2),
//! `format` (11.4.3) and `formatToParts` (11.4.5) over a **single locale**,
//! `en-US`, a single calendar, `gregory`, a single numbering system, `latn`,
//! and the **fixed-offset** time zones.
//!
//! Locale negotiation therefore always resolves to `"en-US"`: `ResolveLocale`
//! with `AvailableLocales = « "en-US" »` falls back to the default locale for
//! every request. That is a real answer for `en`/`en-US` and an honest
//! fallback elsewhere, which is what an implementation with no CLDR data can
//! say. Calendars other than `gregory`/`gregorian` and `iso8601` — which shares
//! the proleptic Gregorian arithmetic and differs only in having no eras — are
//! **rejected** (`RangeError`) rather than accepted and mis-formatted.
//!
//! # Time zones: every offset, no zone database
//!
//! `AvailableNamedTimeZoneIdentifiers()` here is [`INTL_DTF_NAMED_ZONES`] — the
//! UTC aliases and the whole `Etc/GMT±N` family — together with the `UTCOffset`
//! identifiers `IsTimeZoneOffsetString` accepts. Those are exactly the zones
//! whose offset is a constant, so they need no transition data: `Etc/GMT+7` and
//! `-07:00` shift the epoch by the same fixed −420 minutes forever. A
//! geographic identifier such as `America/Vancouver` is still a `RangeError`,
//! because answering it needs the IANA transition table this backend does not
//! carry, and picking one offset for it would be a wrong answer rather than a
//! missing one.
//!
//! An accepted zone is a [`DtfCanonicalTimeZone`]: the identifier
//! `resolvedOptions().timeZone` reports **and** the offset the formatter
//! applies, produced together and stored together. The identifier is the
//! table's, not the caller's spelling — that is
//! `GetAvailableNamedTimeZoneIdentifier` returning `record.[[Identifier]]`, and
//! it is the one rule that makes `"utc"` report `"UTC"` while `"Etc/GMT"`
//! reports `"Etc/GMT"`.
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
//! [`FunctionBuilder::emit_intl_dtf_build_format_with_kind`] with a
//! [`DtfFormatMode`] discriminator, so the field order, the literals and the
//! numeral rendering come from one body of Rust and cannot drift apart.
//!
//! `formatRange` (11.4.6) and `formatRangeToParts` (11.4.7) are the same body
//! again, given a [`DtfFormatTimes`] with two time values. The walk is not
//! duplicated for the second side: it runs inside a wasm `loop` that iterates
//! once or twice, so the emitted function grows by a component copy instead of
//! by a second copy of fifty-odd string-literal selects.
//!
//! Both ends of a range go through the same `HandleDateTimeValue` (11.5.11) the
//! single-date path uses, and [`DtfFormatTimes`] carries **one**
//! [`DtfValueKind`] local for both of them — `SameTemporalType` has already run
//! by the time it is built, so "the two ends were masked under different
//! Temporal field sets" is not a state this code can be in.
//!
//! What the range path does *not* have is CLDR interval-pattern eliding:
//! `Jan 3 – 5, 2019` comes out as two complete sides joined by the fallback
//! separator, which is what `intervalFormatFallback` prescribes when no
//! interval skeleton matches.

use super::super::*;
use super::temporal_plain_date::TemporalCalendarId;

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
        // Shared with [`TimeZoneNameStyle::code`]: the constructor's accepted
        // spellings and the renderer's understood codes are one list.
        codes: INTL_DTF_TIME_ZONE_NAME_CODES,
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

/// Every component slot the field walk can render, in Table 7 order with
/// `fractionalSecondDigits` at its numeric position.
///
/// [`FunctionBuilder::emit_intl_dtf_build_format_with_kind`] keeps exactly one effective
/// local per entry and the Temporal field mask clears by slot, so a component
/// can never be masked under one name and rendered under another.
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

/// The nine properties of `CreateDateTimeFormat` steps 40-41 that clear
/// `needDefaults`.
///
/// `era` and `timeZoneName` are deliberately absent. They are format
/// components for the step-42 `TypeError` — asking for one alongside a
/// `dateStyle` still throws — but the spec's two lists, « weekday, year,
/// month, day » and « dayPeriod, hour, minute, second,
/// fractionalSecondDigits », do not contain them, so
/// `new Intl.DateTimeFormat("en", { timeZoneName: "short" })` still resolves
/// to a year/month/day format that happens to name its zone. The constructor
/// therefore tracks two bits, not one; collapsing them back into a single flag
/// breaks either every `options-conflict.js` or every `era`-only format.
const INTL_DTF_NEED_DEFAULTS_COMPONENTS: [&str; 9] = [
    "weekday",
    "year",
    "month",
    "day",
    "dayPeriod",
    "hour",
    "minute",
    "second",
    "fractionalSecondDigits",
];

/// The `-u-ca` types this implementation answers to, and what each resolves
/// to. `gregorian` is the Unicode alias of `gregory`; `iso8601` shares the
/// proleptic Gregorian arithmetic and differs only in that it has no eras,
/// which this backend never has data to print anyway.
const INTL_DTF_ACCEPTED_CALENDARS: &[(&str, &str)] = &[
    ("gregory", INTL_DTF_RESOLVED_CALENDAR),
    ("gregorian", INTL_DTF_RESOLVED_CALENDAR),
    ("iso8601", "iso8601"),
];

/// `Intl` and `Temporal` must not be able to disagree about a calendar
/// identifier. `INTL_DTF_ACCEPTED_CALENDARS` above and
/// `TemporalCalendarId::spellings`/`canonical` in
/// `builtins/temporal_plain_date.rs` are two independent statements of the same
/// table; this pins them together at compile time, so
/// `new Temporal.PlainDate(2000, 5, 2, "gregorian").calendarId` and
/// `new Intl.DateTimeFormat("en", { calendar: "gregorian" })
///     .resolvedOptions().calendar` cannot answer differently.
///
/// The assertion is deliberately one-directional: `Intl` may accept a spelling
/// `Temporal` does not (a locale extension is not a `[[Calendar]]` slot), but
/// every spelling `Temporal` accepts must resolve to the same canonical form on
/// both sides.
const _: () = {
    let mut calendar_index = 0;
    while calendar_index < TemporalCalendarId::ALL.len() {
        let calendar = TemporalCalendarId::ALL[calendar_index];
        let canonical = calendar.canonical();
        let spellings = calendar.spellings();
        let mut spelling_index = 0;
        while spelling_index < spellings.len() {
            let spelling = spellings[spelling_index];
            let mut row_index = 0;
            let mut found = false;
            while row_index < INTL_DTF_ACCEPTED_CALENDARS.len() {
                let (accepted, resolved) = INTL_DTF_ACCEPTED_CALENDARS[row_index];
                if const_str_eq(accepted, spelling) {
                    assert!(
                        const_str_eq(resolved, canonical),
                        "Intl and Temporal disagree about a calendar's canonical form"
                    );
                    found = true;
                }
                row_index += 1;
            }
            assert!(
                found,
                "Intl.DateTimeFormat does not accept a calendar Temporal accepts"
            );
            spelling_index += 1;
        }
        calendar_index += 1;
    }
};

/// `str` has no `const` equality, so compare the bytes.
const fn const_str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
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

/// The `-u-nu` types this implementation answers to.
const INTL_DTF_ACCEPTED_NUMBERING_SYSTEMS: &[(&str, &str)] = &[(
    INTL_DTF_RESOLVED_NUMBERING_SYSTEM,
    INTL_DTF_RESOLVED_NUMBERING_SYSTEM,
)];

/// An offset from UTC in whole signed minutes.
///
/// The only constructor range-checks, so "the parser forgot to bound the hour"
/// is a construction-site error rather than a wrong-but-plausible formatted
/// time. The bound is the ES `UTCOffset` grammar's, not the IANA world's:
/// `Hour ::: 0 DecimalDigit | 1 DecimalDigit | 20 | 21 | 22 | 23` and
/// `MinuteSecond ::: [0-5] DecimalDigit`, so `±23:59` is representable and
/// `intl402/DateTimeFormat/prototype/resolvedOptions/offset-timezone-basic.js`
/// really does require `-22:23` to be accepted. Anything narrower — `-14:00`,
/// the widest offset any real zone has ever used — would reject conforming
/// input.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct TzOffsetMinutes(i16);

impl TzOffsetMinutes {
    /// The largest `Hour` the `UTCOffset` grammar can spell:
    /// `Hour ::: 0 DecimalDigit | 1 DecimalDigit | 20 | 21 | 22 | 23`.
    ///
    /// A *grammar* fact, and the emitted parser's bound reads it directly.
    /// Deriving it from [`Self::MAX`] instead (`MAX / 60`) would only reproduce
    /// the grammar by coincidence: rounding `MAX` up to `24 * 60`, the obvious
    /// edit for anyone who believed `±24:00` should be spellable, would leave
    /// `MAX % 60 == 0` and the emitted parser would silently start rejecting
    /// `+03:30` and every other offset with non-zero minutes.
    const MAX_HOUR: i64 = 23;
    /// `MinuteSecond ::: [0-5] DecimalDigit`. Also a grammar fact.
    const MAX_MINUTE: i64 = 59;

    /// The largest magnitude the grammar can spell, derived from the two
    /// grammar constants above so the range and the parser cannot disagree.
    const MAX: i16 = (Self::MAX_HOUR * 60 + Self::MAX_MINUTE) as i16;
    const MIN: i16 = -Self::MAX;

    /// The only way to make one. `None` is "outside the `UTCOffset` grammar",
    /// which every caller must turn into a `RangeError` or a compile failure.
    const fn new(minutes: i16) -> Option<Self> {
        match minutes {
            Self::MIN..=Self::MAX => Some(Self(minutes)),
            _ => None,
        }
    }

    /// A whole number of hours, for the `Etc/GMT±N` rows and for UTC itself.
    /// Panics at compile time on an out-of-range row rather than shipping one.
    const fn from_hours(hours: i16) -> Self {
        match Self::new(hours * 60) {
            Some(offset) => offset,
            None => panic!("an Etc/GMT row is outside the UTCOffset range"),
        }
    }

    /// Built through the same constructor as every other row, so there is no
    /// path to a `TzOffsetMinutes` that skipped the range check.
    const UTC: Self = Self::from_hours(0);

    const fn minutes(self) -> i16 {
        self.0
    }

    /// The largest `Hour` and `MinuteSecond` the emitted parser may accept.
    ///
    /// The same two grammar constants [`Self::MAX`] is built from, so the wasm
    /// the parser emits and the Rust the table is built through are bounded by
    /// one pair of numbers. Widening the newtype without widening the parser
    /// (or the reverse) is not expressible.
    const fn max_hour() -> i64 {
        Self::MAX_HOUR
    }

    const fn max_minute() -> i64 {
        Self::MAX_MINUTE
    }
}

/// One row of `AvailableNamedTimeZoneIdentifiers()`.
///
/// `identifier` is what `resolvedOptions().timeZone` reports for **every**
/// ASCII-case-insensitive spelling that matches it, which is what
/// `GetAvailableNamedTimeZoneIdentifier` means by returning
/// `record.[[Identifier]]`.
#[derive(Clone, Copy)]
pub(crate) struct IntlDtfNamedZone {
    pub(crate) identifier: &'static str,
    pub(crate) offset: TzOffsetMinutes,
}

impl IntlDtfNamedZone {
    const fn utc_alias(identifier: &'static str) -> Self {
        Self {
            identifier,
            offset: TzOffsetMinutes::UTC,
        }
    }

    /// An `Etc/GMT±N` row. POSIX inverts the sign: `Etc/GMT+7` is seven hours
    /// *west* of Greenwich, i.e. offset −07:00, which is exactly what
    /// `prototype/format/offset-timezone-gmt-same.js` pins by asserting
    /// `'-07:00'` and `'Etc/GMT+7'` format identically.
    const fn etc_gmt(identifier: &'static str, posix_hours: i16) -> Self {
        Self {
            identifier,
            offset: TzOffsetMinutes::from_hours(-posix_hours),
        }
    }
}

/// `AvailableNamedTimeZoneIdentifiers()` for a backend with no transition data:
/// every IANA Zone or Link name whose offset is a constant.
///
/// The eighteen zero-offset rows are the UTC aliases of `etcetera`/`backward`;
/// the rest are the `Etc/GMT+1`..`Etc/GMT+12` and `Etc/GMT-1`..`Etc/GMT-14`
/// families. Nothing here is canonicalised away: `timezone-utc.js` wants
/// `"utc"` to report `"UTC"` and `canonicalize-utc-timezone.js` wants
/// `"Etc/GMT"` to report `"Etc/GMT"`, and returning the row's own identifier
/// satisfies both without a second table.
pub(crate) const INTL_DTF_NAMED_ZONES: &[IntlDtfNamedZone] = &[
    IntlDtfNamedZone::utc_alias(INTL_DTF_RESOLVED_TIME_ZONE),
    IntlDtfNamedZone::utc_alias("GMT"),
    IntlDtfNamedZone::utc_alias("Etc/UTC"),
    IntlDtfNamedZone::utc_alias("Etc/GMT"),
    IntlDtfNamedZone::utc_alias("Etc/Universal"),
    IntlDtfNamedZone::utc_alias("Etc/Zulu"),
    IntlDtfNamedZone::utc_alias("Etc/Greenwich"),
    IntlDtfNamedZone::utc_alias("Etc/UCT"),
    IntlDtfNamedZone::utc_alias("UCT"),
    IntlDtfNamedZone::utc_alias("Universal"),
    IntlDtfNamedZone::utc_alias("Zulu"),
    IntlDtfNamedZone::utc_alias("Greenwich"),
    IntlDtfNamedZone::utc_alias("GMT+0"),
    IntlDtfNamedZone::utc_alias("GMT-0"),
    IntlDtfNamedZone::utc_alias("GMT0"),
    IntlDtfNamedZone::utc_alias("Etc/GMT+0"),
    IntlDtfNamedZone::utc_alias("Etc/GMT-0"),
    IntlDtfNamedZone::utc_alias("Etc/GMT0"),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+1", 1),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+2", 2),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+3", 3),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+4", 4),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+5", 5),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+6", 6),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+7", 7),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+8", 8),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+9", 9),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+10", 10),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+11", 11),
    IntlDtfNamedZone::etc_gmt("Etc/GMT+12", 12),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-1", -1),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-2", -2),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-3", -3),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-4", -4),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-5", -5),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-6", -6),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-7", -7),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-8", -8),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-9", -9),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-10", -10),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-11", -11),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-12", -12),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-13", -13),
    IntlDtfNamedZone::etc_gmt("Etc/GMT-14", -14),
];

const fn intl_dtf_ascii_lower_byte(byte: u8) -> u8 {
    if byte >= b'A' && byte <= b'Z' {
        byte + 32
    } else {
        byte
    }
}

const fn intl_dtf_str_eq(left: &str, right: &str) -> bool {
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

const fn intl_dtf_ascii_eq_ignore_case(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if intl_dtf_ascii_lower_byte(left[index]) != intl_dtf_ascii_lower_byte(right[index]) {
            return false;
        }
        index += 1;
    }
    true
}

/// The lookup is ASCII-case-insensitive, so two rows differing only in case
/// would make the winner depend on table order. Rejected at compile time.
const _: () = {
    let mut i = 0;
    while i < INTL_DTF_NAMED_ZONES.len() {
        let mut j = i + 1;
        while j < INTL_DTF_NAMED_ZONES.len() {
            assert!(
                !intl_dtf_ascii_eq_ignore_case(
                    INTL_DTF_NAMED_ZONES[i].identifier,
                    INTL_DTF_NAMED_ZONES[j].identifier,
                ),
                "two INTL_DTF_NAMED_ZONES rows share an ASCII-case-insensitive identifier",
            );
            j += 1;
        }
        i += 1;
    }
};

/// The six `timeZoneName` widths, as a closed set.
///
/// [`TimeZoneNameStyle::utc_name`] has no fallback arm, so a style cannot be
/// added without deciding what it prints for the named UTC family — which is
/// the half CLDR `en` has real names for and the half Test262 reads back. It
/// covers exactly the zero-offset rows of [`INTL_DTF_NAMED_ZONES`], not every
/// zone whose offset happens to be zero: `'+00:00'` is an offset identifier and
/// takes the GMT form below.
///
/// For every other zone all six styles deliberately share one answer, the
/// localized GMT format `GMT±HH:MM` (or the bare `GMT` that CLDR `en`'s
/// `gmtZeroFormat` gives a zero-offset offset zone), pre-rendered into
/// [`HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET`]. CLDR `en` would elide the
/// leading zero and a whole `:00` for the three narrow styles — `GMT+5` rather
/// than `GMT+05:00` — and that elision is not implemented, on purpose: no
/// Test262 case in the corpus observes a `timeZoneName` under a non-zero
/// offset, and rendering two widths inside the format walk costs six string
/// concatenations in the one function whose size budget is known to be tight.
/// The answer it does give is a correct GMT offset, not a plausible-looking
/// wrong name.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeZoneNameStyle {
    Short,
    Long,
    ShortOffset,
    LongOffset,
    ShortGeneric,
    LongGeneric,
}

impl TimeZoneNameStyle {
    /// Table order, which is also [`INTL_DTF_COMPONENT_OPTIONS`]'s
    /// `timeZoneName` order.
    const ALL: [Self; 6] = [
        Self::Short,
        Self::Long,
        Self::ShortOffset,
        Self::LongOffset,
        Self::ShortGeneric,
        Self::LongGeneric,
    ];

    /// The spelling `resolvedOptions` reports, which is also the key the option
    /// table matches on.
    const fn spelling(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Long => "long",
            Self::ShortOffset => "shortOffset",
            Self::LongOffset => "longOffset",
            Self::ShortGeneric => "shortGeneric",
            Self::LongGeneric => "longGeneric",
        }
    }

    /// What CLDR `en` prints for the UTC zone family, byte for byte.
    ///
    /// These are literals rather than `GMT+00:00` renderings because `en` has
    /// real names for offset zero and `constructor-options-timeZoneName-valid.js`
    /// plus `format/temporal-plaindate-formatting-timezonename.js` observe them.
    const fn utc_name(self) -> &'static str {
        match self {
            Self::Short => "UTC",
            Self::Long => "Coordinated Universal Time",
            Self::ShortOffset => "GMT",
            Self::LongOffset => "GMT+00:00",
            Self::ShortGeneric => "UTC",
            Self::LongGeneric => "UTC",
        }
    }

    /// The runtime code stored in [`HEAP_INTL_DTF_TIME_ZONE_NAME_OFFSET`],
    /// looked up in the one table the constructor and `resolvedOptions` share
    /// so the enum cannot drift from the codes on the heap.
    const fn code(self) -> i64 {
        let codes = INTL_DTF_TIME_ZONE_NAME_CODES;
        let mut index = 0;
        while index < codes.len() {
            if intl_dtf_str_eq(codes[index].0, self.spelling()) {
                return codes[index].1;
            }
            index += 1;
        }
        panic!("a TimeZoneNameStyle has no code in INTL_DTF_TIME_ZONE_NAME_CODES")
    }
}

/// The `(spelling, code)` list shared by [`INTL_DTF_COMPONENT_OPTIONS`]'s
/// `timeZoneName` row and [`TimeZoneNameStyle::code`]. One list, so the
/// renderer cannot answer to a code the constructor never writes.
const INTL_DTF_TIME_ZONE_NAME_CODES: &[(&str, i64)] = &[
    ("short", 1),
    ("long", 2),
    ("shortOffset", 3),
    ("longOffset", 4),
    ("shortGeneric", 5),
    ("longGeneric", 6),
];

/// Every code in the table belongs to a style and every style has a code:
/// neither list can grow without the other.
const _: () = {
    assert!(
        INTL_DTF_TIME_ZONE_NAME_CODES.len() == TimeZoneNameStyle::ALL.len(),
        "INTL_DTF_TIME_ZONE_NAME_CODES and TimeZoneNameStyle::ALL disagree",
    );
    let mut index = 0;
    while index < TimeZoneNameStyle::ALL.len() {
        // `code()` panics at compile time when a style is missing from the
        // list, so calling it for every style is the other half of the check.
        assert!(TimeZoneNameStyle::ALL[index].code() > 0);
        index += 1;
    }
};

/// `FormatOffsetTimeZoneIdentifier` and the offset renderer both need the sign
/// as a literal; naming them keeps the pool derivation honest.
const INTL_DTF_OFFSET_SIGNS: [&str; 2] = ["+", "-"];
const INTL_DTF_GMT_PREFIX: &str = "GMT";

/// A Temporal type `Intl.DateTimeFormat` knows how to render, and the two
/// field sets ECMA-402 11.5.11 `HandleDateTimeValue` gives it.
///
/// `allowed` and `defaults` are both written as record slot offsets — the same
/// offsets [`INTL_DTF_COMPONENT_OPTIONS`] stores through — so the mask and the
/// default fill cannot disagree about which component they mean, and a slot
/// that is defaulted is by construction a slot that survives the mask.
pub(crate) struct IntlDtfTemporalKind {
    /// The runtime discriminator written into `kind_local`. Zero is reserved:
    /// it always means the legacy Number/Date path, where `TimeClip` applies.
    pub(crate) code: i64,
    /// The `OBJECT_INTERNAL_BRAND_*` the format function dispatches on.
    pub(crate) brand: u64,
    /// The constructor name, for the `toLocaleString` error messages.
    pub(crate) type_name: &'static str,
    /// Every component slot this type may render. The rest are forced to zero
    /// after the `dateStyle`/`timeStyle` expansion, which is what lets
    /// `dateStyle: "full"` on a `PlainDate` produce a byte-identical string to
    /// the legacy path while `timeStyle: "long"` on a `PlainDateTime` silently
    /// drops the zone name.
    pub(crate) allowed: &'static [u64],
    /// `(slot, code)` pairs installed when the format asked for no components
    /// at all — the type's own `needDefaults` answer, replacing the
    /// constructor's date-shaped guess.
    pub(crate) defaults: &'static [(u64, i64)],
    /// `(property, slot)` of the style option this type has no fields for.
    /// `toLocaleString` rejects it with a `TypeError` before formatting; the
    /// `format` function never sees it because a style and a Temporal receiver
    /// only ever meet through `toLocaleString`.
    pub(crate) rejected_style: Option<(&'static str, u64)>,
    /// `[[IsPlain]]` from 11.5.11, i.e. whether the epoch value this type
    /// reduces to is an instant that the resolved zone shifts, or wall-clock
    /// fields that it must leave alone. A mandatory field, so a new Temporal
    /// brand cannot be added without answering the question.
    pub(crate) basis: DtfTimeBasis,
}

/// Whether a time value is an exact point on the timeline or already local.
///
/// This is `[[IsPlain]]` from ECMA-402 11.5.11 promoted to a type. It decides
/// the single place the resolved time zone's offset is added, and getting it
/// wrong is invisible under `UTC`: at `+13:00` a `Temporal.PlainDate`, anchored
/// at noon, would slide a whole day.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DtfTimeBasis {
    /// A legacy `Number`/`Date` time value or a `Temporal.Instant`: an exact
    /// instant, so the components are read in the resolved zone's local time.
    Exact,
    /// A `Temporal.Plain*` value: the epoch milliseconds it was reduced to
    /// already *are* its wall-clock fields, and adding an offset would move
    /// them.
    Plain,
}

/// The `allowed` set shared by the instant-like types: an exact point on the
/// timeline has every field a legacy `Date` has.
const INTL_DTF_ALL_COMPONENT_SLOTS: &[u64] = &INTL_DTF_FORMAT_COMPONENT_SLOTS;

/// The date half of the default fill, `{ year, month, day }` all `numeric`.
const INTL_DTF_DEFAULT_DATE: [(u64, i64); 3] = [
    (HEAP_INTL_DTF_YEAR_OFFSET, 2),
    (HEAP_INTL_DTF_MONTH_OFFSET, 2),
    (HEAP_INTL_DTF_DAY_OFFSET, 2),
];

/// The time half, `{ hour, minute, second }` all `numeric`.
const INTL_DTF_DEFAULT_TIME: [(u64, i64); 3] = [
    (HEAP_INTL_DTF_HOUR_OFFSET, 2),
    (HEAP_INTL_DTF_MINUTE_OFFSET, 2),
    (HEAP_INTL_DTF_SECOND_OFFSET, 2),
];

/// `Temporal.ZonedDateTime` is the one branded value the formatting entry
/// points must refuse: it carries its own time zone, which cannot be reconciled
/// with the formatter's. Raised from `format`/`formatToParts` and from
/// `formatRange`/`formatRangeToParts` alike, so the message names none of them.
const INTL_DTF_ZONED_DATE_TIME_UNSUPPORTED: &str =
    "Intl.DateTimeFormat does not support Temporal.ZonedDateTime";

/// `AdjustDateTimeStyleFormat` left nothing to print.
const INTL_DTF_EMPTY_TEMPORAL_FORMAT: &str =
    "The requested format has no fields in common with this Temporal type";

/// Neither a `UTCOffset` identifier nor a row of [`INTL_DTF_NAMED_ZONES`].
const INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE: &str = "Unsupported timeZone option";

/// `PartitionDateTimeRangePattern` step 5: `SameTemporalType(x, y)` is false.
///
/// A legacy `Number` counts as its own "type" for this purpose, so mixing a
/// `Date` with any Temporal object lands here too — which is exactly what
/// `formatRange/fails-on-distinct-temporal-types.js` asserts.
const INTL_DTF_RANGE_DIFFERENT_TYPES_MESSAGE: &str =
    "Intl.DateTimeFormat.prototype.formatRange startDate and endDate must be the same type";

/// Table 6 of ECMA-402 11.5.11, one row per branded type. The `format`
/// function walks it to dispatch on the receiver's brand and
/// `toLocaleString` indexes it by type, so the two paths cannot resolve a
/// value to different fields.
pub(crate) const INTL_DTF_TEMPORAL_KINDS: &[IntlDtfTemporalKind] = &[
    IntlDtfTemporalKind {
        code: 1,
        brand: OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE,
        type_name: "Temporal.PlainDate",
        allowed: &[
            HEAP_INTL_DTF_WEEKDAY_OFFSET,
            HEAP_INTL_DTF_ERA_OFFSET,
            HEAP_INTL_DTF_YEAR_OFFSET,
            HEAP_INTL_DTF_MONTH_OFFSET,
            HEAP_INTL_DTF_DAY_OFFSET,
        ],
        defaults: &INTL_DTF_DEFAULT_DATE,
        rejected_style: Some(("timeStyle", HEAP_INTL_DTF_TIME_STYLE_OFFSET)),
        basis: DtfTimeBasis::Plain,
    },
    IntlDtfTemporalKind {
        code: 2,
        brand: OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH,
        type_name: "Temporal.PlainYearMonth",
        allowed: &[
            HEAP_INTL_DTF_ERA_OFFSET,
            HEAP_INTL_DTF_YEAR_OFFSET,
            HEAP_INTL_DTF_MONTH_OFFSET,
        ],
        defaults: &[
            (HEAP_INTL_DTF_YEAR_OFFSET, 2),
            (HEAP_INTL_DTF_MONTH_OFFSET, 2),
        ],
        rejected_style: Some(("timeStyle", HEAP_INTL_DTF_TIME_STYLE_OFFSET)),
        basis: DtfTimeBasis::Plain,
    },
    IntlDtfTemporalKind {
        code: 3,
        brand: OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY,
        type_name: "Temporal.PlainMonthDay",
        allowed: &[HEAP_INTL_DTF_MONTH_OFFSET, HEAP_INTL_DTF_DAY_OFFSET],
        defaults: &[
            (HEAP_INTL_DTF_MONTH_OFFSET, 2),
            (HEAP_INTL_DTF_DAY_OFFSET, 2),
        ],
        rejected_style: Some(("timeStyle", HEAP_INTL_DTF_TIME_STYLE_OFFSET)),
        basis: DtfTimeBasis::Plain,
    },
    IntlDtfTemporalKind {
        code: 4,
        brand: OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_TIME,
        type_name: "Temporal.PlainTime",
        allowed: &[
            HEAP_INTL_DTF_DAY_PERIOD_OFFSET,
            HEAP_INTL_DTF_HOUR_OFFSET,
            HEAP_INTL_DTF_MINUTE_OFFSET,
            HEAP_INTL_DTF_SECOND_OFFSET,
            HEAP_INTL_DTF_FRACTIONAL_SECOND_DIGITS_OFFSET,
        ],
        defaults: &INTL_DTF_DEFAULT_TIME,
        rejected_style: Some(("dateStyle", HEAP_INTL_DTF_DATE_STYLE_OFFSET)),
        basis: DtfTimeBasis::Plain,
    },
    IntlDtfTemporalKind {
        code: 5,
        brand: OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME,
        type_name: "Temporal.PlainDateTime",
        allowed: &[
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
        ],
        defaults: &[
            (HEAP_INTL_DTF_YEAR_OFFSET, 2),
            (HEAP_INTL_DTF_MONTH_OFFSET, 2),
            (HEAP_INTL_DTF_DAY_OFFSET, 2),
            (HEAP_INTL_DTF_HOUR_OFFSET, 2),
            (HEAP_INTL_DTF_MINUTE_OFFSET, 2),
            (HEAP_INTL_DTF_SECOND_OFFSET, 2),
        ],
        rejected_style: None,
        basis: DtfTimeBasis::Plain,
    },
    IntlDtfTemporalKind {
        code: 6,
        brand: OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT,
        type_name: "Temporal.Instant",
        allowed: INTL_DTF_ALL_COMPONENT_SLOTS,
        defaults: &[
            (HEAP_INTL_DTF_YEAR_OFFSET, 2),
            (HEAP_INTL_DTF_MONTH_OFFSET, 2),
            (HEAP_INTL_DTF_DAY_OFFSET, 2),
            (HEAP_INTL_DTF_HOUR_OFFSET, 2),
            (HEAP_INTL_DTF_MINUTE_OFFSET, 2),
            (HEAP_INTL_DTF_SECOND_OFFSET, 2),
        ],
        rejected_style: None,
        // The one Temporal type that is a point on the timeline rather than a
        // wall clock, so the resolved zone genuinely applies to it.
        basis: DtfTimeBasis::Exact,
    },
];

/// The `kind_local` code for `Temporal.ZonedDateTime`.
///
/// It is not an [`INTL_DTF_TEMPORAL_KINDS`] row because there is no field set
/// to give it: `HandleDateTimeValue` refuses the type outright. It still needs
/// a code, because `SameTemporalType` has to be able to say that two
/// `ZonedDateTime`s *are* the same type before the refusal fires.
const INTL_DTF_ZONED_DATE_TIME_KIND_CODE: i64 = 7;

/// Code 0 is the legacy path and [`INTL_DTF_ZONED_DATE_TIME_KIND_CODE`] is
/// nobody's row; a table edit that collides with either would silently make two
/// types compare equal under `SameTemporalType`.
const _: () = {
    let mut index = 0;
    while index < INTL_DTF_TEMPORAL_KINDS.len() {
        assert!(
            INTL_DTF_TEMPORAL_KINDS[index].code != 0,
            "code 0 is reserved for the legacy Number/Date path",
        );
        assert!(
            INTL_DTF_TEMPORAL_KINDS[index].code != INTL_DTF_ZONED_DATE_TIME_KIND_CODE,
            "a Temporal row collides with the Temporal.ZonedDateTime kind code",
        );
        let mut other = index + 1;
        while other < INTL_DTF_TEMPORAL_KINDS.len() {
            assert!(
                INTL_DTF_TEMPORAL_KINDS[index].code != INTL_DTF_TEMPORAL_KINDS[other].code,
                "two INTL_DTF_TEMPORAL_KINDS rows share a code",
            );
            other += 1;
        }
        index += 1;
    }
};

/// `NoonTimeRecord()`: the time of day a date-only Temporal value is anchored
/// at. Noon rather than midnight is what keeps the two ends of the Temporal
/// range, `-271821-04-19` and `+275760-09-13`, inside the representable
/// millisecond span instead of falling off it by half a day.
const INTL_DTF_TEMPORAL_NOON_MILLISECONDS: f64 = 43_200_000.0;
const INTL_DTF_MILLISECONDS_PER_DAY: f64 = 86_400_000.0;
/// The unit [`HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET`] is stored in,
/// scaled to the millisecond time values everything else here uses.
const INTL_DTF_MILLISECONDS_PER_MINUTE: f64 = 60_000.0;

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

/// Which artefact [`FunctionBuilder::emit_intl_dtf_build_format_with_kind`] produces.
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

/// Whether the parts this walk produces carry a `source` property, and where
/// the current side's `source` string lives.
///
/// [`DtfSourceAttribution::None`] is the single-date path, and it carries no
/// local at all: [`FunctionBuilder::emit_dtf_append`] then has nothing to read,
/// so `formatToParts` physically *cannot* grow a `source` property and
/// `formatRangeToParts` cannot lose one. That is the invariant, and it is a
/// match arm rather than a runtime `if` precisely because no Test262 case
/// inspects `source` on the single-date path — a review would be the only thing
/// standing between a mistake here and a silent spec violation.
#[derive(Clone, Copy)]
enum DtfSourceAttribution {
    /// `FormatDateTimePattern` (11.5.7): `{ type, value }` and nothing else.
    None,
    /// `PartitionDateTimeRangePattern` (11.5.9): `{ type, value, source }`,
    /// where `source_local` holds `"shared"`, `"startRange"` or `"endRange"`.
    Range { source_local: u32 },
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
    /// The `source` the pending literal was created under. A literal is
    /// attributed to the side that *asked* for it, not to whichever side
    /// happens to flush it, which is what makes the range separator `"shared"`
    /// while an intra-side `", "` stays with its own side.
    pending_source_local: u32,
    /// 1 once at least one non-literal field has been emitted.
    emitted_local: u32,
    scratch_local: u32,
    source: DtfSourceAttribution,
}

/// The time value(s) one `emit_intl_dtf_build_format_with_kind` call formats.
///
/// `second: None` is `format`/`formatToParts`: one date, no loop, no `source`.
/// `second: Some(_)` is `formatRange`/`formatRangeToParts`.
///
/// `kind` is deliberately **one** field for both sides. `SameTemporalType`
/// (11.5.9 step 5) has already run by the time a range builds this, so two
/// different kinds is not a state the formatter can be handed; making it one
/// field is what turns "did anybody check that?" into "there is nowhere to put
/// the second answer".
pub(crate) struct DtfFormatTimes {
    /// `x`.
    pub(crate) first: u32,
    /// `y`, for the two range entry points only.
    pub(crate) second: Option<u32>,
    /// The local holding the shared [`DtfValueKind::code`].
    pub(crate) kind: u32,
}

/// A `format`/`formatRange` argument that carried an
/// `OBJECT_INTERNAL_BRAND_TEMPORAL_*` slot.
///
/// Split out from [`DtfValueKind`] so that [`Self::brand`] is *infallible*.
/// With one enum covering both the branded and the unbranded case, the brand
/// dispatch had to `.expect()` its way out of an `Option<u64>` that the
/// iterator it walked already guaranteed was `Some` — and a future brandless
/// variant would then have compiled and panicked during emission instead of
/// failing `cargo check`, which is the whole point of spelling the domain as an
/// enum.
#[derive(Clone, Copy)]
pub(crate) enum DtfBrandedKind {
    /// One of [`INTL_DTF_TEMPORAL_KINDS`].
    Temporal(&'static IntlDtfTemporalKind),
    /// `Temporal.ZonedDateTime`, which `HandleDateTimeValue` refuses — but only
    /// after `SameTemporalType` has had its say.
    ZonedDateTime,
}

impl DtfBrandedKind {
    /// Every branded kind the dispatch has to recognise, in the order it tests
    /// them.
    fn all() -> impl Iterator<Item = Self> {
        INTL_DTF_TEMPORAL_KINDS
            .iter()
            .map(Self::Temporal)
            .chain(std::iter::once(Self::ZonedDateTime))
    }

    /// The `OBJECT_INTERNAL_BRAND_*` this kind dispatches on. Total, by
    /// construction.
    const fn brand(self) -> u64 {
        match self {
            Self::Temporal(kind) => kind.brand,
            Self::ZonedDateTime => OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME,
        }
    }

    const fn code(self) -> i64 {
        match self {
            Self::Temporal(kind) => kind.code,
            Self::ZonedDateTime => INTL_DTF_ZONED_DATE_TIME_KIND_CODE,
        }
    }
}

/// What one `format`/`formatRange` argument turned out to be, after
/// `ToDateTimeFormattable` (11.4.6 step 4) and the brand dispatch.
///
/// The emitted code stores [`DtfValueKind::code`] in a local; this enum is the
/// Rust-level domain the dispatch is *generated* from, so a brand can neither
/// be dispatched to a code no row owns nor to a row with no brand.
#[derive(Clone, Copy)]
pub(crate) enum DtfValueKind {
    /// Not a Temporal object: `ToNumber` ran and `TimeClip` applies. The
    /// *absence* of a brand match, not a brand of its own, which is why it is
    /// not a [`DtfBrandedKind`].
    Legacy,
    Branded(DtfBrandedKind),
}

impl DtfValueKind {
    const fn code(self) -> i64 {
        match self {
            Self::Legacy => 0,
            Self::Branded(kind) => kind.code(),
        }
    }
}

/// The two halves of a resolved time zone, as the locals holding them.
///
/// Produced in exactly one place —
/// [`FunctionBuilder::emit_intl_dtf_time_zone_option`] — and written to the
/// record in exactly one place, [`Self::store`]. Before this pairing the record
/// held a bare identifier payload with no offset beside it, so a new acceptance
/// path could store an identifier and leave the formatter reading a stale
/// offset; now the type has nowhere to put half an answer.
///
/// This is the *reserved* half: three locals that exist but have not been
/// written. It deliberately has no `store` and is deliberately not `Copy`. The
/// record slots can only be written through a [`DtfResolvedTimeZone`], and
/// [`FunctionBuilder::emit_intl_dtf_time_zone_option`] is the only thing that
/// produces one — it consumes this. Dropping that call therefore fails
/// `cargo check` instead of silently storing three zero-initialised slots, i.e.
/// an identifier payload of 0 beside an offset of 0. The move checker, not a
/// comment, is what makes "produced only by the option reader" true.
pub(crate) struct DtfCanonicalTimeZone {
    /// The string payload `resolvedOptions().timeZone` reports.
    identifier_local: u32,
    /// The signed whole minutes `PartitionDateTimePattern` adds to an exact
    /// time value. A raw `i64`, not an f64 bit pattern.
    offset_minutes_local: u32,
    /// The pre-rendered localized GMT name for any zone outside the named UTC
    /// family, or 0 for a member of it.
    gmt_name_local: u32,
}

impl DtfCanonicalTimeZone {
    /// Reserves the triple. Reserved together so they can be released
    /// together, which is what keeps the temp-local stack LIFO.
    fn reserve(builder: &mut FunctionBuilder<'_>) -> Self {
        Self {
            identifier_local: builder.reserve_temp_local(),
            offset_minutes_local: builder.reserve_temp_local(),
            gmt_name_local: builder.reserve_temp_local(),
        }
    }
}

/// A [`DtfCanonicalTimeZone`] whose three locals have all been written by
/// [`FunctionBuilder::emit_intl_dtf_time_zone_option`].
pub(crate) struct DtfResolvedTimeZone(DtfCanonicalTimeZone);

impl DtfResolvedTimeZone {
    /// The only writer of the three record slots. `resolvedOptions` reads the
    /// identifier, the component walk reads the offset and the `timeZoneName`
    /// field reads the name; none can be written without the others having
    /// been written first.
    fn store(&self, builder: &FunctionBuilder<'_>, record_local: u32, function: &mut Function) {
        for (offset, local) in [
            (HEAP_INTL_DTF_TIME_ZONE_OFFSET, self.0.identifier_local),
            (
                HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET,
                self.0.offset_minutes_local,
            ),
            (
                HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET,
                self.0.gmt_name_local,
            ),
        ] {
            builder.store_i64_local_at_offset(record_local, offset, local, function);
        }
    }

    fn release(self, builder: &mut FunctionBuilder<'_>) {
        builder.release_temp_local(self.0.gmt_name_local);
        builder.release_temp_local(self.0.offset_minutes_local);
        builder.release_temp_local(self.0.identifier_local);
    }
}

/// The broken-down components of one side of a format.
///
/// Naming the set lets the range path derive it twice — once per side — and
/// copy the selected side into the locals the field walk reads, instead of
/// emitting the walk itself twice.
#[derive(Clone, Copy)]
struct DtfComponentLocals {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    ms: u32,
    weekday_index: u32,
    display_year: u32,
}

impl DtfComponentLocals {
    fn locals(self) -> [u32; 9] {
        [
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.ms,
            self.weekday_index,
            self.display_year,
        ]
    }
}

/// Everything `PartitionDateTimeRangePattern` needs that the single-date path
/// does not have.
#[derive(Clone, Copy)]
struct DtfRangeLocals {
    /// The `y` time value; `times.first` is `x`.
    second_time: u32,
    start: DtfComponentLocals,
    end: DtfComponentLocals,
    /// 0 while the start side is being walked, 1 for the end side.
    side: u32,
    /// 1 when the sides are practically equal, otherwise 2.
    side_limit: u32,
    practically_equal: u32,
}

fn emit_dtf_copy_components(
    from: DtfComponentLocals,
    to: DtfComponentLocals,
    function: &mut Function,
) {
    for (source, dest) in from.locals().into_iter().zip(to.locals()) {
        function.instruction(&Instruction::LocalGet(source));
        function.instruction(&Instruction::LocalSet(dest));
    }
}

/// Upper bound on emitted parts: eleven fields with a literal between each,
/// plus the era and fractional-second extras. Rounded up so the array never
/// needs to grow.
const INTL_DTF_MAX_PARTS: i64 = 48;

/// The range bound: both sides plus the separator literal between them.
const INTL_DTF_MAX_RANGE_PARTS: i64 = 2 * INTL_DTF_MAX_PARTS + 1;

/// CLDR `en`'s `intervalFormatFallback` connector — space, EN DASH, space.
///
/// No Test262 case hard-codes this: every one of them derives the separator
/// from our own `formatRangeToParts` output first, so the only requirement is
/// that `formatRange` and `formatRangeToParts` agree, which they do by both
/// reading this constant.
const INTL_DTF_RANGE_SEPARATOR: &str = " \u{2013} ";

/// ECMA-402 11.4.6 step 3 / 11.4.7 step 3.
const INTL_DTF_RANGE_UNDEFINED_MESSAGE: &str =
    "Intl.DateTimeFormat.prototype.formatRange startDate and endDate must be defined";

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
    /// by `-`; anything else is a `RangeError`. A well-formed type this
    /// implementation has no data for is a `RangeError` too, rather than a
    /// silent substitution.
    ///
    /// `accepted` pairs each spelling with what it resolves to, and
    /// `resolved_local` — when given — receives that canonical string, so
    /// `resolvedOptions()` reports what the request actually became. The
    /// caller seeds `resolved_local` with the default before calling.
    fn emit_intl_dtf_unicode_type_option(
        &mut self,
        options_payload_local: u32,
        options_tag_local: u32,
        property: &str,
        accepted: &[(&str, &str)],
        resolved_local: Option<u32>,
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
        for (spelling, canonical) in accepted {
            self.emit_dtf_set_string(expected_local, spelling, function);
            self.emit_string_payload_equality_i32(value_payload_local, expected_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_const(ok_local, 1, function);
            if let Some(resolved_local) = resolved_local {
                self.emit_dtf_set_string(resolved_local, canonical, function);
            }
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

    /// `Get(options, "timeZone")` followed by `CreateDateTimeFormat` steps
    /// 30-31.
    ///
    /// Spec order, and the order below: the `UTCOffset` grammar first, then
    /// `GetAvailableNamedTimeZoneIdentifier`, then `RangeError`. The two spec
    /// branches raise the *same* error for a string this implementation cannot
    /// answer — step 30's "does not parse as `UTCOffset[~SubMinutePrecision]`"
    /// and step 31's "no such identifier" are both `RangeError` — so trying the
    /// parser, falling through to the table, and throwing once at the end is
    /// observationally identical to writing the branches out separately.
    /// `'+15:59:00'` is a well-formed *sub-minute* offset string and still a
    /// `RangeError` here, which is exactly what step 30.a.ii asks for.
    ///
    /// An accepted zone yields **all three** parts of a
    /// [`DtfResolvedTimeZone`]: an identifier with no offset beside it is not a
    /// value this function can return. It consumes the reserved
    /// [`DtfCanonicalTimeZone`] and is the only producer of the resolved form,
    /// so a caller cannot store a zone this never wrote.
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
        let lowered_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();
        let ok_local = self.reserve_temp_local();
        let parsed_local = self.reserve_temp_local();
        let minutes_local = self.reserve_temp_local();
        let gmt_scratch_local = self.reserve_temp_local();

        // `SystemTimeZoneIdentifier()` is `"UTC"` for this backend: there is no
        // host zone to read, and a wrong guess would silently shift every
        // default-zone format.
        self.emit_dtf_set_string(zone.identifier_local, INTL_DTF_RESOLVED_TIME_ZONE, function);
        self.emit_dtf_set_const(zone.offset_minutes_local, 0, function);
        self.emit_dtf_set_const(zone.gmt_name_local, 0, function);
        // Explicitly cleared, not left to Wasm's zero-initialisation: temporary
        // locals are pooled and reused, so a stale non-zero value here would
        // make an absent `timeZone` option look like an accepted `UTCOffset`
        // string to the GMT-name selection at the end of this function.
        self.emit_dtf_set_const(parsed_local, 0, function);
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

        // Step 30: IsTimeZoneOffsetString / ParseTimeZoneOffsetString.
        self.emit_intl_dtf_parse_utc_offset(
            value_payload_local,
            minutes_local,
            parsed_local,
            function,
        );
        self.emit_dtf_if_nonzero(parsed_local, function);
        function.instruction(&Instruction::LocalGet(minutes_local));
        function.instruction(&Instruction::LocalSet(zone.offset_minutes_local));
        self.emit_intl_dtf_format_offset_identifier(
            minutes_local,
            zone.identifier_local,
            function,
        )?;
        function.instruction(&Instruction::Else);

        // Step 31: GetAvailableNamedTimeZoneIdentifier, ASCII-case-insensitive,
        // answering with the table's spelling rather than the caller's.
        self.emit_intl_dtf_ascii_lowercase(value_payload_local, lowered_local, function)?;
        self.emit_dtf_set_const(ok_local, 0, function);
        for row in INTL_DTF_NAMED_ZONES {
            self.emit_dtf_set_string(
                expected_local,
                &row.identifier.to_ascii_lowercase(),
                function,
            );
            self.emit_string_payload_equality_i32(lowered_local, expected_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_const(ok_local, 1, function);
            self.emit_dtf_set_string(zone.identifier_local, row.identifier, function);
            self.emit_dtf_set_const(
                zone.offset_minutes_local,
                row.offset.minutes() as i64,
                function,
            );
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(ok_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // The `timeZoneName` field's localized GMT answer, rendered once here
        // instead of four times inside the format walk. A zero payload is the
        // sentinel for "use CLDR `en`'s real names for the UTC-named zone
        // family" — `UTC`, `Etc/GMT`, `Zulu`, `Greenwich` and the rest of
        // `INTL_DTF_NAMED_ZONES`' zero-offset rows.
        //
        // The discriminator is deliberately **not** "the offset is zero". An
        // *offset* identifier reports the localized GMT format whatever its
        // value, so `timeZone: '+00:00'` is `GMT`, not
        // `Coordinated Universal Time`: `IsTimeZoneOffsetString` accepted it, so
        // it is not the named UTC zone and has no CLDR name of its own.
        // `parsed_local` is exactly "step 30 accepted this as `UTCOffset`", and
        // it is still zero when the option was absent (the default zone is the
        // named `UTC`, which keeps the CLDR names).
        function.instruction(&Instruction::LocalGet(zone.offset_minutes_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_string(zone.gmt_name_local, INTL_DTF_GMT_PREFIX, function);
        self.emit_intl_dtf_format_offset_identifier(
            zone.offset_minutes_local,
            gmt_scratch_local,
            function,
        )?;
        self.emit_concat_string_payloads_local(zone.gmt_name_local, gmt_scratch_local, function)?;
        function.instruction(&Instruction::LocalSet(zone.gmt_name_local));
        function.instruction(&Instruction::Else);
        // A zero-offset *offset* zone. CLDR `en`'s `gmtZeroFormat` is the bare
        // `GMT`, for every width — not `GMT+00:00`.
        self.emit_dtf_if_nonzero(parsed_local, function);
        self.emit_dtf_set_string(zone.gmt_name_local, INTL_DTF_GMT_PREFIX, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            gmt_scratch_local,
            minutes_local,
            parsed_local,
            ok_local,
            expected_local,
            lowered_local,
            value_tag_local,
            value_payload_local,
            key_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(DtfResolvedTimeZone(zone))
    }

    /// `UTCOffset[~SubMinutePrecision]` (ECMA-262 21.4.1.34.1) and nothing else.
    ///
    /// `ok_local` becomes 1 exactly when the whole string is
    /// `ASCIISign Hour (TimeSeparator? MinuteSecond)?`, and `minutes_local`
    /// then holds the signed offset in whole minutes. Only three byte lengths
    /// are legal — 3 (`+HH`), 5 (`+HHMM`) and 6 (`+HH:MM`) — so the eight-byte
    /// sub-minute form `+HH:MM:SS` is refused by the length test before a byte
    /// is examined. That is why `'+15:59:00'` throws.
    ///
    /// Byte 0 must be ASCII `'+'` (0x2B) or `'-'` (0x2D). U+2212 MINUS SIGN is
    /// three WTF-8 bytes beginning 0xE2, so it can never reach the sign test
    /// and `offset-timezone-no-unicode-minus-sign.js` needs no special case.
    ///
    /// `Hour` is `0 DecimalDigit | 1 DecimalDigit | 20 | 21 | 22 | 23`, which
    /// for two decimal digits is precisely `hour <= 23`; `MinuteSecond` is
    /// `[0-5] DecimalDigit`, precisely `minute <= 59`.
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
        function.instruction(&Instruction::I64Const(TzOffsetMinutes::max_hour()));
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
        function.instruction(&Instruction::I64Const(TzOffsetMinutes::max_minute()));
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

    /// `br_if depth` unless the byte is `[0-9]`, for the offset parser's
    /// four digit positions.
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

    /// `FormatOffsetTimeZoneIdentifier(offsetMinutes)`: `±HH:MM`, both fields
    /// always present, and `'+'` for zero.
    ///
    /// `'-00'` and `'-00:00'` therefore both report `"+00:00"`, which is what
    /// `resolvedOptions/offset-timezone-change.js` pins, and `'+03'` reports
    /// `"+03:00"` rather than echoing the three-byte request back.
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
            self.emit_dtf_number_string(field_local, 2, piece_local, function)?;
            self.emit_concat_string_payloads_local(dest_local, piece_local, function)?;
            function.instruction(&Instruction::LocalSet(dest_local));
        }

        for local in [field_local, piece_local, magnitude_local] {
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
        let time_zone = DtfCanonicalTimeZone::reserve(self);
        let calendar_local = self.reserve_temp_local();
        let explicit_local = self.reserve_temp_local();
        // 1 once a component from `INTL_DTF_NEED_DEFAULTS_COMPONENTS` was
        // requested — the *other* half of `explicit_local`, which also counts
        // `era` and `timeZoneName`.
        let defaults_cleared_local = self.reserve_temp_local();
        let need_defaults_local = self.reserve_temp_local();
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
        self.emit_dtf_set_string(calendar_local, INTL_DTF_RESOLVED_CALENDAR, function);
        self.emit_intl_dtf_unicode_type_option(
            options_payload_local,
            options_tag_local,
            "calendar",
            INTL_DTF_ACCEPTED_CALENDARS,
            Some(calendar_local),
            function,
        )?;
        self.emit_intl_dtf_unicode_type_option(
            options_payload_local,
            options_tag_local,
            "numberingSystem",
            INTL_DTF_ACCEPTED_NUMBERING_SYSTEMS,
            None,
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

        // Steps 29-31: timeZone, as an identifier and an offset together. The
        // reserved triple is consumed here and comes back resolved; there is no
        // other way to obtain a value `store` will accept.
        let time_zone = self.emit_intl_dtf_time_zone_option(
            options_payload_local,
            options_tag_local,
            time_zone,
            function,
        )?;

        // Step 36: Table 7, in table order, with fractionalSecondDigits
        // spliced in after `second`.
        self.emit_dtf_set_const(explicit_local, 0, function);
        self.emit_dtf_set_const(defaults_cleared_local, 0, function);
        for (option, dest_local) in INTL_DTF_COMPONENT_OPTIONS.iter().zip(&component_locals) {
            self.emit_intl_dtf_string_option(
                options_payload_local,
                options_tag_local,
                option,
                *dest_local,
                Some(present_local),
                function,
            )?;
            self.emit_intl_dtf_note_component_present(
                option.property,
                explicit_local,
                defaults_cleared_local,
                present_local,
                function,
            );
            if option.property == INTL_DTF_FRACTIONAL_SECOND_DIGITS_AFTER {
                self.emit_intl_dtf_fractional_second_digits_option(
                    options_payload_local,
                    options_tag_local,
                    fractional_local,
                    present_local,
                    function,
                )?;
                self.emit_intl_dtf_note_component_present(
                    "fractionalSecondDigits",
                    explicit_local,
                    defaults_cleared_local,
                    present_local,
                    function,
                );
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

        // Steps 40-41 and 44: `needDefaults` survives `era` and `timeZoneName`
        // — see [`INTL_DTF_NEED_DEFAULTS_COMPONENTS`] — and when it does, the
        // `defaults` of a `date`-required formatter make year, month and day
        // "numeric". A Temporal receiver overrides that fill with its own at
        // format time, which is why the bit is stored rather than consumed.
        function.instruction(&Instruction::LocalGet(date_style_local));
        function.instruction(&Instruction::LocalGet(time_style_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalGet(defaults_cleared_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(need_defaults_local));
        self.emit_dtf_if_nonzero(need_defaults_local, function);
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
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_CALENDAR_OFFSET,
            calendar_local,
            function,
        );
        {
            let payload = self.strings.payload(INTL_DTF_RESOLVED_NUMBERING_SYSTEM);
            self.store_i64_const_at_offset(
                record_local,
                HEAP_INTL_DTF_NUMBERING_SYSTEM_OFFSET,
                payload as u64,
                function,
            );
        }
        time_zone.store(self, record_local, function);
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
        self.store_i64_local_at_offset(
            record_local,
            HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET,
            need_defaults_local,
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
            need_defaults_local,
            defaults_cleared_local,
            explicit_local,
            calendar_local,
        ] {
            self.release_temp_local(local);
        }
        time_zone.release(self);
        for local in [
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

    /// Folds one component's presence into the constructor's two bits.
    ///
    /// `explicit_local` is `hasExplicitFormatComponents`, which every Table 7
    /// property feeds; `defaults_cleared_local` is the narrower step-40/41
    /// question, which `era` and `timeZoneName` do not answer. Routing both
    /// through one function is what stops a later reader from "simplifying"
    /// them back into a single OR.
    fn emit_intl_dtf_note_component_present(
        &self,
        property: &str,
        explicit_local: u32,
        defaults_cleared_local: u32,
        present_local: u32,
        function: &mut Function,
    ) {
        for dest_local in [
            Some(explicit_local),
            INTL_DTF_NEED_DEFAULTS_COMPONENTS
                .contains(&property)
                .then_some(defaults_cleared_local),
        ]
        .into_iter()
        .flatten()
        {
            function.instruction(&Instruction::LocalGet(dest_local));
            function.instruction(&Instruction::LocalGet(present_local));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(dest_local));
        }
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
        // The pending literal keeps the attribution it was created under; the
        // field takes the side that is being walked right now.
        let literal_source = match sink.source {
            DtfSourceAttribution::None => DtfSourceAttribution::None,
            DtfSourceAttribution::Range { .. } => DtfSourceAttribution::Range {
                source_local: sink.pending_source_local,
            },
        };
        self.emit_dtf_if_nonzero(sink.pending_literal_local, function);
        self.emit_dtf_append(
            sink,
            "literal",
            sink.pending_literal_local,
            literal_source,
            function,
        )?;
        self.emit_dtf_set_const(sink.pending_literal_local, 0, function);
        function.instruction(&Instruction::End);
        self.emit_dtf_append(sink, part_type, value_local, sink.source, function)?;
        self.emit_dtf_set_const(sink.emitted_local, 1, function);
        Ok(())
    }

    /// The one place a part reaches the output. In `String` mode it is
    /// concatenated; in `Parts` mode a `{ type, value }` object is appended,
    /// gaining a third `source` property exactly when `source` says so.
    fn emit_dtf_append(
        &mut self,
        sink: &DtfFormatSink,
        part_type: &'static str,
        value_local: u32,
        source: DtfSourceAttribution,
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
                match source {
                    DtfSourceAttribution::None => {}
                    DtfSourceAttribution::Range { source_local } => {
                        self.emit_dtf_set_string(key_local, "source", function);
                        self.emit_object_append_data_property_with_flags(
                            object_local,
                            key_local,
                            source_local,
                            tag_local,
                            true,
                            true,
                            true,
                            function,
                        )?;
                    }
                }
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

    /// Sets the literal to emit before the next field, attributed to the side
    /// currently being walked.
    fn emit_dtf_pending(&mut self, sink: &DtfFormatSink, text: &str, function: &mut Function) {
        self.emit_dtf_set_string(sink.pending_literal_local, text, function);
        if let DtfSourceAttribution::Range { source_local } = sink.source {
            function.instruction(&Instruction::LocalGet(source_local));
            function.instruction(&Instruction::LocalSet(sink.pending_source_local));
        }
    }

    /// The date components of one side, exactly as
    /// `PartitionDateTimePattern` needs them: `MakeDay`-style fields, the
    /// weekday index rebased so 0 is Sunday, and the era year.
    ///
    /// # The one place a time zone exists
    ///
    /// `offset_minutes_local` is added here and nowhere else in the crate. The
    /// caller has already collapsed 11.5.11's `[[IsPlain]]` into it — zero for
    /// a `Temporal.Plain*` value whose epoch milliseconds already *are* its
    /// wall clock, the resolved zone's offset for an instant — so this function
    /// has one rule and no cases, and a second application of the offset would
    /// have to be written somewhere visibly wrong.
    fn emit_dtf_components_from_time(
        &mut self,
        time_local: u32,
        offset_minutes_local: u32,
        comps: DtfComponentLocals,
        function: &mut Function,
    ) {
        // `LocalTime(t) = t + offset`, in the f64-bit-pattern convention every
        // `Date` helper here uses. The offset local is a raw signed integer, so
        // it is converted rather than reinterpreted.
        let time_local = {
            let local_time_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(time_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(offset_minutes_local));
            function.instruction(&Instruction::F64ConvertI64S);
            function.instruction(&Instruction::F64Const(Ieee64::from(
                INTL_DTF_MILLISECONDS_PER_MINUTE,
            )));
            function.instruction(&Instruction::F64Mul);
            function.instruction(&Instruction::F64Add);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(local_time_local));
            local_time_local
        };
        self.emit_date_components_from_time(
            time_local,
            comps.year,
            comps.month,
            comps.day,
            comps.hour,
            comps.minute,
            comps.second,
            comps.ms,
            function,
        );
        self.emit_date_day_from_time(time_local, comps.weekday_index, function);
        function.instruction(&Instruction::LocalGet(comps.weekday_index));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(4.0)));
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(comps.weekday_index));
        self.emit_date_positive_mod(comps.weekday_index, 7.0, function);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(comps.weekday_index));

        // Proleptic Gregorian year 0 is 1 BC, so the displayed year is the era
        // year, never a zero or a negative.
        function.instruction(&Instruction::LocalGet(comps.year));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(comps.year));
        function.instruction(&Instruction::LocalSet(comps.display_year));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::LocalGet(comps.year));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(comps.display_year));
        function.instruction(&Instruction::End);

        self.release_temp_local(time_local);
    }

    /// `PartitionDateTimeRangePattern` steps 7-12: are the two sides
    /// indistinguishable under the resolved components? If so the range
    /// collapses to a single formatted date whose every part is `"shared"`.
    ///
    /// `codes` is `[era, year, month, day, dayPeriod, hour, minute, second,
    /// fractionalSecondDigits]`. `weekday` and `timeZoneName` are absent
    /// because step 11's table does not list them as range fields.
    fn emit_dtf_practical_equality(
        &mut self,
        codes: [u32; 9],
        range: DtfRangeLocals,
        function: &mut Function,
    ) {
        let [e_era, e_year, e_month, e_day, e_day_period, e_hour, e_minute, e_second, e_fractional] =
            codes;
        let a = range.start;
        let b = range.end;

        self.emit_dtf_set_const(range.practically_equal, 1, function);

        // The era differs exactly when one side is AD and the other BC.
        self.emit_dtf_if_nonzero(e_era, function);
        for year_local in [a.year, b.year] {
            function.instruction(&Instruction::LocalGet(year_local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
            function.instruction(&Instruction::F64Gt);
        }
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(range.practically_equal, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // The spec compares the raw record fields, not the rendered numerals:
        // `hour` is the 0-23 value whatever the resolved hour cycle, and
        // `fractionalSecondDigits` compares whole milliseconds.
        for (code_local, left, right) in [
            (e_year, a.display_year, b.display_year),
            (e_month, a.month, b.month),
            (e_day, a.day, b.day),
            (e_hour, a.hour, b.hour),
            (e_minute, a.minute, b.minute),
            (e_second, a.second, b.second),
            (e_fractional, a.ms, b.ms),
        ] {
            self.emit_dtf_if_nonzero(code_local, function);
            function.instruction(&Instruction::LocalGet(left));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::LocalGet(right));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_dtf_set_const(range.practically_equal, 0, function);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        // `dayPeriod` has no numeric field to compare, so compare the rendered
        // names. Every value `emit_dtf_day_period_value` can produce is a pool
        // literal, so equal strings are the identical payload and `i64.ne` is
        // exact. A dayPeriod that ever built a string at runtime would break
        // that and would have to compare contents instead.
        let left_period_local = self.reserve_temp_local();
        let right_period_local = self.reserve_temp_local();
        self.emit_dtf_if_nonzero(e_day_period, function);
        self.emit_dtf_day_period_value(
            e_day_period,
            a.hour,
            a.minute,
            a.second,
            a.ms,
            left_period_local,
            function,
        );
        self.emit_dtf_day_period_value(
            e_day_period,
            b.hour,
            b.minute,
            b.second,
            b.ms,
            right_period_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(left_period_local));
        function.instruction(&Instruction::LocalGet(right_period_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_dtf_set_const(range.practically_equal, 0, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(right_period_local);
        self.release_temp_local(left_period_local);

        self.emit_dtf_if_nonzero(range.practically_equal, function);
        self.emit_dtf_set_const(range.side_limit, 1, function);
        function.instruction(&Instruction::Else);
        self.emit_dtf_set_const(range.side_limit, 2, function);
        function.instruction(&Instruction::End);
    }

    fn reserve_dtf_components(&mut self) -> DtfComponentLocals {
        DtfComponentLocals {
            year: self.reserve_temp_local(),
            month: self.reserve_temp_local(),
            day: self.reserve_temp_local(),
            hour: self.reserve_temp_local(),
            minute: self.reserve_temp_local(),
            second: self.reserve_temp_local(),
            ms: self.reserve_temp_local(),
            weekday_index: self.reserve_temp_local(),
            display_year: self.reserve_temp_local(),
        }
    }

    fn release_dtf_components(&mut self, comps: DtfComponentLocals) {
        for local in comps.locals().into_iter().rev() {
            self.release_temp_local(local);
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    /// `PartitionDateTimePattern` (ECMA-402 11.5.6) for `en-US`/`gregory`/
    /// `latn` and the record's resolved zone, in the one shape `format` and
    /// `formatToParts` share, and — with a second time value —
    /// `PartitionDateTimeRangePattern` (11.5.9), which `formatRange` and
    /// `formatRangeToParts` share.
    ///
    /// Each time is a time value in milliseconds **UTC**: the caller has run
    /// `ToDateTimeFormattable` and, on the legacy path only, `TimeClip`. The
    /// resolved zone's offset is applied here rather than by the caller, once,
    /// in [`Self::emit_dtf_components_from_time`].
    ///
    /// # Why the range path is a wasm loop and not a second copy
    ///
    /// The field walk below is the largest body this crate emits — fifty-seven
    /// string-literal selects for the month and weekday names alone. Emitting
    /// it twice inside one function is the shape that trips Cranelift's
    /// per-function size limit, whose only recovery is a size-optimized
    /// recompile of the *whole* module. Wrapping it in a `loop` that runs once
    /// or twice grows the body by a copy of the component set instead, and the
    /// single-date callers still emit no loop at all.
    ///
    /// There is deliberately **no** wrapper that pins the kind to the legacy
    /// code. Every entry point — including both range entry points — runs
    /// `HandleDateTimeValue` and hands the answer in, so "this caller cannot
    /// see a Temporal value" is not something any caller may assert on its own
    /// behalf any more.
    ///
    /// `times.kind` holds a [`DtfValueKind::code`]: zero for the legacy
    /// Number/Date path, otherwise an [`IntlDtfTemporalKind::code`].
    pub(crate) fn emit_intl_dtf_build_format_with_kind(
        &mut self,
        record_local: u32,
        times: DtfFormatTimes,
        mode: DtfFormatMode,
        out_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind_local = times.kind;
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
        // The resolved zone's offset as stored...
        let zone_offset_local = self.reserve_temp_local();
        // ...the offset actually added to the time value, which is that one for
        // an exact instant and zero for a `Temporal.Plain*` wall clock...
        let applied_offset_local = self.reserve_temp_local();
        // ...and the pre-rendered `timeZoneName` for a non-zero offset, 0 for
        // the UTC family.
        let zone_gmt_name_local = self.reserve_temp_local();
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

        // The locals the field walk reads. On the range path the loop head
        // copies the selected side into them.
        let current = DtfComponentLocals {
            year: year_local,
            month: month_local,
            day: day_local,
            hour: hour_local,
            minute: minute_local,
            second: second_local,
            ms: ms_local,
            weekday_index: weekday_index_local,
            display_year: display_year_local,
        };

        let sink = DtfFormatSink {
            mode,
            text_local: self.reserve_temp_local(),
            array_local: self.reserve_temp_local(),
            buffer_local: self.reserve_temp_local(),
            length_local: self.reserve_temp_local(),
            pending_literal_local: self.reserve_temp_local(),
            pending_source_local: self.reserve_temp_local(),
            emitted_local: self.reserve_temp_local(),
            scratch_local: self.reserve_temp_local(),
            // `source` is observable only through `formatRangeToParts`, so the
            // String mode of a range still emits no `source` property and no
            // attribution bookkeeping.
            source: match (mode, &times.second) {
                (DtfFormatMode::Parts, Some(_)) => DtfSourceAttribution::Range {
                    source_local: self.reserve_temp_local(),
                },
                _ => DtfSourceAttribution::None,
            },
        };

        // Range-only locals, reserved after everything above so the
        // single-date path allocates exactly the locals it allocated before.
        let range = match times.second {
            None => None,
            Some(second_time) => Some(DtfRangeLocals {
                second_time,
                start: self.reserve_dtf_components(),
                end: self.reserve_dtf_components(),
                side: self.reserve_temp_local(),
                side_limit: self.reserve_temp_local(),
                practically_equal: self.reserve_temp_local(),
            }),
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
            (
                HEAP_INTL_DTF_TIME_ZONE_OFFSET_MINUTES_OFFSET,
                zone_offset_local,
            ),
            (HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET, zone_gmt_name_local),
        ] {
            self.load_i64_to_local_from_offset(record_local, offset, local, function);
        }
        self.emit_dtf_set_const(join_at_local, 0, function);

        // ECMA-402 11.5.11 `[[IsPlain]]`, resolved once for the whole body.
        //
        // The legacy `Number`/`Date` path is `kind == 0` and is an exact
        // instant, so the loaded offset is the starting answer and only the
        // plain Temporal rows overwrite it. The `match` is exhaustive on
        // purpose: a Temporal row added without a `basis` — or with a basis
        // this code has not been taught — will not compile, and that is the
        // whole point, because the mistake is invisible under `UTC` and shifts
        // a `PlainDate` by a day at `+13:00`.
        function.instruction(&Instruction::LocalGet(zone_offset_local));
        function.instruction(&Instruction::LocalSet(applied_offset_local));
        for kind in INTL_DTF_TEMPORAL_KINDS {
            match kind.basis {
                DtfTimeBasis::Exact => {}
                DtfTimeBasis::Plain => {
                    self.emit_dtf_if_code_eq(kind_local, kind.code, function);
                    self.emit_dtf_set_const(applied_offset_local, 0, function);
                    function.instruction(&Instruction::End);
                }
            }
        }

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

        // --- Temporal field adjustment ---------------------------------------
        //
        // ECMA-402 11.5.11 `HandleDateTimeValue` / `AdjustDateTimeStyleFormat`.
        // A Temporal value may only be rendered with the fields its type
        // actually has, so every component outside the type's `allowed` set is
        // cleared here — *after* the style expansion, which is what makes
        // `dateStyle: "full"` on a `PlainDate` byte-identical to the legacy
        // path while `timeStyle: "long"` on a `PlainDateTime` quietly loses its
        // zone name.
        //
        // When the format asked for no components at all, the constructor's
        // date-shaped guess is replaced by the type's own defaults first. That
        // one stored bit is the whole reason `{ era: "narrow" }` renders a
        // `PlainTime` as a time while `{ year: "numeric" }` refuses to render
        // it at all: the former never cleared `needDefaults`, the latter did.
        let effective_slots: [(u64, u32); 11] = [
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
        ];
        debug_assert_eq!(
            effective_slots.map(|(slot, _)| slot),
            INTL_DTF_FORMAT_COMPONENT_SLOTS
        );
        let need_defaults_local = self.reserve_temp_local();
        self.emit_dtf_if_nonzero(kind_local, function);
        self.load_i64_to_local_from_offset(
            record_local,
            HEAP_INTL_DTF_NEED_DEFAULTS_OFFSET,
            need_defaults_local,
            function,
        );
        for kind in INTL_DTF_TEMPORAL_KINDS {
            self.emit_dtf_if_code_eq(kind_local, kind.code, function);
            self.emit_dtf_if_nonzero(need_defaults_local, function);
            for (slot, code) in kind.defaults {
                let local = effective_slots
                    .iter()
                    .find(|(candidate, _)| candidate == slot)
                    .map(|(_, local)| *local)
                    .expect("a Temporal default names a component slot");
                debug_assert!(
                    kind.allowed.contains(slot),
                    "a defaulted slot must survive the mask"
                );
                self.emit_dtf_set_const(local, *code, function);
            }
            function.instruction(&Instruction::End);
            for (slot, local) in effective_slots {
                if !kind.allowed.contains(&slot) {
                    self.emit_dtf_set_const(local, 0, function);
                }
            }
            function.instruction(&Instruction::End);
        }
        // An empty format is `AdjustDateTimeStyleFormat` finding no overlap at
        // all: `{ year: "numeric" }` cannot render a `Temporal.PlainTime`.
        for (index, (_, local)) in effective_slots.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(*local));
            if index > 0 {
                function.instruction(&Instruction::I64Or);
            }
        }
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
        function.instruction(&Instruction::End);
        self.release_temp_local(need_defaults_local);

        // --- date components ------------------------------------------------
        match range {
            None => {
                self.emit_dtf_components_from_time(
                    times.first,
                    applied_offset_local,
                    current,
                    function,
                );
            }
            Some(range) => {
                self.emit_dtf_components_from_time(
                    times.first,
                    applied_offset_local,
                    range.start,
                    function,
                );
                self.emit_dtf_components_from_time(
                    range.second_time,
                    applied_offset_local,
                    range.end,
                    function,
                );
                self.emit_dtf_practical_equality(
                    [
                        e_era,
                        e_year,
                        e_month,
                        e_day,
                        e_day_period,
                        e_hour,
                        e_minute,
                        e_second,
                        e_fractional,
                    ],
                    range,
                    function,
                );
            }
        }

        // --- sink -----------------------------------------------------------
        // Loop-invariant: the output accumulates across both sides.
        self.emit_dtf_set_string(sink.text_local, "", function);
        self.emit_dtf_set_const(sink.pending_literal_local, 0, function);
        self.emit_dtf_set_const(sink.emitted_local, 0, function);
        self.emit_dtf_set_const(sink.length_local, 0, function);
        if mode == DtfFormatMode::Parts {
            self.emit_dtf_set_const(
                sink.scratch_local,
                match range {
                    None => INTL_DTF_MAX_PARTS,
                    Some(_) => INTL_DTF_MAX_RANGE_PARTS,
                },
                function,
            );
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

        // --- one iteration per side ------------------------------------------
        if let Some(range) = range {
            self.emit_dtf_set_const(range.side, 0, function);
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(range.side));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            emit_dtf_copy_components(range.start, current, function);
            self.emit_dtf_set_const(sink.pending_literal_local, 0, function);
            if let DtfSourceAttribution::Range { source_local } = sink.source {
                // 11.5.9 step 13: a collapsed range is entirely `"shared"`.
                self.emit_dtf_if_nonzero(range.practically_equal, function);
                self.emit_dtf_set_string(source_local, "shared", function);
                function.instruction(&Instruction::Else);
                self.emit_dtf_set_string(source_local, "startRange", function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(source_local));
                function.instruction(&Instruction::LocalSet(sink.pending_source_local));
            }
            function.instruction(&Instruction::Else);
            emit_dtf_copy_components(range.end, current, function);
            // The separator is appended outright rather than parked in
            // `pending_literal`. A pending literal is overwritable — the
            // fractional-second branch, for one, sets `"."` unconditionally —
            // and a separator that some later field silently clobbers would
            // turn `1/3/2019 – 1/5/2019` into `1/3/20191/5/2019` with nothing
            // to catch it. Appending it here also keeps it outside both sides'
            // bookkeeping, which is what `"shared"` means.
            self.emit_dtf_set_const(sink.pending_literal_local, 0, function);
            self.emit_dtf_set_string(value_local, INTL_DTF_RANGE_SEPARATOR, function);
            if let DtfSourceAttribution::Range { source_local } = sink.source {
                self.emit_dtf_set_string(source_local, "shared", function);
            }
            self.emit_dtf_append(&sink, "literal", value_local, sink.source, function)?;
            if let DtfSourceAttribution::Range { source_local } = sink.source {
                self.emit_dtf_set_string(source_local, "endRange", function);
                function.instruction(&Instruction::LocalGet(source_local));
                function.instruction(&Instruction::LocalSet(sink.pending_source_local));
            }
            function.instruction(&Instruction::End);
            // Both must be reset per side. A stale `emitted` would send the end
            // side's first field down its `if emitted { pending = ", " }` arm,
            // so `1/3/2019 – 1/5/2019` would come out as `1/3/2019 – , 1/5/2019`;
            // a stale `body_last` would put a `/` in front of the end side's
            // month. `time_started` is reset inside the walk already.
            self.emit_dtf_set_const(sink.emitted_local, 0, function);
            self.emit_dtf_set_const(body_last_local, 0, function);
        }

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
        self.emit_dtf_time_zone_name_value(
            e_time_zone_name,
            zone_gmt_name_local,
            value_local,
            function,
        );
        self.emit_dtf_push(&sink, "timeZoneName", value_local, function)?;
        function.instruction(&Instruction::End);

        if let Some(range) = range {
            function.instruction(&Instruction::LocalGet(range.side));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(range.side));
            function.instruction(&Instruction::LocalGet(range.side));
            function.instruction(&Instruction::LocalGet(range.side_limit));
            function.instruction(&Instruction::I64LtU);
            function.instruction(&Instruction::BrIf(0));
            function.instruction(&Instruction::End);
        }

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

        if let Some(range) = range {
            for local in [range.practically_equal, range.side_limit, range.side] {
                self.release_temp_local(local);
            }
            self.release_dtf_components(range.end);
            self.release_dtf_components(range.start);
        }
        if let DtfSourceAttribution::Range { source_local } = sink.source {
            self.release_temp_local(source_local);
        }

        for local in [
            sink.scratch_local,
            sink.emitted_local,
            sink.pending_source_local,
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
            zone_gmt_name_local,
            applied_offset_local,
            zone_offset_local,
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

    /// The `timeZoneName` field, derived from the resolved zone's offset rather
    /// than assumed to be UTC.
    ///
    /// The **UTC-named zone family** keeps CLDR `en`'s real names byte for
    /// byte — [`TimeZoneNameStyle::utc_name`] — because
    /// `constructor-options-timeZoneName-valid.js` and
    /// `format/temporal-plaindate-formatting-timezonename.js` read them back.
    ///
    /// Every other zone uses the localized GMT name the constructor already
    /// rendered into [`HEAP_INTL_DTF_TIME_ZONE_GMT_NAME_OFFSET`], which doubles
    /// as the discriminator: a zero payload *is* "this zone is a named member
    /// of the UTC family". Note that this is **not** the same question as "is
    /// the offset zero" — an offset identifier such as `'+00:00'` has offset
    /// zero and still gets the GMT name, because it is not the named `UTC`
    /// zone. `emit_intl_dtf_time_zone_option` is where that distinction is
    /// made. This body is emitted once per `format`, `formatToParts`,
    /// `formatRange` and `formatRangeToParts`, so building the name here would
    /// have cost ten inline string concatenations four times over in the one
    /// function whose size budget is known to be tight.
    ///
    /// No Test262 case in the current corpus observes a `timeZoneName` under a
    /// non-UTC zone, so this is insurance rather than points — but the
    /// alternative, leaving the constant table in place, would print
    /// `"Coordinated Universal Time"` for `timeZone: "+03:00"`, and a
    /// plausible-looking wrong answer is worse than a missing one.
    fn emit_dtf_time_zone_name_value(
        &mut self,
        style_code_local: u32,
        gmt_name_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) {
        // A non-zero offset has a pre-rendered name and every style shares it.
        self.emit_dtf_if_nonzero(gmt_name_local, function);
        function.instruction(&Instruction::LocalGet(gmt_name_local));
        function.instruction(&Instruction::LocalSet(dest_local));
        function.instruction(&Instruction::Else);
        for style in TimeZoneNameStyle::ALL {
            self.emit_dtf_if_code_eq(style_code_local, style.code(), function);
            self.emit_dtf_set_string(dest_local, style.utc_name(), function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
    }
}

impl<'a> FunctionBuilder<'a> {
    /// `HandleDateTimeValue` (ECMA-402 11.5.11).
    ///
    /// `undefined` means "now". A branded Temporal object is reduced to an
    /// epoch-millisecond instant and reports which of
    /// [`INTL_DTF_TEMPORAL_KINDS`] it is through `kind_local`; anything else
    /// goes through `ToNumber` and `TimeClip` with `kind_local` left at zero,
    /// and a non-finite result is a `RangeError`.
    ///
    /// `TimeClip` is deliberately **not** applied on the Temporal branches:
    /// `Temporal.PlainDate` reaches four years beyond the `Date` range in each
    /// direction and must still render, and `emit_date_components_from_time`
    /// is pure f64 arithmetic that does not care.
    fn emit_intl_dtf_handle_date_time_value(
        &mut self,
        argument_index: usize,
        time_local: u32,
        kind_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();

        self.emit_dtf_set_const(kind_local, 0, function);
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

        // Brand dispatch. Every branch writes both `kind_local` and
        // `time_local`, so a kind can never be reported without the instant it
        // was derived from.
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        // `Temporal.ZonedDateTime` carries its own zone, which cannot be
        // reconciled with the formatter's; the specification refuses it here
        // rather than picking one.
        self.emit_dtf_if_code_eq(
            brand_local,
            OBJECT_INTERNAL_BRAND_TEMPORAL_ZONED_DATE_TIME as i64,
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
        for kind in INTL_DTF_TEMPORAL_KINDS {
            self.emit_dtf_if_code_eq(brand_local, kind.brand as i64, function);
            self.emit_dtf_set_const(kind_local, kind.code, function);
            self.load_i64_to_local_from_offset(
                value_payload_local,
                HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                record_local,
                function,
            );
            self.emit_intl_dtf_temporal_epoch_milliseconds(
                kind.brand,
                record_local,
                time_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
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
        function.instruction(&Instruction::End);

        for local in [
            record_local,
            brand_local,
            value_tag_local,
            value_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The epoch-millisecond instant a branded Temporal record stands for.
    ///
    /// The three partial-date types share the `Temporal.PlainDate` layout and
    /// are anchored at **noon** — `NoonTimeRecord()` — which is what keeps
    /// `+275760-09-13` and `-271821-04-19` renderable instead of rounding off
    /// the end of the range. `Temporal.PlainTime` is anchored on the ISO epoch
    /// day, so day zero, and its date fields are masked away later anyway.
    ///
    /// # Representation
    ///
    /// Temporal record slots hold **raw signed `i64`**, while `MakeDay` and
    /// `MakeTime` take the f64 bit patterns every `Date` local is in. Each
    /// field is therefore widened on the way out, in the one loop below;
    /// reinterpreting instead of converting would read an integer as a
    /// denormal and silently produce 1970.
    fn emit_intl_dtf_temporal_epoch_milliseconds(
        &mut self,
        brand: u64,
        record_local: u32,
        time_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let year_local = self.reserve_temp_local();
        let month_local = self.reserve_temp_local();
        let day_local = self.reserve_temp_local();
        let day_number_local = self.reserve_temp_local();
        let within_day_local = self.reserve_temp_local();
        let hour_local = self.reserve_temp_local();
        let minute_local = self.reserve_temp_local();
        let second_local = self.reserve_temp_local();
        let ms_local = self.reserve_temp_local();

        // Every branch ends with `time = day * 86400000 + withinDay`, so the
        // two halves are combined in exactly one place.
        match brand {
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE
            | OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_YEAR_MONTH
            | OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_MONTH_DAY
            | OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME => {
                let is_date_time = brand == OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_DATE_TIME;
                let (year_offset, month_offset, day_offset) = if is_date_time {
                    (
                        HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_YEAR_OFFSET,
                        HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_MONTH_OFFSET,
                        HEAP_TEMPORAL_PLAIN_DATE_TIME_ISO_DAY_OFFSET,
                    )
                } else {
                    (
                        HEAP_TEMPORAL_PLAIN_DATE_ISO_YEAR_OFFSET,
                        HEAP_TEMPORAL_PLAIN_DATE_ISO_MONTH_OFFSET,
                        HEAP_TEMPORAL_PLAIN_DATE_ISO_DAY_OFFSET,
                    )
                };
                // `MakeDay` takes the zero-based month a `Date` uses; the ISO
                // slot is one-based, so the bias is applied on the integer
                // before it is widened.
                for (offset, local, bias) in [
                    (year_offset, year_local, 0_i64),
                    (month_offset, month_local, -1),
                    (day_offset, day_local, 0),
                ] {
                    self.emit_intl_dtf_load_temporal_integer_as_float(
                        record_local,
                        offset,
                        bias,
                        local,
                        function,
                    );
                }
                self.emit_date_make_day(
                    year_local,
                    month_local,
                    day_local,
                    day_number_local,
                    function,
                );
                if is_date_time {
                    for (offset, local) in [
                        (HEAP_TEMPORAL_PLAIN_DATE_TIME_HOUR_OFFSET, hour_local),
                        (HEAP_TEMPORAL_PLAIN_DATE_TIME_MINUTE_OFFSET, minute_local),
                        (HEAP_TEMPORAL_PLAIN_DATE_TIME_SECOND_OFFSET, second_local),
                        (HEAP_TEMPORAL_PLAIN_DATE_TIME_MILLISECOND_OFFSET, ms_local),
                    ] {
                        self.emit_intl_dtf_load_temporal_integer_as_float(
                            record_local,
                            offset,
                            0,
                            local,
                            function,
                        );
                    }
                    self.emit_date_make_time(
                        hour_local,
                        minute_local,
                        second_local,
                        ms_local,
                        within_day_local,
                        function,
                    );
                } else {
                    function.instruction(&Instruction::F64Const(Ieee64::from(
                        INTL_DTF_TEMPORAL_NOON_MILLISECONDS,
                    )));
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(within_day_local));
                }
            }
            OBJECT_INTERNAL_BRAND_TEMPORAL_PLAIN_TIME => {
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(day_number_local));
                for (offset, local) in [
                    (HEAP_TEMPORAL_PLAIN_TIME_HOUR_OFFSET, hour_local),
                    (HEAP_TEMPORAL_PLAIN_TIME_MINUTE_OFFSET, minute_local),
                    (HEAP_TEMPORAL_PLAIN_TIME_SECOND_OFFSET, second_local),
                    (HEAP_TEMPORAL_PLAIN_TIME_MILLISECOND_OFFSET, ms_local),
                ] {
                    self.emit_intl_dtf_load_temporal_integer_as_float(
                        record_local,
                        offset,
                        0,
                        local,
                        function,
                    );
                }
                self.emit_date_make_time(
                    hour_local,
                    minute_local,
                    second_local,
                    ms_local,
                    within_day_local,
                    function,
                );
            }
            OBJECT_INTERNAL_BRAND_TEMPORAL_INSTANT => {
                self.emit_temporal_epoch_nanoseconds_record_to_milliseconds(
                    record_local,
                    HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_PAYLOAD_OFFSET,
                    HEAP_TEMPORAL_INSTANT_EPOCH_NANOSECONDS_TAG_OFFSET,
                    time_local,
                    function,
                );
                for local in [
                    ms_local,
                    second_local,
                    minute_local,
                    hour_local,
                    within_day_local,
                    day_number_local,
                    day_local,
                    month_local,
                    year_local,
                ] {
                    self.release_temp_local(local);
                }
                return Ok(());
            }
            _ => {
                return Err(EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: Intl.DateTimeFormat cannot reduce this Temporal brand to an instant",
                ));
            }
        }

        function.instruction(&Instruction::LocalGet(day_number_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            INTL_DTF_MILLISECONDS_PER_DAY,
        )));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(within_day_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(time_local));

        for local in [
            ms_local,
            second_local,
            minute_local,
            hour_local,
            within_day_local,
            day_number_local,
            day_local,
            month_local,
            year_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `dest = (f64)(record[offset] + bias)`, bridging a Temporal record's
    /// integer slot to the f64-bit-pattern convention every `Date` helper uses.
    fn emit_intl_dtf_load_temporal_integer_as_float(
        &mut self,
        record_local: u32,
        offset: u64,
        bias: i64,
        dest_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(record_local, offset, dest_local, function);
        function.instruction(&Instruction::LocalGet(dest_local));
        if bias != 0 {
            function.instruction(&Instruction::I64Const(bias));
            function.instruction(&Instruction::I64Add);
        }
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dest_local));
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
        let kind_local = self.reserve_temp_local();
        let out_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(object_local));
        self.load_i64_to_local_from_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.emit_intl_dtf_handle_date_time_value(0, time_local, kind_local, function)?;
        self.emit_intl_dtf_build_format_with_kind(
            record_local,
            DtfFormatTimes {
                first: time_local,
                second: None,
                kind: kind_local,
            },
            DtfFormatMode::String,
            out_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            out_local,
            kind_local,
            time_local,
            record_local,
            object_local,
        ] {
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
        let kind_local = self.reserve_temp_local();
        let out_local = self.reserve_temp_local();

        self.emit_intl_dtf_record_from_receiver(
            record_local,
            "Intl.DateTimeFormat.prototype.formatToParts",
            function,
        )?;
        self.emit_intl_dtf_handle_date_time_value(0, time_local, kind_local, function)?;
        self.emit_intl_dtf_build_format_with_kind(
            record_local,
            DtfFormatTimes {
                first: time_local,
                second: None,
                kind: kind_local,
            },
            DtfFormatMode::Parts,
            out_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [out_local, kind_local, time_local, record_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// ECMA-402 11.4.6 steps 3-5 followed by 11.5.9 steps 2-6, in that order.
    ///
    /// This is the range half of `HandleDateTimeValue` (11.5.11), and the order
    /// is the whole point of the function:
    ///
    /// 1. Either argument `undefined` is a `TypeError`, decided before any
    ///    coercion — so `formatRange(poison, undefined)` never reaches the
    ///    poisoned `valueOf`. The range entry points have no "now" default,
    ///    which is the one way they differ from
    ///    [`Self::emit_intl_dtf_handle_date_time_value`].
    /// 2. `ToDateTimeFormattable` on argument 0, **then** on argument 1. A
    ///    branded Temporal object is kept as-is and reports its
    ///    [`DtfValueKind`]; anything else goes through `ToNumber`.
    /// 3. *Only now* `SameTemporalType`. Running it earlier is exactly the bug
    ///    `to-datetime-formattable-with-different-arg-kinds.js` measures: it
    ///    counts `valueOf` calls and requires one per argument that is not a
    ///    Temporal object, even though the call is about to throw.
    /// 4. Two `Temporal.ZonedDateTime`s are the *same* type, so the refusal
    ///    that `HandleDateTimeValue` owes them comes after step 3, not before.
    /// 5. `TimeClip` and the non-finite `RangeError` apply to the legacy path
    ///    only. `Temporal.PlainDate` reaches four years beyond the `Date` range
    ///    in each direction and must still render.
    fn emit_intl_dtf_range_argument_values(
        &mut self,
        x_local: u32,
        y_local: u32,
        kind_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let x_payload_local = self.reserve_temp_local();
        let x_tag_local = self.reserve_temp_local();
        let y_payload_local = self.reserve_temp_local();
        let y_tag_local = self.reserve_temp_local();
        let x_kind_local = self.reserve_temp_local();
        let y_kind_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, x_payload_local, x_tag_local, function);
        self.emit_builtin_arg_to_locals(1, y_payload_local, y_tag_local, function);

        for tag_local in [x_tag_local, y_tag_local] {
            function.instruction(&Instruction::LocalGet(tag_local));
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

        // Step 2: `ToDateTimeFormattable` per argument, argument 0 first.
        for (tag_local, payload_local, side_kind_local) in [
            (x_tag_local, x_payload_local, x_kind_local),
            (y_tag_local, y_payload_local, y_kind_local),
        ] {
            self.emit_dtf_set_const(side_kind_local, DtfValueKind::Legacy.code(), function);
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                payload_local,
                HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                brand_local,
                function,
            );
            for kind in DtfBrandedKind::all() {
                self.emit_dtf_if_code_eq(brand_local, kind.brand() as i64, function);
                self.emit_dtf_set_const(side_kind_local, kind.code(), function);
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
            // `IsTemporalObject(value)` is false, so `ToNumber` runs — and it
            // runs here, inside the per-argument loop, which is what keeps the
            // observable `valueOf` order argument-0-then-argument-1.
            function.instruction(&Instruction::LocalGet(side_kind_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_value_to_number_payload(tag_local, payload_local, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            self.emit_return_current_completion_if_throw(function);
            function.instruction(&Instruction::End);
        }

        // Step 3: `SameTemporalType(x, y)`. Two legacy values are both kind 0
        // and therefore equal, so the whole condition is just "the kinds
        // differ" — a legacy value next to any Temporal one included, which is
        // what `fails-on-distinct-temporal-types.js` asserts.
        function.instruction(&Instruction::LocalGet(x_kind_local));
        function.instruction(&Instruction::LocalGet(y_kind_local));
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

        // The two sides now provably agree, so one local carries both.
        function.instruction(&Instruction::LocalGet(x_kind_local));
        function.instruction(&Instruction::LocalSet(kind_local));

        // Step 4: a `Temporal.ZonedDateTime` carries its own zone, which cannot
        // be reconciled with the formatter's, so `HandleDateTimeValue` refuses
        // it — with the same message the single-date path uses.
        self.emit_dtf_if_code_eq(kind_local, DtfBrandedKind::ZonedDateTime.code(), function);
        self.emit_throw_current_function_realm_type_error(
            INTL_DTF_ZONED_DATE_TIME_UNSUPPORTED,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Step 5, legacy half. `x > y` is *not* an error: that requirement was
        // removed in 2021 and the range simply formats in the order given.
        function.instruction(&Instruction::LocalGet(kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (payload_local, time_local) in [(x_payload_local, x_local), (y_payload_local, y_local)]
        {
            self.emit_date_time_clip(payload_local, time_local, function);
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
        }
        function.instruction(&Instruction::End);

        // Step 5, Temporal half: the same reduction the single-date path uses,
        // and no `TimeClip`.
        for kind in INTL_DTF_TEMPORAL_KINDS {
            self.emit_dtf_if_code_eq(kind_local, kind.code, function);
            for (payload_local, time_local) in
                [(x_payload_local, x_local), (y_payload_local, y_local)]
            {
                self.load_i64_to_local_from_offset(
                    payload_local,
                    HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                    record_local,
                    function,
                );
                self.emit_intl_dtf_temporal_epoch_milliseconds(
                    kind.brand,
                    record_local,
                    time_local,
                    function,
                )?;
            }
            function.instruction(&Instruction::End);
        }

        for local in [
            record_local,
            brand_local,
            y_kind_local,
            x_kind_local,
            y_tag_local,
            y_payload_local,
            x_tag_local,
            x_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The body `formatRange` (11.4.6) and `formatRangeToParts` (11.4.7)
    /// share: brand check, then both arguments, then `FormatDateTimeRange`.
    ///
    /// The brand check runs first, which is what `formatRange.call({})` with
    /// no arguments at all requires — a `TypeError` for the receiver, not for
    /// the missing dates.
    ///
    /// The single `kind_local` handed to the formatter is the one
    /// `SameTemporalType` has already agreed on, so both ends of the range are
    /// masked with the same Temporal field set by construction.
    fn emit_intl_dtf_format_range(
        &mut self,
        method: &str,
        mode: DtfFormatMode,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record_local = self.reserve_temp_local();
        let x_local = self.reserve_temp_local();
        let y_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();
        let out_local = self.reserve_temp_local();

        self.emit_intl_dtf_record_from_receiver(record_local, method, function)?;
        self.emit_intl_dtf_range_argument_values(x_local, y_local, kind_local, function)?;
        self.emit_intl_dtf_build_format_with_kind(
            record_local,
            DtfFormatTimes {
                first: x_local,
                second: Some(y_local),
                kind: kind_local,
            },
            mode,
            out_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(out_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        let result_tag = match mode {
            DtfFormatMode::String => ValueKind::String.tag() as i64,
            DtfFormatMode::Parts => ValueKind::Array.tag() as i64,
        };
        function.instruction(&Instruction::I64Const(result_tag));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [out_local, kind_local, y_local, x_local, record_local] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// `Intl.DateTimeFormat.prototype.formatRange` — ECMA-402 11.4.6.
    pub(crate) fn emit_intl_date_time_format_format_range(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_dtf_format_range(
            "Intl.DateTimeFormat.prototype.formatRange",
            DtfFormatMode::String,
            function,
        )
    }

    /// `Intl.DateTimeFormat.prototype.formatRangeToParts` — ECMA-402 11.4.7.
    pub(crate) fn emit_intl_date_time_format_format_range_to_parts(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_intl_dtf_format_range(
            "Intl.DateTimeFormat.prototype.formatRangeToParts",
            DtfFormatMode::Parts,
            function,
        )
    }

    /// The whole body of every `Temporal.X.prototype.toLocaleString`.
    ///
    /// The Temporal proposal defines these as
    /// `new Intl.DateTimeFormat(locales, options).format(this)` with one extra
    /// rejection, and that is literally what this emits: the constructor and
    /// the `get format` accessor are called through the ordinary builtin call
    /// path, so `x.toLocaleString(l, o)` and
    /// `new Intl.DateTimeFormat(l, o).format(x)` are the same computation and
    /// cannot drift — which is precisely what most of these tests assert.
    ///
    /// Going through the cached bound format function also means the field
    /// walk is emitted **once** per module rather than once per Temporal type;
    /// inlining [`Self::emit_intl_dtf_build_format_with_kind`] into seven more bodies is
    /// the obvious implementation and the one that trips the per-function size
    /// limit.
    ///
    /// The caller has already brand-checked the receiver, because only it
    /// knows which `[[Initialized...]]` slot to name in the message, and the
    /// check is observable before the arguments are read. It names the type by
    /// the same `OBJECT_INTERNAL_BRAND_*` constant it checked against, so the
    /// method and the field set it formats with cannot disagree.
    pub(crate) fn emit_intl_dtf_temporal_to_locale_string(
        &mut self,
        brand: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = INTL_DTF_TEMPORAL_KINDS
            .iter()
            .find(|kind| kind.brand == brand)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: Intl.DateTimeFormat has no field set for this Temporal brand",
                )
            })?;
        let this_payload_local = self.reserve_temp_local();
        let this_tag_local = self.reserve_temp_local();
        let locales_payload_local = self.reserve_temp_local();
        let locales_tag_local = self.reserve_temp_local();
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let dtf_payload_local = self.reserve_temp_local();
        let dtf_tag_local = self.reserve_temp_local();
        let format_payload_local = self.reserve_temp_local();
        let format_tag_local = self.reserve_temp_local();
        let record_local = self.reserve_temp_local();
        let style_local = self.reserve_temp_local();

        self.compile_this_to_locals(this_payload_local, this_tag_local, function)?;
        self.emit_builtin_arg_to_locals(0, locales_payload_local, locales_tag_local, function);
        self.emit_builtin_arg_to_locals(1, options_payload_local, options_tag_local, function);

        let constructor_meta = self
            .functions
            .get(&StandardBuiltinId::IntlDateTimeFormatConstructor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Intl.DateTimeFormat`",
                )
            })?;
        // 11.1.1 step 1 substitutes the active function object for NewTarget on
        // a plain call, so this returns an instance rather than throwing.
        self.emit_direct_js_call(
            &constructor_meta,
            None,
            &[
                (locales_payload_local, locales_tag_local),
                (options_payload_local, options_tag_local),
            ],
            dtf_payload_local,
            dtf_tag_local,
            function,
        )?;

        // `[[Required]]`: a style this type has no fields for is a TypeError,
        // and it has to be raised here because `AdjustDateTimeStyleFormat`
        // would otherwise quietly render the half that does overlap. The other
        // half of every `options-conflict.js` — an explicit component next to
        // a style — already threw inside the constructor.
        if let Some((property, style_offset)) = kind.rejected_style {
            let message = intl_dtf_temporal_style_message(kind.type_name, property);
            self.load_i64_to_local_from_offset(
                dtf_payload_local,
                HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                record_local,
                function,
            );
            self.load_i64_to_local_from_offset(record_local, style_offset, style_local, function);
            self.emit_dtf_if_nonzero(style_local, function);
            self.emit_throw_current_function_realm_type_error(
                &message,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }

        let format_getter_meta = self
            .functions
            .get(&StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `get Intl.DateTimeFormat.prototype.format`",
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
            style_local,
            record_local,
            format_tag_local,
            format_payload_local,
            dtf_tag_local,
            dtf_payload_local,
            options_tag_local,
            options_payload_local,
            locales_tag_local,
            locales_payload_local,
            this_tag_local,
            this_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}

/// The `[[Required]]` rejection message, built in one place so the pool and
/// the emitter cannot spell it differently.
fn intl_dtf_temporal_style_message(type_name: &str, property: &str) -> String {
    format!("{type_name}.prototype.toLocaleString does not support the {property} option")
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
        "formatRange",
        "formatRangeToParts",
        "type",
        "value",
        "source",
        "shared",
        "startRange",
        "endRange",
        INTL_DTF_RANGE_SEPARATOR,
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
        INTL_DTF_GMT_PREFIX,
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
    values.push(INTL_DTF_UNSUPPORTED_TIME_ZONE_MESSAGE.to_string());
    // Every named zone contributes two literals: the identifier the record
    // stores and reports, and the ASCII-lowercased form the lookup compares
    // against. Derived from the table, so a row added there can never reference
    // a string the data section is missing.
    for row in INTL_DTF_NAMED_ZONES {
        values.push(row.identifier.to_string());
        values.push(row.identifier.to_ascii_lowercase());
    }
    // The `timeZoneName` renderer: the six zero-offset literals plus the pieces
    // the localized GMT format is concatenated from.
    for style in TimeZoneNameStyle::ALL {
        values.push(style.utc_name().to_string());
    }
    for sign in INTL_DTF_OFFSET_SIGNS {
        values.push(sign.to_string());
    }
    for method in [
        "Intl.DateTimeFormat.prototype.resolvedOptions",
        "get Intl.DateTimeFormat.prototype.format",
        "Intl.DateTimeFormat.prototype.formatToParts",
        "Intl.DateTimeFormat.prototype.formatRange",
        "Intl.DateTimeFormat.prototype.formatRangeToParts",
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
        INTL_DTF_RANGE_UNDEFINED_MESSAGE,
        INTL_DTF_RANGE_DIFFERENT_TYPES_MESSAGE,
        INTL_DTF_ZONED_DATE_TIME_UNSUPPORTED,
        INTL_DTF_EMPTY_TEMPORAL_FORMAT,
    ] {
        values.push(value.to_string());
    }
    // The Temporal spellings the `-u-ca` option accepts, and the one rejection
    // message each Temporal `toLocaleString` can raise. Both are derived from
    // the tables the emitters read, so a row added there cannot reference a
    // string the data section is missing.
    for (spelling, canonical) in INTL_DTF_ACCEPTED_CALENDARS
        .iter()
        .chain(INTL_DTF_ACCEPTED_NUMBERING_SYSTEMS)
    {
        values.push((*spelling).to_string());
        values.push((*canonical).to_string());
    }
    for kind in INTL_DTF_TEMPORAL_KINDS {
        if let Some((property, _)) = kind.rejected_style {
            values.push(intl_dtf_temporal_style_message(kind.type_name, property));
        }
    }
    values
}
