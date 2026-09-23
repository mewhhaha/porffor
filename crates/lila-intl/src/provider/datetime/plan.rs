use crate::datetime::*;
use crate::TimeZoneSelection;

use super::{
    locale,
    pattern::{Glue, Pattern},
    profile::{Calendar, Locale, Profile},
};

pub(super) mod adjustment;
mod candidates;
pub(super) mod components;

use components::{components, Required};

#[derive(Clone)]
pub(super) struct Format {
    pub(super) pattern: Pattern,
    pub(super) components: DateTimeComponents,
    pub(super) date: Option<Pattern>,
    pub(super) time: Option<Pattern>,
    pub(super) range_glue: Option<Glue>,
}
pub(super) struct ValidatedPlan<'p> {
    pub(super) locale: &'p Locale,
    pub(super) calendar: &'p Calendar,
    pub(super) calendar_kind: DateTimeCalendar,
    pub(super) numbering: String,
    pub(super) time_zone: TimeZoneSelection,
    pub(super) hour_cycle: DateTimeHourCycle,
    formats: [Option<Format>; 7],
}
impl ValidatedPlan<'_> {
    pub(super) fn available_formats(&self) -> DateTimeFormatAvailability {
        DateTimeFormatAvailability::from_available_kinds(
            [
                DateTimeValueKind::PlainDate,
                DateTimeValueKind::PlainYearMonth,
                DateTimeValueKind::PlainMonthDay,
                DateTimeValueKind::PlainTime,
                DateTimeValueKind::PlainDateTime,
            ]
            .into_iter()
            .filter(|kind| self.formats[index(*kind)].is_some()),
        )
    }
    pub(super) fn format(&self, kind: DateTimeValueKind) -> Result<&Format, DateTimeFormatError> {
        self.formats[index(kind)]
            .as_ref()
            .ok_or(DateTimeFormatError::UnavailableFormat)
    }
    pub(super) fn legacy_components(&self) -> DateTimeComponents {
        // Selection always constructs the legacy entry before publication.
        self.formats[0]
            .as_ref()
            .expect("validated plan has its legacy format")
            .components
    }
}
fn index(kind: DateTimeValueKind) -> usize {
    match kind {
        DateTimeValueKind::Legacy => 0,
        DateTimeValueKind::Instant => 1,
        DateTimeValueKind::PlainDate => 2,
        DateTimeValueKind::PlainYearMonth => 3,
        DateTimeValueKind::PlainMonthDay => 4,
        DateTimeValueKind::PlainTime => 5,
        DateTimeValueKind::PlainDateTime => 6,
    }
}

pub(super) fn select<'p>(
    profile: &'p Profile,
    request: &DateTimePlanRequest,
) -> Result<ValidatedPlan<'p>, DateTimeFormatError> {
    let locale = locale::validate(profile, &request.locale)?;
    let calendar = locale.calendar(request.locale.calendar);
    match request.matcher {
        DateTimeFormatMatcher::Basic | DateTimeFormatMatcher::BestFit => {}
    }
    match (request.required, request.defaults) {
        (DateTimeRequired::Any, _)
        | (DateTimeRequired::Date, DateTimeDefaults::Date)
        | (DateTimeRequired::Time, DateTimeDefaults::Time) => {}
        _ => {
            return Err(DateTimeFormatError::InvalidRequest(
                "inconsistent required/defaults caller profile",
            ));
        }
    }
    let candidates = candidates::Candidates::new(
        locale,
        calendar,
        request.locale.hour_cycle,
        request.locale.numbering_system.as_str(),
    )?;
    let formats = match request.selection {
        DateTimeStyleSelection::Components(options) => {
            let required = match request.required {
                DateTimeRequired::Any => Required::Any,
                DateTimeRequired::Date => Required::Date,
                DateTimeRequired::Time => Required::Time,
            };
            let pick =
                |required, inherit_all, defaults| -> Result<Option<Format>, DateTimeFormatError> {
                    components::defaults(options, required, inherit_all, defaults)
                        .map(|fields| candidates.best(fields))
                        .transpose()
                };
            [
                pick(required, true, request.defaults)?,
                pick(Required::Any, true, DateTimeDefaults::All)?,
                pick(Required::Date, false, DateTimeDefaults::Date)?,
                pick(Required::YearMonth, false, DateTimeDefaults::Date)?,
                pick(Required::MonthDay, false, DateTimeDefaults::Date)?,
                pick(Required::Time, false, DateTimeDefaults::Time)?,
                pick(Required::Any, false, DateTimeDefaults::All)?,
            ]
        }
        DateTimeStyleSelection::Styles(styles) => {
            if (matches!(request.required, DateTimeRequired::Date) && styles.time().is_some())
                || (matches!(request.required, DateTimeRequired::Time) && styles.date().is_some())
            {
                return Err(DateTimeFormatError::InvalidRequest(
                    "style conflicts with caller required fields",
                ));
            }
            let best = candidates.style(styles)?;
            let adjust = |required| {
                let filtered = components::filter(best.components, required);
                if filtered == best.components {
                    Ok(best.clone())
                } else {
                    candidates.best(filtered)
                }
            };
            [
                Some(best.clone()),
                Some(best.clone()),
                if styles.date().is_some() {
                    Some(adjust(Required::Date)?)
                } else {
                    None
                },
                if styles.date().is_some() {
                    Some(adjust(Required::YearMonth)?)
                } else {
                    None
                },
                if styles.date().is_some() {
                    Some(adjust(Required::MonthDay)?)
                } else {
                    None
                },
                if styles.time().is_some() {
                    Some(adjust(Required::Time)?)
                } else {
                    None
                },
                Some(adjust(Required::Any)?),
            ]
        }
    };
    if formats[index(DateTimeValueKind::Legacy)].is_none()
        || formats[index(DateTimeValueKind::Instant)].is_none()
    {
        return Err(super::profile::invalid(
            "mandatory exact-input plan is absent",
        ));
    }
    Ok(ValidatedPlan {
        locale,
        calendar,
        calendar_kind: request.locale.calendar,
        numbering: request.locale.numbering_system.as_str().to_owned(),
        time_zone: request.time_zone.clone(),
        hour_cycle: request.locale.hour_cycle,
        formats,
    })
}

pub(super) fn plain(pattern: Pattern) -> Format {
    Format {
        components: components(&pattern),
        pattern,
        date: None,
        time: None,
        range_glue: None,
    }
}
