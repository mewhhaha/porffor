//! Checked compiler-native Chinese/Dangi years and their immutable Wasm image.
//!
//! The pinned ICU public Date API supplies retained astronomy/cache records.
//! Only the catalog constructor can publish rows; emitted helpers never invoke
//! a provider or choose a different model after a failed calculation.

use super::{EmitError, StringPool, STATIC_DATA_OFFSET};
use icu_calendar::{
    cal::{Chinese, Dangi},
    types::CyclicYear,
    AsCalendar, Calendar, Date, DateDuration, Ref,
};
use std::{fmt, panic::catch_unwind, sync::OnceLock};

pub(crate) const TEMPORAL_EAST_ASIAN_YEAR_ENVELOPE: i64 = 1 << 34;
pub(crate) const EAST_ASIAN_MILLIS_PER_DAY: i64 = 86_400_000;
pub(crate) const EAST_ASIAN_MEAN_YEAR_MILLIS: i64 =
    (EAST_ASIAN_MILLIS_PER_DAY as i128 * 146_097 / 400) as i64;
pub(crate) const EAST_ASIAN_MEAN_SOLAR_TERM_MILLIS: i64 = EAST_ASIAN_MEAN_YEAR_MILLIS / 12;
pub(crate) const EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS: i64 =
    (EAST_ASIAN_MILLIS_PER_DAY as i128 * 295_305_888_531 / 10_000_000_000) as i64;

const FIRST_RELATED_YEAR: i32 = -3653;
const AFTER_LAST_RELATED_YEAR: i32 = 4704;
const RETAINED_YEAR_COUNT: usize = (AFTER_LAST_RELATED_YEAR - FIRST_RELATED_YEAR) as usize;
const FIRST_START_RD: i64 = -1_334_565;
const EXCLUSIVE_END_RD: i64 = 1_717_776;

/// The sole native kind also consumed by calendar admission and the year model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalEastAsianCalendar {
    Chinese,
    Dangi,
}

impl TemporalEastAsianCalendar {
    const fn lower_observer_offset_millis(self) -> i64 {
        match self {
            Self::Chinese => 1397 * 3_600_000 / 180,
            Self::Dangi => 3809 * 3_600_000 / 450,
        }
    }

    const fn upper_observer_offset_millis(self) -> i64 {
        match self {
            Self::Chinese => 8 * 3_600_000,
            Self::Dangi => 9 * 3_600_000,
        }
    }
}

pub(crate) const EAST_ASIAN_YEAR_ROW_BYTES: u64 = 8;
pub(crate) const EAST_ASIAN_MONTH_MASK: u32 = (1 << 13) - 1;
pub(crate) const EAST_ASIAN_LEAP_ORDINAL_SHIFT: u32 = 13;
pub(crate) const EAST_ASIAN_NEW_YEAR_OFFSET_SHIFT: u32 = 17;

/// Both words use a four-byte load; MonthPrefix is extended as signed i32.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TemporalEastAsianYearRowSlot {
    Packed,
    MonthPrefix,
}

impl TemporalEastAsianYearRowSlot {
    pub(crate) const fn byte_offset(self) -> u64 {
        match self {
            Self::Packed => 0,
            Self::MonthPrefix => 4,
        }
    }
}

#[derive(Debug)]
enum CatalogError {
    ProviderInvariant,
    Date {
        calendar: TemporalEastAsianCalendar,
        year: i32,
        message: String,
    },
    IsoDate(String),
    InvalidYear {
        calendar: TemporalEastAsianCalendar,
        year: i32,
        reason: &'static str,
    },
    ArithmeticRange,
    ImageRange,
    Allocation,
}

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderInvariant => formatter.write_str("pinned provider year invariant failed"),
            Self::Date {
                calendar,
                year,
                message,
            } => {
                write!(formatter, "{calendar:?} related year {year}: {message}")
            }
            Self::IsoDate(message) => write!(formatter, "ISO catalog anchor: {message}"),
            Self::InvalidYear {
                calendar,
                year,
                reason,
            } => {
                write!(formatter, "{calendar:?} related year {year}: {reason}")
            }
            Self::ArithmeticRange => formatter.write_str("catalog scalar arithmetic overflow"),
            Self::ImageRange => formatter.write_str("catalog image exceeds Wasm memory32"),
            Self::Allocation => formatter.write_str("catalog allocation failed"),
        }
    }
}

