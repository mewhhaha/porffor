use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use super::{LocaleHourCycles, LocaleHourCyclesProfileError};
use crate::DateTimeHourCycle;

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
    cycles: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRegion {
    region: String,
    cycles: Vec<String>,
}

struct Selector {
    selector: Box<str>,
    cycles: LocaleHourCycles,
}

pub(crate) struct LocaleHourCyclesProfile {
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

impl LocaleHourCyclesProfile {
    pub(crate) fn from_bytes(
        bytes: &[u8],
        digest: [u8; 32],
    ) -> Result<Self, LocaleHourCyclesProfileError> {
        let actual: [u8; 32] = Sha256::digest(bytes).into();
        if actual != digest {
            return Err(LocaleHourCyclesProfileError::Digest);
        }
        let raw: RawProfile =
            serde_json::from_slice(bytes).map_err(|_| LocaleHourCyclesProfileError::Encoding)?;
        if raw.schema != 1 {
            return Err(LocaleHourCyclesProfileError::Schema);
        }
        if raw.algorithm != "locale-hour-cycles-cldr47-v1"
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
            return Err(LocaleHourCyclesProfileError::Revision);
        }
        let mut regions: Vec<Selector> = Vec::with_capacity(raw.regions.len());
        for row in raw.regions {
            if !region_spelling(&row.region) || row.region == "ZZ" {
                return Err(LocaleHourCyclesProfileError::Region);
            }
            if regions
                .last()
                .is_some_and(|previous| previous.selector.as_ref() >= row.region.as_str())
            {
                return Err(LocaleHourCyclesProfileError::Order);
            }
            let cycles = row
                .cycles
                .iter()
                .map(|value| {
                    DateTimeHourCycle::parse(value).ok_or(LocaleHourCyclesProfileError::Cycles)
                })
                .collect::<Result<Vec<_>, _>>()?;
            regions.push(Selector {
                selector: row.region.into_boxed_str(),
                cycles: LocaleHourCycles::checked(cycles)?,
            });
        }
        let mut selectors: Vec<Selector> = Vec::with_capacity(raw.selectors.len());
        for row in raw.selectors {
            if !selector_spelling(&row.selector) {
                return Err(LocaleHourCyclesProfileError::Selector);
            }
            if selectors
                .last()
                .is_some_and(|previous| previous.selector.as_ref() >= row.selector.as_str())
            {
                return Err(LocaleHourCyclesProfileError::Order);
            }
            let cycles = row
                .cycles
                .iter()
                .map(|value| {
                    DateTimeHourCycle::parse(value).ok_or(LocaleHourCyclesProfileError::Cycles)
                })
                .collect::<Result<Vec<_>, _>>()?;
            selectors.push(Selector {
                selector: row.selector.into_boxed_str(),
                cycles: LocaleHourCycles::checked(cycles)?,
            });
        }
        if regions.len() != 292
            || selectors.len() != 275
            || !regions.iter().any(|row| row.selector.as_ref() == "001")
            || !selectors.iter().any(|row| row.selector.as_ref() == "001")
        {
            return Err(LocaleHourCyclesProfileError::Coverage);
        }
        Ok(Self {
            regions: regions.into_boxed_slice(),
            selectors: selectors.into_boxed_slice(),
        })
    }

    fn selector(&self, selector: &str) -> Option<&LocaleHourCycles> {
        self.selectors
            .binary_search_by(|row| row.selector.as_ref().cmp(selector))
            .ok()
            .map(|index| &self.selectors[index].cycles)
    }

    /// Recognized sparse territories have explicit generated world-inherited rows.
    /// An unavailable override yields None so the caller can try the base region.
    pub(crate) fn cycles_for_region(
        &self,
        language: &str,
        region: &str,
    ) -> Option<&LocaleHourCycles> {
        let index = self
            .regions
            .binary_search_by(|row| row.selector.as_ref().cmp(region))
            .ok()?;
        self.selector(&format!("{language}-{region}"))
            .or(Some(&self.regions[index].cycles))
    }
}
