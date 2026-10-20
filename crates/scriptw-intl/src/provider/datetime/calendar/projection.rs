use icu_calendar::{cal::Gregorian, types::YearInfo, Date};

use super::{
    kind::{CalendarId, EraName},
    month::Month,
    CalendarKernels,
};
use crate::datetime::{DateTimeFormatError, DateTimeIsoFields};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::provider::datetime) enum CalendarYear {
    Era { name: EraName, year: i32 },
    Cyclic { year: u8, related_iso: i32 },
}

/// Chinese/Dangi `from_codes` accepts the related ISO year, not extended year.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::provider::datetime) enum CodeYear {
    Extended(i32),
    RelatedIso(i32),
}

#[cfg(test)]
impl CodeYear {
    pub(in crate::provider::datetime) const fn value(self) -> i32 {
        match self {
            Self::Extended(year) | Self::RelatedIso(year) => year,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::provider::datetime) struct CalendarProjection {
    year: CalendarYear,
    #[cfg(test)]
    extended_year: i32,
    month: Month,
    day: u8,
    weekday: u8,
}

impl CalendarProjection {
    pub(in crate::provider::datetime) fn from_iso(
        calendar: CalendarId,
        fields: DateTimeIsoFields,
        kernels: &CalendarKernels,
    ) -> Result<Self, DateTimeFormatError> {
        // The caller owns TimeClip / Temporal input-kind limits. This adapter
        // rejects malformed ISO days, without inventing a universal range claim.
        let iso = Date::try_new_iso(fields.year, fields.month, fields.day)
            .map_err(|_| DateTimeFormatError::InvalidRequest("invalid local ISO date"))?;
        let weekday = iso.day_of_week() as u8 % 7;
        let date = iso.to_calendar(kernels.get(calendar));
        let year_info = if calendar == CalendarId::Iso8601
            || (calendar == CalendarId::Japanese && fields.year < 1873)
        {
            // ISO formatting uses Gregorian labels. The Japanese contract uses
            // Gregorian eras through 1872-12-31, despite native Meiji starting1868.
            iso.to_calendar(Gregorian).year()
        } else {
            date.year()
        };
        let year = match year_info {
            YearInfo::Era(era) if !matches!(calendar, CalendarId::Chinese | CalendarId::Dangi) => {
                CalendarYear::Era {
                    name: EraName::from_native(calendar, era.era.as_str())?,
                    year: era.year,
                }
            }
            YearInfo::Cyclic(cyclic)
                if matches!(calendar, CalendarId::Chinese | CalendarId::Dangi)
                    && (1..=60).contains(&cyclic.year) =>
            {
                CalendarYear::Cyclic {
                    year: cyclic.year,
                    related_iso: cyclic.related_iso,
                }
            }
            // ICU YearInfo is non-exhaustive; unknown future kinds fail here.
            _ => {
                return Err(DateTimeFormatError::InvalidProfile(
                    "unexpected native calendar year kind".into(),
                ));
            }
        };
        let month = Month::from_native(calendar, date.month(), date.months_in_year())?;
        let day = date.day_of_month().0;
        if !(1..=date.days_in_month()).contains(&day) {
            return Err(DateTimeFormatError::InvalidProfile(
                "invalid native calendar day".into(),
            ));
        }
        Ok(Self {
            year,
            #[cfg(test)]
            extended_year: date.extended_year(),
            month,
            day,
            weekday,
        })
    }

    pub(in crate::provider::datetime) const fn year(self) -> CalendarYear {
        self.year
    }
    #[cfg(test)]
    pub(in crate::provider::datetime) const fn extended_year(self) -> i32 {
        self.extended_year
    }
    #[cfg(test)]
    pub(in crate::provider::datetime) const fn code_year(self) -> CodeYear {
        match self.year {
            CalendarYear::Cyclic { related_iso, .. } => CodeYear::RelatedIso(related_iso),
            CalendarYear::Era { .. } => CodeYear::Extended(self.extended_year),
        }
    }
    pub(in crate::provider::datetime) const fn month(self) -> Month {
        self.month
    }
    pub(in crate::provider::datetime) const fn day(self) -> u8 {
        self.day
    }
    pub(in crate::provider::datetime) const fn weekday(self) -> u8 {
        self.weekday
    }
}
