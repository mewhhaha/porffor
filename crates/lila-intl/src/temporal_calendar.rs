//! Primitive Temporal calendar requests answered by the pinned ICU calendar
//! kernel. The compiler sends calendar fields and date arithmetic here only
//! after JavaScript property reads and coercions have completed.

use core::fmt;

use crate::datetime::DateTimeCalendar;

/// The complete calendar set accepted by Temporal in this runtime.
///
/// Wire values are stable and intentionally independent of ICU's internal
/// calendar-kind discriminants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TemporalCalendar {
    Gregorian = 0,
    Iso8601 = 1,
    Chinese = 2,
    Buddhist = 3,
    Indian = 4,
    Persian = 5,
    Roc = 6,
    Dangi = 7,
    IslamicCivil = 8,
    Coptic = 9,
    Ethioaa = 10,
    Ethiopic = 11,
    Hebrew = 12,
    IslamicTabular = 13,
    IslamicUmmAlQura = 14,
    Japanese = 15,
}

impl TemporalCalendar {
    pub const ALL: [Self; 16] = [
        Self::Gregorian,
        Self::Iso8601,
        Self::Chinese,
        Self::Buddhist,
        Self::Indian,
        Self::Persian,
        Self::Roc,
        Self::Dangi,
        Self::IslamicCivil,
        Self::Coptic,
        Self::Ethioaa,
        Self::Ethiopic,
        Self::Hebrew,
        Self::IslamicTabular,
        Self::IslamicUmmAlQura,
        Self::Japanese,
    ];

    pub const fn wire(self) -> i64 {
        self as i64
    }

    pub const fn from_wire(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Gregorian),
            1 => Some(Self::Iso8601),
            2 => Some(Self::Chinese),
            3 => Some(Self::Buddhist),
            4 => Some(Self::Indian),
            5 => Some(Self::Persian),
            6 => Some(Self::Roc),
            7 => Some(Self::Dangi),
            8 => Some(Self::IslamicCivil),
            9 => Some(Self::Coptic),
            10 => Some(Self::Ethioaa),
            11 => Some(Self::Ethiopic),
            12 => Some(Self::Hebrew),
            13 => Some(Self::IslamicTabular),
            14 => Some(Self::IslamicUmmAlQura),
            15 => Some(Self::Japanese),
            _ => None,
        }
    }

    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Gregorian => "gregory",
            Self::Iso8601 => "iso8601",
            Self::Chinese => "chinese",
            Self::Buddhist => "buddhist",
            Self::Indian => "indian",
            Self::Persian => "persian",
            Self::Roc => "roc",
            Self::Dangi => "dangi",
            Self::IslamicCivil => "islamic-civil",
            Self::Coptic => "coptic",
            Self::Ethioaa => "ethioaa",
            Self::Ethiopic => "ethiopic",
            Self::Hebrew => "hebrew",
            Self::IslamicTabular => "islamic-tbla",
            Self::IslamicUmmAlQura => "islamic-umalqura",
            Self::Japanese => "japanese",
        }
    }

    pub fn parse(identifier: &str) -> Option<Self> {
        DateTimeCalendar::parse(identifier).map(|calendar| match calendar {
            DateTimeCalendar::Gregorian => Self::Gregorian,
            DateTimeCalendar::Iso8601 => Self::Iso8601,
            DateTimeCalendar::Chinese => Self::Chinese,
            DateTimeCalendar::Buddhist => Self::Buddhist,
            DateTimeCalendar::Indian => Self::Indian,
            DateTimeCalendar::Persian => Self::Persian,
            DateTimeCalendar::Roc => Self::Roc,
            DateTimeCalendar::Dangi => Self::Dangi,
            DateTimeCalendar::IslamicCivil => Self::IslamicCivil,
            DateTimeCalendar::Coptic => Self::Coptic,
            DateTimeCalendar::Ethioaa => Self::Ethioaa,
            DateTimeCalendar::Ethiopic => Self::Ethiopic,
            DateTimeCalendar::Hebrew => Self::Hebrew,
            DateTimeCalendar::IslamicTabular => Self::IslamicTabular,
            DateTimeCalendar::IslamicUmmAlQura => Self::IslamicUmmAlQura,
            DateTimeCalendar::Japanese => Self::Japanese,
        })
    }
}

/// A Temporal era code. The calendar remains part of the request, so repeated
/// strings such as `am`, `ce`, and `bce` cannot accidentally select another
/// calendar's era.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TemporalCalendarEra {
    Ce = 1,
    Bce = 2,
    Buddhist = 3,
    Shaka = 4,
    Persian = 5,
    Roc = 6,
    Broc = 7,
    Ah = 8,
    Bh = 9,
    Coptic = 10,
    Ethioaa = 11,
    EthiopicAm = 12,
    Hebrew = 13,
    Meiji = 14,
    Taisho = 15,
    Showa = 16,
    Heisei = 17,
    Reiwa = 18,
    JapaneseCe = 19,
    JapaneseBce = 20,
}

