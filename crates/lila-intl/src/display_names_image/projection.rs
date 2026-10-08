//! Exact native locale rows and reachable typed pools from the pinned source.

use super::DISPLAY_NAMES_PROFILE;
use crate::number_format::options::CurrencyCode;
use crate::provider::LocaleCanonicalizationData;
use crate::{
    CanonicalLocaleId, CustomProfileId, DisplayNamesProfiles, IntlDataImageError, IntlDataProfile,
    LocaleDataImage, LocaleId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const MAGIC: &[u8; 8] = b"LILADNP1";
const HEADER_BYTES: usize = 20;
const MAX_DESCRIPTOR_BYTES: usize = 16 * 1024;
const STYLES: &[&str] = &["long", "short", "narrow"];
const DOMAINS: &[&str] = &[
    "language",
    "region",
    "script",
    "language_script",
    "currency",
    "calendar",
    "date_time_field",
    "variant",
];

struct SelectedLocales {
    locales: Box<[CanonicalLocaleId]>,
}
impl SelectedLocales {
    fn projected(
        full: &DisplayNamesProfiles,
        requested: &[LocaleId],
        canonicalizer: &LocaleCanonicalizationData,
    ) -> Result<Self, IntlDataImageError> {
        let available = full.available_locales().collect::<Vec<_>>();
        if requested.is_empty() || requested.len() > available.len() {
            return Err(IntlDataImageError::consumer(
                "DisplayNames projection locale extent",
            ));
        }
        let mut selected = BTreeSet::new();
        for requested in requested {
            let canonical = canonicalizer
                .canonicalize(requested)
                .map_err(IntlDataImageError::consumer)?;
            let index = available
                .binary_search_by(|locale| locale.as_str().cmp(canonical.as_str()))
                .map_err(|_| {
                    IntlDataImageError::consumer(format!(
                        "DisplayNames projection locale {} is absent from the pinned catalogue",
                        canonical.as_str(),
                    ))
                })?;
            if !selected.insert((*available[index]).clone()) {
                return Err(IntlDataImageError::consumer(
                    "DisplayNames projection contains duplicate canonical locales",
                ));
            }
        }
        let default = available
            .binary_search_by(|locale| locale.as_str().cmp("en-US"))
            .map_err(IntlDataImageError::consumer)?;
        selected.insert((*available[default]).clone());
        Ok(Self {
            locales: selected.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        })
    }

    fn locales(&self) -> &[CanonicalLocaleId] {
        &self.locales
    }
}

/// Exact rederivation binds these selected locales to their physical payload.
/// The native decoder accepts this owner, never arbitrary bytes or row lists.
pub(crate) struct DisplayNamesCatalogue<'a> {
    bytes: &'a [u8],
    locales: Box<[CanonicalLocaleId]>,
    currency_codes: Option<Box<[CurrencyCode]>>,
}
impl DisplayNamesCatalogue<'_> {
    pub(crate) fn bytes(&self) -> &[u8] {
        self.bytes
    }
    pub(crate) fn locales(&self) -> &[CanonicalLocaleId] {
        &self.locales
    }
    pub(crate) fn currency_codes(&self) -> Option<&[CurrencyCode]> {
        self.currency_codes.as_deref()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema: u16,
    custom_id: String,
    default_locale: String,
    full_profile_sha256: [u8; 32],
    locale_image_sha256: [u8; 32],
    public_locales: Vec<String>,
    source_pool_indices: Vec<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    currency_codes: Option<Vec<String>>,
}

fn pool_reference(
    row: &serde_json::Value,
    style: &str,
    domain: &str,
    pool_count: usize,
) -> Result<usize, IntlDataImageError> {
    row["styles"][style][domain]
        .as_u64()
        .and_then(|index| usize::try_from(index).ok())
        .filter(|&index| index < pool_count)
        .ok_or_else(|| IntlDataImageError::consumer("pinned DisplayNames pool reference is absent"))
}

