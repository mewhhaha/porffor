//! The closed option domains the Temporal builtins share.
//!
//! Every one of these was previously spelled as a bare `i64` constant,
//! redeclared once per Temporal type, with the numeric encoding restated at
//! each call site. A unit code, a rounding mode, an `overflow` value and a
//! `calendarName` value all have the same Rust type when they are `i64`, and
//! they share numeric ranges: `TEMPORAL_UNIT_SECOND` and
//! `TEMPORAL_ROUNDING_MODE_HALF_EXPAND` were both `6`. Passing one where the
//! other belongs compiled, and review could not see it either.
//!
//! The `code()` values here are the encoding written into emitted Wasm and are
//! deliberately identical to the constants they replace: overflow 0/1,
//! calendarName 0..=3, units 0..=9 with `auto` 10, unset -1 and invalid -2,
//! rounding modes 0..=8, offset 0..=3.

/// A Temporal unit. Declaration order *is* code order: a smaller code names a
/// *larger* unit, which is why `Ord` is not derived — the derived order would
/// read backwards against the domain. Comparisons go through
/// [`TemporalUnit::is_larger_than`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl TemporalUnit {
    /// Declaration order, which is also code order.
    pub(crate) const ALL: [TemporalUnit; 10] = [
        TemporalUnit::Year,
        TemporalUnit::Month,
        TemporalUnit::Week,
        TemporalUnit::Day,
        TemporalUnit::Hour,
        TemporalUnit::Minute,
        TemporalUnit::Second,
        TemporalUnit::Millisecond,
        TemporalUnit::Microsecond,
        TemporalUnit::Nanosecond,
    ];

    /// The `i64` written into emitted Wasm. Smaller code, larger unit.
    pub(crate) const fn code(self) -> i64 {
        match self {
            TemporalUnit::Year => 0,
            TemporalUnit::Month => 1,
            TemporalUnit::Week => 2,
            TemporalUnit::Day => 3,
            TemporalUnit::Hour => 4,
            TemporalUnit::Minute => 5,
            TemporalUnit::Second => 6,
            TemporalUnit::Millisecond => 7,
            TemporalUnit::Microsecond => 8,
            TemporalUnit::Nanosecond => 9,
        }
    }

    /// Index into the ten-slot `Temporal.Duration` field arrays. Deliberately a
    /// *separate* exhaustive match from [`TemporalUnit::code`], even though the
    /// two numberings coincide today: the emitted-Wasm encoding and the Rust
    /// array layout are independent decisions and a future change to one must
    /// not silently move the other.
    pub(crate) const fn duration_field_index(self) -> usize {
        match self {
            TemporalUnit::Year => 0,
            TemporalUnit::Month => 1,
            TemporalUnit::Week => 2,
            TemporalUnit::Day => 3,
            TemporalUnit::Hour => 4,
            TemporalUnit::Minute => 5,
            TemporalUnit::Second => 6,
            TemporalUnit::Millisecond => 7,
            TemporalUnit::Microsecond => 8,
            TemporalUnit::Nanosecond => 9,
        }
    }

    pub(crate) const fn singular(self) -> &'static str {
        match self {
            TemporalUnit::Year => "year",
            TemporalUnit::Month => "month",
            TemporalUnit::Week => "week",
            TemporalUnit::Day => "day",
            TemporalUnit::Hour => "hour",
            TemporalUnit::Minute => "minute",
            TemporalUnit::Second => "second",
            TemporalUnit::Millisecond => "millisecond",
            TemporalUnit::Microsecond => "microsecond",
            TemporalUnit::Nanosecond => "nanosecond",
        }
    }

    pub(crate) const fn plural(self) -> &'static str {
        match self {
            TemporalUnit::Year => "years",
            TemporalUnit::Month => "months",
            TemporalUnit::Week => "weeks",
            TemporalUnit::Day => "days",
            TemporalUnit::Hour => "hours",
            TemporalUnit::Minute => "minutes",
            TemporalUnit::Second => "seconds",
            TemporalUnit::Millisecond => "milliseconds",
            TemporalUnit::Microsecond => "microseconds",
            TemporalUnit::Nanosecond => "nanoseconds",
        }
    }

    /// `None` for the calendar units, which have no fixed length.
    pub(crate) const fn nanoseconds(self) -> Option<i64> {
        match self {
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week => None,
            TemporalUnit::Day => Some(86_400_000_000_000),
            TemporalUnit::Hour => Some(3_600_000_000_000),
            TemporalUnit::Minute => Some(60_000_000_000),
            TemporalUnit::Second => Some(1_000_000_000),
            TemporalUnit::Millisecond => Some(1_000_000),
            TemporalUnit::Microsecond => Some(1_000),
            TemporalUnit::Nanosecond => Some(1),
        }
    }

    /// `MaximumTemporalDurationRoundingIncrement`. `None` where the unit has no
    /// maximum, which is year through day.
    pub(crate) const fn maximum_rounding_increment(self) -> Option<i64> {
        match self {
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                None
            }
            TemporalUnit::Hour => Some(24),
            TemporalUnit::Minute | TemporalUnit::Second => Some(60),
            TemporalUnit::Millisecond | TemporalUnit::Microsecond | TemporalUnit::Nanosecond => {
                Some(1_000)
            }
        }
    }

    pub(crate) const fn is_calendar_unit(self) -> bool {
        match self {
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week => true,
            TemporalUnit::Day
            | TemporalUnit::Hour
            | TemporalUnit::Minute
            | TemporalUnit::Second
            | TemporalUnit::Millisecond
            | TemporalUnit::Microsecond
            | TemporalUnit::Nanosecond => false,
        }
    }

    /// The domain's order, which is the *reverse* of the code order.
    pub(crate) const fn is_larger_than(self, other: TemporalUnit) -> bool {
        self.code() < other.code()
    }
}

