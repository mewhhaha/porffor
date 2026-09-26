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
    /// One checked data set per [`CalendarData`], indexed by its position.
    pub(super) calendars: [Calendar; CalendarData::ALL.len()],
    pub(super) zones: raw::ZoneNames,
}

/// A CLDR `<calendar type=…>` data set. ISO 8601 formats with Gregorian data;
/// every other DateTimeFormat calendar owns its own names and patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CalendarData {
    Gregorian,
    Chinese,
    Buddhist,
    Indian,
    Persian,
    Roc,
    Dangi,
    IslamicCivil,
}

/// How a calendar's year field is represented and named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum YearKind {
    /// Era-numbered years; the slice lists every CLDR era type it can produce.
    Eras(&'static [u8]),
    /// Sexagenary years with a related ISO year and leap months.
    Cyclic,
}

impl CalendarData {
    pub(super) const ALL: [Self; 8] = [
        Self::Gregorian,
        Self::Chinese,
        Self::Buddhist,
        Self::Indian,
        Self::Persian,
        Self::Roc,
        Self::Dangi,
        Self::IslamicCivil,
    ];

    /// The LDML calendar type selected by `selector.json`.
    pub(super) const fn source(self) -> &'static str {
        match self {
            Self::Gregorian => "gregorian",
            Self::Chinese => "chinese",
            Self::Buddhist => "buddhist",
            Self::Indian => "indian",
            Self::Persian => "persian",
            Self::Roc => "roc",
            Self::Dangi => "dangi",
            Self::IslamicCivil => "islamic-civil",
        }
    }

    pub(super) const fn index(self) -> usize {
        match self {
            Self::Gregorian => 0,
            Self::Chinese => 1,
            Self::Buddhist => 2,
            Self::Indian => 3,
            Self::Persian => 4,
            Self::Roc => 5,
            Self::Dangi => 6,
            Self::IslamicCivil => 7,
        }
    }

    /// CLDR47 era types reachable from the pinned ICU4X arithmetic; see
    /// `calendar::convert` for the era-code mapping.
    pub(super) const fn years(self) -> YearKind {
        match self {
            Self::Gregorian | Self::Roc | Self::IslamicCivil => YearKind::Eras(&[0, 1]),
            Self::Buddhist | Self::Indian | Self::Persian => YearKind::Eras(&[0]),
            Self::Chinese | Self::Dangi => YearKind::Cyclic,
        }
    }
}

const _: () = {
    let mut index = 0;
    while index < CalendarData::ALL.len() {
        assert!(CalendarData::ALL[index].index() == index);
        index += 1;
    }
};

impl DateTimeCalendar {
    pub(super) const fn data(self) -> CalendarData {
        match self {
            Self::Gregorian | Self::Iso8601 => CalendarData::Gregorian,
            Self::Chinese => CalendarData::Chinese,
            Self::Buddhist => CalendarData::Buddhist,
            Self::Indian => CalendarData::Indian,
            Self::Persian => CalendarData::Persian,
            Self::Roc => CalendarData::Roc,
            Self::Dangi => CalendarData::Dangi,
            Self::IslamicCivil => CalendarData::IslamicCivil,
        }
    }
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
        &self.calendars[calendar.data().index()]
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
            || selector.calendar_identifiers.len() != DateTimeCalendar::ALL.len()
            || !selector
                .calendars
                .iter()
                .map(String::as_str)
                .eq(DateTimeCalendar::ALL.iter().map(|calendar| calendar.as_str()))
            || DateTimeCalendar::ALL.iter().any(|calendar| {
                selector
                    .calendar_identifiers
                    .get(calendar.as_str())
                    .map(String::as_str)
                    != Some(calendar.data().source())
            })
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
