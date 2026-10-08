//! The actual native DisplayNames tables and their selected Locale data owner.

use crate::display_names::{
    DisplayNameRequest, DisplayNameResult, DisplayNamesError, DisplayNamesProfiles,
    DISPLAY_NAMES_PROFILE,
};
use crate::image::{DataImageComponent, DataImageEnvelope};
use crate::locale_image::{embedded_locale_data_image, LocaleDataImage};
use crate::number_format::options::CurrencyCode;
use crate::provider::LocaleCanonicalizationData;
use crate::{CustomProfileId, IntlDataDigest, IntlDataImageError, IntlDataProfile, LocaleId};
use core::fmt;
use std::sync::{Arc, OnceLock};

mod projection;
pub(crate) use projection::DisplayNamesCatalogue;

pub const INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION: &str = "lila.intl-display-names-data.v1";
// This names the checked native JSON schema, not an ICU provider marker.
pub(crate) const DISPLAY_NAMES_IMAGE_MARKERS: &[&str] = &["lila/display-names/profiles/v1"];

struct AdmittedDisplayNamesData {
    envelope: DataImageEnvelope,
    locale_digest: IntlDataDigest,
    canonicalizer: LocaleCanonicalizationData,
    profiles: Arc<DisplayNamesProfiles>,
    currency_codes: Option<Box<[CurrencyCode]>>,
}

/// Exact pinned data or a rederived row/pool projection, complete native table
/// admission and the selected Locale canonicalizer construct this owner.
#[derive(Clone)]
pub struct DisplayNamesDataImage(Arc<AdmittedDisplayNamesData>);

impl fmt::Debug for DisplayNamesDataImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DisplayNamesDataImage")
            .field("profile", self.profile())
            .field("digest", &self.digest())
            .field(
                "available_locales",
                &self.0.profiles.available_locales().len(),
            )
            .finish()
    }
}

impl DisplayNamesDataImage {
    pub(crate) fn require_complete_source(&self) -> Result<(), IntlDataImageError> {
        if self.0.envelope.blob() != DISPLAY_NAMES_PROFILE {
            return Err(IntlDataImageError::IncompleteConformance);
        }
        Ok(())
    }

    pub fn for_profile(
        profile: IntlDataProfile,
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::DisplayNames,
                &profile,
                DISPLAY_NAMES_IMAGE_MARKERS,
                DISPLAY_NAMES_PROFILE,
            )?,
            locale,
        )
    }

    /// Select complete native locale rows and their reachable typed name pools.
    /// The selected Locale image supplies canonicalization and must carry the
    /// same Custom ID. The actual en-US fallback is retained once.
    pub fn for_custom_projection(
        id: &CustomProfileId,
        requested_locales: &[LocaleId],
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let payload = projection::produce(id, requested_locales, locale)?;
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::DisplayNames,
                &IntlDataProfile::Custom(id.clone()),
                DISPLAY_NAMES_IMAGE_MARKERS,
                &payload,
            )?,
            locale,
        )
    }

    /// Retain selected native locale rows and only the requested currency names.
    /// Other name domains stay complete; absent currencies use the normal
    /// DisplayNames code/none fallback. None retains every original locale row.
    pub fn for_custom_data_projection(
        id: &CustomProfileId,
        public_locales: Option<&[LocaleId]>,
        currency_codes: &[CurrencyCode],
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let payload = projection::produce_data(id, public_locales, currency_codes, locale)?;
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::DisplayNames,
                &IntlDataProfile::Custom(id.clone()),
                DISPLAY_NAMES_IMAGE_MARKERS,
                &payload,
            )?,
            locale,
        )
    }

    pub fn from_bytes(
        bytes: Arc<[u8]>,
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        let envelope = DataImageEnvelope::decode(
            bytes,
            DataImageComponent::DisplayNames,
            DISPLAY_NAMES_IMAGE_MARKERS,
        )?;
        if envelope.profile() != locale.profile() {
            return Err(IntlDataImageError::consumer(
                "DisplayNames and Locale image profiles differ",
            ));
        }
        // Reserved-language evidence is derived once per complete admission,
        // rather than rebuilt for each name key in the native catalogue.
        let canonicalizer =
            LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
        let (profiles, currency_codes) = if envelope.blob() == DISPLAY_NAMES_PROFILE {
            (
                DisplayNamesProfiles::from_image_data(envelope.blob(), &canonicalizer),
                None,
            )
        } else {
            let catalogue =
                projection::admit(envelope.blob(), envelope.profile(), locale, &canonicalizer)?;
            (
                DisplayNamesProfiles::from_projected_image_data(&catalogue, &canonicalizer),
                catalogue
                    .currency_codes()
                    .map(|codes| codes.to_vec().into_boxed_slice()),
            )
        };
        let profiles = profiles.map_err(IntlDataImageError::consumer)?;
        Ok(Self(Arc::new(AdmittedDisplayNamesData {
            envelope,
            locale_digest: locale.digest(),
            canonicalizer,
            profiles: Arc::new(profiles),
            currency_codes,
        })))
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
    pub fn profiles(&self) -> Arc<DisplayNamesProfiles> {
        Arc::clone(&self.0.profiles)
    }
    pub(crate) fn profiles_ref(&self) -> &DisplayNamesProfiles {
        &self.0.profiles
    }
    pub(crate) fn locale_digest(&self) -> IntlDataDigest {
        self.0.locale_digest
    }
    pub(crate) fn currency_codes(&self) -> Option<&[CurrencyCode]> {
        self.0.currency_codes.as_deref()
    }

    /// Check the selected template owner before code validation or lookup, then
    /// consume its retained canonicalizer and native name tables together.
    pub fn display_name(
        &self,
        request: &DisplayNameRequest,
    ) -> Result<DisplayNameResult, DisplayNamesError> {
        self.0
            .profiles
            .display_name_with_data(request, &self.0.canonicalizer)
    }
}

pub fn embedded_display_names_data_image() -> Result<DisplayNamesDataImage, IntlDataImageError> {
    static IMAGE: OnceLock<Result<DisplayNamesDataImage, IntlDataImageError>> = OnceLock::new();
    IMAGE
        .get_or_init(|| {
            let locale = embedded_locale_data_image()?;
            DisplayNamesDataImage::for_profile(IntlDataProfile::Minimal, &locale)
        })
        .clone()
}

#[cfg(test)]
mod tests;
