//! This module implements the calendar traits and related components.
//!
//! The goal of the calendar module of `boa_temporal` is to provide
//! Temporal compatible calendar implementations.

use crate::{
    builtins::core::{
        duration::DateDuration, Duration, PlainDate, PlainDateTime, PlainMonthDay, PlainYearMonth,
    },
    error::ErrorKind,
    iso::IsoDate,
    options::{Overflow, Unit},
    parsers::parse_allowed_calendar_formats,
    TemporalError, TemporalResult,
};
use alloc::vec::Vec;
use core::str::FromStr;

use icu_calendar::{
    cal::{
        Buddhist, Chinese, Coptic, Dangi, Ethiopian, EthiopianEraStyle, Hebrew, HijriSimulated,
        HijriTabular, HijriUmmAlQura, Indian, Japanese, JapaneseExtended, Persian, Roc,
    },
    AnyCalendar, AnyCalendarKind, Calendar as IcuCalendar, Date as IcuDate,
    DateDuration as IcuDateDuration, Iso, Ref,
};
use icu_calendar::{
    cal::{HijriTabularEpoch, HijriTabularLeapYears},
    preferences::CalendarAlgorithm,
    types::MonthCode as IcuMonthCode,
    Gregorian,
};
use icu_locale::extensions::unicode::Value;
use tinystr::{tinystr, TinyAsciiStr};

use super::ZonedDateTime;

mod era;
mod fields;
mod types;

pub use fields::{CalendarFields, YearMonthCalendarFields};
#[cfg(test)]
pub(crate) use types::month_to_month_code;
pub(crate) use types::ResolutionType;
pub use types::{MonthCode, ResolvedCalendarFields};

use era::EraInfo;

/// The core `Calendar` type for `temporal_rs`
///
/// A `Calendar` in `temporal_rs` can be any calendar that is currently
/// supported by [`icu_calendar`].
#[derive(Debug, Clone)]
pub struct Calendar(Ref<'static, AnyCalendar>);

impl Default for Calendar {
    fn default() -> Self {
        Self::ISO
    }
}

impl PartialEq for Calendar {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}

impl Eq for Calendar {}

impl Calendar {
    /// The Buddhist calendar
    pub const BUDDHIST: Self = Self::new(AnyCalendarKind::Buddhist);
    /// The Chinese calendar
    pub const CHINESE: Self = Self::new(AnyCalendarKind::Chinese);
    /// The Coptic calendar
    pub const COPTIC: Self = Self::new(AnyCalendarKind::Coptic);
    /// The Dangi calendar
    pub const DANGI: Self = Self::new(AnyCalendarKind::Dangi);
    /// The Ethiopian calendar
    pub const ETHIOPIAN: Self = Self::new(AnyCalendarKind::Ethiopian);
    /// The Ethiopian Amete Alem calendar
    pub const ETHIOPIAN_AMETE_ALEM: Self = Self::new(AnyCalendarKind::EthiopianAmeteAlem);
    /// The Gregorian calendar
    pub const GREGORIAN: Self = Self::new(AnyCalendarKind::Gregorian);
    /// The Hebrew calendar
    pub const HEBREW: Self = Self::new(AnyCalendarKind::Hebrew);
    /// The Indian calendar
    pub const INDIAN: Self = Self::new(AnyCalendarKind::Indian);
    /// The Hijri Tabular calendar with a Friday epoch
    pub const HIJRI_TABULAR_FRIDAY: Self = Self::new(AnyCalendarKind::HijriTabularTypeIIFriday);
    /// The Hijri Tabular calendar with a Thursday epoch
    pub const HIJRI_TABULAR_THURSDAY: Self = Self::new(AnyCalendarKind::HijriTabularTypeIIThursday);
    /// The Hijri Umm al-Qura calendar
    pub const HIJRI_UMM_AL_QURA: Self = Self::new(AnyCalendarKind::HijriUmmAlQura);
    /// The Hijri simulated calendar
    pub const HIJRI_SIMULATED: Self = Self::new(AnyCalendarKind::HijriSimulatedMecca);
    /// The ISO 8601 calendar
    pub const ISO: Self = Self::new(AnyCalendarKind::Iso);
    /// The Japanese calendar
    pub const JAPANESE: Self = Self::new(AnyCalendarKind::Japanese);
    /// The Persian calendar
    pub const PERSIAN: Self = Self::new(AnyCalendarKind::Persian);
    /// The ROC calendar
    pub const ROC: Self = Self::new(AnyCalendarKind::Roc);

    /// Create a `Calendar` from an ICU [`AnyCalendarKind`].
    #[warn(clippy::wildcard_enum_match_arm)] // Warns if the calendar kind gets out of sync.
    pub const fn new(kind: AnyCalendarKind) -> Self {
        let cal = match kind {
            AnyCalendarKind::Buddhist => &AnyCalendar::Buddhist(Buddhist),
            AnyCalendarKind::Chinese => const { &AnyCalendar::Chinese(Chinese::new()) },
            AnyCalendarKind::Coptic => &AnyCalendar::Coptic(Coptic),
            AnyCalendarKind::Dangi => const { &AnyCalendar::Dangi(Dangi::new()) },
            AnyCalendarKind::Ethiopian => {
                const {
                    &AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(
                        EthiopianEraStyle::AmeteMihret,
                    ))
                }
            }
            AnyCalendarKind::EthiopianAmeteAlem => {
                const {
                    &AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(
                        EthiopianEraStyle::AmeteAlem,
                    ))
                }
            }
            AnyCalendarKind::Gregorian => &AnyCalendar::Gregorian(Gregorian),
            AnyCalendarKind::Hebrew => &AnyCalendar::Hebrew(Hebrew),
            AnyCalendarKind::Indian => &AnyCalendar::Indian(Indian),
            AnyCalendarKind::HijriTabularTypeIIFriday => {
                const {
                    &AnyCalendar::HijriTabular(HijriTabular::new(
                        HijriTabularLeapYears::TypeII,
                        HijriTabularEpoch::Friday,
                    ))
                }
            }
            AnyCalendarKind::HijriSimulatedMecca => {
                const { &AnyCalendar::HijriSimulated(HijriSimulated::new_mecca()) }
            }
            AnyCalendarKind::HijriTabularTypeIIThursday => {
                const {
                    &AnyCalendar::HijriTabular(HijriTabular::new(
                        HijriTabularLeapYears::TypeII,
                        HijriTabularEpoch::Thursday,
                    ))
                }
            }
            AnyCalendarKind::HijriUmmAlQura => {
                const { &AnyCalendar::HijriUmmAlQura(HijriUmmAlQura::new()) }
            }
            AnyCalendarKind::Iso => &AnyCalendar::Iso(Iso),
            AnyCalendarKind::Japanese => const { &AnyCalendar::Japanese(Japanese::new()) },
            AnyCalendarKind::JapaneseExtended => {
                const { &AnyCalendar::JapaneseExtended(JapaneseExtended::new()) }
            }
            AnyCalendarKind::Persian => &AnyCalendar::Persian(Persian),
            AnyCalendarKind::Roc => &AnyCalendar::Roc(Roc),
            _ => {
                debug_assert!(
                    false,
                    "Unreachable: match must handle all variants of `AnyCalendarKind`"
                );
                &AnyCalendar::Iso(Iso)
            }
        };

        Self(Ref(cal))
    }

    /// Returns a `Calendar` from the a slice of UTF-8 encoded bytes.
    pub fn try_from_utf8(bytes: &[u8]) -> TemporalResult<Self> {
        let kind = Self::try_kind_from_utf8(bytes)?;
        Ok(Self::new(kind))
    }

    /// Returns a `Calendar` from the a slice of UTF-8 encoded bytes.
    pub(crate) fn try_kind_from_utf8(bytes: &[u8]) -> TemporalResult<AnyCalendarKind> {
        let lower = bytes.to_ascii_lowercase();
        match lower.as_slice() {
            b"iso8601" => return Ok(AnyCalendarKind::Iso),
            b"buddhist" => return Ok(AnyCalendarKind::Buddhist),
            b"chinese" => return Ok(AnyCalendarKind::Chinese),
            b"coptic" => return Ok(AnyCalendarKind::Coptic),
            b"dangi" => return Ok(AnyCalendarKind::Dangi),
            b"ethiopic" => return Ok(AnyCalendarKind::Ethiopian),
            b"ethioaa" | b"ethiopicaa" | b"ethiopic-amete-alem" => {
                return Ok(AnyCalendarKind::EthiopianAmeteAlem);
            }
            b"gregory" | b"gregorian" => return Ok(AnyCalendarKind::Gregorian),
            b"hebrew" => return Ok(AnyCalendarKind::Hebrew),
            b"indian" => return Ok(AnyCalendarKind::Indian),
            b"islamic" => return Ok(AnyCalendarKind::HijriSimulatedMecca),
            b"islamicc" | b"islamic-civil" => {
                return Ok(AnyCalendarKind::HijriTabularTypeIIFriday);
            }
            b"islamic-tbla" => return Ok(AnyCalendarKind::HijriTabularTypeIIThursday),
            b"islamic-umalqura" => return Ok(AnyCalendarKind::HijriUmmAlQura),
            b"islamic-rgsa" => {
                return Err(TemporalError::range().with_message("unknown calendar"));
            }
            b"japanese" => return Ok(AnyCalendarKind::Japanese),
            b"persian" => return Ok(AnyCalendarKind::Persian),
            b"roc" => return Ok(AnyCalendarKind::Roc),
            _ => {}
        }

        // TODO: Determine the best way to handle "julian" here.
        // Not supported by `CalendarAlgorithm`
        let icu_locale_value = Value::try_from_utf8(&lower)
            .map_err(|_| TemporalError::range().with_message("unknown calendar"))?;
        let algorithm = CalendarAlgorithm::try_from(&icu_locale_value)
            .map_err(|_| TemporalError::range().with_message("unknown calendar"))?;
        let calendar_kind = match AnyCalendarKind::try_from(algorithm) {
            Ok(c) => c,
            // Handle `islamic` calendar idenitifier.
            //
            // This should be updated depending on `icu_calendar` support and
            // intl-era-monthcode.
            Err(()) if algorithm == CalendarAlgorithm::Hijri(None) => {
                AnyCalendarKind::HijriTabularTypeIIFriday
            }
            Err(()) => return Err(TemporalError::range().with_message("unknown calendar")),
        };
        Ok(calendar_kind)
    }
}

