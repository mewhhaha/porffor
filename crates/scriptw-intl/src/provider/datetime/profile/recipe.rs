use super::*;

pub(super) const CALENDARS: [CalendarId; 16] = [
    CalendarId::Gregory,
    CalendarId::Iso8601,
    CalendarId::Chinese,
    CalendarId::Buddhist,
    CalendarId::Coptic,
    CalendarId::Dangi,
    CalendarId::Ethioaa,
    CalendarId::Ethiopic,
    CalendarId::Hebrew,
    CalendarId::Indian,
    CalendarId::IslamicCivil,
    CalendarId::IslamicTbla,
    CalendarId::IslamicUmalqura,
    CalendarId::Japanese,
    CalendarId::Persian,
    CalendarId::Roc,
];
const NEW_LOCALES: [&str; 13] = [
    "en",
    "en-US",
    "ar",
    "ar-EG",
    "zh",
    "zh-Hans",
    "zh-Hans-CN",
    "de",
    "fr",
    "it",
    "ja",
    "ko",
    "hi",
];

/// A closed source-data recipe, independent of public calendar/wire admission.
#[derive(Clone)]
pub(super) struct ProfileRecipe {
    _checked: (),
    calendars: Box<[CalendarId]>,
    requested_calendars: Option<Box<[DateTimeCalendar]>>,
    numbering_systems: Option<Box<[crate::number_format::NumberingSystemOption]>>,
    localized_zones: Option<Box<[Box<str>]>>,
}

impl ProfileRecipe {
    pub(super) fn from_raw(
        version: u32,
        selector: &raw::Selector,
        eras: &raw::EraSupplement,
    ) -> Result<Self, DateTimeFormatError> {
        Self::from_selected_raw(version, selector, eras, &NEW_LOCALES, &CALENDARS)
    }

    pub(super) fn from_projection(
        version: u32,
        selector: &raw::Selector,
        eras: &raw::EraSupplement,
        catalogue: &crate::datetime_image::projection::DateTimeCatalogue<'_>,
    ) -> Result<Self, DateTimeFormatError> {
        let locales = catalogue
            .locales()
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>();
        let expected = match catalogue.calendar_types() {
            None => CALENDARS.to_vec(),
            Some(_) => catalogue
                .service_calendars()
                .iter()
                .copied()
                .map(CalendarId::from_admitted)
                .collect::<Vec<_>>(),
        };
        let mut recipe = Self::from_selected_raw(version, selector, eras, &locales, &expected)?;
        recipe.requested_calendars = catalogue
            .calendar_types()
            .map(|types| types.to_vec().into_boxed_slice());
        recipe.numbering_systems = catalogue
            .numbering_systems()
            .map(|systems| systems.to_vec().into_boxed_slice());
        recipe.localized_zones = catalogue
            .localized_zones()
            .map(|names| names.to_vec().into_boxed_slice());
        Ok(recipe)
    }

    fn from_selected_raw(
        version: u32,
        selector: &raw::Selector,
        eras: &raw::EraSupplement,
        locales: &[&str],
        expected: &[CalendarId],
    ) -> Result<Self, DateTimeFormatError> {
        if version != 2 || selector.schema_version != 1 || selector.release != "47.0.0"
            || selector.commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || selector.minimum_draft != "contributed" || selector.default_locale != "en-US"
            || selector.alt_selection != "ascii date/time patterns when supplied; default names; short territory for generic location names"
        {
            return Err(invalid("unreviewed date/time profile source identity"));
        }
        if eras.identifier != "selected-calendar-eras"
            || eras.cldr_release != "48.0.0"
            || eras.cldr_commit != "acd6d88ae493633240e19a87a721076a8a75c310"
            || eras.source_manifest_sha256
                != "d551300d46e7a64558d60bb2732b702902aa68ae60ee62284d0a67b314c25932"
        {
            return Err(invalid("unreviewed selected calendar-era supplement"));
        }
        if selector.calendars.len() != expected.len()
            || selector.calendar_identifiers.len() != expected.len()
            || selector
                .locales
                .iter()
                .map(String::as_str)
                .ne(locales.iter().copied())
            || selector
                .calendars
                .iter()
                .map(String::as_str)
                .ne(expected.iter().map(|kind| kind.as_str()))
            || expected.iter().any(|kind| {
                selector
                    .calendar_identifiers
                    .get(kind.as_str())
                    .map(String::as_str)
                    != Some(CalendarDataKind::for_calendar(*kind).as_str())
            })
        {
            return Err(invalid("unreviewed exact locale/calendar source recipe"));
        }
        Ok(Self {
            _checked: (),
            calendars: expected.to_vec().into_boxed_slice(),
            requested_calendars: None,
            numbering_systems: None,
            localized_zones: None,
        })
    }

    pub(super) fn calendars_for_locale(&self, default: DateTimeCalendar) -> Vec<CalendarId> {
        self.calendars
            .iter()
            .copied()
            .filter(|kind| match &self.requested_calendars {
                None => true,
                Some(requested) => {
                    *kind == CalendarId::from_admitted(default)
                        || requested
                            .iter()
                            .any(|calendar| *kind == CalendarId::from_admitted(*calendar))
                }
            })
            .collect()
    }
    pub(super) fn numbering_for_locale(
        &self,
        default: &str,
        digits: &BTreeMap<String, [char; 10]>,
    ) -> BTreeSet<String> {
        digits
            .keys()
            .filter(|name| {
                self.numbering_systems.as_ref().is_none_or(|systems| {
                    name.as_str() == default
                        || systems.iter().any(|value| value.name() == name.as_str())
                })
            })
            .cloned()
            .collect()
    }
    pub(super) fn localized_zones(&self) -> Option<&[Box<str>]> {
        self.localized_zones.as_deref()
    }
}
