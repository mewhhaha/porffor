//! Image-backed List data admitted by all nine actual formatter constructors.

use crate::image::DataImageEnvelope;
use crate::locale_image::{embedded_locale_data_image, LocaleDataImage};
use crate::{
    CustomListProfile, DataImageComponent, IntlDataDigest, IntlDataImageError, IntlDataProfile,
    ListProfiles,
};
use core::fmt;
use icu_provider::prelude::*;
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use icu_provider_blob::BlobDataProvider;
use std::sync::{Arc, OnceLock};

pub(crate) mod projection;

pub(crate) const LIST_IMAGE_MARKERS: &[&str] = &["list/and/v1", "list/or/v1", "list/unit/v1"];
pub const INTL_LIST_DATA_CUSTOM_SECTION: &str = "lila.intl-list-data.v1";
const PINNED_BLOB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/list-data.blob"));

struct ListImageInner {
    envelope: DataImageEnvelope,
    locale_digest: IntlDataDigest,
    profiles: Arc<ListProfiles>,
}

/// Only complete image/frame and actual nine-way formatter admission constructs
/// this owner. The profiles retain their loaded immutable data payloads.
#[derive(Clone)]
pub struct ListDataImage {
    inner: Arc<ListImageInner>,
}

impl fmt::Debug for ListDataImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListDataImage")
            .field("profile", self.profile())
            .field("digest", &self.digest())
            .field(
                "available_locales",
                &self.inner.profiles.available_locales().len(),
            )
            .finish()
    }
}

impl ListDataImage {
    pub(crate) fn require_complete_source(&self) -> Result<(), IntlDataImageError> {
        if self.inner.envelope.blob() != PINNED_BLOB {
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
                DataImageComponent::ListFormat,
                &profile,
                LIST_IMAGE_MARKERS,
                PINNED_BLOB,
            )?,
            locale,
        )
    }

    pub fn from_bytes(
        bytes: Arc<[u8]>,
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        // ICU 2's binary List DFA decoder drops conditional patterns on a
        // big-endian host. Refuse that representation instead of changing its
        // Spanish/Hebrew formatting semantics through silent data loss.
        if cfg!(target_endian = "big") {
            return Err(IntlDataImageError::consumer(
                "ICU List binary conditional-pattern images require a little-endian host",
            ));
        }
        let envelope =
            DataImageEnvelope::decode(bytes, DataImageComponent::ListFormat, LIST_IMAGE_MARKERS)?;
        if envelope.profile() != locale.profile() {
            return Err(IntlDataImageError::consumer(
                "List and Locale image profiles differ",
            ));
        }
        let profiles = if envelope.blob() == PINNED_BLOB {
            let provider = BlobDataProvider::try_new_from_blob(envelope.blob().into())
                .map_err(IntlDataImageError::consumer)?;
            ListProfiles::from_image_data(&provider, locale)
                .map_err(IntlDataImageError::consumer)?
        } else {
            let (provider, catalogue) =
                projection::admit(envelope.blob(), envelope.profile(), locale)?;
            let typed =
                LocaleFallbackProvider::new(provider.as_deserializing(), locale.fallbacker());
            ListProfiles::from_data_provider(&typed, &catalogue)
                .map_err(IntlDataImageError::consumer)?
        };
        Ok(Self {
            inner: Arc::new(ListImageInner {
                envelope,
                locale_digest: locale.digest(),
                profiles: Arc::new(profiles),
            }),
        })
    }

    /// Projects the pinned physical loads for the selected public List locales
    /// and all retained native Duration associations. Other service payloads
    /// remain complete; this does not promise a general filtered Intl profile.
    pub fn for_custom_projection(
        selection: &CustomListProfile,
        locale: &LocaleDataImage,
    ) -> Result<Self, IntlDataImageError> {
        if cfg!(target_endian = "big") {
            return Err(IntlDataImageError::consumer(
                "ICU List binary conditional-pattern images require a little-endian host",
            ));
        }
        let payload = projection::produce(selection, locale)?;
        Self::from_bytes(
            DataImageEnvelope::encode(
                DataImageComponent::ListFormat,
                &IntlDataProfile::Custom(selection.id().clone()),
                LIST_IMAGE_MARKERS,
                &payload,
            )?,
            locale,
        )
    }

    pub fn bytes(&self) -> Arc<[u8]> {
        self.inner.envelope.bytes()
    }

    pub fn digest(&self) -> IntlDataDigest {
        self.inner.envelope.digest()
    }

    pub fn profile(&self) -> &IntlDataProfile {
        self.inner.envelope.profile()
    }

    pub fn profiles(&self) -> Arc<ListProfiles> {
        Arc::clone(&self.inner.profiles)
    }

    pub(crate) fn locale_digest(&self) -> IntlDataDigest {
        self.inner.locale_digest
    }

    pub(crate) fn profiles_ref(&self) -> &ListProfiles {
        &self.inner.profiles
    }
}