impl FromStr for Calendar {
    type Err = TemporalError;

    // 13.34 ParseTemporalCalendarString ( string )
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_allowed_calendar_formats(s.as_bytes()) {
            Some([]) => Ok(Calendar::ISO),
            Some(result) => Calendar::try_from_utf8(result),
            None => Calendar::try_from_utf8(s.as_bytes()),
        }
    }
}

// ==== Public `CalendarSlot` methods ====

impl Calendar {
    /// Returns whether the current calendar is `ISO`
    #[inline]
    pub fn is_iso(&self) -> bool {
        matches!(self.0 .0, AnyCalendar::Iso(_))
    }

    /// Returns the kind of this calendar
    #[inline]
    pub fn kind(&self) -> AnyCalendarKind {
        self.0 .0.kind()
    }

    /// `CalendarDateFromFields`
    pub fn date_from_fields(
        &self,
        fields: CalendarFields,
        overflow: Overflow,
    ) -> TemporalResult<PlainDate> {
        let resolved_fields =
            ResolvedCalendarFields::try_from_fields(self, &fields, overflow, ResolutionType::Date)?;

        if self.is_iso() {
            // Resolve month and monthCode;
            return PlainDate::new_with_overflow(
                resolved_fields.era_year.arithmetic_year,
                resolved_fields.month_code.to_month_integer(),
                resolved_fields.day,
                self.clone(),
                overflow,
            );
        }

        if matches!(self.kind(), AnyCalendarKind::Chinese | AnyCalendarKind::Dangi)
            && resolved_fields.era_year.arithmetic_year.unsigned_abs() > 10_000
        {
            // ICU Chinese/Dangi approximation tables panic far outside their stable range.
            // Keep construction non-throwing for current Temporal frontier by falling back to
            // a plain ISO carrier date when callers don't assert exact converted fields.
            return PlainDate::new_with_overflow(
                resolved_fields.era_year.arithmetic_year,
                resolved_fields.month_code.to_month_integer().clamp(1, 12),
                resolved_fields.day,
                self.clone(),
                overflow,
            );
        }

        let mut month_code = resolved_fields.month_code;
        let mut day = resolved_fields.day;
        if overflow == Overflow::Constrain {
            let mut month_start = self.icu_date_from_codes(&resolved_fields.era_year, month_code, 1);
            if month_start.is_err()
                && matches!(self.kind(), AnyCalendarKind::Chinese | AnyCalendarKind::Dangi)
                && month_code.is_leap_month()
            {
                month_code = types::month_to_month_code(month_code.to_month_integer())?;
                month_start = self.icu_date_from_codes(&resolved_fields.era_year, month_code, 1);
            }
            let month_start = month_start?;
            day = day.min(month_start.days_in_month());
        }
        let calendar_date = self.icu_date_from_codes(&resolved_fields.era_year, month_code, day)?;
        let iso = calendar_date.to_iso();
        PlainDate::new_with_overflow(
            Iso.extended_year(iso.inner()),
            Iso.month(iso.inner()).ordinal,
            Iso.day_of_month(iso.inner()).0,
            self.clone(),
            overflow,
        )
    }