impl TemporalCalendarEra {
    pub const ALL: [Self; 20] = [
        Self::Ce,
        Self::Bce,
        Self::Buddhist,
        Self::Shaka,
        Self::Persian,
        Self::Roc,
        Self::Broc,
        Self::Ah,
        Self::Bh,
        Self::Coptic,
        Self::Ethioaa,
        Self::EthiopicAm,
        Self::Hebrew,
        Self::Meiji,
        Self::Taisho,
        Self::Showa,
        Self::Heisei,
        Self::Reiwa,
        Self::JapaneseCe,
        Self::JapaneseBce,
    ];

    pub const fn wire(self) -> i64 {
        self as i64
    }

    pub const fn from_wire(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::Ce),
            2 => Some(Self::Bce),
            3 => Some(Self::Buddhist),
            4 => Some(Self::Shaka),
            5 => Some(Self::Persian),
            6 => Some(Self::Roc),
            7 => Some(Self::Broc),
            8 => Some(Self::Ah),
            9 => Some(Self::Bh),
            10 => Some(Self::Coptic),
            11 => Some(Self::Ethioaa),
            12 => Some(Self::EthiopicAm),
            13 => Some(Self::Hebrew),
            14 => Some(Self::Meiji),
            15 => Some(Self::Taisho),
            16 => Some(Self::Showa),
            17 => Some(Self::Heisei),
            18 => Some(Self::Reiwa),
            19 => Some(Self::JapaneseCe),
            20 => Some(Self::JapaneseBce),
            _ => None,
        }
    }

    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Ce | Self::JapaneseCe => "ce",
            Self::Bce | Self::JapaneseBce => "bce",
            Self::Buddhist => "be",
            Self::Shaka => "shaka",
            Self::Persian => "ap",
            Self::Roc => "roc",
            Self::Broc => "broc",
            Self::Ah => "ah",
            Self::Bh => "bh",
            Self::Coptic => "am",
            Self::Ethioaa => "aa",
            Self::EthiopicAm => "am",
            Self::Hebrew => "am",
            Self::Meiji => "meiji",
            Self::Taisho => "taisho",
            Self::Showa => "showa",
            Self::Heisei => "heisei",
            Self::Reiwa => "reiwa",
        }
    }

    pub const fn calendar(self) -> TemporalCalendar {
        match self {
            Self::Ce | Self::Bce => TemporalCalendar::Gregorian,
            Self::Buddhist => TemporalCalendar::Buddhist,
            Self::Shaka => TemporalCalendar::Indian,
            Self::Persian => TemporalCalendar::Persian,
            Self::Roc | Self::Broc => TemporalCalendar::Roc,
            Self::Ah | Self::Bh => TemporalCalendar::IslamicCivil,
            Self::Coptic => TemporalCalendar::Coptic,
            Self::Ethioaa => TemporalCalendar::Ethioaa,
            Self::EthiopicAm => TemporalCalendar::Ethiopic,
            Self::Hebrew => TemporalCalendar::Hebrew,
            Self::Meiji
            | Self::Taisho
            | Self::Showa
            | Self::Heisei
            | Self::Reiwa
            | Self::JapaneseCe
            | Self::JapaneseBce => TemporalCalendar::Japanese,
        }
    }

    pub const fn allowed_in(self, calendar: TemporalCalendar) -> bool {
        match self {
            Self::Ah | Self::Bh => matches!(
                calendar,
                TemporalCalendar::IslamicCivil
                    | TemporalCalendar::IslamicTabular
                    | TemporalCalendar::IslamicUmmAlQura
            ),
            Self::Ethioaa => matches!(
                calendar,
                TemporalCalendar::Ethioaa | TemporalCalendar::Ethiopic
            ),
            _ => self.calendar() as u8 == calendar as u8,
        }
    }

    pub fn from_icu(calendar: TemporalCalendar, code: &str) -> Option<Self> {
        let era = match (calendar, code) {
            (TemporalCalendar::Gregorian, "ce") => Self::Ce,
            (TemporalCalendar::Gregorian, "bce") => Self::Bce,
            (TemporalCalendar::Buddhist, "be") => Self::Buddhist,
            (TemporalCalendar::Indian, "shaka") => Self::Shaka,
            (TemporalCalendar::Persian, "ap") => Self::Persian,
            (TemporalCalendar::Roc, "roc") => Self::Roc,
            (TemporalCalendar::Roc, "broc") => Self::Broc,
            (
                TemporalCalendar::IslamicCivil
                | TemporalCalendar::IslamicTabular
                | TemporalCalendar::IslamicUmmAlQura,
                "ah",
            ) => Self::Ah,
            (
                TemporalCalendar::IslamicCivil
                | TemporalCalendar::IslamicTabular
                | TemporalCalendar::IslamicUmmAlQura,
                "bh",
            ) => Self::Bh,
            (TemporalCalendar::Coptic, "am") => Self::Coptic,
            (TemporalCalendar::Ethioaa, "aa") => Self::Ethioaa,
            (TemporalCalendar::Ethiopic, "aa") => Self::Ethioaa,
            (TemporalCalendar::Ethiopic, "am") => Self::EthiopicAm,
            (TemporalCalendar::Hebrew, "am") => Self::Hebrew,
            (TemporalCalendar::Japanese, "meiji") => Self::Meiji,
            (TemporalCalendar::Japanese, "taisho") => Self::Taisho,
            (TemporalCalendar::Japanese, "showa") => Self::Showa,
            (TemporalCalendar::Japanese, "heisei") => Self::Heisei,
            (TemporalCalendar::Japanese, "reiwa") => Self::Reiwa,
            (TemporalCalendar::Japanese, "ce") => Self::JapaneseCe,
            (TemporalCalendar::Japanese, "bce") => Self::JapaneseBce,
            _ => return None,
        };
        Some(era)
    }
}