fn invalid_year(
    calendar: TemporalEastAsianCalendar,
    year: i32,
    reason: &'static str,
) -> CatalogError {
    CatalogError::InvalidYear {
        calendar,
        year,
        reason,
    }
}

fn iso_rd(year: i32, month: u8, day: u8) -> Result<i64, CatalogError> {
    Date::try_new_iso(year, month, day)
        .map(|date| date.to_rata_die().to_i64_date())
        .map_err(|error| CatalogError::IsoDate(error.to_string()))
}

/// No raw provider bytes or unchecked alternate row constructor are exposed.
struct ValidatedYear {
    start_rd: i64,
    end_rd: i64,
    mask: u32,
    leap_ordinal: u8,
    new_year_offset: u8,
    month_count: u8,
}

impl ValidatedYear {
    fn from_date<A>(
        mut date: Date<A>,
        calendar: TemporalEastAsianCalendar,
        year: i32,
    ) -> Result<(Self, Date<A>), CatalogError>
    where
        A: AsCalendar + Copy,
        A::Calendar: Calendar<Year = CyclicYear>,
    {
        if date.cyclic_year().related_iso != year
            || date.month().ordinal != 1
            || date.day_of_month().0 != 1
        {
            return Err(invalid_year(
                calendar,
                year,
                "year does not start at its first month/day",
            ));
        }
        let start_rd = date.to_rata_die().to_i64_date();
        let month_count = date.months_in_year();
        if !matches!(month_count, 12 | 13) {
            return Err(invalid_year(calendar, year, "month count is not 12 or 13"));
        }
        let offset = start_rd - iso_rd(year, 1, 1)?;
        if !(18..=52).contains(&offset) {
            return Err(invalid_year(
                calendar,
                year,
                "New Year offset is outside 18..52",
            ));
        }
        let declared_days = date.days_in_year();
        let mut next_start = start_rd;
        let mut mask = 0u32;
        let mut leap_ordinal = 0u8;
        let mut next_regular = 1u8;
        for ordinal in 1..=month_count {
            let month = date.month();
            if date.cyclic_year().related_iso != year
                || month.ordinal != ordinal
                || date.months_in_year() != month_count
                || date.days_in_year() != declared_days
                || date.day_of_month().0 != 1
                || date.to_rata_die().to_i64_date() != next_start
            {
                return Err(invalid_year(
                    calendar,
                    year,
                    "month advancement broke the year interval",
                ));
            }
            let (number, is_leap) = month.standard_code.parsed().ok_or_else(|| {
                invalid_year(
                    calendar,
                    year,
                    "provider emitted an invalid canonical MonthCode",
                )
            })?;
            let spelling = format!("M{number:02}{}", if is_leap { "L" } else { "" });
            if month.standard_code.to_string() != spelling || !(1..=12).contains(&number) {
                return Err(invalid_year(
                    calendar,
                    year,
                    "provider emitted a noncanonical MonthCode",
                ));
            }
            if is_leap {
                if leap_ordinal != 0 || ordinal == 1 || number != next_regular - 1 {
                    return Err(invalid_year(
                        calendar,
                        year,
                        "leap code does not follow its regular month",
                    ));
                }
                leap_ordinal = ordinal;
            } else {
                if number != next_regular {
                    return Err(invalid_year(
                        calendar,
                        year,
                        "regular MonthCodes are not ordered 1..12",
                    ));
                }
                next_regular += 1;
            }
            let days = date.days_in_month();
            if !matches!(days, 29 | 30) {
                return Err(invalid_year(calendar, year, "month length is not 29 or 30"));
            }
            if days == 30 {
                mask |= 1 << u32::from(ordinal - 1);
            }
            let last = date.added(DateDuration::new(0, 0, 0, i32::from(days) - 1));
            if last.cyclic_year().related_iso != year
                || last.month().ordinal != ordinal
                || last.month().standard_code != month.standard_code
                || last.day_of_month().0 != days
                || last.to_rata_die().to_i64_date() != next_start + i64::from(days) - 1
                || last.to_iso().to_rata_die() != last.to_rata_die()
            {
                return Err(invalid_year(
                    calendar,
                    year,
                    "month endpoint does not match its declared length",
                ));
            }
            next_start += i64::from(days);
            date.add(DateDuration::new(0, 1, 0, 0));
        }
        if next_regular != 13 || month_count != 12 + u8::from(leap_ordinal != 0) {
            return Err(invalid_year(
                calendar,
                year,
                "year does not contain twelve regular months and its optional leap",
            ));
        }
        if date.cyclic_year().related_iso != year + 1
            || date.month().ordinal != 1
            || date.day_of_month().0 != 1
            || date.to_rata_die().to_i64_date() != next_start
            || next_start - start_rd != i64::from(declared_days)
        {
            return Err(invalid_year(
                calendar,
                year,
                "month intervals do not meet the following New Year",
            ));
        }
        let july = iso_rd(year, 7, 1)?;
        if !(start_rd <= july && july < next_start) {
            return Err(invalid_year(
                calendar,
                year,
                "related year does not contain ISO July 1",
            ));
        }
        Ok((
            Self {
                start_rd,
                end_rd: next_start,
                mask,
                leap_ordinal,
                new_year_offset: offset as u8,
                month_count,
            },
            date,
        ))
    }
}

