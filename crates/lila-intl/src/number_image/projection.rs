//! Canonical physical Number/Plural closure with private dependent locale views.

use super::{PinnedNumberSource, PINNED_BINARY, PINNED_DESCRIPTOR};
use crate::number_format::options::CurrencyCode;
use crate::number_format::{
    encode_selected, encode_selected_currencies, encode_selected_numbering, NumberLocaleDomains,
    NumberingSystemOption,
};
use crate::provider::LocaleCanonicalizationData;
use crate::{
    CustomProfileId, IntlDataImageError, IntlDataProfile, ListDataImage, LocaleDataImage, LocaleId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const MAGIC: &[u8; 8] = b"LILANUP1";
const HEADER_BYTES: usize = 20;
const MAX_DESCRIPTOR_BYTES: usize = 128 * 1024;

/// The physical bytes and their three domains have one admission authority.
/// Neither arbitrary domain lists nor an independently decoded table can mint it.
pub(crate) struct NumberCatalogue<'a> {
    binary: &'a [u8],
    public: Box<[Box<str>]>,
    relative_time: Box<[Box<str>]>,
    duration: Box<[Box<str>]>,
    physical: Box<[Box<str>]>,
    defaults: Box<[(Box<str>, u8)]>,
    currency_codes: Option<Box<[CurrencyCode]>>,
    numbering_systems: Option<Box<[NumberingSystemOption]>>,
}
impl NumberCatalogue<'_> {
    pub(crate) fn currency_codes(&self) -> Option<&[CurrencyCode]> {
        self.currency_codes.as_deref()
    }
    pub(crate) fn numbering_systems(&self) -> Option<&[NumberingSystemOption]> {
        self.numbering_systems.as_deref()
    }
    pub(crate) fn binary(&self) -> &[u8] {
        self.binary
    }
    pub(crate) fn physical_locales(&self) -> &[Box<str>] {
        &self.physical
    }
    pub(crate) fn domains(&self) -> NumberLocaleDomains {
        NumberLocaleDomains::Projected {
            public: self.public.clone(),
            relative_time: self.relative_time.clone(),
            duration: self.duration.clone(),
            defaults: self.defaults.clone(),
            currency_codes: self.currency_codes.clone(),
            numbering_systems: self.numbering_systems.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema: u16,
    custom_id: String,
    default_locale: String,
    full_number_binary_sha256: [u8; 32],
    full_number_descriptor_sha256: [u8; 32],
    relative_time_source_sha256: [u8; 32],
    duration_source_sha256: [u8; 32],
    locale_image_sha256: [u8; 32],
    list_image_sha256: [u8; 32],
    public_locales: Vec<String>,
    relative_time_locales: Option<Vec<String>>,
    duration_locales: Option<Vec<String>>,
    required_relative_time_locales: Vec<String>,
    required_duration_locales: Vec<String>,
    default_numbering: Vec<(String, u8)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    currency_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    numbering_systems: Option<Vec<String>>,
}

struct DerivedCatalogue {
    public: Box<[Box<str>]>,
    relative_time: Box<[Box<str>]>,
    duration: Box<[Box<str>]>,
    physical: Box<[Box<str>]>,
    defaults: Box<[(Box<str>, u8)]>,
    currency_codes: Option<Box<[CurrencyCode]>>,
    numbering_systems: Option<Box<[NumberingSystemOption]>>,
}

fn select_public(
    requested: &[LocaleId],
    locale: &LocaleDataImage,
    source: &PinnedNumberSource,
) -> Result<Box<[Box<str>]>, IntlDataImageError> {
    let available = source.profiles().available_locales();
    if requested.is_empty() || requested.len() > available.len() {
        return Err(IntlDataImageError::consumer(
            "Number projection locale extent",
        ));
    }
    let authority =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    let mut selected = BTreeSet::new();
    for requested in requested {
        let canonical = authority
            .canonicalize(requested)
            .map_err(IntlDataImageError::consumer)?;
        let index = available
            .binary_search_by(|name| name.as_ref().cmp(canonical.as_str()))
            .map_err(|_| {
                IntlDataImageError::consumer(format!(
                    "Number projection locale {} is absent from the pinned catalogue",
                    canonical.as_str()
                ))
            })?;
        if !selected.insert(available[index].clone()) {
            return Err(IntlDataImageError::consumer(
                "Number projection contains duplicate canonical locales",
            ));
        }
    }
    let default = available
        .binary_search_by(|name| name.as_ref().cmp("en-US"))
        .map_err(IntlDataImageError::consumer)?;
    selected.insert(available[default].clone());
    Ok(selected.into_iter().collect::<Vec<_>>().into_boxed_slice())
}

fn names(values: &[Box<str>]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn project(
    id: &CustomProfileId,
    public_locales: &[LocaleId],
    relative_time_locales: Option<&[LocaleId]>,
    duration_locales: Option<&[LocaleId]>,
    locale: &LocaleDataImage,
    lists: &ListDataImage,
) -> Result<(Vec<u8>, DerivedCatalogue), IntlDataImageError> {
    project_data(
        id,
        Some(public_locales),
        None,
        None,
        relative_time_locales,
        duration_locales,
        locale,
        lists,
    )
}

fn project_data(
    id: &CustomProfileId,
    public_locales: Option<&[LocaleId]>,
    currency_codes: Option<&[CurrencyCode]>,
    numbering_systems: Option<&[NumberingSystemOption]>,
    relative_time_locales: Option<&[LocaleId]>,
    duration_locales: Option<&[LocaleId]>,
    locale: &LocaleDataImage,
    lists: &ListDataImage,
) -> Result<(Vec<u8>, DerivedCatalogue), IntlDataImageError> {
    let profile = IntlDataProfile::Custom(id.clone());
    if locale.profile() != &profile
        || lists.profile() != &profile
        || lists.locale_digest() != locale.digest()
    {
        return Err(IntlDataImageError::consumer(
            "Number projection, Locale and List foundations differ",
        ));
    }
    let source = super::pinned_number_source()?;
    let canonical_systems = numbering_systems.map(|systems| {
        let mut values = systems.to_vec();
        values.sort_unstable_by(|a, b| a.name().cmp(b.name()));
        values
    });
    let numbering_systems = canonical_systems.as_deref();
    if let Some(systems) = numbering_systems {
        if systems.is_empty()
            || systems
                .windows(2)
                .any(|pair| pair[0].name() >= pair[1].name())
            || systems.iter().any(|value| {
                source
                    .profiles()
                    .numbering_systems()
                    .binary_search_by(|name| name.as_ref().cmp(value.name()))
                    .is_err()
            })
        {
            return Err(IntlDataImageError::consumer(
                "Number numbering selection must contain distinct pinned positional systems",
            ));
        }
    }
    let canonical_currencies = currency_codes.map(|codes| {
        let mut codes = codes.to_vec();
        codes.sort_unstable_by_key(|code| code.clone().ascii());
        codes
    });
    let currency_codes = canonical_currencies.as_deref();
    let public = match public_locales {
        Some(locales) => select_public(locales, locale, &source)?,
        None => source
            .profiles()
            .available_locales()
            .to_vec()
            .into_boxed_slice(),
    };
    if let Some(codes) = currency_codes {
        if codes.is_empty()
            || codes
                .windows(2)
                .any(|pair| pair[0].clone().ascii() >= pair[1].clone().ascii())
            || codes
                .iter()
                .any(|code| !source.profiles().has_currency_data("en-US", code))
        {
            return Err(IntlDataImageError::consumer(
                "Number currency selection must contain sorted distinct pinned codes",
            ));
        }
    }
    let relative_time = crate::relative_time_image::projection::number_dependency_locales(
        relative_time_locales,
        locale,
        &source,
    )?;
    let duration = crate::duration_image::projection::number_dependency_locales(
        duration_locales,
        locale,
        lists,
        &source,
    )?;
    let physical = public
        .iter()
        .chain(relative_time.iter())
        .chain(duration.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .into_boxed_slice();
    // Locale information remains unfiltered. Its complete default-nu authority
    // is small and does not retain the excluded locales' heavy format tables.
    let defaults = source
        .profiles()
        .available_locales()
        .iter()
        .map(|name| {
            let index = source
                .profiles()
                .default_numbering_index(name)
                .ok_or_else(|| {
                    IntlDataImageError::consumer("pinned Number default-nu authority is incomplete")
                })?;
            Ok((name.clone(), index))
        })
        .collect::<Result<Vec<_>, IntlDataImageError>>()?
        .into_boxed_slice();
    let descriptor = Descriptor {
        schema: if numbering_systems.is_some() {
            3
        } else if currency_codes.is_some() {
            2
        } else {
            1
        },
        custom_id: id.as_str().to_owned(),
        default_locale: "en-US".to_owned(),
        full_number_binary_sha256: Sha256::digest(PINNED_BINARY).into(),
        full_number_descriptor_sha256: Sha256::digest(PINNED_DESCRIPTOR).into(),
        relative_time_source_sha256: crate::relative_time_image::projection::source_profile_sha256(
        ),
        duration_source_sha256: crate::duration_image::projection::source_profile_sha256(),
        locale_image_sha256: *locale.digest().as_bytes(),
        list_image_sha256: *lists.digest().as_bytes(),
        public_locales: names(&public),
        relative_time_locales: relative_time_locales.map(|_| names(&relative_time)),
        duration_locales: duration_locales.map(|_| names(&duration)),
        required_relative_time_locales: names(&relative_time),
        required_duration_locales: names(&duration),
        default_numbering: defaults
            .iter()
            .map(|(name, index)| (name.to_string(), *index))
            .collect(),
        currency_codes: currency_codes.map(|codes| {
            codes
                .iter()
                .map(|code| code.clone().ascii().into_iter().map(char::from).collect())
                .collect()
        }),
        numbering_systems: numbering_systems.map(|systems| {
            systems
                .iter()
                .map(|value| value.name().to_owned())
                .collect()
        }),
    };
    let binary = match (currency_codes, numbering_systems) {
        (codes, Some(systems)) => encode_selected_numbering(&source, &physical, codes, systems),
        (Some(codes), None) => encode_selected_currencies(&source, &physical, codes),
        (None, None) => encode_selected(&source, &physical),
    }
    .map_err(IntlDataImageError::consumer)?;
    Ok((
        frame_payload(&descriptor, &binary)?,
        DerivedCatalogue {
            public,
            relative_time,
            duration,
            physical,
            defaults,
            currency_codes: currency_codes.map(|codes| codes.to_vec().into_boxed_slice()),
            numbering_systems: numbering_systems.map(|systems| systems.to_vec().into_boxed_slice()),
        },
    ))
}

fn frame_payload(descriptor: &Descriptor, binary: &[u8]) -> Result<Vec<u8>, IntlDataImageError> {
    let descriptor = serde_json::to_vec(descriptor).map_err(IntlDataImageError::consumer)?;
    if descriptor.len() > MAX_DESCRIPTOR_BYTES || binary.len() > PINNED_BINARY.len() {
        return Err(IntlDataImageError::consumer("Number projection extent"));
    }
    let extent = HEADER_BYTES
        .checked_add(descriptor.len())
        .and_then(|n| n.checked_add(binary.len()))
        .filter(|&n| n <= crate::MAX_INTL_COMPONENT_IMAGE_BYTES)
        .ok_or_else(|| IntlDataImageError::consumer("Number projection extent"))?;
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
        &u64::try_from(binary.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(&descriptor);
    payload.extend_from_slice(binary);
    Ok(payload)
}

pub(super) fn produce(
    id: &CustomProfileId,
    public_locales: &[LocaleId],
    relative_time_locales: Option<&[LocaleId]>,
    duration_locales: Option<&[LocaleId]>,
    locale: &LocaleDataImage,
    lists: &ListDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    project(
        id,
        public_locales,
        relative_time_locales,
        duration_locales,
        locale,
        lists,
    )
    .map(|(payload, _)| payload)
}

pub(super) fn produce_data(
    id: &CustomProfileId,
    public_locales: Option<&[LocaleId]>,
    currency_codes: &[CurrencyCode],
    relative_time_locales: Option<&[LocaleId]>,
    duration_locales: Option<&[LocaleId]>,
    locale: &LocaleDataImage,
    lists: &ListDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    project_data(
        id,
        public_locales,
        Some(currency_codes),
        None,
        relative_time_locales,
        duration_locales,
        locale,
        lists,
    )
    .map(|(payload, _)| payload)
}

pub(super) fn produce_numbering(
    id: &CustomProfileId,
    public_locales: Option<&[LocaleId]>,
    currency_codes: Option<&[CurrencyCode]>,
    systems: &[NumberingSystemOption],
    relative_time_locales: Option<&[LocaleId]>,
    duration_locales: Option<&[LocaleId]>,
    locale: &LocaleDataImage,
    lists: &ListDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    project_data(
        id,
        public_locales,
        currency_codes,
        Some(systems),
        relative_time_locales,
        duration_locales,
        locale,
        lists,
    )
    .map(|(payload, _)| payload)
}

pub(super) fn admit<'a>(
    payload: &'a [u8],
    profile: &IntlDataProfile,
    locale: &LocaleDataImage,
    lists: &ListDataImage,
) -> Result<NumberCatalogue<'a>, IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(IntlDataImageError::consumer(
            "a projected Number payload requires Custom",
        ));
    };
    let (descriptor, binary) = split_payload(payload)?;
    let parse = |values: &[String]| {
        values
            .iter()
            .map(|name| LocaleId::parse(name.as_str()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(IntlDataImageError::consumer)
    };
    let public = parse(&descriptor.public_locales)?;
    let relative_time = descriptor
        .relative_time_locales
        .as_deref()
        .map(parse)
        .transpose()?;
    let duration = descriptor
        .duration_locales
        .as_deref()
        .map(parse)
        .transpose()?;
    let currencies = descriptor
        .currency_codes
        .as_deref()
        .map(|codes| {
            codes
                .iter()
                .map(|code| CurrencyCode::parse(code).map_err(IntlDataImageError::consumer))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let systems = descriptor
        .numbering_systems
        .as_deref()
        .map(|systems| {
            systems
                .iter()
                .map(|name| {
                    NumberingSystemOption::parse(name).map_err(IntlDataImageError::consumer)
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let (expected, derived) = project_data(
        id,
        Some(&public),
        currencies.as_deref(),
        systems.as_deref(),
        relative_time.as_deref(),
        duration.as_deref(),
        locale,
        lists,
    )?;
    // This binds the complete typed reachable table closure, canonical domains,
    // source pins, complete small default-nu authority and selected foundations.
    // No dependent image digest appears here, so construction remains acyclic.
    if payload != expected.as_slice() {
        return Err(IntlDataImageError::consumer(
            "Number projection differs from its pinned derivation",
        ));
    }
    Ok(NumberCatalogue {
        binary,
        public: derived.public,
        relative_time: derived.relative_time,
        duration: derived.duration,
        physical: derived.physical,
        defaults: derived.defaults,
        currency_codes: derived.currency_codes,
        numbering_systems: derived.numbering_systems,
    })
}

fn split_payload(payload: &[u8]) -> Result<(Descriptor, &[u8]), IntlDataImageError> {
    if payload.len() < HEADER_BYTES || &payload[..8] != MAGIC {
        return Err(IntlDataImageError::consumer(
            "invalid Number projection framing",
        ));
    }
    let descriptor_length = u32::from_le_bytes(
        payload[8..12]
            .try_into()
            .map_err(IntlDataImageError::consumer)?,
    ) as usize;
    let binary_length = usize::try_from(u64::from_le_bytes(
        payload[12..20]
            .try_into()
            .map_err(IntlDataImageError::consumer)?,
    ))
    .map_err(IntlDataImageError::consumer)?;
    let binary_start = HEADER_BYTES
        .checked_add(descriptor_length)
        .filter(|_| {
            descriptor_length <= MAX_DESCRIPTOR_BYTES && binary_length <= PINNED_BINARY.len()
        })
        .ok_or_else(|| IntlDataImageError::consumer("invalid Number projection extent"))?;
    if binary_start.checked_add(binary_length) != Some(payload.len()) {
        return Err(IntlDataImageError::consumer(
            "invalid Number projection extent",
        ));
    }
    let descriptor = serde_json::from_slice(&payload[HEADER_BYTES..binary_start])
        .map_err(IntlDataImageError::consumer)?;
    Ok((descriptor, &payload[binary_start..]))
}

#[cfg(test)]
mod tests;
