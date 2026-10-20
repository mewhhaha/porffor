//! Only this owner publishes locale proofs after complete nine-profile admission.
use super::*;
use crate::list_image::projection::ListCatalogue;
use icu_list::options::{ListFormatterOptions, ListLength};
use icu_list::provider::{ListAndV1, ListOrV1, ListUnitV1};
use icu_list::{ListFormatter, ListFormatterPreferences};
use icu_provider::prelude::*;
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu_provider_blob::BlobDataProvider;
use std::cell::RefCell;

/// Capture metadata from the exact load consumed by ListFormatter. There is no
/// separate lookup whose effective locale could become detached from its data.
struct AdmissionProvider<'a, P: ?Sized> {
    provider: &'a P,
    effective: RefCell<Option<DataLocale>>,
}
impl<M: DataMarker, P: ?Sized> DataProvider<M> for AdmissionProvider<'_, P>
where
    P: DataProvider<M>,
{
    fn load(&self, request: DataRequest) -> Result<DataResponse<M>, DataError> {
        let response = <P as DataProvider<M>>::load(self.provider, request)?;
        let mut effective = self.effective.borrow_mut();
        if effective.is_some() {
            return Err(DataError::custom("multiple ListFormatter admission loads"));
        }
        *effective = Some(
            response
                .metadata
                .locale
                .clone()
                .unwrap_or_else(|| request.id.locale.clone()),
        );
        Ok(response)
    }
}

#[derive(Debug)]
pub(super) struct ListProfile {
    locale: CanonicalLocaleId,
    formatters: [ListFormatter; 9],
    #[cfg(test)]
    effective_locales: [CanonicalLocaleId; 9],
}
impl ListProfile {
    pub(super) fn locale(&self) -> &CanonicalLocaleId {
        &self.locale
    }
    pub(super) fn formatter(&self, kind: ListType, style: ListStyle) -> &ListFormatter {
        &self.formatters[profile_index(kind, style)]
    }
}
const fn profile_index(kind: ListType, style: ListStyle) -> usize {
    let base = match kind {
        ListType::Conjunction => 0,
        ListType::Disjunction => 3,
        ListType::Unit => 6,
    };
    base + match style {
        ListStyle::Long => 0,
        ListStyle::Short => 1,
        ListStyle::Narrow => 2,
    }
}
#[derive(Debug)]
pub struct ListProfiles {
    profiles: Box<[Arc<ListProfile>]>,
    public_profiles: Box<[usize]>,
    duration_profiles: Box<[(CanonicalLocaleId, usize)]>,
}
pub fn embedded_list_profiles() -> Result<&'static ListProfiles, ListFormatOperationError> {
    crate::list_image::embedded_list_data_image_ref()
        .map(|image| image.profiles_ref())
        .map_err(data_error)
}
fn data_error(error: impl fmt::Display) -> ListFormatOperationError {
    ListFormatOperationError::Data(error.to_string().into_boxed_str())
}
impl ListProfiles {
    pub(crate) fn from_image_data(
        image: &BlobDataProvider,
        locale_image: &crate::locale_image::LocaleDataImage,
    ) -> Result<Self, ListFormatOperationError> {
        let provider =
            LocaleFallbackProvider::new(image.as_deserializing(), locale_image.fallbacker());
        let catalogue = ListCatalogue::full(locale_image).map_err(data_error)?;
        Self::from_data_provider(&provider, &catalogue)
    }