/// A Temporal month code in its normalized numeric form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemporalMonthCode {
    month: u8,
    leap: bool,
}

impl TemporalMonthCode {
    pub fn new(month: u8, leap: bool) -> Result<Self, InvalidTemporalCalendarRequest> {
        if !(1..=13).contains(&month) {
            return Err(InvalidTemporalCalendarRequest(
                "month code number is outside M01..M13",
            ));
        }
        Ok(Self { month, leap })
    }

    pub const fn month(self) -> u8 {
        self.month
    }

    pub const fn is_leap(self) -> bool {
        self.leap
    }
}

/// `overflow` accepted by calendar field construction and date addition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemporalCalendarOverflow {
    Constrain = 0,
    Reject = 1,
}

impl TemporalCalendarOverflow {
    pub const fn wire(self) -> i64 {
        self as i64
    }

    pub const fn from_wire(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Constrain),
            1 => Some(Self::Reject),
            _ => None,
        }
    }
}

/// A date in the ISO calendar's proleptic numbering, as stored in Temporal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalIsoDate {
    year: i32,
    month: u8,
    day: u8,
}

impl TemporalIsoDate {
    pub const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub const fn year(self) -> i32 {
        self.year
    }

    pub const fn month(self) -> u8 {
        self.month
    }

    pub const fn day(self) -> u8 {
        self.day
    }
}

/// Fields already read and converted by `Temporal.PlainDate.from`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalCalendarDateFields {
    year: Option<i32>,
    era: Option<TemporalCalendarEra>,
    era_year: Option<i32>,
    month: Option<u8>,
    month_code: Option<TemporalMonthCode>,
    day: u8,
}

impl TemporalCalendarDateFields {
    pub const fn new(
        year: Option<i32>,
        era: Option<TemporalCalendarEra>,
        era_year: Option<i32>,
        month: Option<u8>,
        month_code: Option<TemporalMonthCode>,
        day: u8,
    ) -> Self {
        Self {
            year,
            era,
            era_year,
            month,
            month_code,
            day,
        }
    }

    pub const fn year(self) -> Option<i32> {
        self.year
    }
    pub const fn era(self) -> Option<TemporalCalendarEra> {
        self.era
    }
    pub const fn era_year(self) -> Option<i32> {
        self.era_year
    }
    pub const fn month(self) -> Option<u8> {
        self.month
    }
    pub const fn month_code(self) -> Option<TemporalMonthCode> {
        self.month_code
    }
    pub const fn day(self) -> u8 {
        self.day
    }
}

/// Integral calendar date duration after JavaScript number conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalCalendarDuration {
    years: i64,
    months: i64,
    weeks: i64,
    days: i64,
}

impl TemporalCalendarDuration {
    pub const fn new(years: i64, months: i64, weeks: i64, days: i64) -> Self {
        Self {
            years,
            months,
            weeks,
            days,
        }
    }

    pub const fn years(self) -> i64 {
        self.years
    }
    pub const fn months(self) -> i64 {
        self.months
    }
    pub const fn weeks(self) -> i64 {
        self.weeks
    }
    pub const fn days(self) -> i64 {
        self.days
    }
}

/// One closed calendar query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalCalendarQuery {
    Fields {
        date: TemporalIsoDate,
    },
    FromFields {
        fields: TemporalCalendarDateFields,
        overflow: TemporalCalendarOverflow,
    },
    DateAdd {
        date: TemporalIsoDate,
        duration: TemporalCalendarDuration,
        overflow: TemporalCalendarOverflow,
    },
}

/// One primitive calendar request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalCalendarRequest {
    calendar: TemporalCalendar,
    query: TemporalCalendarQuery,
}

impl TemporalCalendarRequest {
    pub const fn new(calendar: TemporalCalendar, query: TemporalCalendarQuery) -> Self {
        Self { calendar, query }
    }

    pub const fn calendar(self) -> TemporalCalendar {
        self.calendar
    }

    pub const fn query(self) -> TemporalCalendarQuery {
        self.query
    }