/// Seconds per unit, for the four units the `Temporal.Duration` second/subsecond
/// split works in. Ordered largest unit first, so the enumerate index is the
/// number of entries to skip when the balancing starts at that unit.
pub(crate) const TEMPORAL_UNIT_SECONDS: [(TemporalUnit, i64); 4] = [
    (TemporalUnit::Day, 86_400),
    (TemporalUnit::Hour, 3_600),
    (TemporalUnit::Minute, 60),
    (TemporalUnit::Second, 1),
];

/// The six wall-clock units, which are exactly the units a `Temporal.PlainTime`
/// may name and exactly the units with both a fixed nanosecond length and a
/// finite rounding-increment maximum.
///
/// This exists so `Temporal.PlainTime`'s six-slot arrays — the field locals, the
/// nanosecond scales and the rounding maxima — stop being four independent
/// numberings held together by `TEMPORAL_UNIT_HOUR + index`. Declaration order
/// is the field order the record uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalTimeUnit {
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl TemporalTimeUnit {
    pub(crate) const ALL: [TemporalTimeUnit; 6] = [
        TemporalTimeUnit::Hour,
        TemporalTimeUnit::Minute,
        TemporalTimeUnit::Second,
        TemporalTimeUnit::Millisecond,
        TemporalTimeUnit::Microsecond,
        TemporalTimeUnit::Nanosecond,
    ];

    pub(crate) const fn unit(self) -> TemporalUnit {
        match self {
            TemporalTimeUnit::Hour => TemporalUnit::Hour,
            TemporalTimeUnit::Minute => TemporalUnit::Minute,
            TemporalTimeUnit::Second => TemporalUnit::Second,
            TemporalTimeUnit::Millisecond => TemporalUnit::Millisecond,
            TemporalTimeUnit::Microsecond => TemporalUnit::Microsecond,
            TemporalTimeUnit::Nanosecond => TemporalUnit::Nanosecond,
        }
    }

    pub(crate) const fn code(self) -> i64 {
        self.unit().code()
    }

    /// Total, unlike [`TemporalUnit::nanoseconds`].
    pub(crate) const fn nanoseconds(self) -> i64 {
        match self {
            TemporalTimeUnit::Hour => 3_600_000_000_000,
            TemporalTimeUnit::Minute => 60_000_000_000,
            TemporalTimeUnit::Second => 1_000_000_000,
            TemporalTimeUnit::Millisecond => 1_000_000,
            TemporalTimeUnit::Microsecond => 1_000,
            TemporalTimeUnit::Nanosecond => 1,
        }
    }

    /// Total, unlike [`TemporalUnit::maximum_rounding_increment`].
    pub(crate) const fn maximum_rounding_increment(self) -> i64 {
        match self {
            TemporalTimeUnit::Hour => 24,
            TemporalTimeUnit::Minute | TemporalTimeUnit::Second => 60,
            TemporalTimeUnit::Millisecond
            | TemporalTimeUnit::Microsecond
            | TemporalTimeUnit::Nanosecond => 1_000,
        }
    }
}