struct RetainedYear {
    packed: u32,
    month_prefix: i32,
}

impl RetainedYear {
    fn new(year: &ValidatedYear, month_prefix: i64) -> Result<Self, CatalogError> {
        let month_prefix =
            i32::try_from(month_prefix).map_err(|_| CatalogError::ArithmeticRange)?;
        Ok(Self {
            packed: year.mask
                | (u32::from(year.leap_ordinal) << EAST_ASIAN_LEAP_ORDINAL_SHIFT)
                | (u32::from(year.new_year_offset) << EAST_ASIAN_NEW_YEAR_OFFSET_SHIFT),
            month_prefix,
        })
    }

    fn to_le_bytes(&self) -> [u8; EAST_ASIAN_YEAR_ROW_BYTES as usize] {
        let mut bytes = [0; EAST_ASIAN_YEAR_ROW_BYTES as usize];
        for slot in [
            TemporalEastAsianYearRowSlot::Packed,
            TemporalEastAsianYearRowSlot::MonthPrefix,
        ] {
            let word = match slot {
                TemporalEastAsianYearRowSlot::Packed => self.packed.to_le_bytes(),
                TemporalEastAsianYearRowSlot::MonthPrefix => self.month_prefix.to_le_bytes(),
            };
            let offset = slot.byte_offset() as usize;
            bytes[offset..offset + word.len()].copy_from_slice(&word);
        }
        bytes
    }
}

#[derive(Clone, Copy, Debug)]
struct ModelAnchors {
    lower_reference_moon_millis: i64,
    upper_reference_moon_millis: i64,
    lower_solstice_millis: i64,
    upper_solstice_millis: i64,
}

impl ModelAnchors {
    fn new(calendar: TemporalEastAsianCalendar, unix_epoch_rd: i64) -> Result<Self, CatalogError> {
        let anchor = |year, month, day, time, observer| -> Result<i64, CatalogError> {
            let day = iso_rd(year, month, day)? - unix_epoch_rd;
            i64::try_from(
                i128::from(day) * i128::from(EAST_ASIAN_MILLIS_PER_DAY)
                    + i128::from(time)
                    + i128::from(observer),
            )
            .map_err(|_| CatalogError::ArithmeticRange)
        };
        let lower = calendar.lower_observer_offset_millis();
        let upper = calendar.upper_observer_offset_millis();
        let moon_time = ((18 * 60 + 14) * 60 * 1000) as i64;
        let solstice_time = ((7 * 60 + 44) * 60 * 1000) as i64;
        Ok(Self {
            lower_reference_moon_millis: anchor(2000, 1, 6, moon_time, lower)?,
            upper_reference_moon_millis: anchor(2000, 1, 6, moon_time, upper)?,
            lower_solstice_millis: anchor(1999, 12, 22, solstice_time, lower)?,
            upper_solstice_millis: anchor(1999, 12, 22, solstice_time, upper)?,
        })
    }

