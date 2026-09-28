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
    pub(super) algorithmic: BTreeMap<String, AlgorithmicField>,
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
    Coptic,
    Ethioaa,
    Ethiopic,
    Hebrew,
    IslamicTabular,
    IslamicUmmAlQura,
    Japanese,
}

/// How a calendar's year field is represented and named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum YearKind {
    /// Era-numbered years; the slice lists every CLDR era type it can produce.
    Eras(&'static [u8]),
    /// Sexagenary years with a related ISO year and leap months.
    Cyclic,
}

pub(super) enum AlgorithmicField {
    FiniteDays(Vec<String>),
    /// CLDR's `jpanyear`: year one is 元; other integers use Latin digits.
    JapaneseYearOne(String),
}

impl CalendarData {
    pub(super) const ALL: [Self; 15] = [
        Self::Gregorian,
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
            Self::Coptic => "coptic",
            Self::Ethioaa => "ethiopic-amete-alem",
            Self::Ethiopic => "ethiopic",
            Self::Hebrew => "hebrew",
            Self::IslamicTabular => "islamic-tbla",
            Self::IslamicUmmAlQura => "islamic-umalqura",
            Self::Japanese => "japanese",
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
            Self::Coptic => 8,
            Self::Ethioaa => 9,
            Self::Ethiopic => 10,
            Self::Hebrew => 11,
            Self::IslamicTabular => 12,
            Self::IslamicUmmAlQura => 13,
            Self::Japanese => 14,
        }
    }

    /// CLDR47 era types reachable from the pinned ICU4X arithmetic; see
    /// `calendar::convert` for the era-code mapping.
    pub(super) const fn years(self) -> YearKind {
        match self {
            Self::Gregorian
            | Self::Roc
            | Self::IslamicCivil
            | Self::IslamicTabular
            | Self::IslamicUmmAlQura
            | Self::Ethiopic => YearKind::Eras(&[0, 1]),
            Self::Buddhist
            | Self::Indian
            | Self::Persian
            | Self::Coptic
            | Self::Ethioaa
            | Self::Hebrew => YearKind::Eras(&[0]),
            Self::Japanese => YearKind::Eras(&[232, 233, 234, 235, 236, 237, 238]),
            Self::Chinese | Self::Dangi => YearKind::Cyclic,
        }
    }

    pub(super) const fn months(self) -> u8 {
        match self {
            Self::Coptic | Self::Ethioaa | Self::Ethiopic | Self::Hebrew => 13,
            Self::Gregorian
            | Self::Chinese
            | Self::Buddhist
            | Self::Indian
            | Self::Persian
            | Self::Roc
            | Self::Dangi
            | Self::IslamicCivil
            | Self::IslamicTabular
            | Self::IslamicUmmAlQura
            | Self::Japanese => 12,
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
            Self::Coptic => CalendarData::Coptic,
            Self::Ethioaa => CalendarData::Ethioaa,
            Self::Ethiopic => CalendarData::Ethiopic,
            Self::Hebrew => CalendarData::Hebrew,
            Self::IslamicTabular => CalendarData::IslamicTabular,
            Self::IslamicUmmAlQura => CalendarData::IslamicUmmAlQura,
            Self::Japanese => CalendarData::Japanese,
        }
    }
}
pub(super) struct Calendar {
    pub(super) names: FieldNames,
    pub(super) styles: [Style; 4],
    pub(super) available: Vec<Pattern>,
    pub(super) range_styles: [Style; 4],
    pub(super) range_available: Vec<Pattern>,
    pub(super) intervals: Vec<Interval>,
    pub(super) interval_fallback: Glue,
    pub(super) append_zone: Glue,
    pub(super) append_era: Option<Glue>,
}
#[derive(Clone)]
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
            || raw.pattern_context != "scalar_ascii"
            || selector.schema_version != 1
            || selector.release != "47.0.0"
            || selector.commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || selector.minimum_draft != "contributed"
            || selector.alt_selection
                != "ascii date/time patterns when supplied; default names; short territory for generic location names"
            || selector.calendar_identifiers.len() != DateTimeCalendar::ALL.len()
            || !selector
                .calendars
                .iter()
                .map(String::as_str)
                .eq(DateTimeCalendar::ALL
                    .iter()
                    .map(|calendar| calendar.as_str()))
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
            if row.minimum != 1
                || row.values.iter().any(String::is_empty)
                || row.source.is_empty()
                || row.ruleset.is_empty()
                || row.consumed_rules.is_empty()
                || digits.contains_key(&row.identifier)
            {
                return Err(invalid("invalid finite algorithmic date field"));
            }
            let field = match (row.field, row.method.as_str(), row.identifier.as_str()) {
                ('d', "finite", _) if row.values.len() == 31 => {
                    AlgorithmicField::FiniteDays(row.values)
                }
                ('y', "one_replaced_latin", "jpanyear")
                    if row.values.len() == 1
                        && row.source == "common/rbnf/ja.xml"
                        && row.ruleset == "spellout-numbering-year-latn" =>
                {
                    AlgorithmicField::JapaneseYearOne(row.values.into_iter().next().unwrap())
                }
                _ => return Err(invalid("unsupported algorithmic date field")),
            };
            if algorithmic.insert(row.identifier, field).is_some() {
                return Err(invalid("duplicate algorithmic date field"));
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
    pub(super) fn with_range_patterns(mut self, source: &str) -> Result<Self, DateTimeFormatError> {
        let raw: raw::RangePatterns = serde_json::from_str(source)
            .map_err(|error| DateTimeFormatError::InvalidProfile(error.to_string()))?;
        if raw.schema_version != 1
            || raw.pattern_context != "range_default"
            || raw.overrides.is_empty()
        {
            return Err(invalid("invalid range-pattern context"));
        }
        let mut keys = BTreeSet::new();
        for row in raw.overrides {
            if !keys.insert((row.locale.clone(), row.calendar.clone())) {
                return Err(invalid("duplicate range-pattern calendar"));
            }
            let locale = self
                .locales
                .iter_mut()
                .find(|locale| locale.identifier.as_str() == row.locale)
                .ok_or_else(|| invalid("unknown range-pattern locale"))?;
            let data = CalendarData::ALL
                .into_iter()
                .find(|data| data.source() == row.calendar)
                .ok_or_else(|| invalid("unknown range-pattern calendar"))?;
            locale.calendars[data.index()].apply_range_overrides(
                row,
                &self.digits,
                &self.algorithmic,
            )?;
        }
        Ok(self)
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
