use std::collections::{BTreeMap, BTreeSet};

use crate::datetime::{DateTimeCalendar, DateTimeFormatError, DateTimeHourCycle, DateTimeKeyword};
use crate::CanonicalLocaleId;

use super::names::{FieldNames, PeriodRules};
use super::pattern::{Glue, Pattern};
use super::raw;

mod construction;

pub(super) struct Profile {
    pub(super) default_locale: String,
    pub(super) locales: Vec<Locale>,
    pub(super) digits: BTreeMap<String, [char; 10]>,
    pub(super) algorithmic: BTreeMap<String, Vec<String>>,
    pub(super) geography: raw::Geography,
}
pub(super) struct Locale {
    pub(super) identifier: CanonicalLocaleId,
    pub(super) territory: String,
    pub(super) default_numbering: String,
    pub(super) decimal: BTreeMap<String, String>,
    pub(super) minus: BTreeMap<String, String>,
    pub(super) default_calendar: DateTimeCalendar,
    pub(super) hour_cycle: DateTimeHourCycle,
    pub(super) hour_cycle12: DateTimeHourCycle,
    pub(super) hour_cycle24: DateTimeHourCycle,
    pub(super) periods: PeriodRules,
    pub(super) gregorian: Calendar,
    pub(super) chinese: Calendar,
    pub(super) zones: raw::ZoneNames,
}
pub(super) struct Calendar {
    pub(super) names: FieldNames,
    pub(super) styles: [Style; 4],
    pub(super) available: Vec<Pattern>,
    pub(super) intervals: Vec<Interval>,
    pub(super) interval_fallback: Glue,
    pub(super) append_zone: Glue,
    pub(super) append_era: Option<Glue>,
}
pub(super) struct Style {
    pub(super) date: Pattern,
    pub(super) time: Pattern,
    pub(super) standard: Glue,
    pub(super) at_time: Glue,
}
pub(super) struct Interval {
    pub(super) difference: char,
    pub(super) pattern: Pattern,
    pub(super) second_start: usize,
    pub(super) latest_first: bool,
}

impl Locale {
    pub(super) fn calendar(&self, calendar: DateTimeCalendar) -> &Calendar {
        match calendar {
            DateTimeCalendar::Gregorian | DateTimeCalendar::Iso8601 => &self.gregorian,
            DateTimeCalendar::Chinese => &self.chinese,
        }
    }
}

impl Profile {
    pub(super) fn from_json(source: &str) -> Result<Self, DateTimeFormatError> {
        let raw: raw::Profile = serde_json::from_str(source)
            .map_err(|error| DateTimeFormatError::InvalidProfile(error.to_string()))?;
        let selector = &raw.selector;
        if raw.schema_version != 1
            || selector.schema_version != 1
            || selector.release != "47.0.0"
            || selector.commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || selector.minimum_draft != "contributed"
            || selector.alt_selection != "ascii date/time patterns when supplied; default names; short territory for generic location names"
            || selector.calendar_identifiers.len() != 3
            || selector.calendars != ["gregory", "iso8601", "chinese"]
            || selector
                .calendar_identifiers
                .get("gregory")
                .map(String::as_str)
                != Some("gregorian")
            || selector
                .calendar_identifiers
                .get("iso8601")
                .map(String::as_str)
                != Some("gregorian")
            || selector
                .calendar_identifiers
                .get("chinese")
                .map(String::as_str)
                != Some("chinese")
        {
            return Err(invalid("unreviewed date/time profile schema or recipe"));
        }
        let mut digits = BTreeMap::new();
        for row in raw.numbering_systems {
            let values: [char; 10] = row
                .digits
                .chars()
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| invalid("positional numbering does not have ten digits"))?;
            DateTimeKeyword::parse(row.identifier.clone())
                .map_err(|_| invalid("invalid numbering identifier"))?;
            if values.iter().collect::<BTreeSet<_>>().len() != 10
                || digits.insert(row.identifier, values).is_some()
            {
                return Err(invalid("duplicate numbering identifier or digit"));
            }
        }
        if digits.len() != 77 {
            return Err(invalid("positional numbering inventory changed"));
        }
        let mut algorithmic = BTreeMap::new();
        for row in raw.algorithmic_fields {
            if row.field != 'd'
                || row.minimum != 1
                || row.values.len() != 31
                || row.values.iter().any(String::is_empty)
                || row.source.is_empty()
                || row.ruleset.is_empty()
                || row.consumed_rules.is_empty()
                || digits.contains_key(&row.identifier)
                || algorithmic.insert(row.identifier, row.values).is_some()
            {
                return Err(invalid("invalid finite algorithmic date field"));
            }
        }
        let mut locales = Vec::new();
        let selected: BTreeSet<_> = selector.locales.iter().cloned().collect();
        if selected.len() != selector.locales.len() || raw.locales.len() != selected.len() {
            return Err(invalid("duplicate or incomplete selected locale inventory"));
        }
        for raw in raw.locales {
            locales.push(Locale::from_raw(raw, &digits, &algorithmic)?);
        }
        locales.sort_by(|left, right| left.identifier.as_str().cmp(right.identifier.as_str()));
        if locales
            .iter()
            .map(|locale| locale.identifier.as_str().to_owned())
            .collect::<BTreeSet<_>>()
            != selected
            || !selected.contains(&selector.default_locale)
        {
            return Err(invalid("locale inventory differs from its recipe"));
        }
        let result = Self {
            default_locale: selector.default_locale.clone(),
            locales,
            digits,
            algorithmic,
            geography: raw.zone_geography,
        };
        super::zones::validate(&result)?;
        result.validate_names()?;
        Ok(result)
    }
    pub(super) fn locale(&self, identifier: &str) -> Option<&Locale> {
        self.locales
            .binary_search_by(|locale| locale.identifier.as_str().cmp(identifier))
            .ok()
            .map(|index| &self.locales[index])
    }
}

pub(super) fn invalid(reason: impl Into<String>) -> DateTimeFormatError {
    DateTimeFormatError::InvalidProfile(reason.into())
}