fn project(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
    canonicalizer: &LocaleCanonicalizationData,
) -> Result<(Vec<u8>, SelectedLocales), IntlDataImageError> {
    if locale.profile() != &IntlDataProfile::Custom(id.clone()) {
        return Err(IntlDataImageError::consumer(
            "DisplayNames projection and Locale foundation profiles differ",
        ));
    }
    // Admit the complete original identity, all thirteen locale rows and every
    // typed pool before selecting any subset of the source JSON.
    let full = DisplayNamesProfiles::from_image_data(DISPLAY_NAMES_PROFILE, canonicalizer)
        .map_err(IntlDataImageError::consumer)?;
    let catalogue = SelectedLocales::projected(&full, requested, canonicalizer)?;
    let mut raw: serde_json::Value =
        serde_json::from_slice(DISPLAY_NAMES_PROFILE).map_err(IntlDataImageError::consumer)?;
    let rows = raw["locales"].as_array().ok_or_else(|| {
        IntlDataImageError::consumer("pinned DisplayNames source locale rows are absent")
    })?;
    let mut selected_rows = catalogue
        .locales()
        .iter()
        .map(|name| {
            rows.iter()
                .find(|row| row["locale"].as_str() == Some(name.as_str()))
                .cloned()
                .ok_or_else(|| IntlDataImageError::consumer("pinned DisplayNames row is absent"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let pools = raw["name_pool"].as_array().ok_or_else(|| {
        IntlDataImageError::consumer("pinned DisplayNames source name pools are absent")
    })?;
    let mut used = BTreeSet::new();
    for row in &selected_rows {
        for &style in STYLES {
            for &domain in DOMAINS {
                used.insert(pool_reference(row, style, domain, pools.len())?);
            }
        }
    }
    // Original source pool order is canonical and duplicate-free. Keeping its
    // subsequence preserves that invariant while removing every unused pool.
    let mut dense = vec![None; pools.len()];
    let mut selected_pools = Vec::with_capacity(used.len());
    let mut source_pool_indices = Vec::with_capacity(used.len());
    for source in used {
        let index = u32::try_from(selected_pools.len()).map_err(IntlDataImageError::consumer)?;
        dense[source] = Some(index);
        selected_pools.push(pools[source].clone());
        source_pool_indices.push(u32::try_from(source).map_err(IntlDataImageError::consumer)?);
    }
    for row in &mut selected_rows {
        for &style in STYLES {
            for &domain in DOMAINS {
                let source = pool_reference(row, style, domain, pools.len())?;
                let index = dense[source].ok_or_else(|| {
                    IntlDataImageError::consumer("pinned DisplayNames pool closure is incomplete")
                })?;
                row["styles"][style][domain] = serde_json::Value::from(index);
            }
        }
    }
    raw["locales"] = serde_json::Value::Array(selected_rows);
    raw["name_pool"] = serde_json::Value::Array(selected_pools);
    let descriptor = Descriptor {
        schema: 1,
        custom_id: id.as_str().to_owned(),
        default_locale: "en-US".to_owned(),
        full_profile_sha256: Sha256::digest(DISPLAY_NAMES_PROFILE).into(),
        locale_image_sha256: *locale.digest().as_bytes(),
        public_locales: catalogue
            .locales()
            .iter()
            .map(|name| name.as_str().to_owned())
            .collect(),
        source_pool_indices,
        currency_codes: None,
    };
    // Canonical object order does not depend on serde_json preserve_order.
    let blob = serde_json::to_vec(&canonical_objects(raw)).map_err(IntlDataImageError::consumer)?;
    Ok((frame_payload(&descriptor, &blob)?, catalogue))
}

fn project_data(
    id: &CustomProfileId,
    requested: Option<&[LocaleId]>,
    currency_codes: &[CurrencyCode],
    locale: &LocaleDataImage,
    canonicalizer: &LocaleCanonicalizationData,
) -> Result<(Vec<u8>, SelectedLocales, Box<[CurrencyCode]>), IntlDataImageError> {
    let all_locales;
    let requested = match requested {
        Some(locales) => locales,
        None => {
            let full = DisplayNamesProfiles::from_image_data(DISPLAY_NAMES_PROFILE, canonicalizer)
                .map_err(IntlDataImageError::consumer)?;
            all_locales = full
                .available_locales()
                .map(|name| LocaleId::parse(name.as_str()).map_err(IntlDataImageError::consumer))
                .collect::<Result<Vec<_>, _>>()?;
            &all_locales
        }
    };
    // The existing producer first admits the entire pinned source and its
    // Locale foundation, then closes every selected row's typed associations.
    let (payload, catalogue) = project(id, requested, locale, canonicalizer)?;
    let (mut descriptor, blob) = split_payload(&payload)?;
    let mut raw: serde_json::Value =
        serde_json::from_slice(blob).map_err(IntlDataImageError::consumer)?;
    let original: serde_json::Value =
        serde_json::from_slice(DISPLAY_NAMES_PROFILE).map_err(IntlDataImageError::consumer)?;
    let source_pools = original["name_pool"]
        .as_array()
        .ok_or_else(|| IntlDataImageError::consumer("pinned DisplayNames pools absent"))?;
    let default = original["locales"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|row| row["locale"].as_str() == Some("en-US"))
        })
        .ok_or_else(|| IntlDataImageError::consumer("pinned DisplayNames en-US absent"))?;
    let currency_pool = pool_reference(default, "long", "currency", source_pools.len())?;
    let source_codes = source_pools[currency_pool]["entries"]
        .as_array()
        .ok_or_else(|| IntlDataImageError::consumer("pinned currency names absent"))?;
    if currency_codes.is_empty() || currency_codes.len() > source_codes.len() {
        return Err(IntlDataImageError::consumer(
            "DisplayNames currency projection extent",
        ));
    }
    let mut selected = BTreeMap::new();
    for code in currency_codes {
        let ascii = code.clone().ascii();
        let text = std::str::from_utf8(&ascii).map_err(IntlDataImageError::consumer)?;
        if !source_codes.iter().any(|row| row[0].as_str() == Some(text)) {
            return Err(IntlDataImageError::consumer(
                "DisplayNames currency absent from pinned en-US",
            ));
        }
        if selected.insert(text.to_owned(), code.clone()).is_some() {
            return Err(IntlDataImageError::consumer(
                "duplicate DisplayNames projection currency",
            ));
        }
    }
    let selected_pools = raw["name_pool"]
        .as_array_mut()
        .ok_or_else(|| IntlDataImageError::consumer("projected DisplayNames pools absent"))?;
    let mut keys = Vec::with_capacity(selected_pools.len());
    let mut unique = BTreeMap::new();
    for (index, pool) in selected_pools.iter_mut().enumerate() {
        if pool["kind"].as_str() == Some("currency") {
            pool["entries"]
                .as_array_mut()
                .ok_or_else(|| IntlDataImageError::consumer("projected currency entries absent"))?
                .retain(|row| {
                    row[0]
                        .as_str()
                        .is_some_and(|code| selected.contains_key(code))
                });
        }
        let canonical = canonical_objects(pool.clone());
        let key = serde_json::to_vec(&canonical).map_err(IntlDataImageError::consumer)?;
        unique
            .entry(key.clone())
            .or_insert((canonical, descriptor.source_pool_indices[index]));
        keys.push(key);
    }
    // Filtering can make distinct Currency pools equal or change their order.
    // Rebuild the actual canonical pool domain, then remap every association.
    let mut dense = BTreeMap::new();
    let mut pools = Vec::with_capacity(unique.len());
    let mut source_pool_indices = Vec::with_capacity(unique.len());
    for (key, (pool, source)) in unique {
        let index = u32::try_from(pools.len()).map_err(IntlDataImageError::consumer)?;
        dense.insert(key, index);
        pools.push(pool);
        source_pool_indices.push(source);
    }
    for row in raw["locales"]
        .as_array_mut()
        .ok_or_else(|| IntlDataImageError::consumer("projected DisplayNames rows absent"))?
    {
        for &style in STYLES {
            for &domain in DOMAINS {
                let old = pool_reference(row, style, domain, keys.len())?;
                row["styles"][style][domain] = serde_json::Value::from(dense[&keys[old]]);
            }
        }
    }
    raw["name_pool"] = serde_json::Value::Array(pools);
    descriptor.schema = 2;
    descriptor.currency_codes = Some(selected.keys().cloned().collect());
    descriptor.source_pool_indices = source_pool_indices;
    let blob = serde_json::to_vec(&canonical_objects(raw)).map_err(IntlDataImageError::consumer)?;
    Ok((
        frame_payload(&descriptor, &blob)?,
        catalogue,
        selected
            .into_values()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    ))
}

fn canonical_objects(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(object) => {
            let mut entries = object.into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            serde_json::Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonical_objects(value)))
                    .collect(),
            )
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonical_objects).collect())
        }
        value => value,
    }
}

