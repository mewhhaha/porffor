//! Actual native Duration template data, admitted with selected Number/List owners.

use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::number_format::NumberProfiles;
use crate::{
    CustomProfileId, DurationProfiles, IntlDataDigest, IntlDataImageError, IntlDataProfile,
    ListDataImage, ListProfiles, LocaleDataImage, LocaleId, NumberProfilesDataImage,
};
use sha2::{Digest, Sha256};
use std::sync::{Arc, OnceLock};

pub(crate) mod projection;
pub(crate) use projection::DurationCatalogue;

pub const INTL_DURATION_DATA_CUSTOM_SECTION: &str = "lila.intl-duration-data.v1";
// This describes the actual native JSON schema, rather than an ICU marker.
const MARKERS: &[&str] = &["lila/duration/profiles/v1"];
const PINNED_PAYLOAD: &[u8] = include_bytes!("duration_format/generated/profile.json");

struct AdmittedDurationData {
    envelope: DataImageEnvelope,
    profiles: Arc<DurationProfiles>,
    locale_digest: IntlDataDigest,
    number_digest: IntlDataDigest,
    list_digest: IntlDataDigest,
}

/// Exact native data admission and complete Number/List association precede
/// publication. The decoded catalogue retains both actual immutable owners.
#[derive(Clone)]
pub struct DurationDataImage(Arc<AdmittedDurationData>);
impl core::fmt::Debug for DurationDataImage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DurationDataImage")
            .field("profile", self.profile())
            .field("digest", &self.digest())
            .field(
                "available_locales",
                &self.0.profiles.available_locales().len(),
            )
            .finish_non_exhaustive()
    }
}
impl DurationDataImage {
    pub(crate) fn require_complete_source(&self) -> Result<(), IntlDataImageError> {
        if self.0.envelope.blob() != PINNED_PAYLOAD {
            return Err(IntlDataImageError::IncompleteConformance);
        }
        Ok(())
    }

    pub fn for_profile(
        profile: IntlDataProfile,
        locale: &LocaleDataImage,
        numbers: &NumberProfilesDataImage,
        lists: &ListDataImage,
    ) -> Result<Self, IntlDataImageError> {
        Self::check_foundations(&profile, locale, numbers, lists)?;
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::DurationFormat,
                &profile,
                MARKERS,
                PINNED_PAYLOAD,
            )?,
            locale,
            numbers,
            lists,
        )
    }
    pub fn from_bytes(
        bytes: Arc<[u8]>,
        locale: &LocaleDataImage,
        numbers: &NumberProfilesDataImage,
        lists: &ListDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let envelope =
            DataImageEnvelope::decode(bytes, DataImageComponent::DurationFormat, MARKERS)?;
        Self::check_foundations(envelope.profile(), locale, numbers, lists)?;
        let profiles = if envelope.blob() == PINNED_PAYLOAD {
            if Sha256::digest(envelope.blob()).as_slice()
                != crate::duration_format::DURATION_PROFILE_SHA256
            {
                return Err(IntlDataImageError::consumer(
                    "Duration payload differs from the exact locked native data",
                ));
            }
            DurationProfiles::from_image_data(
                envelope.blob(),
                &numbers.profiles(),
                &lists.profiles(),
            )
        } else {
            let catalogue =
                projection::admit(envelope.blob(), envelope.profile(), locale, numbers, lists)?;
            DurationProfiles::from_projected_image_data(
                &catalogue,
                &numbers.profiles(),
                &lists.profiles(),
            )
        }
        .map_err(IntlDataImageError::consumer)?;
        if numbers.required_duration_locales().is_some_and(|required| {
            !required
                .iter()
                .map(|name| name.as_ref())
                .eq(profiles.available_locales().map(|name| name.as_str()))
        }) {
            return Err(IntlDataImageError::consumer(
                "Duration catalogue differs from the selected Number dependency domain",
            ));
        }
        Ok(Self(Arc::new(AdmittedDurationData {
            envelope,
            profiles: Arc::new(profiles),
            locale_digest: locale.digest(),
            number_digest: numbers.digest(),
            list_digest: lists.digest(),
        })))
    }

    /// Retain selected complete native locale rows and the actual en-US fallback.
    /// Number and List remain the supplied complete immutable foundations.
    pub fn for_custom_projection(
        id: &CustomProfileId,
        requested_locales: &[LocaleId],
        locale: &LocaleDataImage,
        numbers: &NumberProfilesDataImage,
        lists: &ListDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let payload = projection::produce(id, requested_locales, locale, numbers, lists)?;
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::DurationFormat,
                &IntlDataProfile::Custom(id.clone()),
                MARKERS,
                &payload,
            )?,
            locale,
            numbers,
            lists,
        )
    }
    fn check_foundations(
        profile: &IntlDataProfile,
        locale: &LocaleDataImage,
        numbers: &NumberProfilesDataImage,
        lists: &ListDataImage,
    ) -> Result<(), IntlDataImageError> {
        if profile != locale.profile()
            || profile != numbers.profile()
            || profile != lists.profile()
            || numbers.locale_digest() != locale.digest()
            || lists.locale_digest() != locale.digest()
            || numbers
                .list_digest()
                .is_some_and(|digest| digest != lists.digest())
        {
            return Err(IntlDataImageError::consumer(
                "Duration foundations have different profiles or Locale images",
            ));
        }
        Ok(())
    }
    pub fn bytes(&self) -> Arc<[u8]> {
        self.0.envelope.bytes()
    }
    pub fn digest(&self) -> IntlDataDigest {
        self.0.envelope.digest()
    }
    pub fn profile(&self) -> &IntlDataProfile {
        self.0.envelope.profile()
    }
    pub fn profiles(&self) -> Arc<DurationProfiles> {
        Arc::clone(&self.0.profiles)
    }
    pub(crate) fn profiles_ref(&self) -> &DurationProfiles {
        &self.0.profiles
    }
    pub(crate) fn locale_digest(&self) -> IntlDataDigest {
        self.0.locale_digest
    }
    pub(crate) fn number_digest(&self) -> IntlDataDigest {
        self.0.number_digest
    }
    pub(crate) fn list_digest(&self) -> IntlDataDigest {
        self.0.list_digest
    }
    pub(crate) fn uses_foundations(
        &self,
        numbers: &Arc<NumberProfiles>,
        lists: &Arc<ListProfiles>,
    ) -> bool {
        self.0.profiles.uses_foundations(numbers, lists)
    }
}

pub(crate) fn embedded_duration_data_image_ref(
) -> Result<&'static DurationDataImage, IntlDataImageError> {
    static IMAGE: OnceLock<Result<DurationDataImage, IntlDataImageError>> = OnceLock::new();
    IMAGE
        .get_or_init(|| {
            DurationDataImage::for_profile(
                IntlDataProfile::Minimal,
                &crate::embedded_locale_data_image()?,
                &crate::embedded_number_profiles_data_image()?,
                &crate::embedded_list_data_image()?,
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}
pub fn embedded_duration_data_image() -> Result<DurationDataImage, IntlDataImageError> {
    embedded_duration_data_image_ref().cloned()
}

#[cfg(test)]
mod tests;