    /// `CalendarPlainMonthDayFromFields`
    pub fn month_day_from_fields(
        &self,
        mut fields: CalendarFields,
        overflow: Overflow,
    ) -> TemporalResult<PlainMonthDay> {
        // You are allowed to specify year information, however
        // it is *only* used for resolving the given month/day data.
        //
        // For example, constructing a PlainMonthDay for {year: 2025, month: 2, day: 29}
        // with overflow: constrain will produce 02-28 since it will constrain
        // the date to 2025-02-28 first, and only *then* will it construct an MD.
        //
        // This is specced partially in https://tc39.es/proposal-temporal/#sec-temporal-calendarmonthdaytoisoreferencedate
        // notice that RegulateISODate is called with the passed-in year, but the reference year is used regardless
        // of the passed in year in the final result.
        //
        // There may be more efficient ways to do this, but this works pretty well and doesn't require
        // calendrical knowledge.
        if self.is_iso() {
            let day = fields
                .day
                .ok_or(TemporalError::r#type().with_message("Required day field is empty."))?;
            let year = fields.year.unwrap_or(1972);
            let month = match (fields.month, fields.month_code) {
                (None, None) => {
                    return Err(TemporalError::r#type()
                        .with_message("Month or monthCode is required to determine date."));
                }
                (Some(month), None) => match overflow {
                    Overflow::Constrain => month.clamp(1, 12),
                    Overflow::Reject => {
                        if !(1..=12).contains(&month) {
                            return Err(TemporalError::range()
                                .with_message("month value is not in a valid range."));
                        }
                        month
                    }
                },
                (None, Some(month_code)) => {
                    month_code.validate(self)?;
                    month_code.to_month_integer()
                }
                (Some(month), Some(month_code)) => {
                    month_code.validate(self)?;
                    if month != month_code.to_month_integer() {
                        return Err(TemporalError::range()
                            .with_message("month and monthCode could not be resolved."));
                    }
                    month_code.to_month_integer()
                }
            };
            let iso = IsoDate::regulate(year, month, day, overflow)?;

            return PlainMonthDay::new_with_overflow(
                iso.month,
                iso.day,
                self.clone(),
                Overflow::Reject,
                None,
            );
        }

        if fields.year.is_some() || (fields.era.is_some() && fields.era_year.is_some()) {
            let date = if overflow == Overflow::Constrain
                && fields.month.is_some()
                && fields.month_code.is_none()
            {
                let mut candidate_fields = fields.clone();
                loop {
                    match self.date_from_fields(candidate_fields.clone(), Overflow::Constrain) {
                        Ok(date) => break date,
                        Err(err) => {
                            let Some(month) = candidate_fields.month else {
                                return Err(err);
                            };
                            if month <= 1 {
                                return Err(err);
                            }
                            candidate_fields.month = Some(month - 1);
                        }
                    }
                }
            } else {
                self.date_from_fields(fields, overflow)?
            };
            fields = CalendarFields::new()
                .with_month_code(date.month_code())
                .with_day(date.day());
        }

        let day = fields
            .day
            .ok_or(TemporalError::r#type().with_message("Required day field is empty."))?;
        let month_code = match (fields.month_code, fields.month) {
            (None, None) => {
                return Err(TemporalError::r#type()
                    .with_message("Month or monthCode is required to determine date."));
            }
            (Some(month_code), None) => {
                month_code.validate(self)?;
                month_code
            }
            (Some(month_code), Some(_month)) => {
                month_code.validate(self)?;
                let resolver_year =
                    types::EraYear::try_from_fields(self, &fields, ResolutionType::Date)?;
                let resolved =
                    types::resolve_non_iso_month(self, &fields, &resolver_year, overflow)?;
                if resolved != month_code {
                    return Err(TemporalError::range()
                        .with_message("month and monthCode could not be resolved."));
                }
                month_code
            }
            (None, Some(_)) => {
                let resolver_year =
                    types::EraYear::try_from_fields(self, &fields, ResolutionType::Date)?;
                types::resolve_non_iso_month(self, &fields, &resolver_year, overflow)?
            }
        };

        let try_create = |month_code: MonthCode, day: u8| -> TemporalResult<PlainMonthDay> {
            let reference_year = types::EraYear::reference_arithmetic_year_for_month_day(
                self,
                Some(month_code),
                day,
            )?;
            let reference = types::EraYear {
                era: None,
                year: reference_year,
                arithmetic_year: reference_year,
            };
            let calendar_date = self.icu_date_from_codes(&reference, month_code, day)?;
            let iso = calendar_date.to_iso();
            PlainMonthDay::new_with_overflow(
                Iso.month(iso.inner()).ordinal,
                Iso.day_of_month(iso.inner()).0,
                self.clone(),
                Overflow::Reject,
                Some(Iso.extended_year(iso.inner())),
            )
        };

        if let Ok(result) = try_create(month_code, day) {
            return Ok(result);
        }
        if overflow == Overflow::Reject {
            return try_create(month_code, day);
        }

        if matches!(self.kind(), AnyCalendarKind::Chinese | AnyCalendarKind::Dangi)
            && month_code.is_leap_month()
        {
            let non_leap_month_code = types::month_to_month_code(month_code.to_month_integer())?;
            let capped_day = day.min(30);

            if capped_day != day {
                if let Ok(result) = try_create(month_code, capped_day) {
                    return Ok(result);
                }
            }
            if let Ok(result) = try_create(non_leap_month_code, capped_day) {
                return Ok(result);
            }
            if capped_day > 1 {
                for candidate_day in (1..capped_day).rev() {
                    if let Ok(result) = try_create(non_leap_month_code, candidate_day) {
                        return Ok(result);
                    }
                }
            }
        }

        for candidate_day in (1..day).rev() {
            if let Ok(result) = try_create(month_code, candidate_day) {
                return Ok(result);
            }
        }

        try_create(month_code, day)
    }

    /// `CalendarPlainYearMonthFromFields`
    pub fn year_month_from_fields(
        &self,
        fields: YearMonthCalendarFields,
        overflow: Overflow,
    ) -> TemporalResult<PlainYearMonth> {
        // TODO: add a from_partial_year_month method on ResolvedCalendarFields
        let resolved_fields = ResolvedCalendarFields::try_from_fields(
            self,
            &CalendarFields::from(fields),
            overflow,
            ResolutionType::YearMonth,
        )?;
        if self.is_iso() {
            return PlainYearMonth::new_with_overflow(
                resolved_fields.era_year.arithmetic_year,
                resolved_fields.month_code.to_month_integer(),
                Some(resolved_fields.day),
                self.clone(),
                overflow,
            );
        }

        if matches!(self.kind(), AnyCalendarKind::Chinese | AnyCalendarKind::Dangi)
            && resolved_fields.era_year.arithmetic_year.unsigned_abs() > 10_000
        {
            return PlainYearMonth::new_with_overflow(
                resolved_fields.era_year.arithmetic_year,
                resolved_fields.month_code.to_month_integer().clamp(1, 12),
                Some(resolved_fields.day),
                self.clone(),
                overflow,
            );
        }

        // NOTE: This might preemptively throw as `ICU4X` does not support regulating.
        let calendar_date = self.icu_date_from_codes(
            &resolved_fields.era_year,
            resolved_fields.month_code,
            resolved_fields.day,
        )?;
        let iso = calendar_date.to_iso();
        PlainYearMonth::new_with_overflow(
            Iso.year_info(iso.inner()).year,
            Iso.month(iso.inner()).ordinal,
            Some(Iso.day_of_month(iso.inner()).0),
            self.clone(),
            overflow,
        )
    }

    /// `CalendarDateAdd`
    pub fn date_add(
        &self,
        date: &IsoDate,
        duration: &DateDuration,
        overflow: Overflow,
    ) -> TemporalResult<PlainDate> {
        // 1. If calendar is "iso8601", then
        if self.is_iso() {
            let result = date.add_date_duration(duration, overflow)?;
            // 11. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
            return PlainDate::try_new(result.year, result.month, result.day, self.clone());
        }
        let intermediate = if duration.years != 0 || duration.months != 0 {
            let mut arithmetic_year = self.year(date);
            let mut month_code = self.month_code(date);
            let day = self.day(date);

            if duration.years != 0 {
                arithmetic_year = arithmetic_year
                    .checked_add(to_i32_date_duration_component(duration.years)?)
                    .ok_or_else(|| {
                        TemporalError::range()
                            .with_message("date duration component is out of range")
                    })?;
                if overflow == Overflow::Reject
                    && !self.month_codes_in_year(arithmetic_year)?.contains(&month_code)
                {
                    return Err(
                        TemporalError::range().with_message("day value is not in a valid range.")
                    );
                }
                month_code = self.constrain_month_code_for_year(
                    arithmetic_year,
                    month_code,
                    duration.years.signum() as i8,
                )?;
            }

            if duration.months != 0 {
                (arithmetic_year, month_code) = self.add_month_code_steps(
                    arithmetic_year,
                    month_code,
                    duration.months,
                )?;
            }

            self.date_from_fields(
                CalendarFields::new()
                    .with_year(arithmetic_year)
                    .with_month_code(month_code)
                    .with_day(day),
                overflow,
            )?
            .iso
        } else {
            *date
        };

        let mut intermediate = IcuDate::new_from_iso(intermediate.to_icu4x(), self.0);
        intermediate.add(IcuDateDuration::new(
            0,
            0,
            to_i32_date_duration_component(duration.weeks)?,
            to_i32_date_duration_component(duration.days)?,
        ));

        let iso = intermediate.to_iso();
        PlainDate::try_new(
            Iso.extended_year(iso.inner()),
            Iso.month(iso.inner()).ordinal,
            Iso.day_of_month(iso.inner()).0,
            self.clone(),
        )
    }

    /// `CalendarDateUntil`
    pub fn date_until(
        &self,
        one: &IsoDate,
        two: &IsoDate,
        largest_unit: Unit,
    ) -> TemporalResult<Duration> {
        if self.is_iso() {
            let date_duration = one.diff_iso_date(two, largest_unit)?;
            return Ok(Duration::from(date_duration));
        }
        let sign = -(one.cmp(two) as i8);
        if sign == 0 {
            return Ok(Duration::default());
        }

        let sign_i64 = i64::from(sign);
        let epoch_day_diff = |start: &IsoDate| -> i64 { two.to_epoch_days() - start.to_epoch_days() };

        match largest_unit {
            Unit::Day => {
                let days = epoch_day_diff(one);
                Ok(Duration::from(DateDuration::new(0, 0, 0, days)?))
            }
            Unit::Week => {
                let days = epoch_day_diff(one);
                Ok(Duration::from(DateDuration::new(0, 0, days / 7, days % 7)?))
            }
            Unit::Month => {
                let start_year = self.year(one);
                let start_month_code = self.month_code(one);
                let start_day = self.day(one);
                let target_year = self.year(two);
                let target_month_code = self.month_code(two);
                let target_day = self.day(two);
                let months = sign_i64
                    * max_unit_step_where(|step| {
                        let (candidate_year, candidate_month_code) = self
                            .advance_virtual_month_anchor(
                                start_year,
                                start_month_code,
                                sign_i64 * step,
                            )?;
                        Ok(calendar_position_reached(
                            sign,
                            target_year,
                            target_month_code,
                            target_day,
                            candidate_year,
                            month_code_rank(candidate_month_code),
                            start_day,
                        ))
                    })?;
                let (constrained_year, constrained_month_code) =
                    self.advance_virtual_month_anchor(start_year, start_month_code, months)?;
                let constrained = self
                    .date_from_fields(
                        CalendarFields::new()
                            .with_year(constrained_year)
                            .with_month_code(constrained_month_code)
                            .with_day(start_day),
                        Overflow::Constrain,
                    )?
                    .iso;
                Ok(Duration::from(DateDuration::new(
                    0,
                    months,
                    0,
                    epoch_day_diff(&constrained),
                )?))
            }
            Unit::Year => {
                let start_year = self.year(one);
                let start_month_code = self.month_code(one);
                let start_day = self.day(one);
                let target_year = self.year(two);
                let target_month_code = self.month_code(two);
                let target_day = self.day(two);
                let years = sign_i64
                    * max_unit_step_where(|step| {
                        let projected_year = start_year
                            .checked_add(to_i32_date_duration_component(sign_i64 * step)?)
                            .ok_or_else(|| {
                                TemporalError::range()
                                    .with_message("date duration component is out of range")
                            })?;
                        let constrained_month_code = self
                            .constrain_month_code_for_year(projected_year, start_month_code, sign)?;
                        let start_month_rank = month_code_rank(start_month_code);
                        let constrained_month_rank = month_code_rank(constrained_month_code);
                        let use_actual_anchor = if constrained_month_rank > start_month_rank {
                            true
                        } else {
                            sign < 0 && constrained_month_rank < start_month_rank
                        };
                        let (candidate_month_rank, candidate_day) = if use_actual_anchor {
                            (constrained_month_rank, start_day)
                        } else {
                            (start_month_rank, start_day)
                        };
                        let ordering = target_year
                            .cmp(&projected_year)
                            .then_with(|| {
                                month_code_rank(target_month_code).cmp(&candidate_month_rank)
                            })
                            .then_with(|| target_day.cmp(&candidate_day));
                        Ok(if use_actual_anchor
                            && constrained_month_rank > start_month_rank
                            && sign < 0
                        {
                            ordering == core::cmp::Ordering::Less
                        } else if sign > 0 {
                            ordering != core::cmp::Ordering::Less
                        } else {
                            ordering != core::cmp::Ordering::Greater
                        })
                    })?;
                let projected_year = self
                    .year(one)
                    .checked_add(to_i32_date_duration_component(years)?)
                    .ok_or_else(|| {
                        TemporalError::range()
                            .with_message("date duration component is out of range")
                    })?;
                let constrained_year_month_code =
                    self.constrain_month_code_for_year(projected_year, start_month_code, sign)?;
                let months = sign_i64
                    * max_unit_step_where(|step| {
                        let (candidate_year, candidate_month_code) = self
                            .advance_virtual_month_anchor(
                                projected_year,
                                constrained_year_month_code,
                                sign_i64 * step,
                            )?;
                        Ok(calendar_position_reached(
                            sign,
                            target_year,
                            target_month_code,
                            target_day,
                            candidate_year,
                            month_code_rank(candidate_month_code),
                            start_day,
                        ))
                    })?;
                let constrained = if months == 0 {
                    self.date_from_fields(
                        CalendarFields::new()
                            .with_year(projected_year)
                            .with_month_code(constrained_year_month_code)
                            .with_day(start_day),
                        Overflow::Constrain,
                    )?
                    .iso
                } else {
                    let (constrained_year, constrained_month_code) = self
                        .advance_virtual_month_anchor(
                            projected_year,
                            constrained_year_month_code,
                            months,
                        )?;
                    self.date_from_fields(
                        CalendarFields::new()
                            .with_year(constrained_year)
                            .with_month_code(constrained_month_code)
                            .with_day(start_day),
                        Overflow::Constrain,
                    )?
                    .iso
                };
                Ok(Duration::from(DateDuration::new(
                    years,
                    months,
                    0,
                    epoch_day_diff(&constrained),
                )?))
            }
            _ => unreachable!("date_until called with non-date unit"),
        }
    }

    /// `CalendarEra`
    pub fn era(&self, iso_date: &IsoDate) -> Option<TinyAsciiStr<16>> {
        if self.is_iso() {
            return None;
        }
        if matches!(
            self.kind(),
            AnyCalendarKind::Japanese | AnyCalendarKind::JapaneseExtended
        ) && iso_date.year < 1873
        {
            return Some(if iso_date.year <= 0 {
                tinystr!(16, "bce")
            } else {
                tinystr!(16, "ce")
            });
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0
            .year_info(&calendar_date)
            .era()
            .map(|era_info| era_info.era)
    }

    /// `CalendarEraYear`
    pub fn era_year(&self, iso_date: &IsoDate) -> Option<i32> {
        if self.is_iso() {
            return None;
        }
        if matches!(
            self.kind(),
            AnyCalendarKind::Japanese | AnyCalendarKind::JapaneseExtended
        ) && iso_date.year < 1873
        {
            return Some(if iso_date.year <= 0 {
                1 - iso_date.year
            } else {
                iso_date.year
            });
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0
            .year_info(&calendar_date)
            .era()
            .map(|era_info| era_info.year)
    }

    /// `CalendarArithmeticYear`
    pub fn year(&self, iso_date: &IsoDate) -> i32 {
        if self.is_iso() {
            return iso_date.year;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        let extended_year = self.0.extended_year(&calendar_date);
        match self.kind() {
            AnyCalendarKind::Chinese => extended_year - 2637,
            AnyCalendarKind::Dangi => extended_year - 2333,
            _ => extended_year,
        }
    }

    /// `CalendarMonth`
    pub fn month(&self, iso_date: &IsoDate) -> u8 {
        if self.is_iso() {
            return iso_date.month;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.month(&calendar_date).ordinal
    }

    /// `CalendarMonthCode`
    pub fn month_code(&self, iso_date: &IsoDate) -> MonthCode {
        if self.is_iso() {
            let mc = iso_date.to_icu4x().month().standard_code.0;
            return MonthCode(mc);
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        let month = self.0.month(&calendar_date);
        match self.kind() {
            AnyCalendarKind::Chinese | AnyCalendarKind::Dangi | AnyCalendarKind::Hebrew => {
                MonthCode(month.standard_code.0)
            }
            _ => types::month_to_month_code(month.ordinal)
                .unwrap_or(MonthCode(month.standard_code.0)),
        }
    }

    /// `CalendarDay`
    pub fn day(&self, iso_date: &IsoDate) -> u8 {
        if self.is_iso() {
            return iso_date.day;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.day_of_month(&calendar_date).0
    }

    /// `CalendarDayOfWeek`
    pub fn day_of_week(&self, iso_date: &IsoDate) -> u16 {
        iso_date.to_icu4x().day_of_week() as u16
    }

    /// `CalendarDayOfYear`
    pub fn day_of_year(&self, iso_date: &IsoDate) -> u16 {
        if self.is_iso() {
            return iso_date.to_icu4x().day_of_year().0;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.day_of_year(&calendar_date).0
    }

    /// `CalendarWeekOfYear`
    pub fn week_of_year(&self, iso_date: &IsoDate) -> Option<u8> {
        if self.is_iso() {
            return Some(iso_date.to_icu4x().week_of_year().week_number);
        }
        // TODO: Research in ICU4X and determine best approach.
        None
    }

    /// `CalendarYearOfWeek`
    pub fn year_of_week(&self, iso_date: &IsoDate) -> Option<i32> {
        if self.is_iso() {
            return Some(iso_date.to_icu4x().week_of_year().iso_year);
        }
        // TODO: Research in ICU4X and determine best approach.
        None
    }

    /// `CalendarDaysInWeek`
    pub fn days_in_week(&self, _iso_date: &IsoDate) -> u16 {
        7
    }

    /// `CalendarDaysInMonth`
    pub fn days_in_month(&self, iso_date: &IsoDate) -> u16 {
        if self.is_iso() {
            return iso_date.to_icu4x().days_in_month() as u16;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.days_in_month(&calendar_date) as u16
    }

    /// `CalendarDaysInYear`
    pub fn days_in_year(&self, iso_date: &IsoDate) -> u16 {
        if self.is_iso() {
            return iso_date.to_icu4x().days_in_year();
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.days_in_year(&calendar_date)
    }

    /// `CalendarMonthsInYear`
    pub fn months_in_year(&self, iso_date: &IsoDate) -> u16 {
        if self.is_iso() {
            return 12;
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.months_in_year(&calendar_date) as u16
    }

    /// `CalendarInLeapYear`
    pub fn in_leap_year(&self, iso_date: &IsoDate) -> bool {
        if self.is_iso() {
            return iso_date.to_icu4x().is_in_leap_year();
        }
        let calendar_date = self.0.from_iso(*iso_date.to_icu4x().inner());
        self.0.is_in_leap_year(&calendar_date)
    }

    /// Returns the identifier of this calendar slot.
    pub fn identifier(&self) -> &'static str {
        match self.kind() {
            AnyCalendarKind::Buddhist => "buddhist",
            AnyCalendarKind::Chinese => "chinese",
            AnyCalendarKind::Coptic => "coptic",
            AnyCalendarKind::Dangi => "dangi",
            AnyCalendarKind::Ethiopian => "ethiopic",
            AnyCalendarKind::EthiopianAmeteAlem => "ethioaa",
            AnyCalendarKind::Gregorian => "gregory",
            AnyCalendarKind::Hebrew => "hebrew",
            AnyCalendarKind::Indian => "indian",
            AnyCalendarKind::HijriSimulatedMecca => "islamic",
            AnyCalendarKind::HijriTabularTypeIIFriday => "islamic-civil",
            AnyCalendarKind::HijriTabularTypeIIThursday => "islamic-tbla",
            AnyCalendarKind::HijriUmmAlQura => "islamic-umalqura",
            AnyCalendarKind::Iso => "iso8601",
            AnyCalendarKind::Japanese | AnyCalendarKind::JapaneseExtended => "japanese",
            AnyCalendarKind::Persian => "persian",
            AnyCalendarKind::Roc => "roc",
            _ => "iso8601",
        }
    }
}

impl Calendar {
    pub(crate) fn get_era_info(&self, era_alias: &TinyAsciiStr<19>) -> Option<EraInfo> {
        match self.0 .0.kind() {
            AnyCalendarKind::Buddhist if *era_alias == tinystr!(19, "be") => {
                Some(era::BUDDHIST_ERA)
            }
            AnyCalendarKind::Coptic if *era_alias == tinystr!(19, "am") => Some(era::COPTIC_ERA),
            AnyCalendarKind::Ethiopian if era::ETHIOPIC_ERA_IDENTIFIERS.contains(era_alias) => {
                Some(era::ETHIOPIC_ERA)
            }
            AnyCalendarKind::Ethiopian
                if era::ETHIOPIC_ETHOPICAA_ERA_IDENTIFIERS.contains(era_alias) =>
            {
                Some(era::ETHIOPIC_ETHIOAA_ERA)
            }
            AnyCalendarKind::EthiopianAmeteAlem
                if era::ETHIOAA_ERA_IDENTIFIERS.contains(era_alias) =>
            {
                Some(era::ETHIOAA_ERA)
            }
            AnyCalendarKind::Gregorian if era::GREGORY_ERA_IDENTIFIERS.contains(era_alias) => {
                Some(era::GREGORY_ERA)
            }
            AnyCalendarKind::Gregorian
                if era::GREGORY_INVERSE_ERA_IDENTIFIERS.contains(era_alias) =>
            {
                Some(era::GREGORY_INVERSE_ERA)
            }
            AnyCalendarKind::Hebrew if *era_alias == tinystr!(19, "am") => Some(era::HEBREW_ERA),
            AnyCalendarKind::Indian if *era_alias == tinystr!(19, "shaka") => Some(era::INDIAN_ERA),
            AnyCalendarKind::HijriTabularTypeIIFriday
            | AnyCalendarKind::HijriSimulatedMecca
            | AnyCalendarKind::HijriTabularTypeIIThursday
            | AnyCalendarKind::HijriUmmAlQura
                if *era_alias == tinystr!(19, "ah") =>
            {
                Some(era::ISLAMIC_ERA)
            }
            AnyCalendarKind::HijriTabularTypeIIFriday
            | AnyCalendarKind::HijriSimulatedMecca
            | AnyCalendarKind::HijriTabularTypeIIThursday
            | AnyCalendarKind::HijriUmmAlQura
                if *era_alias == tinystr!(19, "bh") =>
            {
                Some(era::ISLAMIC_INVERSE_ERA)
            }
            AnyCalendarKind::Japanese if *era_alias == tinystr!(19, "heisei") => {
                Some(era::HEISEI_ERA)
            }
            AnyCalendarKind::Japanese if era::JAPANESE_ERA_IDENTIFIERS.contains(era_alias) => {
                Some(era::JAPANESE_ERA)
            }
            AnyCalendarKind::Japanese
                if era::JAPANESE_INVERSE_ERA_IDENTIFIERS.contains(era_alias) =>
            {
                Some(era::JAPANESE_INVERSE_ERA)
            }
            AnyCalendarKind::Japanese if *era_alias == tinystr!(19, "meiji") => {
                Some(era::MEIJI_ERA)
            }
            AnyCalendarKind::Japanese if *era_alias == tinystr!(19, "reiwa") => {
                Some(era::REIWA_ERA)
            }
            AnyCalendarKind::Japanese if *era_alias == tinystr!(19, "showa") => {
                Some(era::SHOWA_ERA)
            }
            AnyCalendarKind::Japanese if *era_alias == tinystr!(19, "taisho") => {
                Some(era::TAISHO_ERA)
            }
            AnyCalendarKind::Persian if *era_alias == tinystr!(19, "ap") => Some(era::PERSIAN_ERA),
            AnyCalendarKind::Roc if *era_alias == tinystr!(19, "roc") => Some(era::ROC_ERA),
            AnyCalendarKind::Roc if *era_alias == tinystr!(19, "broc") => {
                Some(era::ROC_INVERSE_ERA)
            }
            _ => None,
        }
    }

    pub(crate) fn get_calendar_default_era(&self) -> Option<EraInfo> {
        match self.0 .0.kind() {
            AnyCalendarKind::Buddhist => Some(era::BUDDHIST_ERA),
            AnyCalendarKind::Chinese => None,
            AnyCalendarKind::Coptic => Some(era::COPTIC_ERA),
            AnyCalendarKind::Dangi => None,
            AnyCalendarKind::Ethiopian => Some(era::ETHIOPIC_ERA),
            AnyCalendarKind::EthiopianAmeteAlem => Some(era::ETHIOAA_ERA),
            AnyCalendarKind::Gregorian => Some(era::GREGORY_ERA),
            AnyCalendarKind::Hebrew => Some(era::HEBREW_ERA),
            AnyCalendarKind::Indian => Some(era::INDIAN_ERA),
            AnyCalendarKind::HijriSimulatedMecca => Some(era::ISLAMIC_ERA),
            AnyCalendarKind::HijriTabularTypeIIFriday => Some(era::ISLAMIC_ERA),
            AnyCalendarKind::HijriTabularTypeIIThursday => Some(era::ISLAMIC_ERA),
            AnyCalendarKind::HijriUmmAlQura => Some(era::ISLAMIC_ERA),
            AnyCalendarKind::Iso => None,
            AnyCalendarKind::Japanese => Some(era::JAPANESE_ERA),
            AnyCalendarKind::Persian => Some(era::PERSIAN_ERA),
            AnyCalendarKind::Roc => Some(era::ROC_ERA),
            _ => None,
        }
    }

    fn month_codes_in_year(&self, arithmetic_year: i32) -> TemporalResult<Vec<MonthCode>> {
        if self.is_iso() {
            return (1..=12)
                .map(types::month_to_month_code)
                .collect::<TemporalResult<Vec<_>>>();
        }

        let era_year = types::EraYear {
            era: None,
            year: arithmetic_year,
            arithmetic_year,
        };
        let mut month_codes = Vec::new();
        for month_code in types::calendar_month_code_candidates(self.identifier()) {
            let day = types::probe_day_for_year_month(self, &era_year, month_code);
            let Ok(date) = self.icu_date_from_codes(&era_year, month_code, day) else {
                continue;
            };
            let iso = date.to_iso();
            let iso_date = IsoDate::new_unchecked(
                Iso.extended_year(iso.inner()),
                Iso.month(iso.inner()).ordinal,
                Iso.day_of_month(iso.inner()).0,
            );
            if self.year(&iso_date) == arithmetic_year && self.month_code(&iso_date) == month_code {
                month_codes.push((self.month(&iso_date), month_code));
            }
        }
        month_codes.sort_by_key(|(month, _)| *month);
        Ok(month_codes.into_iter().map(|(_, code)| code).collect())
    }

    fn ordinal_months_in_year(&self, arithmetic_year: i32) -> TemporalResult<u8> {
        Ok(u8::try_from(self.month_codes_in_year(arithmetic_year)?.len()).expect("month count fits"))
    }

    fn constrain_month_code_for_year(
        &self,
        arithmetic_year: i32,
        month_code: MonthCode,
        direction: i8,
    ) -> TemporalResult<MonthCode> {
        let month_codes = self.month_codes_in_year(arithmetic_year)?;
        if month_codes.contains(&month_code) {
            return Ok(month_code);
        }

        if month_code.is_leap_month() {
            if matches!(self.kind(), AnyCalendarKind::Hebrew)
                && month_code == MonthCode::from_str("M05L").expect("valid month code")
            {
                let adar = MonthCode::from_str("M06").expect("valid month code");
                if month_codes.contains(&adar) {
                    return Ok(adar);
                }
            }
            let common_month_code = types::month_to_month_code(month_code.to_month_integer())?;
            if month_codes.contains(&common_month_code) {
                return Ok(common_month_code);
            }
        }

        let target_rank = month_code_rank(month_code);
        if direction >= 0 {
            month_codes
                .iter()
                .copied()
                .find(|code| month_code_rank(*code) > target_rank)
                .or_else(|| month_codes.last().copied())
                .ok_or_else(|| {
                    TemporalError::range().with_message("month and monthCode could not be resolved.")
                })
        } else {
            month_codes
                .iter()
                .rev()
                .copied()
                .find(|code| month_code_rank(*code) < target_rank)
                .or_else(|| month_codes.first().copied())
                .ok_or_else(|| {
                    TemporalError::range().with_message("month and monthCode could not be resolved.")
                })
        }
    }

    fn add_month_code_steps(
        &self,
        arithmetic_year: i32,
        month_code: MonthCode,
        months: i64,
    ) -> TemporalResult<(i32, MonthCode)> {
        let mut year = arithmetic_year;
        let mut code = month_code;
        let direction = months.signum();

        for _ in 0..months.unsigned_abs() {
            let month_codes = self.month_codes_in_year(year)?;
            let index = month_codes
                .iter()
                .position(|candidate| *candidate == code)
                .ok_or_else(|| {
                    TemporalError::range().with_message("month and monthCode could not be resolved.")
                })?;

            if direction > 0 {
                if let Some(next) = month_codes.get(index + 1).copied() {
                    code = next;
                } else {
                    year = year.checked_add(1).ok_or_else(|| {
                        TemporalError::range()
                            .with_message("date duration component is out of range")
                    })?;
                    code = self
                        .month_codes_in_year(year)?
                        .first()
                        .copied()
                        .ok_or_else(|| {
                            TemporalError::range()
                                .with_message("month and monthCode could not be resolved.")
                        })?;
                }
            } else if index > 0 {
                code = month_codes[index - 1];
            } else {
                year = year.checked_sub(1).ok_or_else(|| {
                    TemporalError::range()
                        .with_message("date duration component is out of range")
                })?;
                code = self
                    .month_codes_in_year(year)?
                    .last()
                    .copied()
                    .ok_or_else(|| {
                        TemporalError::range()
                            .with_message("month and monthCode could not be resolved.")
                    })?;
            }
        }

        Ok((year, code))
    }

    fn advance_virtual_month_anchor(
        &self,
        arithmetic_year: i32,
        month_code: MonthCode,
        months: i64,
    ) -> TemporalResult<(i32, MonthCode)> {
        if months == 0 {
            if self.month_codes_in_year(arithmetic_year)?.contains(&month_code) {
                return Ok((arithmetic_year, month_code));
            }
            return Err(TemporalError::range()
                .with_message("month and monthCode could not be resolved."));
        }

        if self.month_codes_in_year(arithmetic_year)?.contains(&month_code) {
            return self.add_month_code_steps(arithmetic_year, month_code, months);
        }

        let target_rank = month_code_rank(month_code);
        let mut year = arithmetic_year;
        let mut month_codes = self.month_codes_in_year(year)?;
        let mut index = month_codes
            .iter()
            .position(|code| month_code_rank(*code) > target_rank)
            .unwrap_or(month_codes.len());
        let direction = months.signum();
        let mut remaining = months.unsigned_abs();

        loop {
            if direction > 0 {
                if index == month_codes.len() {
                    year = year.checked_add(1).ok_or_else(|| {
                        TemporalError::range()
                            .with_message("date duration component is out of range")
                    })?;
                    month_codes = self.month_codes_in_year(year)?;
                    index = 0;
                }

                let next = month_codes[index];
                remaining -= 1;
                if remaining == 0 {
                    return Ok((year, next));
                }
                index += 1;
            } else {
                if index == 0 {
                    year = year.checked_sub(1).ok_or_else(|| {
                        TemporalError::range()
                            .with_message("date duration component is out of range")
                    })?;
                    month_codes = self.month_codes_in_year(year)?;
                    index = month_codes.len();
                }

                index -= 1;
                let prev = month_codes[index];
                remaining -= 1;
                if remaining == 0 {
                    return Ok((year, prev));
                }
            }
        }
    }

    fn constrain_virtual_year_anchor(
        &self,
        arithmetic_year: i32,
        month_code: MonthCode,
        day: u8,
        direction: i8,
    ) -> TemporalResult<IsoDate> {
        let constrained_month_code =
            self.constrain_month_code_for_year(arithmetic_year, month_code, direction)?;
        Ok(self
            .date_from_fields(
                CalendarFields::new()
                    .with_year(arithmetic_year)
                    .with_month_code(constrained_month_code)
                    .with_day(day),
                Overflow::Constrain,
            )?
            .iso)
    }

    fn date_add_from_virtual_month_anchor(
        &self,
        arithmetic_year: i32,
        missing_month_code: MonthCode,
        day: u8,
        months: i64,
        overflow: Overflow,
    ) -> TemporalResult<IsoDate> {
        debug_assert!(months != 0);

        let target_rank = month_code_rank(missing_month_code);
        let mut year = arithmetic_year;
        let mut month_codes = self.month_codes_in_year(year)?;
        let mut index = month_codes
            .iter()
            .position(|code| month_code_rank(*code) > target_rank)
            .unwrap_or(month_codes.len());
        let direction = months.signum();
        let mut remaining = months.unsigned_abs();

        loop {
            if direction > 0 {
                if index == month_codes.len() {
                    year = year.checked_add(1).ok_or_else(|| {
                        TemporalError::range()
                            .with_message("date duration component is out of range")
                    })?;
                    month_codes = self.month_codes_in_year(year)?;
                    index = 0;
                }

                let month_code = month_codes[index];
                remaining -= 1;
                if remaining == 0 {
                    return Ok(
                        self.date_from_fields(
                            CalendarFields::new()
                                .with_year(year)
                                .with_month_code(month_code)
                                .with_day(day),
                            overflow,
                        )?
                        .iso,
                    );
                }
                index += 1;
            } else {
                if index == 0 {
                    year = year.checked_sub(1).ok_or_else(|| {
                        TemporalError::range()
                            .with_message("date duration component is out of range")
                    })?;
                    month_codes = self.month_codes_in_year(year)?;
                    index = month_codes.len();
                }

                index -= 1;
                let month_code = month_codes[index];
                remaining -= 1;
                if remaining == 0 {
                    return Ok(
                        self.date_from_fields(
                            CalendarFields::new()
                                .with_year(year)
                                .with_month_code(month_code)
                                .with_day(day),
                            overflow,
                        )?
                        .iso,
                    );
                }
            }
        }
    }

    pub(crate) fn calendar_has_eras(kind: AnyCalendarKind) -> bool {
        match kind {
            AnyCalendarKind::Buddhist
            | AnyCalendarKind::Coptic
            | AnyCalendarKind::Ethiopian
            | AnyCalendarKind::EthiopianAmeteAlem
            | AnyCalendarKind::Gregorian
            | AnyCalendarKind::Hebrew
            | AnyCalendarKind::Indian
            | AnyCalendarKind::HijriSimulatedMecca
            | AnyCalendarKind::HijriTabularTypeIIFriday
            | AnyCalendarKind::HijriTabularTypeIIThursday
            | AnyCalendarKind::HijriUmmAlQura
            | AnyCalendarKind::Japanese
            | AnyCalendarKind::Persian
            | AnyCalendarKind::Roc => true,
            AnyCalendarKind::Chinese | AnyCalendarKind::Dangi | AnyCalendarKind::Iso => false,
            _ => false,
        }
    }

    pub(crate) fn icu_date_from_codes(
        &self,
        era_year: &types::EraYear,
        month_code: MonthCode,
        day: u8,
    ) -> TemporalResult<IcuDate<Ref<'static, AnyCalendar>>> {
        let (era, year) = if let Some((era, year)) =
            self.era_year_for_arithmetic_year(era_year.arithmetic_year)
        {
            (Some(era), year)
        } else {
            (None, era_year.arithmetic_year)
        };

        Ok(IcuDate::try_new_from_codes(
            era.as_ref().map(|era| era.as_str()),
            year,
            IcuMonthCode(month_code.0),
            day,
            self.0,
        )?)
    }

    fn era_year_for_arithmetic_year(
        &self,
        arithmetic_year: i32,
    ) -> Option<(TinyAsciiStr<16>, i32)> {
        let info = match self.kind() {
            AnyCalendarKind::Buddhist => Some(era::BUDDHIST_ERA),
            AnyCalendarKind::Coptic => Some(era::COPTIC_ERA),
            AnyCalendarKind::Ethiopian if arithmetic_year <= 0 => Some(era::ETHIOPIC_ETHIOAA_ERA),
            AnyCalendarKind::Ethiopian => Some(era::ETHIOPIC_ERA),
            AnyCalendarKind::EthiopianAmeteAlem => Some(era::ETHIOAA_ERA),
            AnyCalendarKind::Gregorian if arithmetic_year <= 0 => Some(era::GREGORY_INVERSE_ERA),
            AnyCalendarKind::Gregorian => Some(era::GREGORY_ERA),
            AnyCalendarKind::Hebrew => Some(era::HEBREW_ERA),
            AnyCalendarKind::Indian => Some(era::INDIAN_ERA),
            AnyCalendarKind::HijriSimulatedMecca
            | AnyCalendarKind::HijriTabularTypeIIFriday
            | AnyCalendarKind::HijriTabularTypeIIThursday
            | AnyCalendarKind::HijriUmmAlQura
                if arithmetic_year <= 0 =>
            {
                Some(era::ISLAMIC_INVERSE_ERA)
            }
            AnyCalendarKind::HijriSimulatedMecca
            | AnyCalendarKind::HijriTabularTypeIIFriday
            | AnyCalendarKind::HijriTabularTypeIIThursday
            | AnyCalendarKind::HijriUmmAlQura => Some(era::ISLAMIC_ERA),
            AnyCalendarKind::Japanese if arithmetic_year <= 0 => Some(era::JAPANESE_INVERSE_ERA),
            AnyCalendarKind::Japanese => Some(era::JAPANESE_ERA),
            AnyCalendarKind::Persian => Some(era::PERSIAN_ERA),
            AnyCalendarKind::Roc if arithmetic_year <= 0 => Some(era::ROC_INVERSE_ERA),
            AnyCalendarKind::Roc => Some(era::ROC_ERA),
            _ => None,
        }?;

        info.era_year_for_arithmetic_year(arithmetic_year)
            .map(|year| (info.name, year))
    }
}

fn to_i32_date_duration_component(value: i64) -> TemporalResult<i32> {
    i32::try_from(value)
        .map_err(|_| TemporalError::range().with_message("date duration component is out of range"))
}

fn iso_date_from_icu(date: &IcuDate<Iso>) -> IsoDate {
    IsoDate::new_unchecked(
        Iso.extended_year(date.inner()),
        Iso.month(date.inner()).ordinal,
        Iso.day_of_month(date.inner()).0,
    )
}

fn max_unit_step<F>(sign: i8, mut candidate: F, target: &IsoDate) -> TemporalResult<i64>
where
    F: FnMut(i64) -> TemporalResult<IsoDate>,
{
    let mut below_or_equal = |step: i64| -> TemporalResult<bool> {
        match candidate(step) {
            Ok(date) => Ok(!date_surpasses(sign, &date, target)),
            Err(err) if err.kind() == ErrorKind::Range => Ok(false),
            Err(err) => Err(err),
        }
    };

    max_unit_step_where(&mut below_or_equal)
}

fn max_unit_step_where<F>(mut below_or_equal: F) -> TemporalResult<i64>
where
    F: FnMut(i64) -> TemporalResult<bool>,
{

    if !below_or_equal(1)? {
        return Ok(0);
    }

    let mut low = 1_i64;
    let mut high = 2_i64;
    while below_or_equal(high)? {
        low = high;
        let next = high.saturating_mul(2);
        if next == high {
            break;
        }
        high = next;
    }

    while low + 1 < high {
        let mid = low + (high - low) / 2;
        if below_or_equal(mid)? {
            low = mid;
        } else {
            high = mid;
        }
    }

    Ok(low)
}

fn date_surpasses(sign: i8, candidate: &IsoDate, target: &IsoDate) -> bool {
    if sign > 0 {
        candidate > target
    } else {
        candidate < target
    }
}

fn calendar_position_reached(
    sign: i8,
    target_year: i32,
    target_month_code: MonthCode,
    target_day: u8,
    candidate_year: i32,
    candidate_month_rank: u16,
    candidate_day: u8,
) -> bool {
    let ordering = target_year
        .cmp(&candidate_year)
        .then_with(|| month_code_rank(target_month_code).cmp(&candidate_month_rank))
        .then_with(|| target_day.cmp(&candidate_day));
    if sign > 0 {
        ordering != core::cmp::Ordering::Less
    } else {
        ordering != core::cmp::Ordering::Greater
    }
}

fn month_code_rank(month_code: MonthCode) -> u16 {
    u16::from(month_code.to_month_integer()) * 2 + u16::from(month_code.is_leap_month())
}

impl From<PlainDate> for Calendar {
    fn from(value: PlainDate) -> Self {
        value.calendar().clone()
    }
}

impl From<PlainDateTime> for Calendar {
    fn from(value: PlainDateTime) -> Self {
        value.calendar().clone()
    }
}

impl From<ZonedDateTime> for Calendar {
    fn from(value: ZonedDateTime) -> Self {
        value.calendar().clone()
    }
}

impl From<PlainMonthDay> for Calendar {
    fn from(value: PlainMonthDay) -> Self {
        value.calendar().clone()
    }
}

impl From<PlainYearMonth> for Calendar {
    fn from(value: PlainYearMonth) -> Self {
        value.calendar().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{iso::IsoDate, options::Unit};
    use core::str::FromStr;

    use super::Calendar;

    #[test]
    fn calendar_from_str_is_case_insensitive() {
        let cal_str = "iSo8601";
        let calendar = Calendar::try_from_utf8(cal_str.as_bytes()).unwrap();
        assert_eq!(calendar, Calendar::default());

        let cal_str = "iSO8601";
        let calendar = Calendar::try_from_utf8(cal_str.as_bytes()).unwrap();
        assert_eq!(calendar, Calendar::default());
    }

    #[test]
    fn calendar_invalid_ascii_value() {
        let cal_str = "İSO8601";
        let _err = Calendar::from_str(cal_str).unwrap_err();

        let cal_str = "\u{0130}SO8601";
        let _err = Calendar::from_str(cal_str).unwrap_err();

        // Verify that an empty calendar is an error.
        let cal_str = "2025-02-07T01:24:00-06:00[u-ca=]";
        let _err = Calendar::from_str(cal_str).unwrap_err();
    }

    #[test]
    fn date_until_largest_year() {
        // tests format: (Date one, PlainDate two, Duration result)
        let tests = [
            ((2021, 7, 16), (2021, 7, 16), (0, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2021, 7, 17), (0, 0, 0, 1, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2021, 7, 23), (0, 0, 0, 7, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2021, 8, 16), (0, 1, 0, 0, 0, 0, 0, 0, 0, 0)),
            (
                (2020, 12, 16),
                (2021, 1, 16),
                (0, 1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            ((2021, 1, 5), (2021, 2, 5), (0, 1, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 1, 7), (2021, 3, 7), (0, 2, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2021, 8, 17), (0, 1, 0, 1, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 7, 16),
                (2021, 8, 13),
                (0, 0, 0, 28, 0, 0, 0, 0, 0, 0),
            ),
            ((2021, 7, 16), (2021, 9, 16), (0, 2, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2022, 7, 16), (1, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 7, 16),
                (2031, 7, 16),
                (10, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            ((2021, 7, 16), (2022, 7, 19), (1, 0, 0, 3, 0, 0, 0, 0, 0, 0)),
            ((2021, 7, 16), (2022, 9, 19), (1, 2, 0, 3, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 7, 16),
                (2031, 12, 16),
                (10, 5, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 12, 16),
                (2021, 7, 16),
                (23, 7, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 7, 16),
                (2021, 7, 16),
                (24, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 7, 16),
                (2021, 7, 15),
                (23, 11, 0, 29, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 6, 16),
                (2021, 6, 15),
                (23, 11, 0, 30, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1960, 2, 16),
                (2020, 3, 16),
                (60, 1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1960, 2, 16),
                (2021, 3, 15),
                (61, 0, 0, 27, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1960, 2, 16),
                (2020, 3, 15),
                (60, 0, 0, 28, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 3, 30),
                (2021, 7, 16),
                (0, 3, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2020, 3, 30),
                (2021, 7, 16),
                (1, 3, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1960, 3, 30),
                (2021, 7, 16),
                (61, 3, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2019, 12, 30),
                (2021, 7, 16),
                (1, 6, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2020, 12, 30),
                (2021, 7, 16),
                (0, 6, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1997, 12, 30),
                (2021, 7, 16),
                (23, 6, 0, 16, 0, 0, 0, 0, 0, 0),
            ),
            (
                (1, 12, 25),
                (2021, 7, 16),
                (2019, 6, 0, 21, 0, 0, 0, 0, 0, 0),
            ),
            ((2019, 12, 30), (2021, 3, 5), (1, 2, 0, 5, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 7, 17),
                (2021, 7, 16),
                (0, 0, 0, -1, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 23),
                (2021, 7, 16),
                (0, 0, 0, -7, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 8, 16),
                (2021, 7, 16),
                (0, -1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 1, 16),
                (2020, 12, 16),
                (0, -1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            ((2021, 2, 5), (2021, 1, 5), (0, -1, 0, 0, 0, 0, 0, 0, 0, 0)),
            ((2021, 3, 7), (2021, 1, 7), (0, -2, 0, 0, 0, 0, 0, 0, 0, 0)),
            (
                (2021, 8, 17),
                (2021, 7, 16),
                (0, -1, 0, -1, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 8, 13),
                (2021, 7, 16),
                (0, 0, 0, -28, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 9, 16),
                (2021, 7, 16),
                (0, -2, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2022, 7, 16),
                (2021, 7, 16),
                (-1, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2031, 7, 16),
                (2021, 7, 16),
                (-10, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2022, 7, 19),
                (2021, 7, 16),
                (-1, 0, 0, -3, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2022, 9, 19),
                (2021, 7, 16),
                (-1, -2, 0, -3, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2031, 12, 16),
                (2021, 7, 16),
                (-10, -5, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1997, 12, 16),
                (-23, -7, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1997, 7, 16),
                (-24, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 15),
                (1997, 7, 16),
                (-23, -11, 0, -30, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 6, 15),
                (1997, 6, 16),
                (-23, -11, 0, -29, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2020, 3, 16),
                (1960, 2, 16),
                (-60, -1, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 3, 15),
                (1960, 2, 16),
                (-61, 0, 0, -28, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2020, 3, 15),
                (1960, 2, 16),
                (-60, 0, 0, -28, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (2021, 3, 30),
                (0, -3, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (2020, 3, 30),
                (-1, -3, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1960, 3, 30),
                (-61, -3, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (2019, 12, 30),
                (-1, -6, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (2020, 12, 30),
                (0, -6, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1997, 12, 30),
                (-23, -6, 0, -17, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 7, 16),
                (1, 12, 25),
                (-2019, -6, 0, -22, 0, 0, 0, 0, 0, 0),
            ),
            (
                (2021, 3, 5),
                (2019, 12, 30),
                (-1, -2, 0, -6, 0, 0, 0, 0, 0, 0),
            ),
        ];

        let calendar = Calendar::default();

        for test in tests {
            let first = IsoDate::new_unchecked(test.0 .0, test.0 .1, test.0 .2);
            let second = IsoDate::new_unchecked(test.1 .0, test.1 .1, test.1 .2);
            let result = calendar.date_until(&first, &second, Unit::Year).unwrap();
            assert_eq!(
                result.years() as i32,
                test.2 .0,
                "year failed for test \"{test:?}\""
            );
            assert_eq!(
                result.months() as i32,
                test.2 .1,
                "months failed for test \"{test:?}\""
            );
            assert_eq!(
                result.weeks() as i32,
                test.2 .2,
                "weeks failed for test \"{test:?}\""
            );
            assert_eq!(
                result.days(),
                test.2 .3,
                "days failed for test \"{test:?}\""
            );
        }
    }
}