    pub fn encode(self) -> [u8; TEMPORAL_CALENDAR_REQUEST_BYTES] {
        let mut words = [0_i64; TEMPORAL_CALENDAR_REQUEST_WORDS];
        words[0] = self.query.kind().wire();
        words[1] = self.calendar.wire();
        match self.query {
            TemporalCalendarQuery::Fields { date } => {
                set_date_words(&mut words, 2, date);
            }
            TemporalCalendarQuery::FromFields { fields, overflow } => {
                words[0] = TemporalCalendarQueryKind::FromFields.wire();
                words[5] = fields.year.map_or(0, i64::from);
                words[6] = fields.month.map_or(0, i64::from);
                words[7] = fields.month_code.map_or(0, |code| {
                    i64::from(code.month()) | if code.is_leap() { 1 << 8 } else { 0 }
                });
                words[8] = i64::from(fields.day);
                words[9] = fields.era.map_or(0, TemporalCalendarEra::wire);
                words[10] = fields.era_year.map_or(0, i64::from);
                words[11] = overflow.wire();
                words[16] = FROM_FIELDS_DAY
                    | fields.year.map_or(0, |_| FROM_FIELDS_YEAR)
                    | fields.month.map_or(0, |_| FROM_FIELDS_MONTH)
                    | fields.month_code.map_or(0, |_| FROM_FIELDS_MONTH_CODE)
                    | fields.era.map_or(0, |_| FROM_FIELDS_ERA)
                    | fields.era_year.map_or(0, |_| FROM_FIELDS_ERA_YEAR);
            }
            TemporalCalendarQuery::DateAdd {
                date,
                duration,
                overflow,
            } => {
                words[0] = TemporalCalendarQueryKind::DateAdd.wire();
                set_date_words(&mut words, 2, date);
                words[11] = overflow.wire();
                words[12] = duration.years;
                words[13] = duration.months;
                words[14] = duration.weeks;
                words[15] = duration.days;
            }
        }
        let mut bytes = [0; TEMPORAL_CALENDAR_REQUEST_BYTES];
        for (index, word) in words.into_iter().enumerate() {
            bytes[index * 8..index * 8 + 8].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidTemporalCalendarRequest> {
        if bytes.len() != TEMPORAL_CALENDAR_REQUEST_BYTES {
            return Err(InvalidTemporalCalendarRequest(
                "calendar request has the wrong extent",
            ));
        }
        let mut words = [0_i64; TEMPORAL_CALENDAR_REQUEST_WORDS];
        for (index, word) in words.iter_mut().enumerate() {
            *word = i64::from_le_bytes(
                bytes[index * 8..index * 8 + 8]
                    .try_into()
                    .expect("calendar request word"),
            );
        }
        let kind = TemporalCalendarQueryKind::from_wire(words[0]).ok_or(
            InvalidTemporalCalendarRequest("unknown calendar query kind"),
        )?;
        let calendar = TemporalCalendar::from_wire(words[1])
            .ok_or(InvalidTemporalCalendarRequest("unknown calendar id"))?;
        let date = || decode_date_words(&words, 2);
        let query = match kind {
            TemporalCalendarQueryKind::Fields => {
                require_zero(&words[5..], "fields query has unused request words")?;
                TemporalCalendarQuery::Fields { date: date()? }
            }
            TemporalCalendarQueryKind::FromFields => {
                require_zero(&words[2..5], "from-fields query has unused date words")?;
                require_zero(
                    &words[12..16],
                    "from-fields query has unused duration words",
                )?;
                let overflow = TemporalCalendarOverflow::from_wire(words[11])
                    .ok_or(InvalidTemporalCalendarRequest("unknown overflow mode"))?;
                let flags = words[16];
                let has_era_year = flags & (FROM_FIELDS_ERA | FROM_FIELDS_ERA_YEAR)
                    == (FROM_FIELDS_ERA | FROM_FIELDS_ERA_YEAR);
                if flags & !FROM_FIELDS_ALL != 0
                    || flags & FROM_FIELDS_DAY == 0
                    || flags & FROM_FIELDS_YEAR == 0 && !has_era_year
                {
                    return Err(InvalidTemporalCalendarRequest(
                        "from-fields presence flags are invalid",
                    ));
                }
                if words[5] != 0 && flags & FROM_FIELDS_YEAR == 0
                    || words[6] != 0 && flags & FROM_FIELDS_MONTH == 0
                    || words[7] != 0 && flags & FROM_FIELDS_MONTH_CODE == 0
                    || words[9] != 0 && flags & FROM_FIELDS_ERA == 0
                    || words[10] != 0 && flags & FROM_FIELDS_ERA_YEAR == 0
                {
                    return Err(InvalidTemporalCalendarRequest(
                        "from-fields data and presence flags disagree",
                    ));
                }
                let month = (flags & FROM_FIELDS_MONTH != 0)
                    .then(|| checked_u8(words[6], "month is outside the wire range"))
                    .transpose()?;
                let month_code = if flags & FROM_FIELDS_MONTH_CODE != 0 {
                    let month_number = (words[7] & 0xff) as u8;
                    let leap = match words[7] >> 8 {
                        0 => false,
                        1 => true,
                        _ => {
                            return Err(InvalidTemporalCalendarRequest(
                                "month code flags are invalid",
                            ));
                        }
                    };
                    Some(TemporalMonthCode::new(month_number, leap)?)
                } else {
                    None
                };
                let era = (flags & FROM_FIELDS_ERA != 0)
                    .then(|| {
                        TemporalCalendarEra::from_wire(words[9])
                            .ok_or(InvalidTemporalCalendarRequest("unknown calendar era"))
                    })
                    .transpose()?;
                let era_year = (flags & FROM_FIELDS_ERA_YEAR != 0)
                    .then(|| checked_i32(words[10], "eraYear is outside the wire range"))
                    .transpose()?;
                TemporalCalendarQuery::FromFields {
                    fields: TemporalCalendarDateFields::new(
                        (flags & FROM_FIELDS_YEAR != 0)
                            .then(|| checked_i32(words[5], "year is outside the wire range"))
                            .transpose()?,
                        era,
                        era_year,
                        month,
                        month_code,
                        checked_u8(words[8], "day is outside the wire range")?,
                    ),
                    overflow,
                }
            }
            TemporalCalendarQueryKind::DateAdd => {
                require_zero(&words[5..11], "date-add query has unused field words")?;
                require_zero(&words[16..17], "date-add query has unused flags")?;
                let overflow = TemporalCalendarOverflow::from_wire(words[11])
                    .ok_or(InvalidTemporalCalendarRequest("unknown overflow mode"))?;
                TemporalCalendarQuery::DateAdd {
                    date: date()?,
                    duration: TemporalCalendarDuration::new(
                        words[12], words[13], words[14], words[15],
                    ),
                    overflow,
                }
            }
        };
        Ok(Self { calendar, query })
    }
}

/// Kind of calendar query, stable on the host-call wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemporalCalendarQueryKind {
    Fields = 1,
    FromFields = 2,
    DateAdd = 3,
}

impl TemporalCalendarQueryKind {
    pub const fn wire(self) -> i64 {
        self as i64
    }