fn frame_payload(descriptor: &Descriptor, blob: &[u8]) -> Result<Vec<u8>, IntlDataImageError> {
    let descriptor = serde_json::to_vec(descriptor).map_err(IntlDataImageError::consumer)?;
    if descriptor.len() > MAX_DESCRIPTOR_BYTES || blob.len() > DISPLAY_NAMES_PROFILE.len() {
        return Err(IntlDataImageError::consumer(
            "DisplayNames projection extent",
        ));
    }
    let extent = HEADER_BYTES
        .checked_add(descriptor.len())
        .and_then(|n| n.checked_add(blob.len()))
        .filter(|&n| n <= crate::MAX_INTL_COMPONENT_IMAGE_BYTES)
        .ok_or_else(|| IntlDataImageError::consumer("DisplayNames projection extent"))?;
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(extent)
        .map_err(IntlDataImageError::consumer)?;
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(
        &u32::try_from(descriptor.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(
        &u64::try_from(blob.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(&descriptor);
    payload.extend_from_slice(blob);
    Ok(payload)
}

pub(super) fn produce(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    let canonicalizer =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    project(id, requested, locale, &canonicalizer).map(|(payload, _)| payload)
}

pub(super) fn produce_data(
    id: &CustomProfileId,
    requested: Option<&[LocaleId]>,
    currency_codes: &[CurrencyCode],
    locale: &LocaleDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    let canonicalizer =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    project_data(id, requested, currency_codes, locale, &canonicalizer)
        .map(|(payload, _, _)| payload)
}

pub(super) fn admit<'a>(
    payload: &'a [u8],
    profile: &IntlDataProfile,
    locale: &LocaleDataImage,
    canonicalizer: &LocaleCanonicalizationData,
) -> Result<DisplayNamesCatalogue<'a>, IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(IntlDataImageError::consumer(
            "a projected DisplayNames payload requires Custom",
        ));
    };
    let (descriptor, blob) = split_payload(payload)?;
    let requested = descriptor
        .public_locales
        .iter()
        .map(|name| LocaleId::parse(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(IntlDataImageError::consumer)?;
    let (expected, catalogue, currency_codes) =
        match (descriptor.schema, &descriptor.currency_codes) {
            (1, None) => {
                let (payload, catalogue) = project(id, &requested, locale, canonicalizer)?;
                (payload, catalogue, None)
            }
            (2, Some(codes)) => {
                let codes = codes
                    .iter()
                    .map(|code| CurrencyCode::parse(code).map_err(IntlDataImageError::consumer))
                    .collect::<Result<Vec<_>, _>>()?;
                let (payload, catalogue, codes) =
                    project_data(id, Some(&requested), &codes, locale, canonicalizer)?;
                (payload, catalogue, Some(codes))
            }
            _ => {
                return Err(IntlDataImageError::consumer(
                    "invalid DisplayNames projection schema",
                ))
            }
        };
    // Equality binds complete selected rows, every remapped typed association,
    // exact reachable pool contents, source pins and the actual Locale frame.
    if payload != expected.as_slice() {
        return Err(IntlDataImageError::consumer(
            "DisplayNames projection differs from its pinned derivation",
        ));
    }
    Ok(DisplayNamesCatalogue {
        bytes: blob,
        locales: catalogue.locales,
        currency_codes,
    })
}

fn split_payload(payload: &[u8]) -> Result<(Descriptor, &[u8]), IntlDataImageError> {
    if payload.len() < HEADER_BYTES || &payload[..8] != MAGIC {
        return Err(IntlDataImageError::consumer(
            "invalid DisplayNames projection framing",
        ));
    }
    let descriptor_length = u32::from_le_bytes(
        payload[8..12]
            .try_into()
            .map_err(IntlDataImageError::consumer)?,
    ) as usize;
    let blob_length = usize::try_from(u64::from_le_bytes(
        payload[12..20]
            .try_into()
            .map_err(IntlDataImageError::consumer)?,
    ))
    .map_err(IntlDataImageError::consumer)?;
    let blob_start = HEADER_BYTES
        .checked_add(descriptor_length)
        .filter(|_| {
            descriptor_length <= MAX_DESCRIPTOR_BYTES && blob_length <= DISPLAY_NAMES_PROFILE.len()
        })
        .ok_or_else(|| IntlDataImageError::consumer("invalid DisplayNames projection extent"))?;
    if blob_start.checked_add(blob_length) != Some(payload.len()) {
        return Err(IntlDataImageError::consumer(
            "invalid DisplayNames projection extent",
        ));
    }
    let descriptor = serde_json::from_slice(&payload[HEADER_BYTES..blob_start])
        .map_err(IntlDataImageError::consumer)?;
    Ok((descriptor, &payload[blob_start..]))
}

#[cfg(test)]
mod tests;