    pub(crate) fn from_data_provider<P>(
        provider: &P,
        catalogue: &ListCatalogue,
    ) -> Result<Self, ListFormatOperationError>
    where
        P: DataProvider<ListAndV1> + DataProvider<ListOrV1> + DataProvider<ListUnitV1>,
    {
        let mut profiles = Vec::new();
        profiles
            .try_reserve_exact(catalogue.required_locales().len())
            .map_err(|_| ListFormatOperationError::Resource("locale catalogue allocation"))?;
        for locale in catalogue.required_locales() {
            let parsed: icu_locale::Locale = locale.as_str().parse().map_err(data_error)?;
            let prefs: ListFormatterPreferences = parsed.into();
            let mut formatters = Vec::new();
            #[cfg(test)]
            let mut effective = Vec::new();
            for &kind in ListType::ALL {
                for &style in ListStyle::ALL {
                    let options = ListFormatterOptions::default().with_length(match style {
                        ListStyle::Long => ListLength::Wide,
                        ListStyle::Short => ListLength::Short,
                        ListStyle::Narrow => ListLength::Narrow,
                    });
                    macro_rules! profile {
                        ($constructor:ident) => {{
                            let admission = AdmissionProvider {
                                provider,
                                effective: RefCell::new(None),
                            };
                            let formatter = ListFormatter::$constructor(&admission, prefs, options)
                                .map_err(data_error)?;
                            let actual = admission.effective.into_inner().ok_or_else(|| {
                                data_error("missing ListFormatter admission load")
                            })?;
                            (
                                formatter,
                                CanonicalLocaleId::from_data(actual.to_string())
                                    .map_err(data_error)?,
                            )
                        }};
                    }
                    let (formatter, _actual) = match kind {
                        ListType::Conjunction => profile!(try_new_and_unstable),
                        ListType::Disjunction => profile!(try_new_or_unstable),
                        ListType::Unit => profile!(try_new_unit_unstable),
                    };
                    formatters.push(formatter);
                    #[cfg(test)]
                    effective.push(_actual);
                }
            }
            profiles.push(Arc::new(ListProfile {
                locale: locale.clone(),
                formatters: formatters
                    .try_into()
                    .map_err(|_| ListFormatOperationError::InvalidPartition)?,
                #[cfg(test)]
                effective_locales: effective
                    .try_into()
                    .map_err(|_| ListFormatOperationError::InvalidPartition)?,
            }));
        }
        let locate = |locale: &CanonicalLocaleId| {
            profiles
                .binary_search_by(|profile| profile.locale().as_str().cmp(locale.as_str()))
                .map_err(|_| ListFormatOperationError::InvalidResolvedLocale)
        };
        let public_profiles = catalogue
            .public_locales()
            .iter()
            .map(|locale| locate(locale))
            .collect::<Result<Vec<_>, _>>()?;
        let duration_profiles = catalogue
            .duration_associations()
            .iter()
            .map(|(requested, resolved)| Ok((requested.clone(), locate(resolved)?)))
            .collect::<Result<Vec<_>, ListFormatOperationError>>()?;
        let result = Self {
            profiles: profiles.into_boxed_slice(),
            public_profiles: public_profiles.into_boxed_slice(),
            duration_profiles: duration_profiles.into_boxed_slice(),
        };
        if result.available_locales().len() == 0 {
            return Err(data_error("empty locale catalogue"));
        }
        result.admit(CanonicalLocaleId::from_data("en-US").map_err(data_error)?)?;
        Ok(result)
    }
    pub fn available_locales(&self) -> impl ExactSizeIterator<Item = &CanonicalLocaleId> {
        self.public_profiles
            .iter()
            .map(|&index| self.profiles[index].locale())
    }
    #[cfg(test)]
    pub(super) fn effective_locale<'a>(
        &self,
        locale: &'a ResolvedListLocale,
        kind: ListType,
        style: ListStyle,
    ) -> &'a CanonicalLocaleId {
        // Both locale and effective association belong to the same immutable Arc.
        &locale.profile.effective_locales[profile_index(kind, style)]
    }
    pub(crate) fn admit(
        &self,
        locale: CanonicalLocaleId,
    ) -> Result<ResolvedListLocale, ListFormatOperationError> {
        let public = self
            .public_profiles
            .binary_search_by(|&index| self.profiles[index].locale().as_str().cmp(locale.as_str()))
            .map_err(|_| ListFormatOperationError::InvalidResolvedLocale)?;
        Ok(ResolvedListLocale {
            profile: Arc::clone(&self.profiles[self.public_profiles[public]]),
        })
    }
    /// Only the native Duration association domain can acquire a nonpublic
    /// profile. It retains the very same profile Arc used by this selected owner.
    pub(crate) fn resolve_duration_locale(
        &self,
        requested: &CanonicalLocaleId,
    ) -> Result<ResolvedListLocale, ListFormatOperationError> {
        let index = self
            .duration_profiles
            .binary_search_by(|(locale, _)| locale.as_str().cmp(requested.as_str()))
            .map_err(|_| ListFormatOperationError::InvalidResolvedLocale)?;
        Ok(ResolvedListLocale {
            profile: Arc::clone(&self.profiles[self.duration_profiles[index].1]),
        })
    }
    pub(crate) fn format_parts(
        &self,
        request: FormatListPartsRequest,
    ) -> Result<ListParts, ListFormatOperationError> {
        let retained = &request.configuration().locale().profile;
        let index = self
            .profiles
            .binary_search_by(|profile| profile.locale().as_str().cmp(retained.locale().as_str()))
            .map_err(|_| ListFormatOperationError::InvalidResolvedLocale)?;
        if !Arc::ptr_eq(&self.profiles[index], retained) {
            return Err(ListFormatOperationError::InvalidResolvedLocale);
        }
        format_list_parts(request)
    }
    fn matching(&self, requested: &str, matcher: LocaleMatcher) -> Option<usize> {
        // The permitted best-fit policy is the same prefix search over THIS actual set.
        match matcher {
            LocaleMatcher::Lookup | LocaleMatcher::BestFit => {}
        }
        ListCatalogue::matching(requested, |candidate| {
            self.public_profiles
                .binary_search_by(|&index| self.profiles[index].locale().as_str().cmp(candidate))
                .ok()
        })
        .map(|public| self.public_profiles[public])
    }
    pub(crate) fn resolve(
        &self,
        request: ListLocaleRequest,
    ) -> Result<ResolvedListLocale, ListFormatOperationError> {
        if let Some(index) = request
            .requested
            .iter()
            .find_map(|l| self.matching(l.as_str(), request.matcher))
        {
            return Ok(ResolvedListLocale {
                profile: Arc::clone(&self.profiles[index]),
            });
        }
        self.admit(CanonicalLocaleId::from_data("en-US").map_err(data_error)?)
    }
    pub(crate) fn supported(
        &self,
        request: ListSupportedLocalesRequest,
    ) -> Result<ListSupportedLocalesResult, ListFormatOperationError> {
        let mut result = Vec::new();
        result
            .try_reserve_exact(request.requested.len())
            .map_err(|_| ListFormatOperationError::Resource("supported locales allocation"))?;
        for locale in request.requested {
            if self.matching(locale.as_str(), request.matcher).is_some() {
                result.push(locale);
            }
        }
        Ok(ListSupportedLocalesResult {
            locales: result.into_boxed_slice(),
        })
    }
}
