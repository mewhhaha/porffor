//! Complete native projection of the checked CLDR47 primary rows.

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeProfile {
    pub schema: u32,
    pub cldr_release: String,
    pub cldr_commit: String,
    pub country_icu_commit: String,
    pub source_manifest_sha256: String,
    pub supported_locales: Vec<String>,
    pub fallback_chain: Vec<String>,
    pub rows: Rows,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Rows {
    pub aliases: Vec<(String, String)>,
    pub metazones: Vec<Metazone>,
    pub patterns: Vec<(String, String, String)>,
    pub zones: Vec<Zone>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Metazone {
    pub identifier: String,
    pub names: [[Option<String>; 3]; 2],
    pub preferred: Vec<(String, String)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Zone {
    pub identifier: String,
    pub territory: String,
    pub city: String,
    pub country: Option<String>,
    pub location: Option<String>,
    pub names: [[Option<String>; 3]; 2],
    pub periods: Vec<(i64, i64, String)>,
}
