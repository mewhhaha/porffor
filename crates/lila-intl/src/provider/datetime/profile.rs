use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::datetime::{DateTimeCalendar, DateTimeFormatError, DateTimeHourCycle, DateTimeKeyword};
use crate::CanonicalLocaleId;

use super::names::{FieldNames, PeriodRules};
use super::pattern::{Glue, Pattern};
use super::raw;

mod construction;
mod pools;
mod recipe;
use super::calendar::CalendarId;
use recipe::ProfileRecipe;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
pub(super) enum CalendarDataKind {
    #[serde(rename = "gregorian")]
    Gregorian,
    #[serde(rename = "chinese")]
    Chinese,
    #[serde(rename = "buddhist")]
    Buddhist,
    #[serde(rename = "coptic")]
    Coptic,
    #[serde(rename = "dangi")]
    Dangi,
    #[serde(rename = "ethiopic-amete-alem")]
    Ethioaa,
    #[serde(rename = "ethiopic")]
    Ethiopic,
    #[serde(rename = "hebrew")]
    Hebrew,
    #[serde(rename = "indian")]
    Indian,
    #[serde(rename = "islamic-civil")]
    IslamicCivil,
    #[serde(rename = "islamic-tbla")]
    IslamicTbla,
    #[serde(rename = "islamic-umalqura")]
    IslamicUmalqura,
    #[serde(rename = "japanese")]
    Japanese,
    #[serde(rename = "persian")]
    Persian,
    #[serde(rename = "roc")]
    Roc,
}

impl CalendarDataKind {
    fn for_calendar(calendar: CalendarId) -> Self {
        match calendar {
            CalendarId::Gregory | CalendarId::Iso8601 => Self::Gregorian,
            CalendarId::Chinese => Self::Chinese,
            CalendarId::Buddhist => Self::Buddhist,
            CalendarId::Coptic => Self::Coptic,
            CalendarId::Dangi => Self::Dangi,
            CalendarId::Ethioaa => Self::Ethioaa,
            CalendarId::Ethiopic => Self::Ethiopic,
            CalendarId::Hebrew => Self::Hebrew,
            CalendarId::Indian => Self::Indian,
            CalendarId::IslamicCivil => Self::IslamicCivil,
            CalendarId::IslamicTbla => Self::IslamicTbla,
            CalendarId::IslamicUmalqura => Self::IslamicUmalqura,
            CalendarId::Japanese => Self::Japanese,
            CalendarId::Persian => Self::Persian,
            CalendarId::Roc => Self::Roc,
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Self::Gregorian => "gregorian",
            Self::Chinese => "chinese",
            Self::Buddhist => "buddhist",
            Self::Coptic => "coptic",
            Self::Dangi => "dangi",
            Self::Ethioaa => "ethiopic-amete-alem",
            Self::Ethiopic => "ethiopic",
            Self::Hebrew => "hebrew",
            Self::Indian => "indian",
            Self::IslamicCivil => "islamic-civil",
            Self::IslamicTbla => "islamic-tbla",
            Self::IslamicUmalqura => "islamic-umalqura",
            Self::Japanese => "japanese",
            Self::Persian => "persian",
            Self::Roc => "roc",
        }
    }
    pub(super) const fn month_names(self) -> u8 {
        match self {
            Self::Coptic | Self::Ethioaa | Self::Ethiopic | Self::Hebrew => 13,
            Self::Gregorian
            | Self::Chinese
            | Self::Buddhist
            | Self::Dangi
            | Self::Indian
            | Self::IslamicCivil
            | Self::IslamicTbla
            | Self::IslamicUmalqura
            | Self::Japanese
            | Self::Persian
            | Self::Roc => 12,
        }
    }
    pub(super) fn valid_era(self, index: u8, source: Option<super::names::EraSource>) -> bool {
        use super::names::EraSource;
        if let Some(EraSource::Gregorian) = source {
            return self == Self::Japanese && index <= 1;
        }
        match self {
            Self::Gregorian
            | Self::Ethiopic
            | Self::IslamicCivil
            | Self::IslamicTbla
            | Self::IslamicUmalqura
            | Self::Roc => index <= 1,
            Self::Buddhist | Self::Ethioaa | Self::Hebrew | Self::Indian | Self::Persian => {
                index == 0
            }
            Self::Coptic => index == 1,
            Self::Japanese => (232..=236).contains(&index),
            Self::Chinese | Self::Dangi => false,
        }
    }
}

pub(super) enum AlgorithmicField {
    FiniteDay(Vec<String>),
    JapaneseYear { first_year: String },
}
impl AlgorithmicField {
    fn admits(&self, field: Option<char>) -> bool {
        matches!(
            (self, field),
            (Self::FiniteDay(_), Some('d')) | (Self::JapaneseYear { .. }, Some('y'))
        )
    }
}

