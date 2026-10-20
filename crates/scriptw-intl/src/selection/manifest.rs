//! External input can only construct the existing physical projection owner.
use super::{CustomIntlProfile, InvalidCustomIntlProfile};
use crate::{CustomProfileId, InvalidCustomProfileId};
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub enum InvalidCustomIntlManifest {
    TooLarge,
    Decode(String),
    UnsupportedVersion(u16),
    Identity(InvalidCustomProfileId),
    Projection(InvalidCustomIntlProfile),
}
impl core::fmt::Display for InvalidCustomIntlManifest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooLarge => write!(f, "Custom Intl manifest exceeds 256 KiB"),
            Self::Decode(message) => write!(f, "invalid Custom Intl manifest: {message}; v1 accepts locale filters, v2 requires currency_codes, v3 requires date_time_calendars, v4 requires numbering_systems, v5 requires named_time_zones, v6 requires services"),
            Self::UnsupportedVersion(version) => write!(f, "unsupported Custom Intl manifest version {version}; expected 1, 2, 3, 4, 5 or 6"),
            Self::Identity(error) => write!(f, "{error}"), Self::Projection(error) => write!(f, "{error}"),
        }
    }
}
impl std::error::Error for InvalidCustomIntlManifest {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Identity(error) => Some(error),
            Self::Projection(error) => Some(error),
            Self::TooLarge | Self::Decode(_) | Self::UnsupportedVersion(_) => None,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u16,
    custom_id: String,
    locale_filters: Filters,
}
#[derive(Deserialize)]
struct ManifestVersion {
    schema_version: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CurrencyManifest {
    schema_version: u16,
    custom_id: String,
    #[serde(default)]
    locale_filters: Filters,
    currency_codes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CalendarManifest {
    schema_version: u16,
    custom_id: String,
    #[serde(default)]
    locale_filters: Filters,
    #[serde(default, deserialize_with = "present_locales")]
    currency_codes: Option<Vec<String>>,
    date_time_calendars: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NumberingManifest {
    schema_version: u16,
    custom_id: String,
    #[serde(default)]
    locale_filters: Filters,
    #[serde(default, deserialize_with = "present_locales")]
    currency_codes: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    date_time_calendars: Option<Vec<String>>,
    numbering_systems: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedZoneManifest {
    schema_version: u16,
    custom_id: String,
    #[serde(default)]
    locale_filters: Filters,
    #[serde(default, deserialize_with = "present_locales")]
    currency_codes: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    date_time_calendars: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    numbering_systems: Option<Vec<String>>,
    named_time_zones: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceManifest {
    schema_version: u16,
    custom_id: String,
    services: Vec<String>,
    #[serde(default)]
    locale_filters: Filters,
    #[serde(default, deserialize_with = "present_locales")]
    currency_codes: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    date_time_calendars: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    numbering_systems: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    named_time_zones: Option<Vec<String>>,
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Filters {
    #[serde(default, deserialize_with = "present_locales")]
    list: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    relative_time: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    display_names: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    duration: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    number_plural: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    date_time: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    collator: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_locales")]
    segmenter: Option<Vec<String>>,
}
fn present_locales<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error> {
    // Only absence means full. An explicit null is not an alternate spelling.
    Vec::<String>::deserialize(deserializer).map(Some)
}
fn borrowed(values: &Option<Vec<String>>) -> Option<Vec<&str>> {
    values
        .as_ref()
        .map(|values| values.iter().map(String::as_str).collect())
}
impl CustomIntlProfile {
    pub const MANIFEST_MAX_BYTES: usize = 256 * 1024;
    pub fn from_manifest_json(json: &str) -> Result<Self, InvalidCustomIntlManifest> {
        if json.len() > Self::MANIFEST_MAX_BYTES {
            return Err(InvalidCustomIntlManifest::TooLarge);
        }
        let version: ManifestVersion = serde_json::from_str(json)
            .map_err(|error| InvalidCustomIntlManifest::Decode(error.to_string()))?;
        if version.schema_version == 6 {
            let manifest: ServiceManifest = serde_json::from_str(json)
                .map_err(|error| InvalidCustomIntlManifest::Decode(error.to_string()))?;
            let id = CustomProfileId::parse(manifest.custom_id)
                .map_err(InvalidCustomIntlManifest::Identity)?;
            let fields = manifest.locale_filters;
            let inputs = [
                borrowed(&fields.list),
                borrowed(&fields.relative_time),
                borrowed(&fields.display_names),
                borrowed(&fields.duration),
                borrowed(&fields.number_plural),
                borrowed(&fields.date_time),
                borrowed(&fields.collator),
                borrowed(&fields.segmenter),
            ];
            let services: Vec<_> = manifest.services.iter().map(String::as_str).collect();
            let mut profile = if inputs.iter().any(Option::is_some) {
                Self::new(
                    id,
                    inputs[0].as_deref(),
                    inputs[1].as_deref(),
                    inputs[2].as_deref(),
                    inputs[3].as_deref(),
                    inputs[4].as_deref(),
                    inputs[5].as_deref(),
                    inputs[6].as_deref(),
                    inputs[7].as_deref(),
                )
                .and_then(|profile| profile.with_services(&services))
            } else {
                Self::for_services(id, &services)
            }
            .map_err(InvalidCustomIntlManifest::Projection)?;
            macro_rules! project {
                ($field:ident, $method:ident) => {
                    if let Some(values) = manifest.$field {
                        let values: Vec<_> = values.iter().map(String::as_str).collect();
                        profile = profile
                            .$method(&values)
                            .map_err(InvalidCustomIntlManifest::Projection)?;
                    }
                };
            }
            project!(currency_codes, with_currency_codes);
            project!(date_time_calendars, with_date_time_calendars);
            project!(numbering_systems, with_numbering_systems);
            project!(named_time_zones, with_named_time_zones);
            return Ok(profile);
        }
        let (id, fields, currencies, calendars, numbering, named_zones) =
            match version.schema_version {
                1 => {
                    let manifest: Manifest = serde_json::from_str(json)
                        .map_err(|error| InvalidCustomIntlManifest::Decode(error.to_string()))?;
                    if manifest.schema_version != 1 {
                        return Err(InvalidCustomIntlManifest::UnsupportedVersion(
                            manifest.schema_version,
                        ));
                    }
                    (
                        manifest.custom_id,
                        manifest.locale_filters,
                        None,
                        None,
                        None,
                        None,
                    )
                }
                2 => {
                    let manifest: CurrencyManifest = serde_json::from_str(json)
                        .map_err(|error| InvalidCustomIntlManifest::Decode(error.to_string()))?;
                    if manifest.schema_version != 2 {
                        return Err(InvalidCustomIntlManifest::UnsupportedVersion(
                            manifest.schema_version,
                        ));
                    }
                    (
                        manifest.custom_id,
                        manifest.locale_filters,
                        Some(manifest.currency_codes),
                        None,
                        None,
                        None,
                    )
                }
                3 => {
                    let manifest: CalendarManifest = serde_json::from_str(json)
                        .map_err(|error| InvalidCustomIntlManifest::Decode(error.to_string()))?;
                    if manifest.schema_version != 3 {
                        return Err(InvalidCustomIntlManifest::UnsupportedVersion(
                            manifest.schema_version,
                        ));
                    }
                    (
                        manifest.custom_id,
                        manifest.locale_filters,
                        manifest.currency_codes,
                        Some(manifest.date_time_calendars),
                        None,
                        None,
                    )
                }
                4 => {
                    let manifest: NumberingManifest = serde_json::from_str(json)
                        .map_err(|error| InvalidCustomIntlManifest::Decode(error.to_string()))?;
                    if manifest.schema_version != 4 {
                        return Err(InvalidCustomIntlManifest::UnsupportedVersion(
                            manifest.schema_version,
                        ));
                    }
                    (
                        manifest.custom_id,
                        manifest.locale_filters,
                        manifest.currency_codes,
                        manifest.date_time_calendars,
                        Some(manifest.numbering_systems),
                        None,
                    )
                }
                5 => {
                    let manifest: NamedZoneManifest = serde_json::from_str(json)
                        .map_err(|error| InvalidCustomIntlManifest::Decode(error.to_string()))?;
                    if manifest.schema_version != 5 {
                        return Err(InvalidCustomIntlManifest::UnsupportedVersion(
                            manifest.schema_version,
                        ));
                    }
                    (
                        manifest.custom_id,
                        manifest.locale_filters,
                        manifest.currency_codes,
                        manifest.date_time_calendars,
                        manifest.numbering_systems,
                        Some(manifest.named_time_zones),
                    )
                }
                other => return Err(InvalidCustomIntlManifest::UnsupportedVersion(other)),
            };
        let id =
            CustomProfileId::parse(id.as_str()).map_err(InvalidCustomIntlManifest::Identity)?;
        let list = borrowed(&fields.list);
        let relative_time = borrowed(&fields.relative_time);
        let display_names = borrowed(&fields.display_names);
        let duration = borrowed(&fields.duration);
        let number = borrowed(&fields.number_plural);
        let date_time = borrowed(&fields.date_time);
        let collator = borrowed(&fields.collator);
        let segmenter = borrowed(&fields.segmenter);
        let has_locales = [
            list.is_some(),
            relative_time.is_some(),
            display_names.is_some(),
            duration.is_some(),
            number.is_some(),
            date_time.is_some(),
            collator.is_some(),
            segmenter.is_some(),
        ]
        .into_iter()
        .any(|present| present);
        let currency_input = borrowed(&currencies);
        let calendar_input = borrowed(&calendars);
        let numbering_input = borrowed(&numbering);
        let named_input = borrowed(&named_zones);
        let profile = if has_locales {
            Self::new(
                id,
                list.as_deref(),
                relative_time.as_deref(),
                display_names.as_deref(),
                duration.as_deref(),
                number.as_deref(),
                date_time.as_deref(),
                collator.as_deref(),
                segmenter.as_deref(),
            )
        } else if let Some(codes) = currency_input.as_deref() {
            Self::for_currency_codes(id, codes)
        } else if let Some(calendars) = calendar_input.as_deref() {
            Self::for_date_time_calendars(id, calendars)
        } else if let Some(systems) = numbering_input.as_deref() {
            Self::for_numbering_systems(id, systems)
        } else if let Some(zones) = named_input.as_deref() {
            Self::for_named_time_zones(id, zones)
        } else {
            Err(InvalidCustomIntlProfile::NoProjection)
        };
        let mut profile = profile.map_err(InvalidCustomIntlManifest::Projection)?;
        if profile.currency_codes().is_none() {
            if let Some(codes) = currency_input.as_deref() {
                profile = profile
                    .with_currency_codes(codes)
                    .map_err(InvalidCustomIntlManifest::Projection)?;
            }
        }
        if profile.date_time_calendars().is_none() {
            if let Some(calendars) = calendar_input.as_deref() {
                profile = profile
                    .with_date_time_calendars(calendars)
                    .map_err(InvalidCustomIntlManifest::Projection)?;
            }
        }
        if profile.numbering_systems().is_none() {
            if let Some(systems) = numbering_input.as_deref() {
                profile = profile
                    .with_numbering_systems(systems)
                    .map_err(InvalidCustomIntlManifest::Projection)?;
            }
        }
        if profile.named_time_zones().is_none() {
            if let Some(zones) = named_input.as_deref() {
                profile = profile
                    .with_named_time_zones(zones)
                    .map_err(InvalidCustomIntlManifest::Projection)?;
            }
        }
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntlCompilationProfile, IntlDataSelection};
    #[test]
    fn named_zone_v5_canonicalizes_real_aliases_and_projects_actual_dependent_frames() {
        let json = r#"{"schema_version":5,"custom_id":"named-zone-manifest","named_time_zones":["us/eastern"],"numbering_systems":["deva"],"date_time_calendars":["chinese"],"currency_codes":["EUR"],"locale_filters":{"date_time":["fr"]}}"#;
        let from_manifest = CustomIntlProfile::from_manifest_json(json).unwrap();
        let direct = CustomIntlProfile::new(
            CustomProfileId::parse("named-zone-manifest").unwrap(),
            None,
            None,
            None,
            None,
            None,
            Some(&["fr"]),
            None,
            None,
        )
        .unwrap()
        .with_currency_codes(&["EUR"])
        .unwrap()
        .with_date_time_calendars(&["chinese"])
        .unwrap()
        .with_numbering_systems(&["deva"])
        .unwrap()
        .with_named_time_zones(&["America/New_York"])
        .unwrap();
        assert_eq!(from_manifest, direct);
        let owner = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(from_manifest));
        let selected = owner.selected().unwrap();
        let full = IntlDataSelection::new(IntlCompilationProfile::Custom(
            CustomProfileId::parse("named-zone-manifest").unwrap(),
        ));
        for section in [
            crate::INTL_NAMED_TIME_ZONE_DATA_CUSTOM_SECTION,
            crate::INTL_DATETIME_DATA_CUSTOM_SECTION,
            crate::INTL_TIME_ZONE_NAMES_DATA_CUSTOM_SECTION,
        ] {
            let projected = selected
                .component_sections()
                .into_iter()
                .find(|(name, _)| *name == section)
                .unwrap()
                .1;
            let complete = full
                .selected()
                .unwrap()
                .component_sections()
                .into_iter()
                .find(|(name, _)| *name == section)
                .unwrap()
                .1;
            assert!(
                projected.len() < complete.len(),
                "physical named-zone closure {section}"
            );
        }
        assert_eq!(
            selected
                .supported_values(crate::SupportedValuesKey::TimeZone)
                .unwrap()
                .values(),
            full.selected()
                .unwrap()
                .supported_values(crate::SupportedValuesKey::TimeZone)
                .unwrap()
                .values()
        );
        let exported = selected.export_bytes().unwrap();
        let admitted = crate::SelectedIntlDataBundle::from_export_bytes(&exported).unwrap();
        assert_eq!(admitted.export_bytes().unwrap(), exported);
        let only = CustomIntlProfile::from_manifest_json(r#"{"schema_version":5,"custom_id":"named-only","named_time_zones":["US/Eastern","Europe/London"]}"#).unwrap();
        assert_eq!(
            only,
            CustomIntlProfile::for_named_time_zones(
                CustomProfileId::parse("named-only").unwrap(),
                &["Europe/London", "America/New_York"]
            )
            .unwrap()
        );
    }
    #[test]
    fn named_zone_v5_rejects_offsets_unknown_alias_duplicates_and_foreign_manifest_keys() {
        for json in [
            r#"{"schema_version":5,"custom_id":"bad","named_time_zones":[]}"#,
            r#"{"schema_version":5,"custom_id":"bad","named_time_zones":null}"#,
            r#"{"schema_version":5,"custom_id":"bad","named_time_zones":["US/Eastern","America/New_York"]}"#,
            r#"{"schema_version":5,"custom_id":"bad","named_time_zones":["Europe/Unknown"]}"#,
            r#"{"schema_version":5,"custom_id":"bad","named_time_zones":["+05:30"]}"#,
            r#"{"schema_version":5,"custom_id":"bad","named_time_zones":["UTC"],"named_time_zones":["America/New_York"]}"#,
            r#"{"schema_version":5,"custom_id":"bad","named_time_zones":["UTC"],"numbering_systems":null}"#,
            r#"{"schema_version":5,"custom_id":"bad","named_time_zones":["UTC"],"services":["DateTimeFormat"]}"#,
            r#"{"schema_version":1,"custom_id":"bad","locale_filters":{"list":["fr"]},"named_time_zones":["UTC"]}"#,
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":["EUR"],"named_time_zones":["UTC"]}"#,
            r#"{"schema_version":3,"custom_id":"bad","date_time_calendars":["chinese"],"named_time_zones":["UTC"]}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":["deva"],"named_time_zones":["UTC"]}"#,
        ] {
            assert!(
                CustomIntlProfile::from_manifest_json(json).is_err(),
                "{json}"
            );
        }
    }
    #[test]
    fn numbering_v4_projects_both_actual_owners_and_composes_prior_data_dimensions() {
        let json = r#"{"schema_version":4,"custom_id":"numbering-manifest","numbering_systems":["DEVA"],"date_time_calendars":["chinese"],"currency_codes":["EUR"],"locale_filters":{"number_plural":["fr","ar-EG"],"date_time":["fr","ar-EG"]}}"#;
        let from_manifest = CustomIntlProfile::from_manifest_json(json).unwrap();
        let direct = CustomIntlProfile::new(
            CustomProfileId::parse("numbering-manifest").unwrap(),
            None,
            None,
            None,
            None,
            Some(&["ar-EG", "fr"]),
            Some(&["ar-EG", "fr"]),
            None,
            None,
        )
        .unwrap()
        .with_currency_codes(&["EUR"])
        .unwrap()
        .with_date_time_calendars(&["chinese"])
        .unwrap()
        .with_numbering_systems(&["deva"])
        .unwrap();
        assert_eq!(from_manifest, direct);
        let projected =
            IntlDataSelection::new(IntlCompilationProfile::CustomProjection(from_manifest));
        let full = IntlDataSelection::new(IntlCompilationProfile::Custom(
            CustomProfileId::parse("numbering-manifest").unwrap(),
        ));
        let selected = projected.selected().unwrap();
        for section in [
            crate::INTL_NUMBER_DATA_CUSTOM_SECTION,
            crate::INTL_DATETIME_DATA_CUSTOM_SECTION,
        ] {
            let selected_bytes = selected
                .component_sections()
                .into_iter()
                .find(|(name, _)| *name == section)
                .unwrap()
                .1;
            let full_bytes = full
                .selected()
                .unwrap()
                .component_sections()
                .into_iter()
                .find(|(name, _)| *name == section)
                .unwrap()
                .1;
            assert!(
                selected_bytes.len() < full_bytes.len(),
                "physical numbering closure {section}"
            );
        }
        assert_eq!(
            selected
                .supported_values(crate::SupportedValuesKey::NumberingSystem)
                .unwrap()
                .values(),
            full.selected()
                .unwrap()
                .supported_values(crate::SupportedValuesKey::NumberingSystem)
                .unwrap()
                .values()
        );
        let bytes = selected.export_bytes().unwrap();
        let admitted = crate::SelectedIntlDataBundle::from_export_bytes(&bytes).unwrap();
        assert_eq!(admitted.export_bytes().unwrap(), bytes);
        assert_eq!(
            admitted
                .supported_values(crate::SupportedValuesKey::NumberingSystem)
                .unwrap()
                .values()
                .len(),
            78
        );
        let only = CustomIntlProfile::from_manifest_json(r#"{"schema_version":4,"custom_id":"numbering-only","numbering_systems":["LATN","deva"]}"#).unwrap();
        assert_eq!(
            only,
            CustomIntlProfile::for_numbering_systems(
                CustomProfileId::parse("numbering-only").unwrap(),
                &["deva", "latn"]
            )
            .unwrap()
        );
    }
    #[test]
    fn numbering_v4_rejects_unknown_or_ambiguous_data_without_changing_older_manifests() {
        for json in [
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":[]}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":null}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":["DEVA","deva"]}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":["zzzz"]}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":["a"]}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":["deva"],"numbering_systems":["latn"]}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":["deva"],"date_time_calendars":null}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":["deva"],"currency_codes":null}"#,
            r#"{"schema_version":4,"custom_id":"bad","numbering_systems":["deva"],"services":["NumberFormat"]}"#,
            r#"{"schema_version":1,"custom_id":"bad","locale_filters":{"list":["fr"]},"numbering_systems":["deva"]}"#,
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":["EUR"],"numbering_systems":["deva"]}"#,
            r#"{"schema_version":3,"custom_id":"bad","date_time_calendars":["chinese"],"numbering_systems":["deva"]}"#,
        ] {
            assert!(
                CustomIntlProfile::from_manifest_json(json).is_err(),
                "{json}"
            );
        }
        for version in [1, 2, 3] {
            let json = match version {
                1 => r#"{"schema_version":1,"custom_id":"old","locale_filters":{"list":["fr"]}}"#,
                2 => r#"{"schema_version":2,"custom_id":"old","currency_codes":["EUR"]}"#,
                3 => r#"{"schema_version":3,"custom_id":"old","date_time_calendars":["chinese"]}"#,
                _ => unreachable!(),
            };
            assert!(CustomIntlProfile::from_manifest_json(json)
                .unwrap()
                .numbering_systems()
                .is_none());
        }
    }
    #[test]
    fn calendar_v3_composes_real_data_filters_without_narrowing_global_kernel_authority() {
        let json = r#"{"schema_version":3,"custom_id":"calendar-manifest","date_time_calendars":["chinese"],"currency_codes":["JPY","EUR"],"locale_filters":{"date_time":["fr"]}}"#;
        let from_manifest = CustomIntlProfile::from_manifest_json(json).unwrap();
        let direct = CustomIntlProfile::new(
            CustomProfileId::parse("calendar-manifest").unwrap(),
            None,
            None,
            None,
            None,
            None,
            Some(&["fr"]),
            None,
            None,
        )
        .unwrap()
        .with_currency_codes(&["EUR", "JPY"])
        .unwrap()
        .with_date_time_calendars(&["chinese"])
        .unwrap();
        assert_eq!(from_manifest, direct);
        let selection =
            IntlDataSelection::new(IntlCompilationProfile::CustomProjection(from_manifest));
        let selected = selection.selected().unwrap();
        assert_eq!(
            selected
                .supported_values(crate::SupportedValuesKey::Calendar)
                .unwrap()
                .values()
                .len(),
            16
        );
        assert_eq!(
            selected
                .supported_values(crate::SupportedValuesKey::Currency)
                .unwrap()
                .values()
                .iter()
                .map(|code| code.as_ref())
                .collect::<Vec<_>>(),
            ["EUR", "JPY"]
        );
        let admitted =
            crate::SelectedIntlDataBundle::from_export_bytes(&selected.export_bytes().unwrap())
                .unwrap();
        assert_eq!(admitted.identity(), selected.identity());
        assert_eq!(
            admitted
                .supported_values(crate::SupportedValuesKey::Calendar)
                .unwrap()
                .values()
                .len(),
            16
        );
        let only = CustomIntlProfile::from_manifest_json(r#"{"schema_version":3,"custom_id":"calendar-only","date_time_calendars":["gregory","chinese"]}"#).unwrap();
        assert_eq!(
            only,
            CustomIntlProfile::for_date_time_calendars(
                CustomProfileId::parse("calendar-only").unwrap(),
                &["chinese", "gregory"]
            )
            .unwrap()
        );
        for json in [
            r#"{"schema_version":3,"custom_id":"bad","date_time_calendars":[]}"#,
            r#"{"schema_version":3,"custom_id":"bad","date_time_calendars":null}"#,
            r#"{"schema_version":3,"custom_id":"bad","date_time_calendars":["chinese","chinese"]}"#,
            r#"{"schema_version":3,"custom_id":"bad","date_time_calendars":["unknown"]}"#,
            r#"{"schema_version":3,"custom_id":"bad","date_time_calendars":["chinese"],"date_time_calendars":["gregory"]}"#,
            r#"{"schema_version":3,"custom_id":"bad","date_time_calendars":["chinese"],"currency_codes":null}"#,
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":["EUR"],"date_time_calendars":["chinese"]}"#,
        ] {
            assert!(
                CustomIntlProfile::from_manifest_json(json).is_err(),
                "{json}"
            );
        }
    }
    #[test]
    fn currency_v2_rederives_both_physical_owners_and_the_exported_supported_catalogue() {
        let json = r#"{"schema_version":2,"custom_id":"currency-manifest","currency_codes":["jpy","EUR"]}"#;
        let from_manifest = CustomIntlProfile::from_manifest_json(json).unwrap();
        let direct = CustomIntlProfile::for_currency_codes(
            CustomProfileId::parse("currency-manifest").unwrap(),
            &["EUR", "JPY"],
        )
        .unwrap();
        assert_eq!(from_manifest, direct);
        let selection =
            IntlDataSelection::new(IntlCompilationProfile::CustomProjection(from_manifest));
        let selected = selection.selected().unwrap();
        assert_eq!(
            selected
                .supported_values(crate::SupportedValuesKey::Currency)
                .unwrap()
                .values()
                .iter()
                .map(|code| code.as_ref())
                .collect::<Vec<_>>(),
            ["EUR", "JPY"]
        );
        let full = IntlDataSelection::new(IntlCompilationProfile::Custom(
            CustomProfileId::parse("currency-manifest").unwrap(),
        ));
        let original = full.selected().unwrap();
        for ((name, selected), (_, original)) in selected
            .component_sections()
            .into_iter()
            .zip(original.component_sections())
        {
            if matches!(
                name,
                crate::INTL_NUMBER_DATA_CUSTOM_SECTION
                    | crate::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION
            ) {
                assert!(
                    selected.len() < original.len(),
                    "physical currency closure {name}"
                );
            } else if !matches!(
                name,
                crate::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION
                    | crate::INTL_DURATION_DATA_CUSTOM_SECTION
            ) {
                assert_eq!(selected, original, "unfiltered authority {name}");
            }
        }
        let exported = selected.export_bytes().unwrap();
        let admitted = crate::SelectedIntlDataBundle::from_export_bytes(&exported).unwrap();
        assert_eq!(admitted.export_bytes().unwrap(), exported);
        assert_eq!(admitted.identity(), selected.identity());
        assert_eq!(
            selected
                .supported_values(crate::SupportedValuesKey::NumberingSystem)
                .unwrap()
                .values()
                .len(),
            78
        );
    }

    #[test]
    fn currency_v2_composes_with_locale_filters_and_rejects_fake_or_ambiguous_dimensions() {
        let profile = CustomIntlProfile::from_manifest_json(r#"{"schema_version":2,"custom_id":"currency-composition","locale_filters":{"number_plural":["fr"],"display_names":["fr"]},"currency_codes":["EUR"]}"#).unwrap();
        let direct = CustomIntlProfile::new(
            CustomProfileId::parse("currency-composition").unwrap(),
            None,
            None,
            Some(&["fr"]),
            None,
            Some(&["fr"]),
            None,
            None,
            None,
        )
        .unwrap()
        .with_currency_codes(&["EUR"])
        .unwrap();
        assert_eq!(profile, direct);
        for json in [
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":[]}"#,
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":null}"#,
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":["EUR","eur"]}"#,
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":["ZZZ"]}"#,
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":["EUR"],"currency_codes":["JPY"]}"#,
            r#"{"schema_version":1,"custom_id":"bad","locale_filters":{"list":["fr"]},"currency_codes":["EUR"]}"#,
            r#"{"schema_version":2,"custom_id":"bad","currency_codes":["EUR"],"calendars":["gregory"]}"#,
        ] {
            assert!(
                CustomIntlProfile::from_manifest_json(json).is_err(),
                "{json}"
            );
        }
    }
    #[test]
    fn manifest_constructs_the_same_sorted_checked_profile_and_physical_bundle() {
        let manifest = r#"{"schema_version":1,"custom_id":"manifest-source","locale_filters":{
            "list":["fr","en-US"],"relative_time":["fr"],"display_names":["fr"],"duration":["fr"],
            "number_plural":["fr"],"date_time":["fr"],"collator":["fr"],"segmenter":["fr"]}}"#;
        let from_manifest = CustomIntlProfile::from_manifest_json(manifest).unwrap();
        let expected = CustomIntlProfile::new(
            CustomProfileId::parse("manifest-source").unwrap(),
            Some(&["en-US", "fr"]),
            Some(&["fr"]),
            Some(&["fr"]),
            Some(&["fr"]),
            Some(&["fr"]),
            Some(&["fr"]),
            Some(&["fr"]),
            Some(&["fr"]),
        )
        .unwrap();
        assert_eq!(from_manifest, expected);
        let selected =
            IntlDataSelection::new(IntlCompilationProfile::CustomProjection(from_manifest));
        let direct = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(expected));
        let selected = selected.selected().unwrap();
        let direct = direct.selected().unwrap();
        assert_eq!(selected.identity(), direct.identity());
        for ((left_name, left), (right_name, right)) in selected
            .component_sections()
            .into_iter()
            .zip(direct.component_sections())
        {
            assert_eq!(left_name, right_name);
            assert_eq!(left, right);
        }
    }
    #[test]
    fn manifest_rejects_unknown_dimensions_null_duplicates_empty_and_foreign_versions() {
        for filters in [
            r#"{"list":[]}"#,
            r#"{"list":null}"#,
            r#"{"list":["fr","fr"]}"#,
            r#"{"list":["fr"],"list":["en-US"]}"#,
            r#"{"calendars":["gregory"]}"#,
            r#"{"numbering_systems":["latn"]}"#,
            r#"{"currencies":["EUR"]}"#,
            r#"{"named_zones":["UTC"]}"#,
            r#"{"services":["NumberFormat"]}"#,
            "{}",
        ] {
            assert!(CustomIntlProfile::from_manifest_json(&format!(
                r#"{{"schema_version":1,"custom_id":"manifest-source","locale_filters":{filters}}}"#)).is_err(), "{filters}");
        }
        assert!(matches!(
            CustomIntlProfile::from_manifest_json(
                r#"{"schema_version":2,"custom_id":"manifest-source","locale_filters":{"list":["fr"]}}"#
            ),
            Err(InvalidCustomIntlManifest::Decode(_))
        ));
        assert!(CustomIntlProfile::from_manifest_json(
            r#"{"schema_version":1,"custom_id":"bad ID","locale_filters":{"list":["fr"]}}"#
        )
        .is_err());
        assert!(CustomIntlProfile::from_manifest_json(r#"{"schema_version":1,"custom_id":"x","custom_id":"y","locale_filters":{"list":["fr"]}}"#).is_err());
        assert!(matches!(
            CustomIntlProfile::from_manifest_json(
                &" ".repeat(CustomIntlProfile::MANIFEST_MAX_BYTES + 1)
            ),
            Err(InvalidCustomIntlManifest::TooLarge)
        ));
    }
}