    pub const fn from_wire(value: i64) -> Option<Self> {
        match value {
            1 => Some(Self::Fields),
            2 => Some(Self::FromFields),
            3 => Some(Self::DateAdd),
            _ => None,
        }
    }
}

impl TemporalCalendarQuery {
    const fn kind(self) -> TemporalCalendarQueryKind {
        match self {
            Self::Fields { .. } => TemporalCalendarQueryKind::Fields,
            Self::FromFields { .. } => TemporalCalendarQueryKind::FromFields,
            Self::DateAdd { .. } => TemporalCalendarQueryKind::DateAdd,
        }
    }
}

/// All fields projected from one exact ISO date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalCalendarDate {
    iso: TemporalIsoDate,
    year: i32,
    era_year: Option<i32>,
    era: Option<TemporalCalendarEra>,
    month: u8,
    month_code: TemporalMonthCode,
    day: u8,
    months_in_year: u8,
    days_in_month: u8,
    days_in_year: u16,
    day_of_year: u16,
    in_leap_year: bool,
}

impl TemporalCalendarDate {
    pub(crate) const fn new_projected(
        iso: TemporalIsoDate,
        year: i32,
        era_year: Option<i32>,
        era: Option<TemporalCalendarEra>,
        month: u8,
        month_code: TemporalMonthCode,
        day: u8,
        months_in_year: u8,
        days_in_month: u8,
        days_in_year: u16,
        day_of_year: u16,
        in_leap_year: bool,
    ) -> Self {
        Self {
            iso,
            year,
            era_year,
            era,
            month,
            month_code,
            day,
            months_in_year,
            days_in_month,
            days_in_year,
            day_of_year,
            in_leap_year,
        }
    }

    pub const fn iso(self) -> TemporalIsoDate {
        self.iso
    }
    pub const fn year(self) -> i32 {
        self.year
    }
    pub const fn era_year(self) -> Option<i32> {
        self.era_year
    }
    pub const fn era(self) -> Option<TemporalCalendarEra> {
        self.era
    }
    pub const fn month(self) -> u8 {
        self.month
    }
    pub const fn month_code(self) -> TemporalMonthCode {
        self.month_code
    }
    pub const fn day(self) -> u8 {
        self.day
    }
    pub const fn months_in_year(self) -> u8 {
        self.months_in_year
    }
    pub const fn days_in_month(self) -> u8 {
        self.days_in_month
    }
    pub const fn days_in_year(self) -> u16 {
        self.days_in_year
    }
    pub const fn day_of_year(self) -> u16 {
        self.day_of_year
    }
    pub const fn in_leap_year(self) -> bool {
        self.in_leap_year
    }