static EMBEDDED: OnceLock<Result<ListDataImage, IntlDataImageError>> = OnceLock::new();

pub(crate) fn embedded_list_data_image_ref() -> Result<&'static ListDataImage, IntlDataImageError> {
    EMBEDDED
        .get_or_init(|| {
            let locale = embedded_locale_data_image()?;
            ListDataImage::for_profile(IntlDataProfile::Minimal, &locale)
        })
        .as_ref()
        .map_err(Clone::clone)
}

pub fn embedded_list_data_image() -> Result<ListDataImage, IntlDataImageError> {
    embedded_list_data_image_ref().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number_format::options::LocaleMatcher;
    use crate::{
        CanonicalLocaleId, CheckedListConfiguration, CustomProfileId, FormatListPartsRequest,
        ListFormatOperationError, ListLocaleRequest, ListStyle, ListType,
    };

    #[test]
    fn a_minimal_list_image_rejects_a_custom_named_locale_foundation() {
        let image = embedded_list_data_image().expect("admitted pinned Minimal List image");
        let locale = LocaleDataImage::for_profile(IntlDataProfile::Custom(
            CustomProfileId::parse("list-profile-proof").expect("valid Custom component name"),
        ))
        .expect("admitted exact pinned Custom-named Locale image");
        assert_eq!(image.profile(), &IntlDataProfile::Minimal);
        assert!(matches!(
            ListDataImage::from_bytes(image.bytes(), &locale),
            Err(IntlDataImageError::Consumer(_))
        ));
    }

    #[test]
    fn a_valid_image_does_not_admit_another_owners_resolved_configuration() {
        let locale = embedded_locale_data_image().expect("real locale image");
        let first = embedded_list_data_image().expect("real List image");
        let second = ListDataImage::from_bytes(first.bytes(), &locale)
            .expect("independently admitted real List image");
        let first_profiles = first.profiles();
        let resolved = first_profiles
            .resolve(ListLocaleRequest {
                requested: vec![
                    CanonicalLocaleId::from_data("es").expect("canonical requested locale")
                ]
                .into_boxed_slice(),
                matcher: LocaleMatcher::Lookup,
            })
            .expect("actual available locale");
        let request = FormatListPartsRequest::new(
            CheckedListConfiguration::new(resolved, ListType::Conjunction, ListStyle::Long),
            ["España", "Italia"]
                .iter()
                .map(|element| {
                    element
                        .encode_utf16()
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                })
                .collect(),
        )
        .expect("retained raw UTF16 request");
        let parts = first_profiles
            .format_parts(request.clone())
            .expect("configuration remains valid for its actual owner");
        assert_eq!(parts.input_count(), 2);
        assert_eq!(
            second.profiles().format_parts(request),
            Err(ListFormatOperationError::InvalidResolvedLocale)
        );
    }
}