    fn moon_index_on_or_before(day: i64, base: i64) -> Result<i64, CatalogError> {
        let inclusive_end = (i128::from(day) + 1) * i128::from(EAST_ASIAN_MILLIS_PER_DAY) - 1;
        i64::try_from(
            (inclusive_end - i128::from(base))
                .div_euclid(i128::from(EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS)),
        )
        .map_err(|_| CatalogError::ArithmeticRange)
    }

    fn validate_model_year(
        calendar: TemporalEastAsianCalendar,
        year: i32,
        info: &ValidatedYear,
        unix_epoch_rd: i64,
        base: i64,
    ) -> Result<(), CatalogError> {
        let start = info
            .start_rd
            .checked_sub(unix_epoch_rd)
            .ok_or(CatalogError::ArithmeticRange)?;
        let end = info
            .end_rd
            .checked_sub(unix_epoch_rd)
            .ok_or(CatalogError::ArithmeticRange)?;
        let first = Self::moon_index_on_or_before(start, base)?;
        let after = Self::moon_index_on_or_before(end, base)?;
        let moon_day = |index: i64| {
            (i128::from(base) + i128::from(index) * i128::from(EAST_ASIAN_MEAN_LUNAR_MONTH_MILLIS))
                .div_euclid(i128::from(EAST_ASIAN_MILLIS_PER_DAY))
        };
        if moon_day(first) != i128::from(start)
            || moon_day(after) != i128::from(end)
            || after.checked_sub(first) != Some(i64::from(info.month_count))
        {
            return Err(invalid_year(
                calendar,
                year,
                "model month serials do not join its public Date year",
            ));
        }
        Ok(())
    }
}

struct CalendarCatalog {
    years: Box<[RetainedYear]>,
    total_months: i64,
    lower_serial_offset: i64,
    upper_serial_offset: i64,
    anchors: ModelAnchors,
}

impl CalendarCatalog {
    fn from_date<A>(
        first: Date<A>,
        calendar: TemporalEastAsianCalendar,
        unix_epoch_rd: i64,
    ) -> Result<Self, CatalogError>
    where
        A: AsCalendar + Copy,
        A::Calendar: Calendar<Year = CyclicYear>,
    {
        // Validate both complete model years, not merely their boundary dates.
        let (preceding, mut date) =
            ValidatedYear::from_date(first, calendar, FIRST_RELATED_YEAR - 1)?;
        if preceding.end_rd != FIRST_START_RD {
            return Err(invalid_year(
                calendar,
                FIRST_RELATED_YEAR - 1,
                "lower model boundary does not join retained years",
            ));
        }
        let mut years = Vec::new();
        years
            .try_reserve_exact(RETAINED_YEAR_COUNT)
            .map_err(|_| CatalogError::Allocation)?;
        let mut previous_end = preceding.end_rd;
        let mut month_prefix = 0i64;
        for year in FIRST_RELATED_YEAR..AFTER_LAST_RELATED_YEAR {
            let (info, next) = ValidatedYear::from_date(date, calendar, year)?;
            if info.start_rd != previous_end {
                return Err(invalid_year(
                    calendar,
                    year,
                    "retained year starts are not contiguous",
                ));
            }
            years.push(RetainedYear::new(&info, month_prefix)?);
            month_prefix = month_prefix
                .checked_add(i64::from(info.month_count))
                .ok_or(CatalogError::ArithmeticRange)?;
            previous_end = info.end_rd;
            date = next;
        }
        let (following, _) = ValidatedYear::from_date(date, calendar, AFTER_LAST_RELATED_YEAR)?;
        if previous_end != EXCLUSIVE_END_RD || following.start_rd != previous_end {
            return Err(invalid_year(
                calendar,
                AFTER_LAST_RELATED_YEAR,
                "upper model boundary does not join retained years",
            ));
        }
        let first_start_day = FIRST_START_RD
            .checked_sub(unix_epoch_rd)
            .ok_or(CatalogError::ArithmeticRange)?;
        let exclusive_end_day = EXCLUSIVE_END_RD
            .checked_sub(unix_epoch_rd)
            .ok_or(CatalogError::ArithmeticRange)?;
        let anchors = ModelAnchors::new(calendar, unix_epoch_rd)?;
        ModelAnchors::validate_model_year(
            calendar,
            FIRST_RELATED_YEAR - 1,
            &preceding,
            unix_epoch_rd,
            anchors.lower_reference_moon_millis,
        )?;
        ModelAnchors::validate_model_year(
            calendar,
            AFTER_LAST_RELATED_YEAR,
            &following,
            unix_epoch_rd,
            anchors.upper_reference_moon_millis,
        )?;
        let lower_index = ModelAnchors::moon_index_on_or_before(
            first_start_day,
            anchors.lower_reference_moon_millis,
        )?;
        let upper_index = ModelAnchors::moon_index_on_or_before(
            exclusive_end_day,
            anchors.upper_reference_moon_millis,
        )?;
        Ok(Self {
            years: years.into_boxed_slice(),
            total_months: month_prefix,
            lower_serial_offset: lower_index
                .checked_neg()
                .ok_or(CatalogError::ArithmeticRange)?,
            upper_serial_offset: month_prefix
                .checked_sub(upper_index)
                .ok_or(CatalogError::ArithmeticRange)?,
            anchors,
        })
    }
}

