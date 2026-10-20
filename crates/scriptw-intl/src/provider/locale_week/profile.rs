use serde::Deserialize;
use sha2::{Digest as _, Sha256};

use super::{LocaleWeekInfo, LocaleWeekProfileError};

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
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRegion {
    region: String,
    first_day: u32,
    weekend_mask: u32,
}

struct RegionWeek {
    region: Box<str>,
    info: LocaleWeekInfo,
}

pub(crate) struct LocaleWeekProfile {
    rows: Box<[RegionWeek]>,
    world: usize,
}

fn region_spelling(region: &str) -> bool {
    (region.len() == 2 && region.bytes().all(|b| b.is_ascii_uppercase()) && region != "ZZ")
        || (region.len() == 3 && region.bytes().all(|b| b.is_ascii_digit()))
}

impl LocaleWeekProfile {
    pub(crate) fn from_bytes(
        bytes: &[u8],
        digest: [u8; 32],
    ) -> Result<Self, LocaleWeekProfileError> {
        let actual: [u8; 32] = Sha256::digest(bytes).into();
        if actual != digest {
            return Err(LocaleWeekProfileError::Digest);
        }
        let raw: RawProfile =
            serde_json::from_slice(bytes).map_err(|_| LocaleWeekProfileError::Encoding)?;
        if raw.schema != 1 {
            return Err(LocaleWeekProfileError::Schema);
        }
        if raw.algorithm != "locale-week-cldr47-v1"
            || raw.cldr_release != "47.0.0"
            || raw.cldr_commit != "2ef784e3a4168bc2a43cd1b5b9839b6636f5899c"
            || raw.primary_manifest_sha256.len() != 64
            || raw.primary_source_sha256.len() != 64
            || !raw
                .primary_manifest_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
            || !raw
                .primary_source_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err(LocaleWeekProfileError::Revision);
        }
        let mut rows: Vec<RegionWeek> = Vec::with_capacity(raw.regions.len());
        let mut world = None;
        for item in raw.regions {
            if !region_spelling(&item.region) {
                return Err(LocaleWeekProfileError::Region);
            }
            if rows
                .last()
                .is_some_and(|previous| previous.region.as_ref() >= item.region.as_str())
            {
                return Err(LocaleWeekProfileError::RegionOrder);
            }
            let info = LocaleWeekInfo::from_iso_mask(item.first_day, item.weekend_mask)
                .map_err(|_| LocaleWeekProfileError::WeekInfo)?;
            if item.region == "001" {
                world = Some(rows.len());
            }
            rows.push(RegionWeek {
                region: item.region.into_boxed_str(),
                info,
            });
        }
        Ok(Self {
            rows: rows.into_boxed_slice(),
            world: world.ok_or(LocaleWeekProfileError::MissingWorldDefault)?,
        })
    }

    pub(crate) fn get(&self, region: &str) -> Option<&LocaleWeekInfo> {
        self.rows
            .binary_search_by(|row| row.region.as_ref().cmp(region))
            .ok()
            .map(|index| &self.rows[index].info)
    }

    pub(crate) fn world_default(&self) -> &LocaleWeekInfo {
        &self.rows[self.world].info
    }
}