// The time-unit view and the full unit domain must not drift: the six wall-clock
// units are contiguous at the bottom of the code range, and their magnitudes are
// stated in both places.
const _: () = {
    let mut index = 0;
    while index < TemporalTimeUnit::ALL.len() {
        let time_unit = TemporalTimeUnit::ALL[index];
        assert!(time_unit.code() == TemporalUnit::Hour.code() + index as i64);
        match time_unit.unit().nanoseconds() {
            Some(nanoseconds) => assert!(nanoseconds == time_unit.nanoseconds()),
            None => panic!("a wall-clock unit must have a fixed nanosecond length"),
        }
        match time_unit.unit().maximum_rounding_increment() {
            Some(maximum) => assert!(maximum == time_unit.maximum_rounding_increment()),
            None => panic!("a wall-clock unit must have a rounding-increment maximum"),
        }
        index += 1;
    }
};

/// The property and `auto` policy for `GetTemporalUnitValuedOption`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalUnitOptionProperty {
    LargestUnit,
    SmallestUnit,
    Unit,
}

impl TemporalUnitOptionProperty {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            TemporalUnitOptionProperty::LargestUnit => "largestUnit",
            TemporalUnitOptionProperty::SmallestUnit => "smallestUnit",
            TemporalUnitOptionProperty::Unit => "unit",
        }
    }

    pub(crate) const fn allows_auto(self) -> bool {
        match self {
            TemporalUnitOptionProperty::LargestUnit => true,
            TemporalUnitOptionProperty::SmallestUnit | TemporalUnitOptionProperty::Unit => false,
        }
    }
}

/// What a `GetTemporalUnitValuedOption` read can produce. The reader is the one
/// place a JS string becomes a unit; every consumer downstream takes the code
/// this produces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalUnitSlot {
    Unit(TemporalUnit),
    /// `"auto"`, accepted only for [`TemporalUnitOptionProperty::LargestUnit`].
    Auto,
    /// The property was absent.
    Unset,
    /// A string that names no unit.
    Invalid,
}

impl TemporalUnitSlot {
    pub(crate) const fn code(self) -> i64 {
        match self {
            TemporalUnitSlot::Unit(unit) => unit.code(),
            TemporalUnitSlot::Auto => 10,
            TemporalUnitSlot::Unset => -1,
            TemporalUnitSlot::Invalid => -2,
        }
    }
}

/// `ToTemporalRoundingMode`. Declaration order is code order, and the half-*
/// family is contiguous at the top because
/// `emit_temporal_duration_round_up_i32` tests the whole family with one
/// `mode >= HalfCeil` comparison. The `const` block below fails the build if
/// that stops being true.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalRoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

impl TemporalRoundingMode {
    pub(crate) const ALL: [TemporalRoundingMode; 9] = [
        TemporalRoundingMode::Ceil,
        TemporalRoundingMode::Floor,
        TemporalRoundingMode::Expand,
        TemporalRoundingMode::Trunc,
        TemporalRoundingMode::HalfCeil,
        TemporalRoundingMode::HalfFloor,
        TemporalRoundingMode::HalfExpand,
        TemporalRoundingMode::HalfTrunc,
        TemporalRoundingMode::HalfEven,
    ];

    pub(crate) const fn code(self) -> i64 {
        match self {
            TemporalRoundingMode::Ceil => 0,
            TemporalRoundingMode::Floor => 1,
            TemporalRoundingMode::Expand => 2,
            TemporalRoundingMode::Trunc => 3,
            TemporalRoundingMode::HalfCeil => 4,
            TemporalRoundingMode::HalfFloor => 5,
            TemporalRoundingMode::HalfExpand => 6,
            TemporalRoundingMode::HalfTrunc => 7,
            TemporalRoundingMode::HalfEven => 8,
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            TemporalRoundingMode::Ceil => "ceil",
            TemporalRoundingMode::Floor => "floor",
            TemporalRoundingMode::Expand => "expand",
            TemporalRoundingMode::Trunc => "trunc",
            TemporalRoundingMode::HalfCeil => "halfCeil",
            TemporalRoundingMode::HalfFloor => "halfFloor",
            TemporalRoundingMode::HalfExpand => "halfExpand",
            TemporalRoundingMode::HalfTrunc => "halfTrunc",
            TemporalRoundingMode::HalfEven => "halfEven",
        }
    }