struct Catalog {
    chinese: CalendarCatalog,
    dangi: CalendarCatalog,
}

impl Catalog {
    fn build() -> Result<Self, CatalogError> {
        let unix_epoch_rd = iso_rd(1970, 1, 1)?;
        let chinese = Chinese::new();
        let dangi = Dangi::new();
        let first_year = FIRST_RELATED_YEAR - 1;
        let chinese_first = Date::try_new_chinese_with_calendar(first_year, 1, 1, Ref(&chinese))
            .map_err(|error| CatalogError::Date {
                calendar: TemporalEastAsianCalendar::Chinese,
                year: first_year,
                message: error.to_string(),
            })?;
        let dangi_first = Date::try_new_dangi_with_calendar(first_year, 1, 1, Ref(&dangi))
            .map_err(|error| CatalogError::Date {
                calendar: TemporalEastAsianCalendar::Dangi,
                year: first_year,
                message: error.to_string(),
            })?;
        Ok(Self {
            chinese: CalendarCatalog::from_date(
                chinese_first,
                TemporalEastAsianCalendar::Chinese,
                unix_epoch_rd,
            )?,
            dangi: CalendarCatalog::from_date(
                dangi_first,
                TemporalEastAsianCalendar::Dangi,
                unix_epoch_rd,
            )?,
        })
    }
}

static CATALOG: OnceLock<Result<Catalog, CatalogError>> = OnceLock::new();

/// Upstream's infallible Calendar trait panics for an invalid checked record.
/// Capture that failure once at native construction; never select a fallback.
fn catalog() -> Result<&'static Catalog, &'static CatalogError> {
    CATALOG
        .get_or_init(|| match catch_unwind(Catalog::build) {
            Ok(result) => result,
            Err(_) => Err(CatalogError::ProviderInvariant),
        })
        .as_ref()
}

/// Checked addresses and native header values of one retained-year image.
#[derive(Debug)]
pub(crate) struct TemporalEastAsianCalendarImage {
    rows_ptr: u32,
    total_months: i64,
    lower_serial_offset: i64,
    upper_serial_offset: i64,
    anchors: ModelAnchors,
}

