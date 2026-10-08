//! Canonical immutable component framing shared by actual typed consumers.

use crate::{CustomProfileId, IntlDataDigest, IntlDataProfile, IntlDataVersions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::ops::Range;
use std::sync::Arc;

const MAGIC: &[u8; 8] = b"LILAI001";
const SCHEMA: u16 = 1;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
pub const MAX_INTL_COMPONENT_IMAGE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DataImageComponent {
    LocaleTransforms,
    ListFormat,
    Collator,
    NumberProfiles,
    Segmenter,
    DisplayNames,
    RelativeTime,
    DurationFormat,
    NamedTimeZones,
    DateTime,
    TimeZoneNames,
    NativeLocaleInformation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntlDataImageError {
    Framing(&'static str),
    Manifest(Box<str>),
    Version,
    Component,
    MarkerInventory,
    Digest,
    IncompleteConformance,
    Consumer(Box<str>),
}
impl IntlDataImageError {
    pub fn consumer(error: impl fmt::Display) -> Self {
        Self::Consumer(error.to_string().into_boxed_str())
    }
}
impl fmt::Display for IntlDataImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Framing(reason) => write!(f, "invalid immutable Intl image framing: {reason}"),
            Self::Manifest(reason) => write!(f, "invalid immutable Intl image manifest: {reason}"),
            Self::Version => f.write_str("Intl image schema or upstream data versions differ"),
            Self::Component => f.write_str("Intl image belongs to a different typed component"),
            Self::MarkerInventory => {
                f.write_str("Intl image marker inventory is incomplete or incompatible")
            }
            Self::Digest => f.write_str("Intl image content digest does not match its bytes"),
            Self::IncompleteConformance => f.write_str(
                "Conformance Intl requires the complete unprojected pinned component data",
            ),
            Self::Consumer(reason) => write!(f, "Intl image typed data admission failed: {reason}"),
        }
    }
}
impl std::error::Error for IntlDataImageError {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
enum Profile {
    Minimal,
    Custom { id: String },
    Conformance,
}
impl Profile {
    fn from_selected(profile: &IntlDataProfile) -> Result<Self, IntlDataImageError> {
        match profile {
            IntlDataProfile::Minimal => Ok(Self::Minimal),
            IntlDataProfile::Custom(id) => Ok(Self::Custom {
                id: id.as_str().to_owned(),
            }),
            IntlDataProfile::Conformance => Ok(Self::Conformance),
        }
    }
    fn selected(&self) -> Result<IntlDataProfile, IntlDataImageError> {
        match self {
            Self::Minimal => Ok(IntlDataProfile::Minimal),
            Self::Custom { id } => CustomProfileId::parse(id.as_str())
                .map(IntlDataProfile::Custom)
                .map_err(|error| IntlDataImageError::Manifest(error.to_string().into_boxed_str())),
            Self::Conformance => Ok(IntlDataProfile::Conformance),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Versions {
    icu4x: String,
    cldr: String,
    unicode: String,
    icu_data_tag: String,
    segmenter_lstm: String,
    tzdb: String,
}
impl Versions {
    fn pinned() -> Self {
        let versions = IntlDataVersions::PINNED;
        Self {
            icu4x: versions.icu4x.into(),
            cldr: versions.cldr.into(),
            unicode: versions.unicode.into(),
            icu_data_tag: versions.icu_data_tag.into(),
            segmenter_lstm: versions.segmenter_lstm.into(),
            tzdb: versions.tzdb.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u16,
    component: DataImageComponent,
    profile: Profile,
    default_locale: String,
    versions: Versions,
    markers: Vec<String>,
}

fn marker_inventory(names: &[&str]) -> Result<Vec<String>, IntlDataImageError> {
    let mut names = names
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    names.sort_unstable();
    if names.is_empty()
        || names.iter().any(String::is_empty)
        || names.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(IntlDataImageError::MarkerInventory);
    }
    Ok(names)
}

/// Framing admission is private; a family constructor must additionally load
/// and validate the actual typed marker payloads before exposing its image.
#[derive(Debug, Clone)]
pub(crate) struct DataImageEnvelope {
    bytes: Arc<[u8]>,
    blob: Range<usize>,
    digest: IntlDataDigest,
    profile: IntlDataProfile,
}
impl DataImageEnvelope {
    pub(crate) fn encode(
        component: DataImageComponent,
        profile: &IntlDataProfile,
        marker_names: &[&str],
        blob: &[u8],
    ) -> Result<Arc<[u8]>, IntlDataImageError> {
        let manifest = Manifest {
            schema: SCHEMA,
            component,
            profile: Profile::from_selected(profile)?,
            default_locale: "en-US".into(),
            versions: Versions::pinned(),
            markers: marker_inventory(marker_names)?,
        };
        let manifest = serde_json::to_vec(&manifest).map_err(IntlDataImageError::consumer)?;
        if manifest.len() > MAX_MANIFEST_BYTES || blob.is_empty() {
            return Err(IntlDataImageError::Framing(
                "empty payload or excessive manifest",
            ));
        }
        let total = 12usize
            .checked_add(manifest.len())
            .and_then(|n| n.checked_add(8))
            .and_then(|n| n.checked_add(blob.len()))
            .and_then(|n| n.checked_add(32))
            .ok_or(IntlDataImageError::Framing("size overflow"))?;
        if total > MAX_INTL_COMPONENT_IMAGE_BYTES {
            return Err(IntlDataImageError::Framing(
                "image exceeds the admitted resource extent",
            ));
        }
        let mut bytes = Vec::with_capacity(total);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(
            &u32::try_from(manifest.len())
                .map_err(|_| IntlDataImageError::Framing("manifest length"))?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&manifest);
        bytes.extend_from_slice(
            &u64::try_from(blob.len())
                .map_err(|_| IntlDataImageError::Framing("payload length"))?
                .to_le_bytes(),
        );
        bytes.extend_from_slice(blob);
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        bytes.extend_from_slice(&digest);
        Ok(bytes.into())
    }

    pub(crate) fn decode(
        bytes: Arc<[u8]>,
        expected_component: DataImageComponent,
        expected_marker_names: &[&str],
    ) -> Result<Self, IntlDataImageError> {
        if !(53..=MAX_INTL_COMPONENT_IMAGE_BYTES).contains(&bytes.len())
            || bytes.get(..8) != Some(MAGIC.as_slice())
        {
            return Err(IntlDataImageError::Framing("magic or total extent"));
        }
        let manifest_len =
            u32::from_le_bytes(bytes[8..12].try_into().expect("header extent checked")) as usize;
        if manifest_len > MAX_MANIFEST_BYTES {
            return Err(IntlDataImageError::Framing("manifest extent"));
        }
        let manifest_end = 12usize
            .checked_add(manifest_len)
            .ok_or(IntlDataImageError::Framing("manifest overflow"))?;
        let blob_start = manifest_end
            .checked_add(8)
            .ok_or(IntlDataImageError::Framing("payload overflow"))?;
        let manifest_bytes = bytes
            .get(12..manifest_end)
            .ok_or(IntlDataImageError::Framing("truncated manifest"))?;
        let blob_len = bytes
            .get(manifest_end..blob_start)
            .ok_or(IntlDataImageError::Framing("truncated payload length"))?;
        let blob_len = usize::try_from(u64::from_le_bytes(
            blob_len.try_into().expect("eight-byte extent"),
        ))
        .map_err(|_| IntlDataImageError::Framing("unaddressable payload"))?;
        let blob_end = blob_start
            .checked_add(blob_len)
            .ok_or(IntlDataImageError::Framing("payload overflow"))?;
        if blob_len == 0 || blob_end.checked_add(32) != Some(bytes.len()) {
            return Err(IntlDataImageError::Framing(
                "payload extent or trailing bytes",
            ));
        }
        let digest: [u8; 32] = Sha256::digest(&bytes[..blob_end]).into();
        if bytes[blob_end..] != digest {
            return Err(IntlDataImageError::Digest);
        }
        let manifest: Manifest =
            serde_json::from_slice(manifest_bytes).map_err(IntlDataImageError::consumer)?;
        if manifest.schema != SCHEMA
            || manifest.versions != Versions::pinned()
            || manifest.default_locale != "en-US"
        {
            return Err(IntlDataImageError::Version);
        }
        if manifest.component != expected_component {
            return Err(IntlDataImageError::Component);
        }
        if manifest.markers != marker_inventory(expected_marker_names)? {
            return Err(IntlDataImageError::MarkerInventory);
        }
        let profile = manifest.profile.selected()?;
        if serde_json::to_vec(&manifest).map_err(IntlDataImageError::consumer)? != manifest_bytes {
            return Err(IntlDataImageError::Manifest(
                "noncanonical manifest encoding".into(),
            ));
        }
        Ok(Self {
            bytes,
            blob: blob_start..blob_end,
            digest: IntlDataDigest::from_sha256(digest),
            profile,
        })
    }

    pub(crate) fn bytes(&self) -> Arc<[u8]> {
        self.bytes.clone()
    }
    pub(crate) fn blob(&self) -> &[u8] {
        &self.bytes[self.blob.clone()]
    }
    pub(crate) fn digest(&self) -> IntlDataDigest {
        self.digest
    }
    pub(crate) fn profile(&self) -> &IntlDataProfile {
        &self.profile
    }
}

#[cfg(test)]
mod tests;
