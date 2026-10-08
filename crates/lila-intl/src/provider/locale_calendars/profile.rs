use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use super::LocaleCalendarsProfileError;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    schema: u32,
    algorithm: String,
    cldr_release: String,
    cldr_commit: String,
    primary_manifest_sha256: String,
    primary_source_sha256: String,
    regions: Vec<RawRegion>,
    selectors: Vec<RawSelector>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSelector {
    selector: String,
    calendars: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRegion {
    region: String,
    calendars: Vec<String>,
}

struct Selector {
    selector: Box<str>,
    calendars: Box<[Box<str>]>,
}

pub(crate) struct LocaleCalendarsProfile {
    regions: Box<[Selector]>,
    selectors: Box<[Selector]>,
}

fn region_spelling(region: &str) -> bool {
    (region.len() == 2 && region.bytes().all(|byte| byte.is_ascii_uppercase()))
        || (region.len() == 3 && region.bytes().all(|byte| byte.is_ascii_digit()))
}

fn selector_spelling(selector: &str) -> bool {
    if let Some((language, region)) = selector.split_once('-') {
        ((2..=3).contains(&language.len()) || (5..=8).contains(&language.len()))
            && language.bytes().all(|byte| byte.is_ascii_lowercase())
            && region_spelling(region)
    } else {
        region_spelling(selector)
    }
}

fn checked_preferences(names: Vec<String>) -> Result<Box<[Box<str>]>, LocaleCalendarsProfileError> {
    if names.is_empty() || names.len() > 18 {
        return Err(LocaleCalendarsProfileError::Calendars);
    }
    for (i, name) in names.iter().enumerate() {
        if !matches!(
            name.as_str(),
            "buddhist"
                | "chinese"
                | "coptic"
                | "dangi"
                | "ethioaa"
                | "ethiopic"
                | "gregory"
                | "hebrew"
                | "indian"
                | "islamic"
                | "islamic-civil"
                | "islamic-rgsa"
                | "islamic-tbla"
                | "islamic-umalqura"
                | "iso8601"
                | "japanese"
                | "persian"
                | "roc"
        ) || names[..i].contains(name)
        {
            return Err(LocaleCalendarsProfileError::Calendars);
        }
    }
    Ok(names
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice())
}
impl LocaleCalendarsProfile {
    pub(crate) fn from_bytes(
        bytes: &[u8],
        digest: [u8; 32],
    ) -> Result<Self, LocaleCalendarsProfileError> {
        let actual: [u8; 32] = Sha256::digest(bytes).into();
        if actual != digest {
            return Err(LocaleCalendarsProfileError::Digest);
        }
        let raw: RawProfile =
            serde_json::from_slice(bytes).map_err(|_| LocaleCalendarsProfileError::Encoding)?;
        if raw.schema != 1 {
            return Err(LocaleCalendarsProfileError::Schema);
        }
        if raw.algorithm != "locale-calendars-cldr47-v1"
            || raw.cldr_release != "47.0.0"
            || raw.cldr_commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || raw.primary_manifest_sha256.len() != 64
            || !raw
                .primary_manifest_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || raw.primary_source_sha256
                != "3fb813039e4ab5041afc78c27fe35704b48e114513f008722e3e366473ed10b4"
        {
            return Err(LocaleCalendarsProfileError::Revision);
        }
        let mut regions: Vec<Selector> = Vec::with_capacity(raw.regions.len());
        for row in raw.regions {
            if !region_spelling(&row.region) || row.region == "ZZ" {
                return Err(LocaleCalendarsProfileError::Region);
            }
            if regions
                .last()
                .is_some_and(|previous| previous.selector.as_ref() >= row.region.as_str())
            {
                return Err(LocaleCalendarsProfileError::Order);
            }
            let calendars = checked_preferences(row.calendars)?;
            regions.push(Selector {
                selector: row.region.into_boxed_str(),
                calendars,
            });
        }
        let mut selectors: Vec<Selector> = Vec::with_capacity(raw.selectors.len());
        for row in raw.selectors {
            if !selector_spelling(&row.selector) {
                return Err(LocaleCalendarsProfileError::Selector);
            }
            if selectors
                .last()
                .is_some_and(|previous| previous.selector.as_ref() >= row.selector.as_str())
            {
                return Err(LocaleCalendarsProfileError::Order);
            }
            let calendars = checked_preferences(row.calendars)?;
            selectors.push(Selector {
                selector: row.selector.into_boxed_str(),
                calendars,
            });
        }
        if regions.len() != 292
            || selectors.len() != 52
            || !regions.iter().any(|row| row.selector.as_ref() == "001")
            || !selectors.iter().any(|row| row.selector.as_ref() == "001")
        {
            return Err(LocaleCalendarsProfileError::Coverage);
        }
        Ok(Self {
            regions: regions.into_boxed_slice(),
            selectors: selectors.into_boxed_slice(),
        })
    }

    fn selector(&self, selector: &str) -> Option<&[Box<str>]> {
        self.selectors
            .binary_search_by(|row| row.selector.as_ref().cmp(selector))
            .ok()
            .map(|index| self.selectors[index].calendars.as_ref())
    }

    /// Recognized sparse territories have explicit generated world-inherited rows.
    /// An unavailable override yields None so the caller can try the base region.
    pub(crate) fn calendars_for_region(&self, language: &str, region: &str) -> Option<&[Box<str>]> {
        let index = self
            .regions
            .binary_search_by(|row| row.selector.as_ref().cmp(region))
            .ok()?;
        self.selector(&format!("{language}-{region}"))
            .or(Some(self.regions[index].calendars.as_ref()))
    }
}