impl TemporalEastAsianCalendarImage {
    pub(crate) const fn rows_ptr(&self) -> u32 {
        self.rows_ptr
    }
    pub(crate) const fn first_related_year(&self) -> i64 {
        FIRST_RELATED_YEAR as i64
    }
    pub(crate) const fn year_count(&self) -> u32 {
        RETAINED_YEAR_COUNT as u32
    }
    pub(crate) const fn total_months(&self) -> i64 {
        self.total_months
    }
    pub(crate) const fn lower_serial_offset(&self) -> i64 {
        self.lower_serial_offset
    }
    pub(crate) const fn upper_serial_offset(&self) -> i64 {
        self.upper_serial_offset
    }
    pub(crate) const fn lower_reference_moon_millis(&self) -> i64 {
        self.anchors.lower_reference_moon_millis
    }
    pub(crate) const fn upper_reference_moon_millis(&self) -> i64 {
        self.anchors.upper_reference_moon_millis
    }
    pub(crate) const fn lower_solstice_millis(&self) -> i64 {
        self.anchors.lower_solstice_millis
    }
    pub(crate) const fn upper_solstice_millis(&self) -> i64 {
        self.anchors.upper_solstice_millis
    }

    fn append(bytes: &mut Vec<u8>, catalog: &CalendarCatalog) -> Result<Self, CatalogError> {
        let rows_ptr = STATIC_DATA_OFFSET
            .checked_add(u32::try_from(bytes.len()).map_err(|_| CatalogError::ImageRange)?)
            .ok_or(CatalogError::ImageRange)?;
        for row in catalog.years.iter() {
            bytes.extend_from_slice(&row.to_le_bytes());
        }
        Ok(Self {
            rows_ptr,
            total_months: catalog.total_months,
            lower_serial_offset: catalog.lower_serial_offset,
            upper_serial_offset: catalog.upper_serial_offset,
            anchors: catalog.anchors,
        })
    }
}

/// Named images make omission of either kind visible in exhaustive dispatch.
#[derive(Debug)]
pub(crate) struct TemporalEastAsianYearImage {
    chinese: TemporalEastAsianCalendarImage,
    dangi: TemporalEastAsianCalendarImage,
}

impl TemporalEastAsianYearImage {
    pub(crate) fn calendar(
        &self,
        calendar: TemporalEastAsianCalendar,
    ) -> &TemporalEastAsianCalendarImage {
        match calendar {
            TemporalEastAsianCalendar::Chinese => &self.chinese,
            TemporalEastAsianCalendar::Dangi => &self.dangi,
        }
    }

    fn append_to(bytes: &mut Vec<u8>, catalog: &Catalog) -> Result<Self, CatalogError> {
        let alignment = EAST_ASIAN_YEAR_ROW_BYTES as usize;
        let padding = (alignment - bytes.len() % alignment) % alignment;
        let aligned = bytes
            .len()
            .checked_add(padding)
            .ok_or(CatalogError::ImageRange)?;
        let row_bytes = RETAINED_YEAR_COUNT
            .checked_mul(EAST_ASIAN_YEAR_ROW_BYTES as usize)
            .and_then(|one| one.checked_mul(2))
            .ok_or(CatalogError::ImageRange)?;
        let end = aligned
            .checked_add(row_bytes)
            .ok_or(CatalogError::ImageRange)?;
        STATIC_DATA_OFFSET
            .checked_add(u32::try_from(end).map_err(|_| CatalogError::ImageRange)?)
            .ok_or(CatalogError::ImageRange)?;
        bytes
            .try_reserve_exact(end - bytes.len())
            .map_err(|_| CatalogError::Allocation)?;
        bytes.resize(aligned, 0);
        Ok(Self {
            chinese: TemporalEastAsianCalendarImage::append(bytes, &catalog.chinese)?,
            dangi: TemporalEastAsianCalendarImage::append(bytes, &catalog.dangi)?,
        })
    }
}

impl StringPool {
    pub(super) fn append_temporal_east_asian_year_image(&mut self) -> Result<(), EmitError> {
        let catalog = catalog().map_err(|error| {
            EmitError::unsupported(format!("invalid Temporal Chinese/Dangi catalog: {error}"))
        })?;
        self.temporal_east_asian_year_image = Some(
            TemporalEastAsianYearImage::append_to(&mut self.bytes, catalog).map_err(|error| {
                EmitError::unsupported(format!("invalid Temporal Chinese/Dangi image: {error}"))
            })?,
        );
        Ok(())
    }

    pub(crate) fn temporal_east_asian_year_image(&self) -> &TemporalEastAsianYearImage {
        self.temporal_east_asian_year_image
            .as_ref()
            .expect("Temporal calendar compiler requires its checked year image")
    }
}