    pub(crate) const fn is_half(self) -> bool {
        match self {
            TemporalRoundingMode::Ceil
            | TemporalRoundingMode::Floor
            | TemporalRoundingMode::Expand
            | TemporalRoundingMode::Trunc => false,
            TemporalRoundingMode::HalfCeil
            | TemporalRoundingMode::HalfFloor
            | TemporalRoundingMode::HalfExpand
            | TemporalRoundingMode::HalfTrunc
            | TemporalRoundingMode::HalfEven => true,
        }
    }

    /// `NegateRoundingMode`: ceil and floor swap, as do halfCeil and halfFloor;
    /// the sign-symmetric modes are unchanged.
    pub(crate) const fn negated(self) -> Self {
        match self {
            TemporalRoundingMode::Ceil => TemporalRoundingMode::Floor,
            TemporalRoundingMode::Floor => TemporalRoundingMode::Ceil,
            TemporalRoundingMode::HalfCeil => TemporalRoundingMode::HalfFloor,
            TemporalRoundingMode::HalfFloor => TemporalRoundingMode::HalfCeil,
            TemporalRoundingMode::Expand => TemporalRoundingMode::Expand,
            TemporalRoundingMode::Trunc => TemporalRoundingMode::Trunc,
            TemporalRoundingMode::HalfExpand => TemporalRoundingMode::HalfExpand,
            TemporalRoundingMode::HalfTrunc => TemporalRoundingMode::HalfTrunc,
            TemporalRoundingMode::HalfEven => TemporalRoundingMode::HalfEven,
        }
    }
}

// `emit_temporal_duration_round_up_i32` folds the whole half-* family into one
// `mode >= HalfCeil` test, and the rounding-mode reader writes `ALL`'s index as
// the code. Both dependencies are load-bearing and neither is visible at the
// site that relies on it, so assert them at build time.
const _: () = {
    let mut index = 0;
    while index < TemporalRoundingMode::ALL.len() {
        let mode = TemporalRoundingMode::ALL[index];
        assert!(mode.code() == index as i64);
        assert!(mode.is_half() == (index >= TemporalRoundingMode::HalfCeil.code() as usize));
        // Negation must be an involution and must stay inside the family.
        assert!(mode.negated().negated().code() == mode.code());
        assert!(mode.negated().is_half() == mode.is_half());
        index += 1;
    }
};

// `TemporalUnit::ALL` is indexed by code in the unit-option reader, and
// `duration_field_index` is used to index the ten-slot Duration arrays.
const _: () = {
    let mut index = 0;
    while index < TemporalUnit::ALL.len() {
        assert!(TemporalUnit::ALL[index].code() == index as i64);
        assert!(TemporalUnit::ALL[index].duration_field_index() < 10);
        index += 1;
    }
};

/// A string-valued Temporal option whose accepted spellings and emitted codes
/// are fixed by the proposal.
///
/// `DEFAULT` is a required associated const rather than "whatever entry happens
/// to be first in `ALLOWED`": the old reader took a `&[(&str, i64)]` slice and
/// silently treated `allowed[0]` as the default, so reordering the slice
/// changed behaviour with nothing else to see.
pub(crate) trait StringValuedOption: Copy + 'static {
    /// The property name read out of the options bag.
    const PROPERTY: &'static str;
    /// The value used when the property is absent.
    const DEFAULT: Self;
    /// Every accepted value, in the order the reader tests them.
    const ALLOWED: &'static [Self];
    fn name(self) -> &'static str;
    fn code(self) -> i64;
}

/// Whether a `ToTemporal*` conversion owns an observable `overflow` options
/// read.
///
/// `Read` carries the only local pair the conversion may observe. `Omit`
/// carries no placeholder locals, so an internal conversion cannot
/// accidentally read a dummy or unrelated options value.
#[derive(Clone, Copy, Debug)]
pub(super) enum TemporalConversionOverflowOptions {
    Read { payload_local: u32, tag_local: u32 },
    Omit,
}

