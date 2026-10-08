//! Canonical selected native rows and their two reachable physical pool domains.

use super::PINNED_NATIVE;
use crate::provider::{DateTimeProvider, LocaleCanonicalizationData};
use crate::{
    CanonicalLocaleId, CustomProfileId, DateTimeCalendar, IntlDataImageError, IntlDataProfile,
    LocaleDataImage, LocaleId, NamedTimeZoneDataImage,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const MAGIC: &[u8; 8] = b"LILADTP1";
const HEADER_BYTES: usize = 20;
const MAX_DESCRIPTOR_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct CalendarPoolId(usize);
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct ZoneNamePoolId(usize);

struct SelectedLocales(Box<[CanonicalLocaleId]>);
impl SelectedLocales {
    fn new(
        requested: &[LocaleId],
        canonicalizer: &LocaleCanonicalizationData,
    ) -> Result<Self, IntlDataImageError> {
        // This validates the exact complete source identity, every original
        // locale, both full pools and all globals before a subset is derived.
        let available = DateTimeProvider::projection_source_locales(
            core::str::from_utf8(PINNED_NATIVE).map_err(IntlDataImageError::consumer)?,
            canonicalizer,
        )
        .map_err(IntlDataImageError::consumer)?;
        if requested.is_empty() || requested.len() > available.len() {
            return Err(invalid("DateTime projection locale extent"));
        }
        let mut selected = BTreeSet::new();
        for requested in requested {
            let canonical = canonicalizer
                .canonicalize(requested)
                .map_err(IntlDataImageError::consumer)?;
            let index = available
                .binary_search_by(|name| name.as_str().cmp(canonical.as_str()))
                .map_err(|_| {
                    invalid(format!(
                        "DateTime projection locale {} is absent from the pinned catalogue",
                        canonical.as_str()
                    ))
                })?;
            if !selected.insert(available[index].clone()) {
                return Err(invalid(
                    "DateTime projection contains duplicate canonical locales",
                ));
            }
        }
        let default = available
            .binary_search_by(|name| name.as_str().cmp("en-US"))
            .map_err(IntlDataImageError::consumer)?;
        selected.insert(available[default].clone());
        Ok(Self(
            selected.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        ))
    }
}

/// Only exact rederivation can mint this bytes/catalogue association. Both the
/// reduced source recipe and the native decoder consume this same admission.
pub(crate) struct DateTimeCatalogue<'a> {
    bytes: &'a [u8],
    locales: Box<[CanonicalLocaleId]>,
    calendar_types: Option<Box<[DateTimeCalendar]>>,
    service_calendars: Box<[DateTimeCalendar]>,
    numbering_systems: Option<Box<[crate::number_format::NumberingSystemOption]>>,
    localized_zones: Option<Box<[Box<str>]>>,
}
impl DateTimeCatalogue<'_> {
    pub(crate) fn bytes(&self) -> &[u8] {
        self.bytes
    }
    pub(crate) fn locales(&self) -> &[CanonicalLocaleId] {
        &self.locales
    }
    pub(crate) fn calendar_types(&self) -> Option<&[DateTimeCalendar]> {
        self.calendar_types.as_deref()
    }
    pub(crate) fn service_calendars(&self) -> &[DateTimeCalendar] {
        &self.service_calendars
    }
    pub(crate) fn numbering_systems(
        &self,
    ) -> Option<&[crate::number_format::NumberingSystemOption]> {
        self.numbering_systems.as_deref()
    }
    pub(crate) fn localized_zones(&self) -> Option<&[Box<str>]> {
        self.localized_zones.as_deref()
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
    named_time_zone_image_sha256: [u8; 32],
    public_locales: Vec<String>,
    source_calendar_pool_ids: Vec<u32>,
    source_zone_name_pool_ids: Vec<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    calendar_types: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    numbering_systems: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    named_time_zones: Option<Vec<String>>,
}