    pub fn encode(self) -> [u8; TEMPORAL_CALENDAR_RESPONSE_BYTES] {
        let mut words = [0_i64; TEMPORAL_CALENDAR_RESPONSE_WORDS];
        words[0] = TEMPORAL_CALENDAR_STATUS_DATE;
        words[1] = i64::from(self.year);
        words[2] = i64::from(self.era_year.unwrap_or(0));
        words[3] = i64::from(self.month);
        words[4] = i64::from(self.month_code.month());
        words[5] = i64::from(self.month_code.is_leap());
        words[6] = i64::from(self.day);
        words[7] = i64::from(self.months_in_year);
        words[8] = i64::from(self.days_in_month);
        words[9] = self.era.map_or(0, TemporalCalendarEra::wire);
        words[10] = i64::from(self.iso.year());
        words[11] = i64::from(self.iso.month());
        words[12] = i64::from(self.iso.day());
        words[13] = i64::from(self.days_in_year);
        words[14] = i64::from(self.day_of_year);
        words[15] = i64::from(self.in_leap_year);
        encode_words(words)
    }
}

/// Why a Temporal calendar operation throws a `RangeError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TemporalCalendarRangeError {
    InvalidDate = 1,
    Overflow = 2,
    OutOfRange = 3,
}

impl TemporalCalendarRangeError {
    pub const ALL: [Self; 3] = [Self::InvalidDate, Self::Overflow, Self::OutOfRange];

    pub const fn status(self) -> i64 {
        self as i64
    }

    pub const fn message(self) -> &'static str {
        match self {
            Self::InvalidDate => "Invalid Temporal calendar date",
            Self::Overflow => "Temporal calendar date overflow",
            Self::OutOfRange => "Temporal calendar date is outside the supported range",
        }
    }
}

/// The result of one calendar query, as a projected date or a user RangeError.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalCalendarAnswer {
    Date(TemporalCalendarDate),
    RangeError(TemporalCalendarRangeError),
}

