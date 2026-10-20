use super::{raw, DurationError, DurationStyle, DurationUnit, DurationUnitStyle};
use crate::list_format::{
    CheckedListConfiguration, FormatListPartsRequest, ListPart, ListProfiles, ListStyle, ListType,
    ResolvedListLocale,
};
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{NumberLocaleRequest, NumberProfiles};
use crate::CanonicalLocaleId;
use std::sync::Arc;
pub(super) const LOCALES: [&str; 15] = [
    "ar",
    "ar-EG",
    "de",
    "en",
    "en-US",
    "es",
    "fr",
    "hi",
    "it",
    "ja",
    "ko",
    "sr",
    "zh",
    "zh-Hans",
    "zh-Hans-CN",
];
#[derive(Debug, Clone)]
pub(super) struct Locale {
    pub name: CanonicalLocaleId,
    pub list: ResolvedListLocale,
    pub hour_minute: Box<str>,
    pub minute_second: Box<str>,
    pub two_digit_hours: bool,
}
pub struct DurationProfiles {
    pub(super) locales: Box<[Arc<Locale>]>,
    pub(super) numbers: Arc<NumberProfiles>,
    pub(super) lists: Arc<ListProfiles>,
}
impl core::fmt::Debug for DurationProfiles {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DurationProfiles")
            .field("available_locales", &self.locales.len())
            .finish_non_exhaustive()
    }
}
fn placeholder(value: &str, slots: &[u8], optional: bool) -> Result<(), DurationError> {
    let mut rest = value.to_owned();
    if value.is_empty() {
        return Err(DurationError::InvalidProfile);
    }
    for slot in slots {
        let token = format!("{{{slot}}}");
        let n = value.matches(&token).count();
        if n != 1 && !(optional && n == 0) {
            return Err(DurationError::InvalidProfile);
        }
        rest = rest.replace(&token, "");
    }
    if rest.contains(['{', '}']) {
        return Err(DurationError::InvalidProfile);
    }
    Ok(())
}
fn digital(pattern: &raw::Pattern, skeleton: &str) -> Result<(), DurationError> {
    let mut fields = Vec::new();
    let mut widths = Vec::new();
    let mut separators = Vec::new();
    let mut chars = pattern.pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        if !matches!(ch, 'h' | 'H' | 'm' | 's') {
            return Err(DurationError::InvalidProfile);
        }
        let mut width = 1u8;
        while chars.peek() == Some(&ch) {
            width += 1;
            chars.next();
        }
        if width > 2 {
            return Err(DurationError::InvalidProfile);
        }
        fields.push(ch.to_ascii_lowercase());
        widths.push(width);
        if chars.peek().is_some() {
            let mut sep = String::new();
            while chars
                .peek()
                .is_some_and(|c| !matches!(c, 'h' | 'H' | 'm' | 's'))
            {
                let c = chars.next().unwrap();
                if c == '\'' || c.is_ascii_alphabetic() {
                    return Err(DurationError::InvalidProfile);
                }
                sep.push(c);
            }
            if sep.is_empty() {
                return Err(DurationError::InvalidProfile);
            }
            separators.push(sep);
        }
    }
    if fields.iter().collect::<String>() != skeleton
        || widths != pattern.widths
        || separators != pattern.separators
    {
        return Err(DurationError::InvalidProfile);
    }
    Ok(())
}
fn apply(template: &str, a: &str, b: &str) -> String {
    template.replace("{0}", a).replace("{1}", b)
}
fn list_source(patterns: &[String; 4], length: usize) -> String {
    if length == 2 {
        return apply(&patterns[0], "0", "1");
    }
    let mut result = apply(
        &patterns[3],
        &(length - 2).to_string(),
        &(length - 1).to_string(),
    );
    for index in (0..length - 2).rev() {
        result = apply(
            &patterns[if index == 0 { 1 } else { 2 }],
            &index.to_string(),
            &result,
        )
    }
    result
}
pub fn embedded_duration_profiles() -> Result<&'static DurationProfiles, DurationError> {
    crate::duration_image::embedded_duration_data_image_ref()
        .map(|image| image.profiles_ref())
        .map_err(|_| DurationError::InvalidProfile)
}
impl DurationProfiles {
    /// The exact existing native rows, also consumed by List projection closure.
    pub(crate) fn required_list_locales() -> &'static [&'static str] {
        &LOCALES
    }
    pub(crate) fn from_image_data(
        bytes: &[u8],
        numbers: &Arc<NumberProfiles>,
        lists: &Arc<ListProfiles>,
    ) -> Result<Self, DurationError> {
        Self::from_json(
            core::str::from_utf8(bytes).map_err(|_| DurationError::InvalidProfile)?,
            numbers,
            lists,
        )
    }
    pub(super) fn from_json(
        text: &str,
        numbers: &Arc<NumberProfiles>,
        lists: &Arc<ListProfiles>,
    ) -> Result<Self, DurationError> {
        Self::from_json_with(text, numbers, lists, &LOCALES)
    }
    pub(crate) fn from_projected_image_data(
        catalogue: &crate::duration_image::DurationCatalogue<'_>,
        numbers: &Arc<NumberProfiles>,
        lists: &Arc<ListProfiles>,
    ) -> Result<Self, DurationError> {
        // Only exact pinned-source rederivation can bind these bytes and names.
        // Every selected row still passes the complete original native decoder.
        let expected = catalogue
            .locales()
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>();
        Self::from_json_with(
            core::str::from_utf8(catalogue.bytes()).map_err(|_| DurationError::InvalidProfile)?,
            numbers,
            lists,
            &expected,
        )
    }
    fn from_json_with(
        text: &str,
        numbers: &Arc<NumberProfiles>,
        lists: &Arc<ListProfiles>,
        expected_locales: &[&str],
    ) -> Result<Self, DurationError> {
        let raw: raw::Profile =
            serde_json::from_str(text).map_err(|_| DurationError::InvalidProfile)?;
        if raw.schema != 1
            || raw.cldr_commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || raw.default_locale != "en-US"
            || raw.locales.len() != expected_locales.len()
        {
            return Err(DurationError::InvalidProfile);
        }
        let mut locales = Vec::new();
        for (row, &expected) in raw.locales.into_iter().zip(expected_locales) {
            if row.locale != expected || row.units.len() != 30 || row.lists.len() != 3 {
                return Err(DurationError::InvalidProfile);
            }
            // Every local association must resolve exactly through both consumed owners,
            // never through a service's default fallback. They remain immutable owners.
            let name = CanonicalLocaleId::from_data(row.locale.as_str())
                .map_err(|_| DurationError::InvalidLocale)?;
            let number = crate::number_format::resolve_number_locale_in(
                &NumberLocaleRequest {
                    requested: vec![name.clone()].into_boxed_slice(),
                    matcher: LocaleMatcher::Lookup,
                    numbering_system: None,
                },
                crate::number_format::NumberLocaleView::duration(numbers),
            )?;
            if number.formatting().as_str() != expected
                || !crate::number_format::NumberLocaleView::duration(numbers).contains(expected)
            {
                return Err(DurationError::InvalidLocale);
            }
            let list = lists.resolve_duration_locale(&name)?;
            let effective = list.resolved().as_str();
            if effective != expected
                && !expected
                    .strip_prefix(effective)
                    .is_some_and(|suffix| suffix.starts_with('-'))
            {
                return Err(DurationError::InvalidLocale);
            }
            if matches!(expected, "es" | "sr") && effective != expected {
                return Err(DurationError::InvalidLocale);
            }
            for (index, unit) in row.units.iter().enumerate() {
                if DurationUnit::parse(&unit.unit) != Some(DurationUnit::ALL[index / 3])
                    || DurationUnitStyle::parse(&unit.style)
                        != Some(DurationUnitStyle::ALL[index % 3])
                {
                    return Err(DurationError::InvalidProfile);
                }
                for pattern in &unit.patterns {
                    placeholder(pattern, &[0], true)?;
                }
            }
            for (index, source) in row.lists.iter().enumerate() {
                let style = ListStyle::ALL[index];
                if source.style != DurationStyle::ALL[index].name() {
                    return Err(DurationError::InvalidProfile);
                }
                for template in &source.patterns {
                    placeholder(template, &[0, 1], false)?;
                }
                for length in 2..=4 {
                    let input = (0..length)
                        .map(|n| {
                            n.to_string()
                                .encode_utf16()
                                .collect::<Vec<_>>()
                                .into_boxed_slice()
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    let request = FormatListPartsRequest::new(
                        CheckedListConfiguration::new(list.clone(), ListType::Unit, style),
                        input,
                    )?;
                    let partition = lists.format_parts(request.clone())?;
                    let mut actual = String::new();
                    for part in partition.parts() {
                        match part {
                            ListPart::Literal(text) => actual.push_str(
                                &String::from_utf16(text)
                                    .map_err(|_| DurationError::InvalidProfile)?,
                            ),
                            ListPart::Element(index) => actual.push_str(
                                &String::from_utf16(&request.elements()[*index as usize])
                                    .map_err(|_| DurationError::InvalidProfile)?,
                            ),
                        }
                    }
                    if actual != list_source(&source.patterns, length) {
                        return Err(DurationError::InvalidProfile);
                    }
                }
            }
            digital(&row.digital.hm, "hm")?;
            digital(&row.digital.hms, "hms")?;
            digital(&row.digital.ms, "ms")?;
            let hm = &row.digital.hms.separators[0];
            let ms = &row.digital.hms.separators[1];
            if row.digital.hm.separators[0] != *hm
                || row.digital.ms.separators[0] != *ms
                || row.digital.hm.widths[0] != row.digital.hms.widths[0]
            {
                return Err(DurationError::InvalidProfile);
            }
            locales.push(Arc::new(Locale {
                name,
                list,
                hour_minute: hm.clone().into_boxed_str(),
                minute_second: ms.clone().into_boxed_str(),
                two_digit_hours: row.digital.hms.widths[0] == 2,
            }));
        }
        Ok(Self {
            locales: locales.into_boxed_slice(),
            numbers: Arc::clone(numbers),
            lists: Arc::clone(lists),
        })
    }
    pub(crate) fn uses_foundations(
        &self,
        numbers: &Arc<NumberProfiles>,
        lists: &Arc<ListProfiles>,
    ) -> bool {
        Arc::ptr_eq(&self.numbers, numbers) && Arc::ptr_eq(&self.lists, lists)
    }
    pub(crate) fn ensure_numbers(
        &self,
        numbers: &Arc<NumberProfiles>,
    ) -> Result<(), DurationError> {
        if !Arc::ptr_eq(&self.numbers, numbers) {
            return Err(DurationError::InvalidLocale);
        }
        Ok(())
    }
    pub(crate) fn format_parts(
        &self,
        configuration: &super::CheckedDurationConfiguration,
        record: &super::DurationRecord,
        limits: &crate::number_format::PartitionLimits,
    ) -> Result<super::DurationPartition, DurationError> {
        // One selected catalogue owns the exact retained template, Number and
        // List foundations before empty, numeric or textual output can branch.
        self.ensure_configuration(configuration)?;
        super::partition::format(configuration, record, &self.numbers, &self.lists, limits)
    }
    pub(crate) fn ensure_configuration(
        &self,
        configuration: &super::CheckedDurationConfiguration,
    ) -> Result<(), DurationError> {
        let locale = configuration.locale();
        let index = self
            .locales
            .binary_search_by(|profile| profile.name.as_str().cmp(locale.profile.name.as_str()))
            .map_err(|_| DurationError::InvalidLocale)?;
        if !Arc::ptr_eq(&self.locales[index], &locale.profile) {
            return Err(DurationError::InvalidLocale);
        }
        locale.number.ensure_profiles(&self.numbers)?;
        Ok(())
    }
    pub fn available_locales(&self) -> impl ExactSizeIterator<Item = &CanonicalLocaleId> {
        self.locales.iter().map(|p| &p.name)
    }
    pub(super) fn matching(&self, requested: &str) -> Option<&Arc<Locale>> {
        let mut candidate = requested
            .split("-u-")
            .next()
            .unwrap_or(requested)
            .split("-x-")
            .next()
            .unwrap_or(requested);
        loop {
            if let Ok(index) = self
                .locales
                .binary_search_by(|p| p.name.as_str().cmp(candidate))
            {
                return Some(&self.locales[index]);
            }
            let end = candidate.rfind('-')?;
            candidate = &candidate[..end];
            if candidate
                .rsplit('-')
                .next()
                .is_some_and(|part| part.len() == 1)
            {
                candidate = &candidate[..candidate.rfind('-')?]
            }
        }
    }
}