fn invalid(reason: impl core::fmt::Display) -> IntlDataImageError {
    IntlDataImageError::consumer(reason)
}
fn index(value: &serde_json::Value, extent: usize) -> Result<usize, IntlDataImageError> {
    value
        .as_u64()
        .and_then(|index| usize::try_from(index).ok())
        .filter(|&index| index < extent)
        .ok_or_else(|| invalid("pinned DateTime pool reference is absent or out of bounds"))
}
fn calendar_id(
    value: &serde_json::Value,
    extent: usize,
) -> Result<CalendarPoolId, IntlDataImageError> {
    index(value, extent).map(CalendarPoolId)
}
fn zone_name_id(
    value: &serde_json::Value,
    extent: usize,
) -> Result<ZoneNamePoolId, IntlDataImageError> {
    index(value, extent).map(ZoneNamePoolId)
}

fn project(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
    named: &NamedTimeZoneDataImage,
    canonicalizer: &LocaleCanonicalizationData,
) -> Result<(Vec<u8>, SelectedLocales), IntlDataImageError> {
    let profile = IntlDataProfile::Custom(id.clone());
    if locale.profile() != &profile || named.profile() != &profile {
        return Err(invalid(
            "DateTime projection and Locale/IANA profiles differ",
        ));
    }
    let selected = SelectedLocales::new(requested, canonicalizer)?;
    let mut raw: serde_json::Value =
        serde_json::from_slice(PINNED_NATIVE).map_err(IntlDataImageError::consumer)?;
    let rows = raw["locales"]
        .as_array()
        .ok_or_else(|| invalid("pinned DateTime locale rows are absent"))?;
    let mut selected_rows = selected
        .0
        .iter()
        .map(|name| {
            rows.iter()
                .find(|row| row["locale"].as_str() == Some(name.as_str()))
                .cloned()
                .ok_or_else(|| invalid("pinned DateTime selected row is absent"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let calendars = raw["calendar_pool"]
        .as_array()
        .ok_or_else(|| invalid("pinned DateTime calendar pool is absent"))?;
    let zones = raw["zone_name_pool"]
        .as_array()
        .ok_or_else(|| invalid("pinned DateTime zone-name pool is absent"))?;
    let mut used_calendars = BTreeSet::new();
    let mut used_zones = BTreeSet::new();
    for row in &selected_rows {
        let references = row["calendar_refs"]
            .as_array()
            .ok_or_else(|| invalid("pinned DateTime calendar references are absent"))?;
        for reference in references {
            used_calendars.insert(calendar_id(&reference[1], calendars.len())?);
        }
        used_zones.insert(zone_name_id(&row["zone_name_ref"], zones.len())?);
    }
    // Distinct key types make crossing calendar and zone remaps impossible.
    // Original pool order is canonical; keep its reachable subsequence.
    let mut calendar_remap = BTreeMap::new();
    let mut selected_calendars = Vec::with_capacity(used_calendars.len());
    let mut source_calendar_pool_ids = Vec::with_capacity(used_calendars.len());
    for source in used_calendars {
        let dense =
            u32::try_from(selected_calendars.len()).map_err(IntlDataImageError::consumer)?;
        calendar_remap.insert(source, dense);
        selected_calendars.push(calendars[source.0].clone());
        source_calendar_pool_ids
            .push(u32::try_from(source.0).map_err(IntlDataImageError::consumer)?);
    }
    let mut zone_remap = BTreeMap::new();
    let mut selected_zones = Vec::with_capacity(used_zones.len());
    let mut source_zone_name_pool_ids = Vec::with_capacity(used_zones.len());
    for source in used_zones {
        let dense = u32::try_from(selected_zones.len()).map_err(IntlDataImageError::consumer)?;
        zone_remap.insert(source, dense);
        selected_zones.push(zones[source.0].clone());
        source_zone_name_pool_ids
            .push(u32::try_from(source.0).map_err(IntlDataImageError::consumer)?);
    }
    for row in &mut selected_rows {
        let references = row["calendar_refs"]
            .as_array_mut()
            .ok_or_else(|| invalid("pinned DateTime calendar references are absent"))?;
        for reference in references {
            let source = calendar_id(&reference[1], calendars.len())?;
            let dense = calendar_remap
                .get(&source)
                .ok_or_else(|| invalid("incomplete DateTime calendar closure"))?;
            reference[1] = serde_json::Value::from(*dense);
        }
        let source = zone_name_id(&row["zone_name_ref"], zones.len())?;
        let dense = zone_remap
            .get(&source)
            .ok_or_else(|| invalid("incomplete DateTime zone-name closure"))?;
        row["zone_name_ref"] = serde_json::Value::from(*dense);
    }
    let public_locales = selected
        .0
        .iter()
        .map(|name| name.as_str().to_owned())
        .collect::<Vec<_>>();
    raw["selector"]["locales"] = serde_json::json!(&public_locales);
    raw["locales"] = serde_json::Value::Array(selected_rows);
    raw["calendar_pool"] = serde_json::Value::Array(selected_calendars);
    raw["zone_name_pool"] = serde_json::Value::Array(selected_zones);
    let descriptor = Descriptor {
        schema: if named.named_time_zone_selection().is_some() {
            4
        } else {
            1
        },
        custom_id: id.as_str().to_owned(),
        default_locale: "en-US".to_owned(),
        full_profile_sha256: Sha256::digest(PINNED_NATIVE).into(),
        locale_image_sha256: *locale.digest().as_bytes(),
        named_time_zone_image_sha256: *named.digest().as_bytes(),
        public_locales,
        source_calendar_pool_ids,
        source_zone_name_pool_ids,
        calendar_types: None,
        numbering_systems: None,
        named_time_zones: named
            .named_time_zone_selection()
            .map(|names| names.iter().map(|name| name.as_str().to_owned()).collect()),
    };
    if named.named_time_zone_selection().is_some() {
        project_zone_names(&mut raw, named)?;
    }
    let native =
        serde_json::to_vec(&canonical_objects(raw)).map_err(IntlDataImageError::consumer)?;
    Ok((frame(&descriptor, &native)?, selected))
}

fn project_data(
    id: &CustomProfileId,
    requested: Option<&[LocaleId]>,
    calendar_types: &[DateTimeCalendar],
    locale: &LocaleDataImage,
    named: &NamedTimeZoneDataImage,
    canonicalizer: &LocaleCanonicalizationData,
) -> Result<
    (
        Vec<u8>,
        SelectedLocales,
        Box<[DateTimeCalendar]>,
        Box<[DateTimeCalendar]>,
    ),
    IntlDataImageError,
> {
    let all_locales;
    let requested = match requested {
        Some(locales) => locales,
        None => {
            all_locales = DateTimeProvider::projection_source_locales(
                core::str::from_utf8(PINNED_NATIVE).map_err(IntlDataImageError::consumer)?,
                canonicalizer,
            )
            .map_err(IntlDataImageError::consumer)?
            .iter()
            .map(|name| LocaleId::parse(name.as_str()).map_err(IntlDataImageError::consumer))
            .collect::<Result<Vec<_>, _>>()?;
            &all_locales
        }
    };
    let (payload, selected) = project(id, requested, locale, named, canonicalizer)?;
    let (mut descriptor, native) = split(&payload)?;
    let mut raw: serde_json::Value =
        serde_json::from_slice(native).map_err(IntlDataImageError::consumer)?;
    if calendar_types.is_empty() || calendar_types.len() > DateTimeCalendar::ALL.len() {
        return Err(invalid("DateTime calendar selection extent"));
    }
    let mut requested_calendars = BTreeMap::new();
    for &calendar in calendar_types {
        if requested_calendars
            .insert(calendar.as_str(), calendar)
            .is_some()
        {
            return Err(invalid("duplicate DateTime projection calendar"));
        }
    }
    let source_pools = raw["calendar_pool"]
        .as_array()
        .ok_or_else(|| invalid("projected DateTime calendar pool absent"))?
        .clone();
    let mut used = BTreeSet::new();
    let mut service = BTreeSet::new();
    for row in raw["locales"]
        .as_array_mut()
        .ok_or_else(|| invalid("projected DateTime locales absent"))?
    {
        let default = row["calendar_preferences"]
            .as_array()
            .and_then(|preferences| {
                preferences
                    .iter()
                    .find_map(|value| value.as_str().and_then(DateTimeCalendar::parse))
            })
            .ok_or_else(|| invalid("pinned DateTime locale default absent"))?;
        let refs = row["calendar_refs"]
            .as_array_mut()
            .ok_or_else(|| invalid("projected DateTime calendar references absent"))?;
        refs.retain(|reference| {
            reference[0].as_str().is_some_and(|name| {
                name == default.as_str() || requested_calendars.contains_key(name)
            })
        });
        if !refs
            .iter()
            .any(|reference| reference[0].as_str() == Some(default.as_str()))
        {
            return Err(invalid("DateTime default calendar closure incomplete"));
        }
        for reference in refs {
            let name = reference[0]
                .as_str()
                .ok_or_else(|| invalid("calendar association absent"))?;
            service.insert(name.to_owned());
            used.insert(calendar_id(&reference[1], source_pools.len())?);
        }
    }
    let mut remap = BTreeMap::new();
    let mut pools = Vec::with_capacity(used.len());
    let mut source_calendar_pool_ids = Vec::with_capacity(used.len());
    for source in used {
        remap.insert(
            source,
            u32::try_from(pools.len()).map_err(IntlDataImageError::consumer)?,
        );
        pools.push(source_pools[source.0].clone());
        source_calendar_pool_ids.push(descriptor.source_calendar_pool_ids[source.0]);
    }
    for row in raw["locales"]
        .as_array_mut()
        .ok_or_else(|| invalid("projected DateTime locales absent"))?
    {
        for reference in row["calendar_refs"]
            .as_array_mut()
            .ok_or_else(|| invalid("projected DateTime calendar references absent"))?
        {
            let source = calendar_id(&reference[1], source_pools.len())?;
            reference[1] = serde_json::Value::from(remap[&source]);
        }
    }
    let service_calendars = raw["selector"]["calendars"]
        .as_array_mut()
        .ok_or_else(|| invalid("pinned DateTime calendar recipe absent"))?;
    service_calendars.retain(|value| value.as_str().is_some_and(|name| service.contains(name)));
    let service_calendars = service_calendars
        .iter()
        .map(|value| {
            value
                .as_str()
                .and_then(DateTimeCalendar::parse)
                .ok_or_else(|| invalid("invalid DateTime service calendar"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    raw["selector"]["calendar_identifiers"]
        .as_object_mut()
        .ok_or_else(|| invalid("pinned calendar identities absent"))?
        .retain(|name, _| service.contains(name));
    raw["calendar_pool"] = serde_json::Value::Array(pools);
    descriptor.schema = if named.named_time_zone_selection().is_some() {
        4
    } else {
        2
    };
    descriptor.calendar_types = Some(
        requested_calendars
            .keys()
            .map(|name| (*name).to_owned())
            .collect(),
    );
    descriptor.source_calendar_pool_ids = source_calendar_pool_ids;
    let native =
        serde_json::to_vec(&canonical_objects(raw)).map_err(IntlDataImageError::consumer)?;
    Ok((
        frame(&descriptor, &native)?,
        selected,
        requested_calendars
            .into_values()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        service_calendars,
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

fn project_numbering(
    id: &CustomProfileId,
    requested: Option<&[LocaleId]>,
    calendars: Option<&[DateTimeCalendar]>,
    systems: &[crate::number_format::NumberingSystemOption],
    locale: &LocaleDataImage,
    named: &NamedTimeZoneDataImage,
    canonicalizer: &LocaleCanonicalizationData,
) -> Result<
    (
        Vec<u8>,
        SelectedLocales,
        Option<Box<[DateTimeCalendar]>>,
        Box<[DateTimeCalendar]>,
        Box<[crate::number_format::NumberingSystemOption]>,
    ),
    IntlDataImageError,
> {
    let (payload, selected, calendar_types, service_calendars) = match calendars {
        Some(calendars) => {
            let (payload, selected, calendars, service) =
                project_data(id, requested, calendars, locale, named, canonicalizer)?;
            (payload, selected, Some(calendars), service)
        }
        None => {
            let all;
            let requested = match requested {
                Some(locales) => locales,
                None => {
                    all = DateTimeProvider::projection_source_locales(
                        core::str::from_utf8(PINNED_NATIVE)
                            .map_err(IntlDataImageError::consumer)?,
                        canonicalizer,
                    )
                    .map_err(IntlDataImageError::consumer)?
                    .iter()
                    .map(|name| {
                        LocaleId::parse(name.as_str()).map_err(IntlDataImageError::consumer)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                    &all
                }
            };
            let (payload, selected) = project(id, requested, locale, named, canonicalizer)?;
            (
                payload,
                selected,
                None,
                DateTimeCalendar::ALL.to_vec().into_boxed_slice(),
            )
        }
    };
    let (mut descriptor, native) = split(&payload)?;
    let mut raw: serde_json::Value =
        serde_json::from_slice(native).map_err(IntlDataImageError::consumer)?;
    let available = raw["numbering_systems"]
        .as_array()
        .ok_or_else(|| invalid("DateTime digit kernels absent"))?
        .iter()
        .filter_map(|row| row["identifier"].as_str())
        .collect::<BTreeSet<_>>();
    let mut names = BTreeMap::new();
    if systems.is_empty() || systems.len() > available.len() {
        return Err(invalid("DateTime numbering selection extent"));
    }
    for system in systems {
        if !available.contains(system.name())
            || names
                .insert(system.name().to_owned(), system.clone())
                .is_some()
        {
            return Err(invalid(
                "duplicate or nonpositional DateTime numbering selection",
            ));
        }
    }
    for row in raw["locales"]
        .as_array_mut()
        .ok_or_else(|| invalid("DateTime locales absent"))?
    {
        let default = row["default_numbering"]
            .as_str()
            .ok_or_else(|| invalid("DateTime default numbering absent"))?
            .to_owned();
        for field in ["decimal_separators", "minus_signs"] {
            let symbols = row[field]
                .as_array_mut()
                .ok_or_else(|| invalid("DateTime numbering symbols absent"))?;
            symbols.retain(|pair| {
                pair[0]
                    .as_str()
                    .is_some_and(|name| name == default || names.contains_key(name))
            });
            if !symbols
                .iter()
                .any(|pair| pair[0].as_str() == Some(default.as_str()))
            {
                return Err(invalid("DateTime default numbering closure absent"));
            }
        }
    }
    descriptor.schema = if named.named_time_zone_selection().is_some() {
        4
    } else {
        3
    };
    descriptor.numbering_systems = Some(names.keys().cloned().collect());
    let native =
        serde_json::to_vec(&canonical_objects(raw)).map_err(IntlDataImageError::consumer)?;
    Ok((
        frame(&descriptor, &native)?,
        selected,
        calendar_types,
        service_calendars,
        names.into_values().collect::<Vec<_>>().into_boxed_slice(),
    ))
}

fn frame(descriptor: &Descriptor, native: &[u8]) -> Result<Vec<u8>, IntlDataImageError> {
    let descriptor = serde_json::to_vec(descriptor).map_err(IntlDataImageError::consumer)?;
    if descriptor.len() > MAX_DESCRIPTOR_BYTES || native.len() > PINNED_NATIVE.len() {
        return Err(invalid("DateTime projection extent"));
    }
    let extent = HEADER_BYTES
        .checked_add(descriptor.len())
        .and_then(|n| n.checked_add(native.len()))
        .filter(|&n| n <= crate::MAX_INTL_COMPONENT_IMAGE_BYTES)
        .ok_or_else(|| invalid("DateTime projection extent"))?;
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
        &u64::try_from(native.len())
            .map_err(IntlDataImageError::consumer)?
            .to_le_bytes(),
    );
    payload.extend_from_slice(&descriptor);
    payload.extend_from_slice(native);
    Ok(payload)
}

pub(super) fn produce(
    id: &CustomProfileId,
    requested: &[LocaleId],
    locale: &LocaleDataImage,
    named: &NamedTimeZoneDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    let authority =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    project(id, requested, locale, named, &authority).map(|(payload, _)| payload)
}

pub(super) fn produce_named(
    id: &CustomProfileId,
    locale: &LocaleDataImage,
    named: &NamedTimeZoneDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    let authority =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    let requested = DateTimeProvider::projection_source_locales(
        core::str::from_utf8(PINNED_NATIVE).map_err(IntlDataImageError::consumer)?,
        &authority,
    )
    .map_err(IntlDataImageError::consumer)?
    .iter()
    .map(|name| LocaleId::parse(name.as_str()).map_err(IntlDataImageError::consumer))
    .collect::<Result<Vec<_>, _>>()?;
    project(id, &requested, locale, named, &authority).map(|(payload, _)| payload)
}

fn project_zone_names(
    raw: &mut serde_json::Value,
    named: &NamedTimeZoneDataImage,
) -> Result<(), IntlDataImageError> {
    let selection = named
        .named_time_zone_selection()
        .ok_or_else(|| invalid("selected IANA zones absent"))?;
    let mut zones = BTreeSet::new();
    let mut metas = BTreeSet::new();
    for row in raw["zone_geography"]["zones"]
        .as_array()
        .ok_or_else(|| invalid("DateTime geography zones absent"))?
    {
        let selected = row["identifier"]
            .as_str()
            .and_then(|name| crate::TimeZoneId::parse(name).ok())
            .and_then(|name| named.zones_ref().lookup(&name).ok())
            .is_some_and(|identity| {
                selection
                    .binary_search_by(|name| name.as_str().cmp(identity.primary_identifier()))
                    .is_ok()
            });
        if selected {
            zones.insert(row["identifier"].as_str().unwrap().to_owned());
            for period in row["periods"]
                .as_array()
                .ok_or_else(|| invalid("DateTime zone periods absent"))?
            {
                metas.insert(
                    period[2]
                        .as_str()
                        .ok_or_else(|| invalid("DateTime metazone target absent"))?
                        .to_owned(),
                );
            }
        }
    }
    for pool in raw["zone_name_pool"]
        .as_array_mut()
        .ok_or_else(|| invalid("DateTime zone-name pool absent"))?
    {
        pool["zones"]
            .as_array_mut()
            .ok_or_else(|| invalid("localized zone names absent"))?
            .retain(|row| {
                row["identifier"]
                    .as_str()
                    .is_some_and(|name| zones.contains(name))
            });
        pool["metazones"]
            .as_array_mut()
            .ok_or_else(|| invalid("localized metazone names absent"))?
            .retain(|row| {
                row["identifier"]
                    .as_str()
                    .is_some_and(|name| metas.contains(name))
            });
    }
    Ok(())
}

pub(super) fn produce_data(
    id: &CustomProfileId,
    requested: Option<&[LocaleId]>,
    calendar_types: &[DateTimeCalendar],
    locale: &LocaleDataImage,
    named: &NamedTimeZoneDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    let authority =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    project_data(id, requested, calendar_types, locale, named, &authority)
        .map(|(payload, _, _, _)| payload)
}

pub(super) fn produce_numbering(
    id: &CustomProfileId,
    requested: Option<&[LocaleId]>,
    calendars: Option<&[DateTimeCalendar]>,
    systems: &[crate::number_format::NumberingSystemOption],
    locale: &LocaleDataImage,
    named: &NamedTimeZoneDataImage,
) -> Result<Vec<u8>, IntlDataImageError> {
    let authority =
        LocaleCanonicalizationData::from_image(locale).map_err(IntlDataImageError::consumer)?;
    project_numbering(id, requested, calendars, systems, locale, named, &authority)
        .map(|(payload, _, _, _, _)| payload)
}

pub(super) fn admit<'a>(
    payload: &'a [u8],
    profile: &IntlDataProfile,
    locale: &LocaleDataImage,
    named: &NamedTimeZoneDataImage,
    canonicalizer: &LocaleCanonicalizationData,
) -> Result<DateTimeCatalogue<'a>, IntlDataImageError> {
    let IntlDataProfile::Custom(id) = profile else {
        return Err(invalid("a projected DateTime payload requires Custom"));
    };
    let (descriptor, native) = split(payload)?;
    let requested = descriptor
        .public_locales
        .iter()
        .map(|name| LocaleId::parse(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(IntlDataImageError::consumer)?;
    let schema = if descriptor.schema == 4 {
        if descriptor.numbering_systems.is_some() {
            3
        } else if descriptor.calendar_types.is_some() {
            2
        } else {
            1
        }
    } else {
        descriptor.schema
    };
    let (expected, selected, calendar_types, service_calendars, numbering_systems) = match (
        schema,
        &descriptor.calendar_types,
        &descriptor.numbering_systems,
    ) {
        (1, None, None) => {
            let (expected, selected) = project(id, &requested, locale, named, canonicalizer)?;
            (
                expected,
                selected,
                None,
                DateTimeCalendar::ALL.to_vec().into_boxed_slice(),
                None,
            )
        }
        (2, Some(types), None) => {
            let types = types
                .iter()
                .map(|name| {
                    DateTimeCalendar::parse(name)
                        .ok_or_else(|| invalid("invalid DateTime calendar selection"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let (expected, selected, types, service) =
                project_data(id, Some(&requested), &types, locale, named, canonicalizer)?;
            (expected, selected, Some(types), service, None)
        }
        (3, calendars, Some(systems)) => {
            let calendars = calendars
                .as_deref()
                .map(|names| {
                    names
                        .iter()
                        .map(|name| {
                            DateTimeCalendar::parse(name)
                                .ok_or_else(|| invalid("invalid DateTime calendar selection"))
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            let systems = systems
                .iter()
                .map(|name| {
                    crate::number_format::NumberingSystemOption::parse(name)
                        .map_err(IntlDataImageError::consumer)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let (expected, selected, calendars, service, systems) = project_numbering(
                id,
                Some(&requested),
                calendars.as_deref(),
                &systems,
                locale,
                named,
                canonicalizer,
            )?;
            (expected, selected, calendars, service, Some(systems))
        }
        _ => return Err(invalid("invalid DateTime projection schema")),
    };
    if payload != expected.as_slice() {
        return Err(invalid(
            "DateTime projection differs from its pinned derivation",
        ));
    }
    let localized_zones = if named.named_time_zone_selection().is_some() {
        let raw: serde_json::Value =
            serde_json::from_slice(native).map_err(IntlDataImageError::consumer)?;
        Some(
            raw["zone_name_pool"][0]["zones"]
                .as_array()
                .ok_or_else(|| invalid("admitted localized zone names absent"))?
                .iter()
                .map(|row| {
                    row["identifier"]
                        .as_str()
                        .map(Box::<str>::from)
                        .ok_or_else(|| invalid("admitted localized zone identity absent"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
        )
    } else {
        None
    };
    Ok(DateTimeCatalogue {
        bytes: native,
        locales: selected.0,
        calendar_types,
        service_calendars,
        numbering_systems,
        localized_zones,
    })
}

fn split(payload: &[u8]) -> Result<(Descriptor, &[u8]), IntlDataImageError> {
    if payload.len() < HEADER_BYTES || payload.get(..8) != Some(MAGIC.as_slice()) {
        return Err(invalid("invalid DateTime projection framing"));
    }
    let descriptor_length = usize::try_from(u32::from_le_bytes(
        payload[8..12]
            .try_into()
            .map_err(IntlDataImageError::consumer)?,
    ))
    .map_err(IntlDataImageError::consumer)?;
    let native_length = usize::try_from(u64::from_le_bytes(
        payload[12..20]
            .try_into()
            .map_err(IntlDataImageError::consumer)?,
    ))
    .map_err(IntlDataImageError::consumer)?;
    let start = HEADER_BYTES
        .checked_add(descriptor_length)
        .filter(|_| {
            descriptor_length <= MAX_DESCRIPTOR_BYTES && native_length <= PINNED_NATIVE.len()
        })
        .ok_or_else(|| invalid("invalid DateTime projection extent"))?;
    if start.checked_add(native_length) != Some(payload.len()) {
        return Err(invalid("invalid DateTime projection extent"));
    }
    let descriptor = serde_json::from_slice(&payload[HEADER_BYTES..start])
        .map_err(IntlDataImageError::consumer)?;
    Ok((descriptor, &payload[start..]))
}

#[cfg(test)]
mod tests;