impl TemporalCalendarAnswer {
    pub fn encode(self) -> [u8; TEMPORAL_CALENDAR_RESPONSE_BYTES] {
        match self {
            Self::Date(date) => date.encode(),
            Self::RangeError(error) => {
                let mut words = [0_i64; TEMPORAL_CALENDAR_RESPONSE_WORDS];
                words[0] = error.status();
                encode_words(words)
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidTemporalCalendarRequest> {
        if bytes.len() != TEMPORAL_CALENDAR_RESPONSE_BYTES {
            return Err(InvalidTemporalCalendarRequest(
                "calendar answer has the wrong extent",
            ));
        }
        let words = decode_words(bytes);
        if words[0] == TEMPORAL_CALENDAR_STATUS_DATE {
            let month = checked_u8(words[4], "answer month code is outside the wire range")?;
            let leap = match words[5] {
                0 => false,
                1 => true,
                _ => {
                    return Err(InvalidTemporalCalendarRequest(
                        "answer month code flag is invalid",
                    ));
                }
            };
            let era = if words[9] == 0 {
                None
            } else {
                Some(
                    TemporalCalendarEra::from_wire(words[9])
                        .ok_or(InvalidTemporalCalendarRequest("answer has an unknown era"))?,
                )
            };
            let date = Self::Date(TemporalCalendarDate {
                iso: TemporalIsoDate::new(
                    checked_i32(words[10], "answer ISO year is outside the wire range")?,
                    checked_u8(words[11], "answer ISO month is outside the wire range")?,
                    checked_u8(words[12], "answer ISO day is outside the wire range")?,
                ),
                year: checked_i32(words[1], "answer year is outside the wire range")?,
                era_year: era
                    .map(|_| checked_i32(words[2], "answer eraYear is outside the wire range"))
                    .transpose()?,
                era,
                month: checked_u8(words[3], "answer month is outside the wire range")?,
                month_code: TemporalMonthCode::new(month, leap)?,
                day: checked_u8(words[6], "answer day is outside the wire range")?,
                months_in_year: checked_u8(
                    words[7],
                    "answer monthsInYear is outside the wire range",
                )?,
                days_in_month: checked_u8(
                    words[8],
                    "answer daysInMonth is outside the wire range",
                )?,
                days_in_year: u16::try_from(words[13]).map_err(|_| {
                    InvalidTemporalCalendarRequest("answer daysInYear is outside the wire range")
                })?,
                day_of_year: u16::try_from(words[14]).map_err(|_| {
                    InvalidTemporalCalendarRequest("answer dayOfYear is outside the wire range")
                })?,
                in_leap_year: match words[15] {
                    0 => false,
                    1 => true,
                    _ => {
                        return Err(InvalidTemporalCalendarRequest(
                            "answer inLeapYear flag is invalid",
                        ));
                    }
                },
            });
            Ok(date)
        } else {
            let error = TemporalCalendarRangeError::ALL
                .into_iter()
                .find(|error| error.status() == words[0])
                .filter(|_| words[1..].iter().all(|word| *word == 0))
                .ok_or(InvalidTemporalCalendarRequest(
                    "calendar answer status is invalid",
                ))?;
            Ok(Self::RangeError(error))
        }
    }
}

pub const TEMPORAL_CALENDAR_REQUEST_WORDS: usize = 17;
pub const TEMPORAL_CALENDAR_REQUEST_BYTES: usize = TEMPORAL_CALENDAR_REQUEST_WORDS * 8;
pub const TEMPORAL_CALENDAR_REQUEST_KIND_OFFSET: u64 = 0;
pub const TEMPORAL_CALENDAR_REQUEST_CALENDAR_OFFSET: u64 = 8;
pub const TEMPORAL_CALENDAR_REQUEST_ISO_YEAR_OFFSET: u64 = 16;
pub const TEMPORAL_CALENDAR_REQUEST_ISO_MONTH_OFFSET: u64 = 24;
pub const TEMPORAL_CALENDAR_REQUEST_ISO_DAY_OFFSET: u64 = 32;
pub const TEMPORAL_CALENDAR_REQUEST_YEAR_OFFSET: u64 = 40;
pub const TEMPORAL_CALENDAR_REQUEST_MONTH_OFFSET: u64 = 48;
pub const TEMPORAL_CALENDAR_REQUEST_MONTH_CODE_OFFSET: u64 = 56;
pub const TEMPORAL_CALENDAR_REQUEST_DAY_OFFSET: u64 = 64;
pub const TEMPORAL_CALENDAR_REQUEST_ERA_OFFSET: u64 = 72;
pub const TEMPORAL_CALENDAR_REQUEST_ERA_YEAR_OFFSET: u64 = 80;
pub const TEMPORAL_CALENDAR_REQUEST_OVERFLOW_OFFSET: u64 = 88;
pub const TEMPORAL_CALENDAR_REQUEST_DURATION_YEARS_OFFSET: u64 = 96;
pub const TEMPORAL_CALENDAR_REQUEST_DURATION_MONTHS_OFFSET: u64 = 104;
pub const TEMPORAL_CALENDAR_REQUEST_DURATION_WEEKS_OFFSET: u64 = 112;
pub const TEMPORAL_CALENDAR_REQUEST_DURATION_DAYS_OFFSET: u64 = 120;
pub const TEMPORAL_CALENDAR_REQUEST_FLAGS_OFFSET: u64 = 128;
pub const TEMPORAL_CALENDAR_RESPONSE_WORDS: usize = 16;
pub const TEMPORAL_CALENDAR_RESPONSE_BYTES: usize = TEMPORAL_CALENDAR_RESPONSE_WORDS * 8;
pub const TEMPORAL_CALENDAR_RESPONSE_STATUS_OFFSET: u64 = 0;
pub const TEMPORAL_CALENDAR_RESPONSE_YEAR_OFFSET: u64 = 8;
pub const TEMPORAL_CALENDAR_RESPONSE_ERA_YEAR_OFFSET: u64 = 16;
pub const TEMPORAL_CALENDAR_RESPONSE_MONTH_OFFSET: u64 = 24;
pub const TEMPORAL_CALENDAR_RESPONSE_MONTH_CODE_OFFSET: u64 = 32;
pub const TEMPORAL_CALENDAR_RESPONSE_MONTH_CODE_LEAP_OFFSET: u64 = 40;
pub const TEMPORAL_CALENDAR_RESPONSE_DAY_OFFSET: u64 = 48;
pub const TEMPORAL_CALENDAR_RESPONSE_MONTHS_IN_YEAR_OFFSET: u64 = 56;
pub const TEMPORAL_CALENDAR_RESPONSE_DAYS_IN_MONTH_OFFSET: u64 = 64;
pub const TEMPORAL_CALENDAR_RESPONSE_ERA_OFFSET: u64 = 72;
pub const TEMPORAL_CALENDAR_RESPONSE_ISO_YEAR_OFFSET: u64 = 80;
pub const TEMPORAL_CALENDAR_RESPONSE_ISO_MONTH_OFFSET: u64 = 88;
pub const TEMPORAL_CALENDAR_RESPONSE_ISO_DAY_OFFSET: u64 = 96;
pub const TEMPORAL_CALENDAR_RESPONSE_DAYS_IN_YEAR_OFFSET: u64 = 104;
pub const TEMPORAL_CALENDAR_RESPONSE_DAY_OF_YEAR_OFFSET: u64 = 112;
pub const TEMPORAL_CALENDAR_RESPONSE_IN_LEAP_YEAR_OFFSET: u64 = 120;
pub const TEMPORAL_CALENDAR_STATUS_DATE: i64 = 0;
/// Temporal's ISO date limit in epoch days.
pub const TEMPORAL_CALENDAR_ISO_DAYS_LIMIT: i64 = 100_000_000;

pub const TEMPORAL_CALENDAR_FROM_FIELDS_YEAR: i64 = 1;
pub const TEMPORAL_CALENDAR_FROM_FIELDS_MONTH: i64 = 2;
pub const TEMPORAL_CALENDAR_FROM_FIELDS_MONTH_CODE: i64 = 4;
pub const TEMPORAL_CALENDAR_FROM_FIELDS_DAY: i64 = 8;
pub const TEMPORAL_CALENDAR_FROM_FIELDS_ERA: i64 = 16;
pub const TEMPORAL_CALENDAR_FROM_FIELDS_ERA_YEAR: i64 = 32;
const FROM_FIELDS_YEAR: i64 = TEMPORAL_CALENDAR_FROM_FIELDS_YEAR;
const FROM_FIELDS_MONTH: i64 = TEMPORAL_CALENDAR_FROM_FIELDS_MONTH;
const FROM_FIELDS_MONTH_CODE: i64 = TEMPORAL_CALENDAR_FROM_FIELDS_MONTH_CODE;
const FROM_FIELDS_DAY: i64 = TEMPORAL_CALENDAR_FROM_FIELDS_DAY;
const FROM_FIELDS_ERA: i64 = TEMPORAL_CALENDAR_FROM_FIELDS_ERA;
const FROM_FIELDS_ERA_YEAR: i64 = TEMPORAL_CALENDAR_FROM_FIELDS_ERA_YEAR;
const FROM_FIELDS_ALL: i64 = FROM_FIELDS_YEAR
    | FROM_FIELDS_MONTH
    | FROM_FIELDS_MONTH_CODE
    | FROM_FIELDS_DAY
    | FROM_FIELDS_ERA
    | FROM_FIELDS_ERA_YEAR;

/// Malformed host-call bytes, which indicate an ABI or compiler fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTemporalCalendarRequest(pub(crate) &'static str);

impl fmt::Display for InvalidTemporalCalendarRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for InvalidTemporalCalendarRequest {}

/// Provider failure for a decoded Temporal calendar request.
#[derive(Debug)]
pub enum TemporalCalendarError {
    InvalidRequest(InvalidTemporalCalendarRequest),
    InvalidCalendarData(&'static str),
}

impl fmt::Display for TemporalCalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(error) => write!(f, "invalid Temporal calendar request: {error}"),
            Self::InvalidCalendarData(reason) => {
                write!(f, "pinned Temporal calendar data is incompatible: {reason}")
            }
        }
    }
}

impl std::error::Error for TemporalCalendarError {}

impl From<InvalidTemporalCalendarRequest> for TemporalCalendarError {
    fn from(value: InvalidTemporalCalendarRequest) -> Self {
        Self::InvalidRequest(value)
    }
}

fn set_date_words(
    words: &mut [i64; TEMPORAL_CALENDAR_REQUEST_WORDS],
    start: usize,
    date: TemporalIsoDate,
) {
    words[start] = i64::from(date.year());
    words[start + 1] = i64::from(date.month());
    words[start + 2] = i64::from(date.day());
}

fn decode_date_words(
    words: &[i64; TEMPORAL_CALENDAR_REQUEST_WORDS],
    start: usize,
) -> Result<TemporalIsoDate, InvalidTemporalCalendarRequest> {
    Ok(TemporalIsoDate::new(
        checked_i32(words[start], "ISO year is outside the wire range")?,
        checked_u8(words[start + 1], "ISO month is outside the wire range")?,
        checked_u8(words[start + 2], "ISO day is outside the wire range")?,
    ))
}

fn checked_i32(value: i64, message: &'static str) -> Result<i32, InvalidTemporalCalendarRequest> {
    i32::try_from(value).map_err(|_| InvalidTemporalCalendarRequest(message))
}

fn checked_u8(value: i64, message: &'static str) -> Result<u8, InvalidTemporalCalendarRequest> {
    u8::try_from(value).map_err(|_| InvalidTemporalCalendarRequest(message))
}

fn require_zero(
    words: &[i64],
    message: &'static str,
) -> Result<(), InvalidTemporalCalendarRequest> {
    if words.iter().all(|word| *word == 0) {
        Ok(())
    } else {
        Err(InvalidTemporalCalendarRequest(message))
    }
}

fn encode_words(
    words: [i64; TEMPORAL_CALENDAR_RESPONSE_WORDS],
) -> [u8; TEMPORAL_CALENDAR_RESPONSE_BYTES] {
    let mut bytes = [0; TEMPORAL_CALENDAR_RESPONSE_BYTES];
    for (index, word) in words.into_iter().enumerate() {
        bytes[index * 8..index * 8 + 8].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn decode_words(bytes: &[u8]) -> [i64; TEMPORAL_CALENDAR_RESPONSE_WORDS] {
    let mut words = [0_i64; TEMPORAL_CALENDAR_RESPONSE_WORDS];
    for (index, word) in words.iter_mut().enumerate() {
        *word = i64::from_le_bytes(
            bytes[index * 8..index * 8 + 8]
                .try_into()
                .expect("answer word"),
        );
    }
    words
}
