use super::{raw, RelativeStyle, RelativeTimeError, RelativeUnit};
use crate::number_format::options::LocaleMatcher;
use crate::number_format::{NumberFormatKernelError, NumberProfiles, PartitionLimits};
use crate::CanonicalLocaleId;
use std::sync::Arc;

const CLDR_COMMIT: &str = "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c";
const LOCALES: [&str; 14] = [
    "ar",
    "ar-EG",
    "de",
    "en",
    "en-US",
    "fr",
    "hi",
    "it",
    "ja",
    "ko",
    "pl",
    "zh",
    "zh-Hans",
    "zh-Hans-CN",
];

/// Pattern syntax is checked once; rendering never interprets unchecked braces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Pattern {
    Literal(Box<str>),
    Number { before: Box<str>, after: Box<str> },
}
impl Pattern {
    fn new(source: String) -> Result<Self, RelativeTimeError> {
        if source.is_empty() || source.contains('\0') {
            return Err(RelativeTimeError::InvalidProfile("empty/NUL pattern"));
        }
        if let Some((before, after)) = source.split_once("{0}") {
            if before.contains(['{', '}']) || after.contains(['{', '}']) {
                return Err(RelativeTimeError::InvalidProfile("pattern placeholder"));
            }
            Ok(Self::Number {
                before: before.into(),
                after: after.into(),
            })
        } else if source.contains(['{', '}']) {
            Err(RelativeTimeError::InvalidProfile("pattern placeholder"))
        } else {
            Ok(Self::Literal(source.into_boxed_str()))
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Field {
    pub past: [Pattern; 6],
    pub future: [Pattern; 6],
    pub relative: [Option<Box<str>>; 5],
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Locale {
    pub fields: [Field; 24],
}
impl Locale {
    pub fn field(&self, unit: RelativeUnit, style: RelativeStyle) -> &Field {
        &self.fields[unit.index() * 3 + style.index()]
    }
}

#[derive(Debug, Clone)]
pub struct RelativeProfiles {
    locale_names: Box<[&'static str]>,
    locales: Box<[Arc<Locale>]>,
    numbers: Arc<NumberProfiles>,
}
impl RelativeProfiles {
    pub(crate) fn from_json(
        source: &str,
        number_profiles: &Arc<NumberProfiles>,
    ) -> Result<Self, RelativeTimeError> {
        Self::from_json_with_locales(source, &LOCALES, number_profiles)
    }
    pub(crate) fn from_projected_json(
        source: &str,
        catalogue: &crate::relative_time_image::projection::RelativeCatalogue,
        number_profiles: &Arc<NumberProfiles>,
    ) -> Result<Self, RelativeTimeError> {
        Self::from_json_with_locales(source, catalogue.locales(), number_profiles)
    }
    fn from_json_with_locales(
        source: &str,
        locale_names: &[&'static str],
        number_profiles: &Arc<NumberProfiles>,
    ) -> Result<Self, RelativeTimeError> {
        let raw: raw::Profile = serde_json::from_str(source)
            .map_err(|_| RelativeTimeError::InvalidProfile("JSON schema"))?;
        if raw.schema != 1
            || raw.cldr_commit != CLDR_COMMIT
            || raw.locales.len() != locale_names.len()
        {
            return Err(RelativeTimeError::InvalidProfile("source recipe"));
        }
        let mut locales: Vec<Option<Arc<Locale>>> = vec![None; locale_names.len()];
        for locale in raw.locales {
            let slot = locale_names
                .binary_search(&locale.locale.as_str())
                .map_err(|_| RelativeTimeError::InvalidProfile("locale domain"))?;
            if locales[slot].is_some() || locale.fields.len() != 24 {
                return Err(RelativeTimeError::InvalidProfile(
                    "duplicate/incomplete locale",
                ));
            }
            let id = CanonicalLocaleId::from_data(locale.locale)
                .map_err(|_| RelativeTimeError::InvalidProfile("canonical locale"))?;
            // A relative-time locale needs the actual shared decimal/plural owner.
            crate::plural_rules::ResolvedPluralLocale::from_resolved_in(
                id.clone(),
                id.clone(),
                crate::number_format::NumberLocaleView::relative(number_profiles),
            )?;
            let mut fields: [Option<Field>; 24] = std::array::from_fn(|_| None);
            for field in locale.fields {
                let index = field.unit.index() * 3 + field.style.index();
                if fields[index].is_some() {
                    return Err(RelativeTimeError::InvalidProfile("duplicate unit/style"));
                }
                let patterns = |values: [String; 6]| -> Result<[Pattern; 6], RelativeTimeError> {
                    let result = values
                        .into_iter()
                        .map(Pattern::new)
                        .collect::<Result<Vec<_>, _>>()?;
                    result
                        .try_into()
                        .map_err(|_| RelativeTimeError::InvalidProfile("category domain"))
                };
                let mut relative = std::array::from_fn(|_| None);
                for item in field.relative {
                    if !(-2..=2).contains(&item.offset) {
                        return Err(RelativeTimeError::InvalidProfile("auto offset domain"));
                    }
                    let index = (item.offset + 2) as usize;
                    if relative[index].is_some()
                        || item.value.is_empty()
                        || item.value.contains(['{', '}', '\0'])
                    {
                        return Err(RelativeTimeError::InvalidProfile(
                            "duplicate/invalid auto literal",
                        ));
                    }
                    relative[index] = Some(item.value.into_boxed_str());
                }
                fields[index] = Some(Field {
                    past: patterns(field.past)?,
                    future: patterns(field.future)?,
                    relative,
                });
            }
            let fields = fields
                .into_iter()
                .collect::<Option<Vec<_>>>()
                .ok_or(RelativeTimeError::InvalidProfile(
                    "incomplete unit/style domain",
                ))?
                .try_into()
                .map_err(|_| RelativeTimeError::InvalidProfile("field domain"))?;
            locales[slot] = Some(Arc::new(Locale { fields }));
        }
        let locales = locales
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or(RelativeTimeError::InvalidProfile(
                "incomplete locale domain",
            ))?
            .into_boxed_slice();
        Ok(Self {
            locale_names: locale_names.to_vec().into_boxed_slice(),
            locales,
            numbers: Arc::clone(number_profiles),
        })
    }
    pub(crate) fn uses_number_profiles(&self, profiles: &Arc<NumberProfiles>) -> bool {
        Arc::ptr_eq(&self.numbers, profiles)
    }
    pub(super) fn number_profiles(&self) -> &Arc<NumberProfiles> {
        &self.numbers
    }
    pub(crate) fn ensure_number_profiles(
        &self,
        profiles: &Arc<NumberProfiles>,
    ) -> Result<(), RelativeTimeError> {
        if self.uses_number_profiles(profiles) {
            Ok(())
        } else {
            Err(NumberFormatKernelError::InvalidResolvedLocale.into())
        }
    }
    pub(super) fn ensure_locale(
        &self,
        locale: &super::ResolvedRelativeTimeLocale,
    ) -> Result<(), RelativeTimeError> {
        locale.ensure_owner(self, &self.numbers)
    }
    /// Consume only constructor state resolved from this admitted template owner.
    /// This check also precedes the numeric:auto literal return.
    pub fn format_parts(
        &self,
        configuration: &super::RelativeTimeConfiguration,
        value: super::FiniteRelativeNumber,
        unit: RelativeUnit,
        limits: &PartitionLimits,
    ) -> Result<super::RelativePartition, RelativeTimeError> {
        self.ensure_locale(configuration.locale())?;
        super::partition::partition_relative_time(configuration, value, unit, &self.numbers, limits)
    }
    pub fn available_locales(&self) -> &[&'static str] {
        &self.locale_names
    }
    pub(super) fn locale(&self, id: &str) -> Option<&Arc<Locale>> {
        self.locale_names
            .binary_search(&id)
            .ok()
            .map(|slot| &self.locales[slot])
    }
    pub(super) fn matching_locale(&self, requested: &str, matcher: LocaleMatcher) -> Option<&str> {
        match matcher {
            LocaleMatcher::Lookup | LocaleMatcher::BestFit => {}
        }
        let mut candidate = requested;
        loop {
            if let Ok(slot) = self.available_locales().binary_search(&candidate) {
                return Some(self.available_locales()[slot]);
            }
            candidate = &candidate[..candidate.rfind('-')?];
            if candidate
                .rsplit('-')
                .next()
                .is_some_and(|part| part.len() == 1)
            {
                candidate = &candidate[..candidate.rfind('-')?];
            }
        }
    }
    pub fn supported_locales(
        &self,
        requested: &[CanonicalLocaleId],
        matcher: LocaleMatcher,
        limits: &PartitionLimits,
    ) -> Result<Box<[CanonicalLocaleId]>, RelativeTimeError> {
        if requested.len() as u128 > u128::from(limits.part_count()) {
            return Err(RelativeTimeError::Resource("locale count"));
        }
        let mut result = Vec::new();
        result
            .try_reserve(requested.len())
            .map_err(|_| RelativeTimeError::Resource("locale allocation"))?;
        for locale in requested {
            if self.matching_locale(locale.as_str(), matcher).is_some() {
                result.push(super::configuration::canonical_copy(
                    locale.as_str(),
                    limits,
                )?);
            }
        }
        Ok(result.into_boxed_slice())
    }
}

pub fn embedded_relative_profiles() -> Result<&'static RelativeProfiles, RelativeTimeError> {
    crate::relative_time_image::embedded_relative_time_data_image_ref()
        .map(|image| image.profiles_ref())
        .map_err(|_| RelativeTimeError::InvalidProfile("image admission"))
}