pub(super) struct Profile {
    pub(super) default_locale: String,
    pub(super) locales: Vec<Locale>,
    pub(super) digits: BTreeMap<String, [char; 10]>,
    pub(super) algorithmic: BTreeMap<String, AlgorithmicField>,
    pub(super) geography: super::zones::Geography,
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
    pub(super) first_weekday: raw::Weekday,
    calendars: pools::CalendarRecords,
    pub(super) zones: Arc<super::zones::ZoneNames>,
}
pub(super) struct Calendar {
    pub(super) kind: CalendarDataKind,
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
    #[cfg(test)]
    pub(super) fn calendar(&self, calendar: DateTimeCalendar) -> &Calendar {
        self.calendars
            .get(calendar)
            .expect("the complete source control owns this calendar")
    }
    pub(super) fn checked_calendar(&self, calendar: DateTimeCalendar) -> Option<&Calendar> {
        self.calendars.get(calendar)
    }
    pub(super) fn supports_calendar(&self, calendar: DateTimeCalendar) -> bool {
        self.calendars.get(calendar).is_some()
    }
    pub(super) fn calendars(&self) -> impl Iterator<Item = (CalendarId, &Calendar)> {
        self.calendars.iter()
    }
    #[cfg(test)]
    pub(super) fn foundation_calendar(&self, calendar: CalendarId) -> Option<&Calendar> {
        self.calendars.for_kind(calendar)
    }
}

impl Profile {
    pub(super) fn from_json(source: &str) -> Result<Self, DateTimeFormatError> {
        let raw: raw::Profile = serde_json::from_str(source)
            .map_err(|error| DateTimeFormatError::InvalidProfile(error.to_string()))?;
        let recipe =
            ProfileRecipe::from_raw(raw.schema_version, &raw.selector, &raw.era_supplement)?;
        Self::from_raw(raw, recipe)
    }

    pub(super) fn from_projection(
        catalogue: &crate::datetime_image::projection::DateTimeCatalogue<'_>,
    ) -> Result<Self, DateTimeFormatError> {
        let raw: raw::Profile = serde_json::from_slice(catalogue.bytes())
            .map_err(|error| DateTimeFormatError::InvalidProfile(error.to_string()))?;
        let recipe = ProfileRecipe::from_projection(
            raw.schema_version,
            &raw.selector,
            &raw.era_supplement,
            catalogue,
        )?;
        Self::from_raw(raw, recipe)
    }

    fn from_raw(raw: raw::Profile, recipe: ProfileRecipe) -> Result<Self, DateTimeFormatError> {
        let selector = &raw.selector;
        let supplement = &raw.numbering_supplement;
        if supplement.identifier != "tols"
            || supplement.cldr_release != "48.0.0"
            || supplement.cldr_commit != "acd6d88ae493633240e19a87a721076a8a75c310"
            || supplement.unicode_release != "17.0.0"
            || supplement.source_manifest_sha256
                != "c0b70b5f5ffc5940c8f47e97ec99bd9c5596e1b81613901bffaa9a80323dc62f"
        {
            return Err(invalid("unreviewed positional numbering supplement"));
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
        if digits.len() != 78 {
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
                return Err(invalid("invalid algorithmic date field"));
            }
            let value = match (row.field, row.positional_fallback) {
                ('d', None) if row.values.len() == 31 => AlgorithmicField::FiniteDay(row.values),
                ('y', Some(raw::PositionalFallback::Latin))
                    if row.identifier == "jpanyear"
                        && row.source == "common/rbnf/ja.xml"
                        && row.ruleset == "spellout-numbering-year-latn"
                        && row.values == ["元"]
                        && row.consumed_rules
                            == [
                                ("spellout-numbering-year-latn".into(), 0, "=0=".into()),
                                ("spellout-numbering-year-latn".into(), 1, "元".into()),
                                ("spellout-numbering-year-latn".into(), 2, "=0=".into()),
                            ] =>
                {
                    AlgorithmicField::JapaneseYear {
                        first_year: row
                            .values
                            .into_iter()
                            .next()
                            .ok_or_else(|| invalid("missing first year"))?,
                    }
                }
                _ => return Err(invalid("unreviewed algorithmic date-field recipe")),
            };
            if algorithmic.insert(row.identifier, value).is_some() {
                return Err(invalid("duplicate algorithmic date-field recipe"));
            }
        }
        let geography = super::zones::Geography::from_raw(raw.zone_geography)?;
        let mut pools = pools::Pools::from_raw(
            raw.calendar_pool,
            raw.zone_name_pool,
            &digits,
            &algorithmic,
            &geography,
            recipe,
        )?;
        let mut locales = Vec::new();
        let selected: BTreeSet<_> = selector.locales.iter().cloned().collect();
        if selected.len() != selector.locales.len() || raw.locales.len() != selected.len() {
            return Err(invalid("duplicate or incomplete selected locale inventory"));
        }
        for raw in raw.locales {
            locales.push(Locale::from_raw(raw, &digits, &mut pools)?);
        }
        pools.validate_coverage()?;
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
            geography,
        };
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