/// `GetTemporalOverflowOption`. One definition for all seven Temporal types.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalOverflow {
    Constrain,
    Reject,
}

impl TemporalOverflow {
    pub(crate) const fn code(self) -> i64 {
        match self {
            TemporalOverflow::Constrain => 0,
            TemporalOverflow::Reject => 1,
        }
    }
}

impl StringValuedOption for TemporalOverflow {
    const PROPERTY: &'static str = "overflow";
    const DEFAULT: Self = TemporalOverflow::Constrain;
    const ALLOWED: &'static [Self] = &[TemporalOverflow::Constrain, TemporalOverflow::Reject];

    fn name(self) -> &'static str {
        match self {
            TemporalOverflow::Constrain => "constrain",
            TemporalOverflow::Reject => "reject",
        }
    }

    fn code(self) -> i64 {
        TemporalOverflow::code(self)
    }
}

/// `GetTemporalShowCalendarNameOption`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ShowCalendarName {
    Auto,
    Always,
    Never,
    Critical,
}

impl ShowCalendarName {
    pub(crate) const fn code(self) -> i64 {
        match self {
            ShowCalendarName::Auto => 0,
            ShowCalendarName::Always => 1,
            ShowCalendarName::Never => 2,
            ShowCalendarName::Critical => 3,
        }
    }
}

impl StringValuedOption for ShowCalendarName {
    const PROPERTY: &'static str = "calendarName";
    const DEFAULT: Self = ShowCalendarName::Auto;
    const ALLOWED: &'static [Self] = &[
        ShowCalendarName::Auto,
        ShowCalendarName::Always,
        ShowCalendarName::Never,
        ShowCalendarName::Critical,
    ];

    fn name(self) -> &'static str {
        match self {
            ShowCalendarName::Auto => "auto",
            ShowCalendarName::Always => "always",
            ShowCalendarName::Never => "never",
            ShowCalendarName::Critical => "critical",
        }
    }

    fn code(self) -> i64 {
        ShowCalendarName::code(self)
    }
}

/// `GetTemporalDisambiguationOption`. This backend resolves only `UTC` and
/// fixed offsets, so the value is read, validated and then deliberately
/// dropped — see `ZonedDateTimeOptionKey::destination`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Disambiguation {
    Compatible,
    Earlier,
    Later,
    Reject,
}

impl StringValuedOption for Disambiguation {
    const PROPERTY: &'static str = "disambiguation";
    const DEFAULT: Self = Disambiguation::Compatible;
    const ALLOWED: &'static [Self] = &[
        Disambiguation::Compatible,
        Disambiguation::Earlier,
        Disambiguation::Later,
        Disambiguation::Reject,
    ];

    fn name(self) -> &'static str {
        match self {
            Disambiguation::Compatible => "compatible",
            Disambiguation::Earlier => "earlier",
            Disambiguation::Later => "later",
            Disambiguation::Reject => "reject",
        }
    }

    fn code(self) -> i64 {
        match self {
            Disambiguation::Compatible => 0,
            Disambiguation::Earlier => 1,
            Disambiguation::Later => 2,
            Disambiguation::Reject => 3,
        }
    }
}

/// `GetTemporalOffsetOption`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum OffsetOption {
    Reject,
    Use,
    Prefer,
    Ignore,
}

impl StringValuedOption for OffsetOption {
    const PROPERTY: &'static str = "offset";
    const DEFAULT: Self = OffsetOption::Reject;
    const ALLOWED: &'static [Self] = &[
        OffsetOption::Reject,
        OffsetOption::Use,
        OffsetOption::Prefer,
        OffsetOption::Ignore,
    ];

    fn name(self) -> &'static str {
        match self {
            OffsetOption::Reject => "reject",
            OffsetOption::Use => "use",
            OffsetOption::Prefer => "prefer",
            OffsetOption::Ignore => "ignore",
        }
    }

    fn code(self) -> i64 {
        match self {
            OffsetOption::Reject => 0,
            OffsetOption::Use => 1,
            OffsetOption::Prefer => 2,
            OffsetOption::Ignore => 3,
        }
    }
}
