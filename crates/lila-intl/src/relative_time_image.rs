//! The consumed CLDR47 native RelativeTimeFormat templates and numeric owner.

use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::number_format::NumberProfiles;
use crate::{
    CustomProfileId, IntlDataDigest, IntlDataImageError, IntlDataProfile, LocaleDataImage,
    LocaleId, NumberProfilesDataImage, RelativeProfiles,
};
use std::sync::{Arc, OnceLock};

pub(crate) mod projection;

pub const INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION: &str = "lila.intl-relative-time.v1";
// A genuine Lila-native schema descriptor, rather than an ICU marker.
pub(crate) const RELATIVE_TIME_IMAGE_MARKERS: &[&str] = &["lila/relative-time/profiles/v1"];
const PINNED_PROFILE: &[u8] = include_bytes!("relative_time_format/generated/profile.json");

struct AdmittedRelativeTimeData {
    envelope: DataImageEnvelope,
    profiles: Arc<RelativeProfiles>,
    locale_digest: IntlDataDigest,
    number_digest: IntlDataDigest,
}

/// Admission decodes the complete native template catalogue and retains the
/// exact selected NumberFormat/PluralRules owner before exposing any consumer.
#[derive(Clone)]
pub struct RelativeTimeDataImage(Arc<AdmittedRelativeTimeData>);
impl core::fmt::Debug for RelativeTimeDataImage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RelativeTimeDataImage")
            .field("profile", self.profile())
            .field("digest", &self.digest())
            .finish_non_exhaustive()
    }
}
impl RelativeTimeDataImage {
    pub(crate) fn require_complete_source(&self) -> Result<(), IntlDataImageError> {
        if self.0.envelope.blob() != PINNED_PROFILE {
            return Err(IntlDataImageError::IncompleteConformance);
        }
        Ok(())
    }

    pub fn for_profile(
        profile: IntlDataProfile,
        locale: &LocaleDataImage,
        numbers: &NumberProfilesDataImage,
    ) -> Result<Self, IntlDataImageError> {
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::RelativeTime,
                &profile,
                RELATIVE_TIME_IMAGE_MARKERS,
                PINNED_PROFILE,
            )?,
            locale,
            numbers,
        )
    }
    pub fn from_bytes(
        bytes: Arc<[u8]>,
        locale: &LocaleDataImage,
        numbers: &NumberProfilesDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let envelope = DataImageEnvelope::decode(
            bytes,
            DataImageComponent::RelativeTime,
            RELATIVE_TIME_IMAGE_MARKERS,
        )?;
        if envelope.profile() != locale.profile()
            || envelope.profile() != numbers.profile()
            || numbers.locale_digest() != locale.digest()
        {
            return Err(IntlDataImageError::consumer(
                "RelativeTime, Number and Locale profiles or foundations differ",
            ));
        }
        let profiles = if envelope.blob() == PINNED_PROFILE {
            let source =
                core::str::from_utf8(envelope.blob()).map_err(IntlDataImageError::consumer)?;
            RelativeProfiles::from_json(source, &numbers.profiles())
                .map_err(IntlDataImageError::consumer)?
        } else {
            let (source, catalogue) =
                projection::admit(envelope.blob(), envelope.profile(), locale, numbers)?;
            let source = core::str::from_utf8(source).map_err(IntlDataImageError::consumer)?;
            RelativeProfiles::from_projected_json(source, &catalogue, &numbers.profiles())
                .map_err(IntlDataImageError::consumer)?
        };
        if numbers
            .required_relative_time_locales()
            .is_some_and(|required| {
                !required
                    .iter()
                    .map(|name| name.as_ref())
                    .eq(profiles.available_locales().iter().copied())
            })
        {
            return Err(IntlDataImageError::consumer(
                "RelativeTime catalogue differs from the selected Number dependency domain",
            ));
        }
        Ok(Self(Arc::new(AdmittedRelativeTimeData {
            envelope,
            profiles: Arc::new(profiles),
            locale_digest: locale.digest(),
            number_digest: numbers.digest(),
        })))
    }
    /// Selects actual pinned template rows and retains the complete selected
    /// Number foundation. Public RelativeTime support is limited to these rows
    /// and the required en-US fallback.
    pub fn for_custom_projection(
        id: &CustomProfileId,
        requested_locales: &[LocaleId],
        locale: &LocaleDataImage,
        numbers: &NumberProfilesDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let payload = projection::produce(id, requested_locales, locale, numbers)?;
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::RelativeTime,
                &IntlDataProfile::Custom(id.clone()),
                RELATIVE_TIME_IMAGE_MARKERS,
                &payload,
            )?,
            locale,
            numbers,
        )
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
    pub fn profiles(&self) -> Arc<RelativeProfiles> {
        Arc::clone(&self.0.profiles)
    }
    pub(crate) fn profiles_ref(&self) -> &RelativeProfiles {
        &self.0.profiles
    }
    pub(crate) fn locale_digest(&self) -> IntlDataDigest {
        self.0.locale_digest
    }
    pub(crate) fn number_digest(&self) -> IntlDataDigest {
        self.0.number_digest
    }
    pub(crate) fn uses_number_profiles(&self, profiles: &Arc<NumberProfiles>) -> bool {
        self.0.profiles.uses_number_profiles(profiles)
    }
}

pub(crate) fn embedded_relative_time_data_image_ref(
) -> Result<&'static RelativeTimeDataImage, IntlDataImageError> {
    static IMAGE: OnceLock<Result<RelativeTimeDataImage, IntlDataImageError>> = OnceLock::new();
    IMAGE
        .get_or_init(|| {
            let locale = crate::embedded_locale_data_image()?;
            let numbers = crate::embedded_number_profiles_data_image()?;
            RelativeTimeDataImage::for_profile(IntlDataProfile::Minimal, &locale, &numbers)
        })
        .as_ref()
        .map_err(Clone::clone)
}
pub fn embedded_relative_time_data_image() -> Result<RelativeTimeDataImage, IntlDataImageError> {
    embedded_relative_time_data_image_ref().cloned()
}

#[cfg(test)]
mod tests;
